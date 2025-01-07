use smallvec::SmallVec;
use std::sync::Arc;
use stream_mandu::common_interface::types::{ArtistInfo, MusicInfo, PlaylistInfo};
use ytui_audio::libmpv::LibmpvPlayer;

mod app_announcement;
mod impl_data_getter;

pub struct DataSink {
    pub(super) player: Arc<LibmpvPlayer>,
    pub(super) music_list: Result<Vec<MusicInfo>, String>,
    pub(super) playlist_list: Result<Vec<PlaylistInfo>, String>,
    pub(super) artist_list: Result<Vec<ArtistInfo>, String>,
    pub(super) queue_list: SmallVec<[(String, String); 5]>,

    has_new_data: bool,
}

impl Default for DataSink {
    fn default() -> Self {
        let mpv_player = LibmpvPlayer::new().unwrap();
        // by-default only stream audio
        mpv_player.disable_video().unwrap();

        Self {
            player: Arc::new(mpv_player),
            music_list: Ok(Vec::new()),
            playlist_list: Ok(Vec::new()),
            artist_list: Ok(Vec::new()),
            queue_list: SmallVec::new(),
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

    pub(super) fn at_or_last<T>(list: &[T], index: usize) -> Option<&T> {
        list.get(index).map(Option::Some).unwrap_or(list.last())
    }
}
