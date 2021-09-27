use crate::ui::{
    self,
    event::{MIDDLE_ARTIST_INDEX, MIDDLE_MUSIC_INDEX, MIDDLE_PLAYLIST_INDEX},
};
use fetcher;
use std::sync::{Arc, Condvar, Mutex};

macro_rules! handle_response {
    ($response: expr, $page: expr, $state_original: expr, $win_index: expr, $target: ident) => {
        let mut state = $state_original.lock().unwrap();
        match $response {
            Ok(data) => {
                state.help = "Press ?";
                state.$target.0 = data;
            }
            Err(e) => {
                match e {
                    fetcher::ReturnAction::Failed => {
                        state.help = "Fetch error..";
                    }
                    fetcher::ReturnAction::EOR => {
                        state.help = "Result end..";
                        // again show the result of first page to create cycle
                        state.fetched_page[$win_index] = Some(0);
                    }
                    fetcher::ReturnAction::Retry => {
                        // the respective function from which the data is exptracted
                        // specify the no of times to retry. Simple rerun the loop if retry is feasible
                        // TODO: Refine this.
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
    let (mut prev_music_page, mut prev_playlist_page, mut prev_artist_page): (
        Option<usize>,
        Option<usize>,
        Option<usize>,
    ) = (None, None, None);

    'communicator_loop: loop {
        let mut state = notifier.wait(state_original.lock().unwrap()).unwrap();
        if state.active == ui::Window::None {
            break 'communicator_loop;
        }

        if state.filled_source.1 != prev_playlistbar_source
            || state.fetched_page[MIDDLE_PLAYLIST_INDEX] != prev_playlist_page
        {
            prev_playlistbar_source = state.filled_source.1.clone();
            prev_playlist_page = state.fetched_page[MIDDLE_PLAYLIST_INDEX];
            state.playlistbar.0.clear();
            let page = state.fetched_page[MIDDLE_PLAYLIST_INDEX].unwrap_or_default();
            std::mem::drop(state);
            match prev_playlistbar_source {
                ui::PlaylistbarSource::Search(ref term) => {
                    let playlists = fetcher.search_playlist(term, page).await;
                    handle_response!(
                        playlists,
                        page,
                        state_original,
                        MIDDLE_PLAYLIST_INDEX,
                        playlistbar
                    );
                }
                ui::PlaylistbarSource::Artist(ref artist_id) => {
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
        if state.filled_source.2 != prev_artistbar_source
            || state.fetched_page[MIDDLE_ARTIST_INDEX] != prev_artist_page
        {
            prev_artistbar_source = state.filled_source.2.clone();
            prev_artist_page = state.fetched_page[MIDDLE_ARTIST_INDEX];
            state.artistbar.0.clear();
            let page = state.fetched_page[MIDDLE_ARTIST_INDEX].unwrap_or_default();
            std::mem::drop(state);
            match prev_artistbar_source {
                ui::ArtistbarSource::Search(ref term) => {
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
        if state.filled_source.0 != prev_musicbar_source
            || state.fetched_page[MIDDLE_MUSIC_INDEX] != prev_music_page
        {
            prev_musicbar_source = state.filled_source.0.clone();
            prev_music_page = state.fetched_page[MIDDLE_MUSIC_INDEX];
            state.musicbar.0.clear();
            let page = state.fetched_page[MIDDLE_MUSIC_INDEX].unwrap_or_default();
            std::mem::drop(state);
            // prev_musicbar_source and current musicbar_source are equal at this point
            match prev_musicbar_source {
                ui::MusicbarSource::Trending => {
                    let trending_music = fetcher.get_trending_music(page).await;
                    handle_response!(
                        trending_music,
                        page,
                        state_original,
                        MIDDLE_MUSIC_INDEX,
                        musicbar
                    );
                }
                ui::MusicbarSource::Search(ref term) => {
                    let musics = fetcher.search_music(term, page).await;
                    handle_response!(musics, page, state_original, MIDDLE_MUSIC_INDEX, musicbar);
                }
                ui::MusicbarSource::Playlist(ref playlist_id) => {
                    let music_content = fetcher.get_playlist_content(&playlist_id, page).await;
                    handle_response!(
                        music_content,
                        page,
                        state_original,
                        MIDDLE_MUSIC_INDEX,
                        musicbar
                    );
                }
                ui::MusicbarSource::Artist(ref artist_id) => {
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
