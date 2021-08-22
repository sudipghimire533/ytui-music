use serde::{self, Deserialize, Serialize};
pub mod utils;
use reqwest;
use serde::de::Deserializer;
use std::{collections::VecDeque, time::Duration};

pub trait ExtendDuration {
    fn to_string(self) -> String;
    fn from_string(inp: &str) -> Duration;
}

fn seconds_to_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let sec: u64 = Deserialize::deserialize(deserializer)?;
    let dur: Duration = Duration::from_secs(sec);
    Ok(dur.to_string())
}

fn yt_vid_id_to_url<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let id: &str = Deserialize::deserialize(deserializer)?;
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

pub struct Fetcher {
    trending_now: Option<Vec<MusicUnit>>,
    servers: [&'static str; 6],
    client: reqwest::Client,
    active_server_index: usize,
}
