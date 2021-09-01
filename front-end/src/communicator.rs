use crate::ui::{
    self,
    event::{MIDDLE_ARTIST_INDEX, MIDDLE_MUSIC_INDEX, MIDDLE_PLAYLIST_INDEX},
};
use fetcher;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

macro_rules! search_request {
    ($query: expr, $page: expr, $state_original: expr, $window_index: expr, $target: ident, $fetcher: expr, $fetcher_func: path) => {{
        let res_data = $fetcher_func($fetcher, $query, $page).await;

        let mut state = $state_original.lock().unwrap();
        match res_data {
            Ok(data) => {
                state.help = "Press ?";
                state.$target = VecDeque::from(data);
                state.fetched_page[$window_index] = Some($page);
            }
            Err(e) => {
                state.$target = VecDeque::new();
                match e {
                    fetcher::ReturnAction::Failed => {
                        state.help = "Search error..";
                        state.fetched_page[$window_index] = None;
                    }
                    fetcher::ReturnAction::EOR => {
                        state.help = "Result end..";
                        state.fetched_page[$window_index] = None;
                    }
                    fetcher::ReturnAction::Retry => {
                        state.help = "temp error..";
                        /* TODO: retry */
                    }
                }
            }
        }
    }};
}

pub async fn communicator<'st, 'nt>(
    state_original: &'st mut Arc<Mutex<ui::State<'_>>>,
    notifier: &'nt mut Arc<Condvar>,
) {
    let mut fetcher = fetcher::Fetcher::new();

    let (mut prev_musicbar_source, mut prev_playlistbar_source, mut prev_artistbar_source) = {
        let state = state_original.lock().unwrap();
        (
            state.filled_source.0.clone(),
            state.filled_source.1.clone(),
            state.filled_source.2.clone(),
        )
    };
    'communicator_loop: loop {
        let mut state = notifier.wait(state_original.lock().unwrap()).unwrap();
        if state.active == ui::Window::None {
            break 'communicator_loop;
        }

        if state.filled_source.1 != prev_playlistbar_source {
            prev_playlistbar_source = state.filled_source.1.clone();
            state.playlistbar.clear();
            std::mem::drop(state);
            notifier.notify_all();
            match prev_playlistbar_source {
                ui::PlaylistbarSource::Search(ref term, page) => {
                    search_request!(
                        term,
                        page,
                        state_original,
                        MIDDLE_PLAYLIST_INDEX,
                        playlistbar,
                        &mut fetcher,
                        fetcher::Fetcher::search_playlist
                    );
                }
                ui::PlaylistbarSource::Artist(_artist_id, _page) => todo!(),
                ui::PlaylistbarSource::Favourates | ui::PlaylistbarSource::RecentlyPlayed => {}
            }
            state_original.lock().unwrap().active = ui::Window::Playlistbar;
            notifier.notify_all();
        } else {
            std::mem::drop(state);
        }

        let mut state = state_original.lock().unwrap();
        if state.filled_source.2 != prev_artistbar_source {
            prev_artistbar_source = state.filled_source.2.clone();
            state.artistbar.clear();
            std::mem::drop(state);
            notifier.notify_all();
            match prev_artistbar_source {
                ui::ArtistbarSource::Search(ref term, page) => {
                    search_request!(
                        term,
                        page,
                        state_original,
                        MIDDLE_ARTIST_INDEX,
                        artistbar,
                        &mut fetcher,
                        fetcher::Fetcher::search_artist
                    );
                }
                ui::ArtistbarSource::RecentlyPlayed | ui::ArtistbarSource::Favourates => {}
            }
            state_original.lock().unwrap().active = ui::Window::Artistbar;
            notifier.notify_all();
        } else {
            std::mem::drop(state);
        }

        let mut state = state_original.lock().unwrap();
        if state.filled_source.0 != prev_musicbar_source {
            prev_musicbar_source = state.filled_source.0.clone();
            state.musicbar.clear();
            std::mem::drop(state);
            notifier.notify_all();
            // prev_musicbar_source and current musicbar_source are equal at this point
            match prev_musicbar_source {
                ui::MusicbarSource::Trending(page) => {
                    let trending_music = fetcher.get_trending_music(page).await;
                    // Lock state only after fetcher is done with web request
                    let mut state = state_original.lock().unwrap();

                    match trending_music {
                        Ok(data) => {
                            state.help = "Press ?";
                            state.musicbar = VecDeque::from(Vec::from(data));
                            state.fetched_page[MIDDLE_MUSIC_INDEX] = Some(page);
                        }
                        Err(e) => {
                            state.playlistbar = VecDeque::new();
                            match e {
                                fetcher::ReturnAction::EOR => {
                                    state.help = "Result end..";
                                    state.fetched_page[MIDDLE_MUSIC_INDEX] = None;
                                }
                                fetcher::ReturnAction::Failed => {
                                    state.help = "Fetch error..";
                                    state.fetched_page[MIDDLE_MUSIC_INDEX] = None;
                                }
                                fetcher::ReturnAction::Retry => {
                                    state.help = "temp error..";
                                    /* TODO: Retry */
                                }
                            }
                        }
                    }
                }
                ui::MusicbarSource::Search(ref term, page) => {
                    search_request!(
                        term,
                        page,
                        state_original,
                        MIDDLE_MUSIC_INDEX,
                        musicbar,
                        &mut fetcher,
                        fetcher::Fetcher::search_music
                    );
                }
                ui::MusicbarSource::Playlist(ref playlist_id, page) => {
                    let playlist_content = fetcher.get_playlist_content(&playlist_id, page).await;
                    let mut state = state_original.lock().unwrap();
                    match playlist_content {
                        Ok(data) => {
                            state.help = "Press ?";
                            state.musicbar = VecDeque::from(data);
                        }
                        Err(e) => {
                            state.musicbar = VecDeque::new();
                            match e {
                                fetcher::ReturnAction::EOR => {
                                    state.help = "Result end..";
                                    state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] = None;
                                }
                                fetcher::ReturnAction::Failed => {
                                    state.help = "Fetch error..";
                                    state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] = None;
                                }
                                fetcher::ReturnAction::Retry => {
                                    state.help = "temp error..";
                                    /* TODO: Retry */
                                }
                            }
                        }
                    }
                }
                ui::MusicbarSource::Artist(_artist_id, _page) => {
                    todo!()
                }
                ui::MusicbarSource::Favourates
                | ui::MusicbarSource::RecentlyPlayed
                | ui::MusicbarSource::YoutubeCommunity => {}
            }
            state_original.lock().unwrap().active = ui::Window::Musicbar;
            notifier.notify_all();
        } else {
            // If above if block is not executed state lock should however be released
            // so that state can be lock again for following if block
            std::mem::drop(state);
        }
    }
}
