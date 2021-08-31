use serde::{self, Deserialize, Serialize};
pub mod utils;
use reqwest;
use serde::de::Deserializer;
use std::{collections::VecDeque, time::Duration};

pub trait ExtendDuration {
    fn to_string(self) -> String;
    fn from_string(inp: &str) -> Duration;
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
    pub name: String,
}
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct PlaylistUnit {
    pub name: String,
}

struct SearchRes {
    music: Vec<MusicUnit>,
    playlist: Vec<PlaylistUnit>,
    artist: Vec<ArtistUnit>,
}

#[derive(Debug)]
pub enum ReturnAction {
    Failed,
    Retry,
    EOR, // End Of Result
}

pub struct Fetcher {
    trending_now: Option<Vec<MusicUnit>>,
    servers: [&'static str; 6],
    search_res: (String, SearchRes),
    client: reqwest::Client,
    active_server_index: usize,
}
