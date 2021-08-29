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
        let state = notifier.wait(state_original.lock().unwrap()).unwrap();
        if state.active == ui::Window::None {
            break 'communicator_loop;
        }
        let to_fetch = state.to_fetch.clone();

        // Now the fetch may take some time
        // so we should not lock the state so can ui keeps responding
        std::mem::drop(state);

        match to_fetch {
            ui::FillFetch::None => {}
            ui::FillFetch::Trending(page) => {
                state_original.lock().unwrap().help = "Fetching...";
                notifier.notify_all();
                let trending_music = fetcher.get_trending_music(page).await;
                let mut state = state_original.lock().unwrap();

                match trending_music {
                    Ok(data) => {
                        state.musicbar = VecDeque::from(Vec::from(data));
                        state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] = Some(page);
                    }
                    Err(_e) => {
                        state.musicbar = VecDeque::new();
                        state.fetched_page[ui::event::MIDDLE_MUSIC_INDEX] = None;
                    }
                }
                state.to_fetch = ui::FillFetch::None;
                state.active = ui::Window::Musicbar;
                state.help = "Press ?";
                notifier.notify_all();
            }
            ui::FillFetch::Search(m_page, p_page, a_page) => {}
        }
    }
}
