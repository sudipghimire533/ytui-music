use super::requests::*;
use super::types as invidious_types;
use super::InvidiousBackend;
use crate::common_interface;
use crate::web_client::WebClient;
use common_interface::StreamMandu;
use std::time::Duration;

#[async_trait::async_trait]
impl<WebC: WebClient + Send + Sync> StreamMandu for InvidiousBackend<WebC> {
    async fn get_music_info(
        &self,
        music_id: &str,
    ) -> common_interface::Result<common_interface::types::MusicInfo> {
        let query = RequestVideoById::new(InvidiousApiQuery::VideoById { video_id: music_id });
        let fetch_result = self.fetch_endpoint(query).await;

        match fetch_result {
            Ok(music_info) => Ok(music_info.into()),
            Err(_e) => todo!(),
        }
    }

    async fn get_playlist_info(
        &self,
        playlist_id: &str,
    ) -> common_interface::Result<common_interface::types::PlaylistInfo> {
        let query = RequestPlaylistById::new(InvidiousApiQuery::PlaylistById { playlist_id });
        let fetch_result = self.fetch_endpoint(query).await;

        match fetch_result {
            Ok(playlist_info) => Ok(playlist_info.into()),
            Err(_e) => todo!(),
        }
    }

    async fn get_artist_info(
        &self,
        artist_id: &str,
    ) -> common_interface::Result<common_interface::types::ArtistInfo> {
        todo!("can not fetch info for artist: {}", artist_id)
    }

    async fn fetch_trending_music(
        &self,
        region: Option<crate::region::IsoRegion>,
    ) -> common_interface::Result<Vec<common_interface::types::MusicInfo>> {
        let query = RequestTrending::new(InvidiousApiQuery::Trending {
            region: region.unwrap_or(crate::region::IsoRegion::NP),
        });
        let fetch_result = self
            .fetch_endpoint(query)
            .await
            .map_err(|e| e.as_error_string())?;
        let result_as_music_list = fetch_result.into_iter().map(Into::into).collect();
        Ok(result_as_music_list)
    }

    async fn search_query(
        &self,
        query: &str,
        find_music: bool,
        find_playlist: bool,
        find_artist: bool,
    ) -> common_interface::Result<common_interface::types::SearchResults> {
        let query = RequestSearch::new(InvidiousApiQuery::Search {
            query,
            find_playlist,
            find_artist,
            find_music,
        });
        let fetch_result = self
            .fetch_endpoint(query)
            .await
            .map_err(|e| e.as_error_string())?;
        let fetch_result_as_search_list = fetch_result.into();
        Ok(fetch_result_as_search_list)
    }
}

impl From<invidious_types::video::VideoInfo> for common_interface::types::MusicInfo {
    fn from(invidious_video: invidious_types::video::VideoInfo) -> Self {
        Self {
            id: invidious_video.video_id,
            title: Some(invidious_video.title),
            length: Some(Duration::from_secs(invidious_video.length_seconds as u64)),
            stream_count: Some(invidious_video.view_count as u64),
            like_count: Some(invidious_video.like_count as i64),
            dislike_count: Some(invidious_video.dislike_count as i64),
            author_id: Some(invidious_video.author_id),
            author_name: Some(invidious_video.author),
            album_id: None,
            album_name: None,
        }
    }
}

impl From<invidious_types::video::SearchVideoUnit> for common_interface::types::MusicInfo {
    fn from(invidious_video: invidious_types::video::SearchVideoUnit) -> Self {
        Self {
            id: invidious_video.video_id,
            title: Some(invidious_video.title),
            length: Some(Duration::from_secs(invidious_video.length_seconds as u64)),
            stream_count: Some(invidious_video.view_count as u64),
            like_count: None,
            dislike_count: None,
            author_id: Some(invidious_video.author_id),
            author_name: Some(invidious_video.author),
            album_id: None,
            album_name: None,
        }
    }
}
impl From<invidious_types::video::TrendingVideo> for common_interface::types::MusicInfo {
    fn from(invidious_video: invidious_types::video::TrendingVideo) -> Self {
        Self {
            id: invidious_video.video_id,
            title: Some(invidious_video.title),
            length: Some(Duration::from_secs(invidious_video.length_seconds as u64)),
            stream_count: Some(invidious_video.view_count as u64),
            like_count: None,
            dislike_count: None,
            author_id: Some(invidious_video.author_id),
            author_name: Some(invidious_video.author),
            album_id: None,
            album_name: None,
        }
    }
}

impl From<invidious_types::playlists::PlaylistVideoUnit> for common_interface::types::MusicInfo {
    fn from(invidious_video: invidious_types::playlists::PlaylistVideoUnit) -> Self {
        Self {
            id: invidious_video.video_id,
            title: Some(invidious_video.title),
            length: Some(Duration::from_secs(invidious_video.length_seconds as u64)),
            stream_count: None,
            like_count: None,
            dislike_count: None,
            author_id: Some(invidious_video.author_id),
            author_name: Some(invidious_video.author),
            album_id: None,
            album_name: None,
        }
    }
}

impl From<invidious_types::common::PlaylistVideoObject> for common_interface::types::MusicInfo {
    fn from(invidious_video: invidious_types::common::PlaylistVideoObject) -> Self {
        Self {
            id: invidious_video.video_id,
            title: Some(invidious_video.title),
            length: Some(Duration::from_secs(invidious_video.length_seconds)),
            stream_count: None,
            like_count: None,
            dislike_count: None,
            author_id: None,
            author_name: None,
            album_id: None,
            album_name: None,
        }
    }
}

impl From<invidious_types::playlists::PlaylistInfo> for common_interface::types::PlaylistInfo {
    fn from(invidious_playlist: invidious_types::playlists::PlaylistInfo) -> Self {
        Self {
            id: invidious_playlist.playlist_id,
            title: Some(invidious_playlist.title),
            items_count: Some(invidious_playlist.video_count as u32),
            author_id: Some(invidious_playlist.author_id),
            author_name: Some(invidious_playlist.author),
            item_list: invidious_playlist
                .videos
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<invidious_types::playlists::SearchPlaylistUnit>
    for common_interface::types::PlaylistInfo
{
    fn from(invidious_playlist: invidious_types::playlists::SearchPlaylistUnit) -> Self {
        Self {
            id: invidious_playlist.playlist_id,
            title: Some(invidious_playlist.title),
            items_count: Some(invidious_playlist.video_count as u32),
            author_id: Some(invidious_playlist.author_id),
            author_name: Some(invidious_playlist.author),
            item_list: invidious_playlist
                .videos
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<invidious_types::channel::ChannelInfo> for common_interface::types::ArtistInfo {
    fn from(invidious_channel: invidious_types::channel::ChannelInfo) -> Self {
        Self {
            id: invidious_channel.author_id,
            name: Some(invidious_channel.author),
            playlist_count: None,
            music_count: None,
        }
    }
}

impl From<invidious_types::channel::SearchChannelUnit> for common_interface::types::ArtistInfo {
    fn from(invidious_channel: invidious_types::channel::SearchChannelUnit) -> Self {
        Self {
            id: invidious_channel.author_id,
            name: Some(invidious_channel.author),
            playlist_count: None,
            music_count: None,
        }
    }
}

impl From<invidious_types::common::SearchResults> for common_interface::types::SearchResults {
    fn from(invidious_search_results: invidious_types::common::SearchResults) -> Self {
        let mut musics = vec![];
        let mut playlists = vec![];
        let mut artists = vec![];

        for search_result in invidious_search_results {
            match search_result {
                invidious_types::common::SearchResult::Video(search_video_unit) => {
                    musics.push(search_video_unit.into());
                }
                invidious_types::common::SearchResult::Playlist(search_playlist_unit) => {
                    playlists.push(search_playlist_unit.into())
                }
                invidious_types::common::SearchResult::Channel(search_channel_unit) => {
                    artists.push(search_channel_unit.into())
                }
            }
        }

        Self {
            musics,
            playlists,
            artists,
        }
    }
}
