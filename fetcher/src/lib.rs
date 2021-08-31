use serde::{self, Deserialize, Serialize};
pub mod utils;
use reqwest;
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

fn yt_vid_id_to_url<'de, D>(input: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let id: &str = Deserialize::deserialize(input)?;
    Ok(format!("https://www.youtube.com/watch?v={id}", id = id))
}

// While fecthing playlist videos from endpoint /playlists/:plid
// response is returned as "videos": [ { <Fields of MusicUnit> } ]
// this structure is only used to convert such response to Vec<MusicUnit>
#[derive(Deserialize, Clone, PartialEq)]
struct FetchPlaylistContentRes {
    videos: Vec<MusicUnit>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct FetchArtistPlaylist {
    playlists: Vec<PlaylistUnit>,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct MusicUnit {
    #[serde(default)]
    pub liked: bool,
    #[serde(alias = "author")]
    pub artist: String,
    #[serde(alias = "title")]
    pub name: String,
    #[serde(alias = "lengthSeconds")]
    #[serde(deserialize_with = "seconds_to_str")]
    pub duration: String,
    #[serde(alias = "videoId")]
    #[serde(deserialize_with = "yt_vid_id_to_url")]
    pub path: String,
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

struct SearchRes {
    music: Vec<MusicUnit>,
    playlist: Vec<PlaylistUnit>,
    artist: Vec<ArtistUnit>,
    last_fetched: i8,
}

#[derive(Debug)]
pub enum ReturnAction {
    Failed,
    Retry,
    EOR, // End Of Result
}

pub struct Fetcher {
    trending_now: Option<Vec<MusicUnit>>,
    playlist_content: (String, Vec<MusicUnit>),
    artist_content: (String, Vec<MusicUnit>, Vec<PlaylistUnit>),
    servers: [&'static str; 6],
    search_res: (String, SearchRes),
    client: reqwest::Client,
    active_server_index: usize,
}
