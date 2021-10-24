use serde::{self, Deserialize, Serialize};
pub mod utils;
use std::time::Duration;

pub trait ExtendDuration {
    fn to_string(self) -> String;
    fn from_string(inp: &str) -> Duration;
}

fn num_to_str<'de, D>(input: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let num: usize = Deserialize::deserialize(input)?;
    let mut res = num.to_string();
    res.shrink_to_fit();
    Ok(res)
}

fn seconds_to_str<'de, D>(input: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Note: If duration is set to 0:0 from the json response
    // the the video may be live ({islive: true, ..} in response)
    // to keep things simple ignore all those details and this will simply return "0:0"
    // this should be documented to inform the user
    let sec: u64 = Deserialize::deserialize(input)?;
    let dur: Duration = Duration::from_secs(sec);
    Ok(dur.to_string())
}

// While fecthing playlist videos from endpoint /playlists/:plid
// response is returned as "videos": [ { <Fields of MusicUnit> } ]
// this structure is only used to convert such response to Vec<MusicUnit>
#[derive(Deserialize, Clone, PartialEq)]
struct FetchPlaylistContentRes {
    videos: Vec<MusicUnit>,
}

// Serve same purpose as described in struct FetchPlaylistContentRes but
// to convert to Vec<PlaylistUnit>
#[derive(Deserialize, Clone, PartialEq)]
struct FetchArtistPlaylist {
    playlists: Vec<PlaylistUnit>,
}

// Represent the single playable music item.
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct MusicUnit {
    // uniquely identifiable id of the youtube channel that represent the publisher of this unit
    // This field exist to make it possible to navigate to the artist channel from the song alone
    // server return this field as `author`
    #[serde(alias = "author")]
    pub artist: String,
    // The name of the music unit itself. This may also contains the unicode or
    // any unprintable character.
    // This field simply serves as the music name to be displayed in list
    // server return this field as `title`
    #[serde(alias = "title")]
    pub name: String,
    #[serde(alias = "lengthSeconds")]
    #[serde(deserialize_with = "seconds_to_str")]
    pub duration: String,
    #[serde(alias = "videoId")]
    pub id: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ArtistUnit {
    #[serde(alias = "author")]
    pub name: String,
    #[serde(alias = "authorId")]
    pub id: String,
    #[serde(alias = "videoCount")]
    #[serde(deserialize_with = "num_to_str")]
    pub video_count: String,
}
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct PlaylistUnit {
    #[serde(alias = "title")]
    pub name: String,
    #[serde(alias = "playlistId")]
    pub id: String,
    pub author: String,
    #[serde(alias = "videoCount")]
    #[serde(deserialize_with = "num_to_str")]
    pub video_count: String,
}

#[derive(Default)]
struct SearchRes {
    music: Vec<MusicUnit>,
    playlist: Vec<PlaylistUnit>,
    artist: Vec<ArtistUnit>,
    query: String,
    last_fetched: i8,
}

#[derive(Default)]
struct ArtistRes {
    music: (String, Vec<MusicUnit>),
    playlist: (String, Vec<PlaylistUnit>),
}

#[derive(Default)]
struct PlaylistRes {
    music: Vec<MusicUnit>,
    id: String,
}

/*
Return type of the fetch function. This indicates different reason on why no data
was returned from the fetcher function as stated below.
*/
#[derive(Debug)]
pub enum ReturnAction {
    // This variat indicates that the fetch has failed and cannot be resolved on retrying
    // This may be due to several reasons including server down, network failure, parse failure
    // Also Failed is active when fetcher had retried and now had exceed the retry count
    Failed,
    // This variant simply indicates that the request has failed but doing the same request for
    // another time may suceed
    Retry,
    // EOR avvrebration of End Of Result indicates that there is nothing more to fetch
    // At this point the corresponding container have all the data either fetched at once
    // like or had fetched the maximum page in pagination fetch
    // A common example is when all the 3/4 page of trending page have been fetched
    // and user had seen the end. or all the search result have been srved
    // At this condition the fetch has technicallt suceed and also tells that
    // another retry on same query will return EOR again and again until new fetch is to be made
    EOR,
}

pub struct Fetcher {
    // None if nothing of the trending music is selected.
    // Stores the vector of music that is trending in music section in specified region
    // trending_now, is never cleared in a session. Unlike many others data container
    // below, this field is only appended and read. When user paginate and there are no more
    // result in container another web request method is made and result is again stored and never cleared.
    // This may bring little delay when user explore for first time in a session but after that everything
    // will be in memory making it smooth.
    trending_now: Option<Vec<MusicUnit>>,

    //playlist_content stores collection of music contained in a playlist
    // first field: (String) holds the unique if of playlist that is being read.
    // All the content in given playlist id is fetched at once and stored meaning
    // only single web request is needed and no other even when user asks for next page.
    // On the other hand this means for a playlist with large amount of content
    // takes more time to initilized.
    // The needed request will return the array of music data. And currently there is no way
    // to fetch only the necessary fields inside music struct. Which means even with playlist
    // of samll size, over data is returend by the data that is just ignored from our side. Thus
    // increasing the network traffic.
    // On the positive side, it is not needed to send multiple request when user explore the content
    // in pagiunation. Just the different chunks of data is returned. Also, this makes possible to
    // feed the player backend with all the content of playlist which means playing from
    // playlist don't have to get interrupted for fetching more data once played tha last item of
    // last fetched data. Playback stops only when playlist ends.
    playlist_content: PlaylistRes,

    /*
    artist_content stores collection of music and also the collection of playlists
    from the channel
    First field: (String) holds the unique id of channel being fetched.
    For more info see documentation on playlist_content above
    */
    artist_content: ArtistRes,

    // List of available servers powered by invidious youtube data fetcher. All the servers should
    // provide same endpoints to make request to and same pattern of return data. Which actually means
    // all the servers must be powered by the same mahor version of invidious backend.
    // Most of the servers do not expect high amount of request to their api. So to protect this
    // it would be better to frequently change the server time to time even in single session.
    // To distribute the load between multiple servers it would be better if this list is kept growing
    // See the utils.rs file to see the format of server url.
    servers: &'static [String],

    // Container to store the result of search result.
    // First field: (String) is the query being searched for.
    search_res: SearchRes,

    // The reqwest client itself. This is only initilized once per session.
    client: reqwest::Client,

    // index that reference the servers[] field.
    // When server need to be changes as described in documentation of servers[] field
    // this index is updated (usually rotated clockwise)
    // TODO:
    // It may be more efficient to directly reference the elemnt from searvers[] rather than
    // storing the index and hence preventing accidintal out-of-index access
    active_server_index: usize,

    // copy of constants.item_per_list
    item_per_page: usize,
    // reference to constants.region in config file
    region: &'static str,
}
