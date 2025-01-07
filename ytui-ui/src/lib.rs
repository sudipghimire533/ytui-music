use smallvec::SmallVec;
use ytui_audio::libmpv::MpvPropertyGet;

mod components;
mod dimension;
pub mod renderer;
mod state;

pub trait DataRequester {
    fn search_new_term(
        &mut self,
        term: String,
        search_for_music: bool,
        search_for_playlist: bool,
        search_for_artist: bool,
    );
    fn quit(&mut self);
    fn play_from_music_pane(&mut self, selected_index: usize);
    fn fetch_from_playlist_pane(&mut self, selected_index: usize);
    fn fetch_from_artist_pane(&mut self, selected_index: usize);
    fn fetch_trending_music(&mut self);
    fn toggle_pause_playback(&mut self);
}

pub trait DataGetter {
    fn startup_overlay_announcement(&self) -> Option<(String, String)>;
    fn get_music_list(&self) -> Vec<[String; 3]>;
    fn get_playlist_list(&self) -> Vec<[String; 2]>;
    fn get_artist_list(&self) -> Vec<String>;
    fn has_new_data(&self) -> bool;
    fn mark_consumed_new_data(&mut self);
}

pub trait PlayerStatsGetter {
    fn get_player_stats(&self) -> PlayerStats;
}

#[derive(Default, Debug)]
pub struct PlayerStats {
    pub total_duration: Option<i64>,
    pub elabsed_duration: Option<i64>,
    pub playback_percent: Option<i64>,
    pub media_title: Option<String>,
    pub paused: Option<bool>,
    pub volume: Option<i64>,
    pub next_in_queue: SmallVec<[String; 5]>,
}

impl PlayerStats {
    fn from_libmpv_player(mpv_player: &ytui_audio::libmpv::LibmpvPlayer) -> Self {
        let total_duration = mpv_player
            .get_property::<i64>(MpvPropertyGet::Duration)
            .unwrap();
        let media_title = mpv_player.get_property(MpvPropertyGet::MediaTitle).unwrap();
        let elabsed_duration = mpv_player
            .get_property::<i64>(MpvPropertyGet::TimePos)
            .unwrap();
        let paused = mpv_player
            .get_property(MpvPropertyGet::PauseStatus)
            .unwrap();
        let volume = mpv_player.get_property(MpvPropertyGet::Volume).unwrap();
        let playback_percent = mpv_player
            .get_property(MpvPropertyGet::PercentPos)
            .unwrap()
            .map(Option::Some)
            .unwrap_or_else(|| {
                if let (Some(total), Some(elapsed)) = (total_duration, elabsed_duration) {
                    Some(elapsed.saturating_sub(100).saturating_div(total))
                } else {
                    None
                }
            });

        let next_in_queue = std::iter::repeat(())
            .take({
                let next_in_queue_count = mpv_player
                    .get_property::<i64>(MpvPropertyGet::PlaylistCount)
                    .unwrap()
                    .unwrap_or_default() as usize;
                std::cmp::min(next_in_queue_count, 5)
            })
            .enumerate()
            .filter_map(|(nth, _)| {
                mpv_player
                    .get_property(MpvPropertyGet::NthPlaylistItemTitle(nth))
                    .unwrap()
            })
            .collect();

        PlayerStats {
            total_duration,
            elabsed_duration,
            media_title,
            paused,
            volume,
            playback_percent,
            next_in_queue,
        }
    }
}
