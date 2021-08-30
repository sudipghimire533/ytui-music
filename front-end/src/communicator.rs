use crate::ui;
use fetcher;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use tokio;

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
                        match e {
                            fetcher::ReturnAction::EOR => state.help = "Trending EOR..",
                            fetcher::ReturnAction::Failed => state.help = "Fetch error..",
                            fetcher::ReturnAction::Retry => { /* TODO: retry */ }
                        }
                        state.musicbar = VecDeque::new();
                        state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] = None;
                    }
                }
                state.to_fetch = ui::FillFetch::None;
                state.active = ui::Window::Musicbar;
                notifier.notify_all();
            }
            ui::FillFetch::Search(query, [m_page, p_page, a_page]) => {
                if let Some(m_page) = m_page {
                    let res_music = fetcher.search_music(query.as_str(), m_page).await;
                    let mut state = state_original.lock().unwrap();

                    match res_music {
                        Ok(data) => {
                            state.help = "Press ?";
                            state.musicbar = VecDeque::from(data);
                            state.active = ui::Window::Musicbar;
                            state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] =
                                Some(m_page as usize);
                        }
                        Err(e) => {
                            match e {
                                fetcher::ReturnAction::Failed => state.help = "Search error..",
                                fetcher::ReturnAction::EOR => state.help = "Search EOR..",
                                fetcher::ReturnAction::Retry => { /* TODO: retry */ }
                            }
                            state.musicbar = VecDeque::new();
                            state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] = None;
                        }
                    }
                }

                let mut state = state_original.lock().unwrap();
                state.to_fetch = ui::FillFetch::None;
                notifier.notify_all();
            }
        }
    }
}
