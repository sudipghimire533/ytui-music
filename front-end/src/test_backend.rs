use crate::ui::{ArtistUnit, MusicUnit, PlaylistUnit};
use serde_json;

fn get_sample_music_list(page: u32) -> Option<Vec<MusicUnit>> {
    let mut all: Vec<MusicUnit> =
        serde_json::from_str(include_str!("../src/test-data/trending.json")).unwrap();
    for i in 0..all.len() {
        all[i].path = "/home/sudip/Music/music.mp3".to_string();
        all[i].name = i.to_string() + "  " + &all[i].name;
    }
    let lower_limit = 10 as usize * page as usize;
    if lower_limit >= all.len() {
        return None;
    }
    let upper_limit = std::cmp::min(lower_limit + 10, all.len());
    Some(all[lower_limit..upper_limit].to_vec())
}

fn get_sample_playlist_list(page: u32) -> Option<Vec<PlaylistUnit>> {
    let mut all: Vec<PlaylistUnit> =
        serde_json::from_str(include_str!("../src/test-data/playlist_sample.json")).unwrap();
    for i in 0..all.len() {
        all[i].name = i.to_string() + "  " + &all[i].name;
    }
    let lower_limit = 10 as usize * page as usize;
    if lower_limit > all.len() {
        return None;
    }
    let upper_limit = std::cmp::min(lower_limit + 10, all.len());
    Some(all[lower_limit..upper_limit].to_vec())
}

fn get_sample_artist_list(page: u32) -> Option<Vec<ArtistUnit>> {
    let mut all: Vec<ArtistUnit> =
        serde_json::from_str(include_str!("../src/test-data/artist_sample.json")).unwrap();
    for i in 0..all.len() {
        all[i].name = i.to_string() + "  " + &all[i].name;
    }
    let lower_limit = 10 as usize * page as usize;
    if lower_limit > all.len() {
        return None;
    }
    let upper_limit = std::cmp::min(lower_limit + 10, all.len());
    Some(all[lower_limit..upper_limit].to_vec())
}
macro_rules! return_music_data {
    ($prefix: expr, $page: expr) => {
        if let Some(data) = get_sample_music_list($page) {
            Some(
                data.iter()
                    .map(|v| MusicUnit {
                        name: $prefix.to_string() + "  " + &v.name,
                        ..v.clone()
                    })
                    .collect(),
            )
        } else {
            None
        }
    };
}

macro_rules! return_playlist_data {
    ($prefix: expr, $page: expr) => {
        if let Some(data) = get_sample_playlist_list($page) {
            Some(
                data.iter()
                    .map(|v| PlaylistUnit {
                        name: $prefix.to_string() + &v.name,
                        ..v.clone()
                    })
                    .collect(),
            )
        } else {
            None
        }
    };
}

macro_rules! return_artist_data {
    ($prefix: expr, $page: expr) => {
        if let Some(data) = get_sample_artist_list($page) {
            Some(
                data.iter()
                    .map(|v| ArtistUnit {
                        name: $prefix.to_string() + &v.name,
                        ..v.clone()
                    })
                    .collect(),
            )
        } else {
            None
        }
    };
}

pub fn get_trending_music(page: u32) -> Option<Vec<MusicUnit>> {
    return_music_data!(" - [trending] - ", page)
}

pub fn get_community_music(page: u32) -> Option<Vec<MusicUnit>> {
    return_music_data!(">> [Community] ", page)
}
pub fn get_favourates_music(page: u32) -> Option<Vec<MusicUnit>> {
    return_music_data!("==>[favourates] ", page)
}
pub fn get_promoted_music(page: u32) -> Option<Vec<MusicUnit>> {
    return_music_data!("<=>[promoted] ", page)
}
pub fn get_recents_music(page: u32) -> Option<Vec<MusicUnit>> {
    return_music_data!("*-* [recents] ", page)
}
pub fn get_following_music(page: u32) -> Option<Vec<MusicUnit>> {
    return_music_data!("%^^ [followings] ", page)
}
pub fn get_saved_playlist(page: u32) -> Option<Vec<PlaylistUnit>> {
    return_playlist_data!("[Saved]<< ", page)
}
pub fn get_following_artist(page: u32) -> Option<Vec<ArtistUnit>> {
    return_artist_data!("[Following ]<< ", page)
}
