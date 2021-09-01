use crate::ui;
use fetcher;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

macro_rules! search_request {
    ($query: expr, $page: expr, $state_original: expr, $window_index: expr, $target: expr, $fetcher: expr, $fetcher_func: path) => {{
        let res_data = $fetcher_func($fetcher, $query.as_str(), $page).await;
        let mut state = $state_original.lock().unwrap();

        let res;
        match res_data {
            Ok(data) => {
                state.help = "Press ?";
                res = VecDeque::from(data);
                state.fetched_page[$window_index] = Some($page);
            }
            Err(e) => {
                res = VecDeque::new();
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

        // $target is recived as state_original.lock().unwrap().<musibac|paylistbar|artistbar>
        // So previous lock to state should be released
        std::mem::drop(state);
        $target = res;
    }};
}

pub async fn communicator<'st, 'nt>(
    state_original: &'st mut Arc<Mutex<ui::State<'_>>>,
    notifier: &'nt mut Arc<Condvar>,
) {
    let mut fetcher = fetcher::Fetcher::new();

    'communicator_loop: loop {
        let to_fetch;
        {
            let state = notifier.wait(state_original.lock().unwrap()).unwrap();
            if state.active == ui::Window::None {
                break 'communicator_loop;
            }
            to_fetch = state.to_fetch.clone();
            // [State Unlocked Here!!] Now the fetch may take some time
            // so we should not lock the state so can ui keeps responding
        }
        match to_fetch {
            ui::FillFetch::None => {}
            ui::FillFetch::Trending(page) => {
                state_original.lock().unwrap().musicbar.clear();

                let trending_music = fetcher.get_trending_music(page).await;
                // Lock state only after fetcher is done with web request
                let mut state = state_original.lock().unwrap();

                match trending_music {
                    Ok(data) => {
                        state.help = "Press ?";
                        state.musicbar = VecDeque::from(Vec::from(data));
                        state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] = Some(page);
                    }
                    Err(e) => {
                        state.playlistbar = VecDeque::new();
                        match e {
                            fetcher::ReturnAction::EOR => {
                                state.help = "Result end..";
                                state.fetched_page[ui::event::MIDDLE_PLAYLIST_INDEX] = None;
                            }
                            fetcher::ReturnAction::Failed => {
                                state.help = "Fetch error..";
                                state.fetched_page[ui::event::MIDDLE_PLAYLIST_INDEX] = None;
                            }
                            fetcher::ReturnAction::Retry => {
                                state.help = "temp error..";
                                /* TODO: Retry */
                            }
                        }
                    }
                }
                state.to_fetch = ui::FillFetch::None;
                state.active = ui::Window::Musicbar;
                notifier.notify_all();
            }
            ui::FillFetch::Playlist => {
                let state = state_original.lock().unwrap();
                if let ui::MusicbarSource::Playlist(playlist_id) = state.filled_source.0.clone() {
                    let page =
                        state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX].unwrap_or_default();
                    std::mem::drop(state); // Always free the lock before sending web request

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
                    state.to_fetch = ui::FillFetch::None;
                    state.active = ui::Window::Musicbar;
                    notifier.notify_all();
                } else {
                    // This is hard error. Instead of panicing(which leaves terminal in bad state)
                    // directly print the info in screen. This will distrub the ui
                    eprintln!("Tried to fetch playlist content but playlist id is unsupported");
                }
            }
            ui::FillFetch::Search(query, [m_page, p_page, a_page]) => {
                if let Some(m_page) = m_page {
                    search_request!(
                        query,
                        m_page,
                        state_original,
                        ui::event::MIDDLE_MUSIC_INDEX,
                        state_original.lock().unwrap().musicbar,
                        &mut fetcher,
                        fetcher::Fetcher::search_music
                    );
                }
                if let Some(p_page) = p_page {
                    search_request!(
                        query,
                        p_page,
                        state_original,
                        ui::event::MIDDLE_PLAYLIST_INDEX,
                        state_original.lock().unwrap().playlistbar,
                        &mut fetcher,
                        fetcher::Fetcher::search_playlist
                    );
                }

                if let Some(a_page) = a_page {
                    search_request!(
                        query,
                        a_page,
                        state_original,
                        ui::event::MIDDLE_ARTIST_INDEX,
                        state_original.lock().unwrap().artistbar,
                        &mut fetcher,
                        fetcher::Fetcher::search_artist
                    );
                }

                state_original.lock().unwrap().to_fetch = ui::FillFetch::None;
                notifier.notify_all();
            }
        }
    }
}
