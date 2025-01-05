impl ytui_ui::DataRequester for super::SourceAction {
    fn search_new_term(
        &mut self,
        term: String,
        search_for_music: bool,
        search_for_playlist: bool,
        search_for_artist: bool,
    ) {
        self.search_query = Some((
            term,
            search_for_music,
            search_for_playlist,
            search_for_artist,
        ))
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn play_from_music_pane(&mut self, selected_index: usize) {
        self.music_play_index = Some(selected_index);
    }

    fn fetch_from_artist_pane(&mut self, selected_index: usize) {
        self.playlist_fetch_index = Some(selected_index);
    }

    fn fetch_from_playlist_pane(&mut self, selected_index: usize) {
        self.playlist_fetch_index = Some(selected_index)
    }

    fn fetch_trending_music(&mut self) {
        self.fetch_trending_music = Some(());
    }

    fn toggle_pause_playback(&mut self) {
        self.pause_playback_toggle = Some(());
    }
}
