use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MusicInfo {
    pub id: String,

    pub title: Option<String>,
    pub length: Option<Duration>,
    pub stream_count: Option<u64>,
    pub like_count: Option<i64>,
    pub dislike_count: Option<i64>,

    pub author_id: Option<String>,
    pub author_name: Option<String>,

    pub album_id: Option<String>,
    pub album_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistInfo {
    pub id: String,

    pub item_list: Vec<MusicInfo>,

    pub title: Option<String>,
    pub items_count: Option<u32>,

    pub author_id: Option<String>,
    pub author_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtistInfo {
    pub id: String,

    pub name: Option<String>,
    pub playlist_count: Option<u64>,
    pub music_count: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResults {
    pub musics: Vec<MusicInfo>,
    pub playlists: Vec<PlaylistInfo>,
    pub artists: Vec<ArtistInfo>,
}
