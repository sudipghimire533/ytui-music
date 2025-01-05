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
}

pub trait DataGetter {
    fn get_music_list(&self) -> impl Iterator<Item = [String; 3]>;
    fn get_playlist_list(&self) -> &[&str; 2];
    fn get_artist_list(&self) -> &[&str];
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
    pub media_title: Option<String>,
}

impl PlayerStats {
    fn from_libmpv_player(mpv_player: &ytui_audio::libmpv::LibmpvPlayer) -> Self {
        let total_duration = mpv_player.get_property(MpvPropertyGet::Duration).unwrap();
        let media_title = mpv_player.get_property(MpvPropertyGet::MediaTitle).unwrap();
        let elabsed_duration = mpv_player.get_property(MpvPropertyGet::TimePos).unwrap();

        PlayerStats {
            total_duration,
            elabsed_duration,
            media_title,
        }
    }
}
