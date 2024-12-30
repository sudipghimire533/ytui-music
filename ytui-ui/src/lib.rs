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
}

pub trait DataGetter {
    fn get_music_list(&self) -> impl Iterator<Item = [std::borrow::Cow<'_, str>; 3]>;
    fn get_playlist_list(&self) -> &[&str; 2];
    fn get_artist_list(&self) -> &[&str];
}
