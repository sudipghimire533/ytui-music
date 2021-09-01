use crate::ui::{
    self,
    event::{MIDDLE_ARTIST_INDEX, MIDDLE_MUSIC_INDEX, MIDDLE_PLAYLIST_INDEX},
};
use fetcher;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

macro_rules! handle_response {
    ($response: expr, $page: expr, $state_original: expr, $win_index: expr, $target: ident) => {
        let mut state = $state_original.lock().unwrap();
        match $response {
            Ok(data) => {
                state.help = "Press ?";
                state.$target = VecDeque::from(data);
                state.fetched_page[$win_index] = Some($page);
            }
            Err(e) => {
                match e {
                    fetcher::ReturnAction::Failed => {
                        state.help = "Fetch error..";
                        state.fetched_page[$win_index] = None;
                    }
                    fetcher::ReturnAction::EOR => {
                        state.help = "Result end..";
                        state.fetched_page[$win_index] = None;
                    }
                    fetcher::ReturnAction::Retry => {
                        // the respective function from which the data is exptracted
                        // specify the no of times to retry. Simple rerun the loop if retry is feasible
                        state.help = "Retrying..";
                        continue;
                    }
                }
            }
        }
        std::mem::drop(state);
    };
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
            match prev_playlistbar_source {
                ui::PlaylistbarSource::Search(ref term, page) => {
                    let playlists = fetcher.search_playlist(term, page).await;
                    handle_response!(
                        playlists,
                        page,
                        state_original,
                        MIDDLE_PLAYLIST_INDEX,
                        playlistbar
                    );
                }
                ui::PlaylistbarSource::Artist(ref artist_id, page) => {
                    let playlist_content = fetcher.get_playlist_of_channel(&artist_id, page).await;
                    handle_response!(
                        playlist_content,
                        page,
                        state_original,
                        MIDDLE_PLAYLIST_INDEX,
                        playlistbar
                    );
                }
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
            match prev_artistbar_source {
                ui::ArtistbarSource::Search(ref term, page) => {
                    let artists = fetcher.search_artist(term, page).await;
                    handle_response!(
                        artists,
                        page,
                        state_original,
                        MIDDLE_ARTIST_INDEX,
                        artistbar
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
            // prev_musicbar_source and current musicbar_source are equal at this point
            match prev_musicbar_source {
                ui::MusicbarSource::Trending(page) => {
                    let trending_music = fetcher.get_trending_music(page).await;
                    handle_response!(
                        trending_music,
                        page,
                        state_original,
                        MIDDLE_MUSIC_INDEX,
                        musicbar
                    );
                }
                ui::MusicbarSource::Search(ref term, page) => {
                    let musics = fetcher.search_music(term, page).await;
                    handle_response!(musics, page, state_original, MIDDLE_MUSIC_INDEX, musicbar);
                }
                ui::MusicbarSource::Playlist(ref playlist_id, page) => {
                    let music_content = fetcher.get_playlist_content(&playlist_id, page).await;
                    handle_response!(
                        music_content,
                        page,
                        state_original,
                        MIDDLE_MUSIC_INDEX,
                        musicbar
                    );
                }
                ui::MusicbarSource::Artist(ref artist_id, page) => {
                    let music_content = fetcher.get_videos_of_channel(&artist_id, page).await;
                    handle_response!(
                        music_content,
                        page,
                        state_original,
                        MIDDLE_MUSIC_INDEX,
                        musicbar
                    );
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
