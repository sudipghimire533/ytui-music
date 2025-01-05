use std::sync::Arc;
use ytui_audio::libmpv::LibmpvPlayer;
use ytui_invidious::invidious::types::{
    channel::SearchChannelUnit,
    playlists::{PlaylistVideoUnit, SearchPlaylistUnit},
    video::{SearchVideoUnit, TrendingVideos},
};

mod impl_data_getter;

pub struct DataSink {
    pub(super) player: Arc<LibmpvPlayer>,
    pub(super) music_list: MusicList,
    pub(super) playlist_list: PlaylistList,
    pub(super) artist_list: ArtistList,

    has_new_data: bool,
}

pub enum MusicList {
    Error(String),
    SearchResult(Vec<SearchVideoUnit>),
    FetchedFromPlaylist(Vec<PlaylistVideoUnit>),
    Trending(TrendingVideos),
}

pub enum PlaylistList {
    Error(String),
    SearchResult(Vec<SearchPlaylistUnit>),
}

pub enum ArtistList {
    Error(String),
    SearchResult(Vec<SearchChannelUnit>),
}

impl Default for DataSink {
    fn default() -> Self {
        let mpv_player = LibmpvPlayer::new().unwrap();
        // by-default only stream audio
        mpv_player.disable_video().unwrap();

        Self {
            player: Arc::new(mpv_player),
            music_list: MusicList::SearchResult(Default::default()),
            playlist_list: PlaylistList::SearchResult(Default::default()),
            artist_list: ArtistList::SearchResult(Default::default()),
            has_new_data: false,
        }
    }
}

impl DataSink {
    pub fn make_player_copy(&self) -> Arc<LibmpvPlayer> {
        Arc::clone(&self.player)
    }

    pub(super) fn mark_new_data_arrival(&mut self) {
        self.has_new_data = true;
    }

    pub(super) fn music_id_at_index_or_last(&self, index: usize) -> Option<&str> {
        match &self.music_list {
            MusicList::Error(_error_message) => None,

            MusicList::SearchResult(music_list) => music_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(music_list.last())
                .map(|music| music.video_id.as_str()),
            MusicList::FetchedFromPlaylist(music_list) => music_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(music_list.last())
                .map(|music| music.video_id.as_str()),
            MusicList::Trending(music_list) => music_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(music_list.last())
                .map(|music| music.video_id.as_str()),
        }
    }

    pub(super) fn playlist_id_at_index_or_last(&self, index: usize) -> Option<&str> {
        match &self.playlist_list {
            PlaylistList::SearchResult(playlist_list) => playlist_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(playlist_list.last())
                .map(|playlist| playlist.playlist_id.as_str()),
            PlaylistList::Error(_error_message) => None,
        }
    }
}
