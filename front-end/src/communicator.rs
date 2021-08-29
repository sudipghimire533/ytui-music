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
            ui::FillFetch::None | ui::FillFetch::Filled => {}
            ui::FillFetch::Trending(page) => {
                state_original.lock().unwrap().help = "Fetching...";
                let trending_music = fetcher.get_trending_music(page).await;
                let mut state = state_original.lock().unwrap();

                match trending_music {
                    Ok(data) => {
                        state.musicbar = VecDeque::from(Vec::from(data));
                        state.to_fetch = ui::FillFetch::Filled;
                        notifier.notify_all();
                    }
                    Err(e) => {}
                }

                state.help = "Press ?";
            }
            ui::FillFetch::Search(m_page, p_page, a_page) => {}
        }
    }
}
