use crate::{
    ui::{
        self,
        event::{MIDDLE_ARTIST_INDEX, MIDDLE_MUSIC_INDEX, MIDDLE_PLAYLIST_INDEX},
    },
    CONFIG,
};
use std::sync::{Arc, Condvar, Mutex};

macro_rules! handle_response {
    ($response: expr, $state_original: expr, $win_index: expr, $target: ident) => {{
        let mut state = $state_original.lock().unwrap();
        // return the boolean which is only truw when response is RETRY
        let mut need_retry = false;
        match $response {
            Ok(data) => {
                state.status = "Sucess..";
                state.$target.0 = data;
            }
            Err(e) => {
                match e {
                    fetcher::ReturnAction::Failed => {
                        state.status = "Fetch error..";
                    }
                    fetcher::ReturnAction::EOR => {
                        state.status = "Result end..";
                        // TODO: Setting this to None means that the next page will always be 0.
                        // That being said when user tries to navigate to previous page after seeing
                        // EOR then still the fetched page will be 0. i.e again started from beginning.
                        // This is desirable when user tries to navigate to next page but is
                        // undesiriable when user tries to navigate to previous page. For now, I can't
                        // think of any workaround except really messing around with fetched_page be
                        // changing the data type (may be new struct storing maximum page before EOR)
                        // and manipulating accordingly. But I have no intention to do so. So this todo
                        // message will be left todo forever
                        // -- END todo --
                        // Setting this to None means that in next iteration condition
                        // state.fetched_page[] != prev_<>_page wil be true but as for the whole if
                        // statement to be true the page should be Some value. That means when EOR is
                        // reched the list will be empty until next iteration. If the if condition
                        // wouldn't have .is_some() check then in next iteration then zeroth page will
                        // be fetched which is undesirable because it confuses weather it is reallt the
                        // next page or zeroth page after EOR
                        state.fetched_page[$win_index] = None;
                    }
                    fetcher::ReturnAction::Retry => {
                        // the respective function from which the data is exptracted
                        // specify the no of times to retry. Simple rerun the loop if retry is feasible
                        state.status = "Retrying..";
                        need_retry = true;
                    }
                }
            }
        }
        std::mem::drop(state);
        need_retry
    }};
}

pub async fn communicator<'st, 'nt>(
    state_original: &'st mut Arc<Mutex<ui::State<'_>>>,
    notifier: &'nt mut Arc<Condvar>,
) {
    let mut fetcher = fetcher::Fetcher::new(&CONFIG.servers.list, &CONFIG.constants.region);

    // variables with prev_ suffex are to be compared with respective current variables from state.
    // This is to check weather anything have changed from previous data request from user so that
    // further request are made or not
    let (mut prev_musicbar_source, mut prev_playlistbar_source, mut prev_artistbar_source) = {
        // Initilization is done inside seperate scope so that this state variable is not visible
        // anywhere after that. It helps my autocomplete in editor
        let state = state_original.lock().unwrap();
        (
            state.filled_source.0.clone(),
            state.filled_source.1.clone(),
            state.filled_source.2.clone(),
        )
    };
    let mut prev_music_page: Option<usize> = None;
    let mut prev_playlist_page: Option<usize> = None;
    let mut prev_artist_page: Option<usize> = None;
    // set these booleans to true when request handeling failed with RETREY response. if this is
    // true then other condition should not have to be true
    let mut need_retry = [false; 3];

    'communicator_loop: loop {
        let mut state = notifier.wait(state_original.lock().unwrap()).unwrap();
        if state.active == ui::Window::None {
            break 'communicator_loop;
        }

        // This block is executed when the source of playlist has changed from previous iteration
        // or new page is requested from the same source. Same pattern is repeated to fill musicbar
        // amd artistbar too.
        // As this statements are executed in loop in short time, creating the condition variable
        // seperatly is worth sacrificing. Anyway It is true when one of below 3 condition are met:
        // 1) the source of to fill playlist is different. i.e in previous loop data was shown from
        //    search and now is needed to fetch the result of trending or seperate search query.
        //    See PlaylistbarSource in ui/mod.rs
        // 2) corresponsing retry is set to true
        // 3) or the source is same but the different page is requested. An extra condition is
        //    added to ensure that it is requesting at least Some page not nothing. eg: when EOR is
        //    reached fetched_page is set to None and for None there is nothing to fetch. See EOR
        //    condition in handle_response! macro
        // UGH!! this if statement condition check is too ugly. I hate it
        if state.filled_source.1 != prev_playlistbar_source
            || need_retry[MIDDLE_PLAYLIST_INDEX]
            || (state.fetched_page[MIDDLE_PLAYLIST_INDEX] != prev_playlist_page
                && state.fetched_page[MIDDLE_PLAYLIST_INDEX].is_some())
        {
            // clear the target so that noone gets confused if it the response from previous or
            // current request
            state.playlistbar.0.clear();

            // condition of if made sure that fetched_page[MIDDLE_PLAYLIST_INDEX] is Some vlaue so
            // unwrapping it is safe.
            let page = state.fetched_page[MIDDLE_PLAYLIST_INDEX].unwrap();

            // Save this source as previous source for next iteration
            prev_playlistbar_source = state.filled_source.1.clone();
            prev_playlist_page = Some(page);

            // early drop the state so ui is not blocked while this thread send web req. See: else
            // block documentation
            std::mem::drop(state);

            // This is the variable from which the response from matching source is set and later
            // handled with handle_response! macro
            let playlist_content;

            // At this point state.filled.source.1 and prev_playlistbar_source is same. As state is
            // already dropped we cant match state.filled.source.1 so match this
            match prev_playlistbar_source {
                ui::PlaylistbarSource::Search(ref term) => {
                    playlist_content = fetcher.search_playlist(term, page).await;
                }
                ui::PlaylistbarSource::Artist(ref artist_id) => {
                    playlist_content = fetcher.get_playlist_of_channel(artist_id, page).await;
                }
                ui::PlaylistbarSource::Favourates | ui::PlaylistbarSource::RecentlyPlayed => {
                    // TODO
                    playlist_content = Ok(Vec::new());
                }
            }

            // if return action is RETRY set so in need_retry so that nex interation will try again
            let retry = handle_response!(
                playlist_content,
                state_original,
                MIDDLE_PLAYLIST_INDEX,
                playlistbar
            );
            need_retry[MIDDLE_PLAYLIST_INDEX] = retry;
            state_original.lock().unwrap().active = ui::Window::Playlistbar;
            notifier.notify_all();
        } else {
            // State is always unlocked in above block and dropped in if block. But when if block
            // condition is not met then the state will never be unlocked so always drop the state.
            // Note, insetad of dropping state inside and outside if statement one could think to
            // drop it outside the if/else block so single drop statement will work. But when if
            // block is executed web request is made and when state is dropped outside if/else
            // block then the state will remain locked until web request is made which may even
            // take indefinite time so which in turn keeps the ui blocked state is dropped early
            // in if block
            std::mem::drop(state);
        }

        // Checks and fills the artistbar.
        let mut state = state_original.lock().unwrap();
        if state.filled_source.2 != prev_artistbar_source
            || need_retry[MIDDLE_ARTIST_INDEX]
            || (state.fetched_page[MIDDLE_ARTIST_INDEX] != prev_artist_page
                && state.fetched_page[MIDDLE_ARTIST_INDEX].is_some())
        {
            state.artistbar.0.clear();
            let page = state.fetched_page[MIDDLE_ARTIST_INDEX].unwrap();
            prev_artistbar_source = state.filled_source.2.clone();
            prev_artist_page = Some(page);
            std::mem::drop(state);

            let artist_content;
            match prev_artistbar_source {
                ui::ArtistbarSource::Search(ref term) => {
                    artist_content = fetcher.search_artist(term, page).await;
                }
                ui::ArtistbarSource::RecentlyPlayed | ui::ArtistbarSource::Favourates => {
                    // TODO:
                    artist_content = Ok(Vec::new());
                }
            }

            let retry = handle_response!(
                artist_content,
                state_original,
                MIDDLE_ARTIST_INDEX,
                artistbar
            );
            need_retry[MIDDLE_ARTIST_INDEX] = retry;
            state_original.lock().unwrap().active = ui::Window::Artistbar;
            notifier.notify_all();
        } else {
            std::mem::drop(state);
        }

        // Checks and fills the musicbar
        let mut state = state_original.lock().unwrap();
        if state.filled_source.0 != prev_musicbar_source
            || need_retry[MIDDLE_MUSIC_INDEX]
            || (state.fetched_page[MIDDLE_MUSIC_INDEX] != prev_music_page
                && state.fetched_page[MIDDLE_MUSIC_INDEX].is_some())
        {
            state.musicbar.0.clear();
            let page = state.fetched_page[MIDDLE_MUSIC_INDEX].unwrap();
            prev_musicbar_source = state.filled_source.0.clone();
            prev_music_page = Some(page);
            std::mem::drop(state);
            // prev_musicbar_source and current musicbar_source are equal at this point
            let music_content;
            match prev_musicbar_source {
                ui::MusicbarSource::Trending => {
                    music_content = fetcher.get_trending_music(page).await;
                }
                ui::MusicbarSource::Search(ref term) => {
                    music_content = fetcher.search_music(term, page).await;
                }
                ui::MusicbarSource::Playlist(ref playlist_id) => {
                    music_content = fetcher.get_playlist_content(playlist_id, page).await;
                }
                ui::MusicbarSource::Artist(ref artist_id) => {
                    music_content = fetcher.get_videos_of_channel(artist_id, page).await;
                }
                ui::MusicbarSource::Favourates | ui::MusicbarSource::RecentlyPlayed => {
                    // TODO: handle each variant with accurate function
                    music_content = Ok(Vec::new());
                }
            }

            let retry =
                handle_response!(music_content, state_original, MIDDLE_MUSIC_INDEX, musicbar);
            need_retry[MIDDLE_MUSIC_INDEX] = retry;
            state_original.lock().unwrap().active = ui::Window::Musicbar;
            notifier.notify_all();
        } else {
            // If above if block is not executed state lock should however be released
            // so that state can be lock again for following if block
            std::mem::drop(state);
        }
    }
}
