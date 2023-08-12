use crate::ui::{self, utils::ExtendMpv};
use config::initilize::{CONFIG, STORAGE};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::{
    convert::TryFrom,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

pub const MIDDLE_MUSIC_INDEX: usize = 0;
pub const MIDDLE_PLAYLIST_INDEX: usize = 1;
pub const MIDDLE_ARTIST_INDEX: usize = 2;

#[derive(Clone)]
enum HeadTo {
    Initial,
    Next,
    Prev,
}

// Helper function to return the index of something depending the current position and direction to
// move to
fn advance_index(current: usize, limit: usize, direction: HeadTo) -> usize {
    // This means that the list is empty.
    if limit == 0 {
        return 0;
    }
    match direction {
        HeadTo::Next => (current + 1) % limit,
        HeadTo::Prev => current.checked_sub(1).unwrap_or(limit - 1) % limit,
        HeadTo::Initial => current,
    }
}

// Helper function to drop the first paramater and call the function in second paramater and
// optional arguments provided in later arguments
// This is used to drop the state and call the function as such pattern is found redundant while
// calling event handeling closure where unlocked state needs to be droppped before calling the
// corresponding handler
macro_rules! drop_and_call {
    // This will call the function in passe in second argument
    // passed function will not accept any argument
    ($state: expr, $callback: expr) => {{
        std::mem::drop($state);
        $callback()
    }};
    // This will call the function recived in second argument and pass the later arguments as that
    // function paramater
    ($state: expr, $callback: expr, $($args: expr)*) => {{
        std::mem::drop($state);
        $callback( $($args)* )
    }};
}

// Heklper function to get the next page depending on the current page and direction to move
// This was mainly created to fetch the next page of the musicbar/playlist bar when user
// hits NEXT_SH_KEY or PREV_SH_KEY
#[inline]
fn get_page(current: &Option<usize>, direction: HeadTo) -> usize {
    let page = match current {
        None => 0,
        Some(prev) => match direction {
            HeadTo::Initial => 0,
            HeadTo::Next => prev + 1,
            HeadTo::Prev => prev.checked_sub(1).unwrap_or_default(),
        },
    };
    page as usize
}

/*
* The event_sender function is running in it's own seperate thread.
* -> A loop is initilized where it waits for any event to happen (keypress and resize for now)
* and call the corresponding closure to handle event.
* -> Inside every closure state that are dependent to this event is checked. eg: checks active
* window shile handleing left/right direction key
* -> To fetch data, required data paramater is set in a state variable which is shared across all
* the threads. And another loop is ran in communicator.rs where it wait checks weather anything
* should be filled from diffrenet source.
*/
pub async fn event_sender(
    state_original: &mut Arc<Mutex<ui::State<'_>>>,
    notifier: &mut Arc<Condvar>,
) {
    // Some predefined source
    let youtube_community_channels = vec![fetcher::ArtistUnit {
        name: "Youtube Music Global Charts".to_string(),
        id: "UCrKZcyOJVWnJ60zM1XWllNw".to_string(),
        video_count: "NaN".to_string(),
    }];

    let download_counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

    // There is several option in sidebar like trending/ favourates,
    // this handler will change the selected option from sidebar depending on the direction user
    // move (Up or DOwn).
    let advance_sidebar = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let current = state.sidebar.selected().unwrap_or_default();
        state.sidebar.select(Some(advance_index(
            current,
            ui::utils::SIDEBAR_LIST_COUNT,
            direction,
        )));
        notifier.notify_all();
    };

    // select the next or previous element in musicbar list. This is done simply by setting the
    // correct index in corresponding TableState
    let advance_music_list = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let next_index;
        match state.musicbar.1.selected() {
            None => next_index = 0,
            Some(current) => {
                next_index = advance_index(current, state.musicbar.0.len(), direction);
            }
        }
        state.musicbar.1.select(Some(next_index));
        notifier.notify_all();
    };

    // simialr to advance_music_list but instead rotate data in `playlistbar` variable of state
    let advance_playlist_list = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let next_index;
        match state.playlistbar.1.selected() {
            None => next_index = 0,
            Some(current) => {
                next_index = advance_index(current, state.playlistbar.0.len(), direction);
            }
        }
        state.playlistbar.1.select(Some(next_index));
        notifier.notify_all();
    };

    // simialr to advance_playlist_list but instead rotate data in `artistbar` variable of state
    let advance_artist_list = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        // if the list is empty then do nothing else.
        // It is necessary to return instantly otherwise the next_index will get value 0 and this
        // closure will endup doing select(Some(0)) to the empty list
        let next_index;
        match state.artistbar.1.selected() {
            None => next_index = 0,
            Some(current) => {
                next_index = advance_index(current, state.artistbar.0.len(), direction);
            }
        }
        state.artistbar.1.select(Some(next_index));
        notifier.notify_all();
    };

    // When active window is set to NONE, it means user had requested to quit the application,
    // This handle will fire when user hits QUIT_SH_KEY
    // Before breaking the loop which this function is running on
    // this closure will simply set the active_window (`active`) to None so that functions in other
    // thread can also respond to the event (which is usally again breking the running loop in
    // thread)
    let quit = |force_quit: bool| -> bool {
        let mut state = state_original.lock().unwrap();
        // Do not quit when some download is in progress as it may leave partial file on the disk.
        // If it is urgent required to quit the application user should also press ALT key along
        // with CTRL and QUIT key
        if !force_quit && *download_counter.lock().unwrap() > 0 {
            state.active = ui::Window::Popup(
                "Error",
                "Some download are in progress. Press this shortcut with ALT key to force quit"
                    .to_string(),
            );
            return false;
        }

        // setting active window to None is to quit
        state.active = ui::Window::None;
        // Also make sure databse is flushed.
        if let Err(err) = STORAGE.lock().unwrap().cache_flush() {
            eprintln!("Cannot flush the storage db. Error: {err}", err = err);
        }

        notifier.notify_all();
        true
    };

    // This handler will fire up when user request to move between sections like musicbar, sidebar
    // etc. Similar handler moveto_next_window / moveto_prev_window are not merged as these
    // closures as these handlers are frequently called so avoid more branching
    let moveto_next_window = || {
        let mut state = state_original.lock().unwrap();
        state.active = state.active.next();
        notifier.notify_all();
    };

    let moveto_prev_window = || {
        let mut state = state_original.lock().unwrap();
        state.active = state.active.prev();
        notifier.notify_all();
    };

    // This handler is fired when user press ESC key,
    // if searchbar is active clear the content in search bar and move to next window
    // if helpbar is active anway move to sidebar just to hide the help window
    let handle_esc = || {
        let mut state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Searchbar | ui::Window::Popup(..) => {
                state.search.0.clear();
                drop_and_call!(state, moveto_next_window);
            }
            ui::Window::BottomControl => {
                drop_and_call!(state, moveto_next_window);
            }
            ui::Window::Sidebar
            | ui::Window::Musicbar
            | ui::Window::Playlistbar
            | ui::Window::Artistbar => {
                state.active = ui::Window::BottomControl;
                notifier.notify_all();
            }
            ui::Window::None => {
                unreachable!();
            }
        }
    };

    // This handler is fired when user press BACKSPACE key
    // backspace key will pop the last character from search query if pressed from searchbar
    // and if this key is pressed from somewhere else other than searchbar then will simply
    // move to previous window
    let handle_backspace = || {
        let mut state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Searchbar => {
                state.search.0.pop();
                notifier.notify_all();
            }
            _ => drop_and_call!(state, moveto_prev_window),
        }
    };

    // This is fires when user press any character key
    // this will simpley push the recived character in search query term and update state
    // so can the added character becomes visible
    let handle_search_input = |ch| {
        state_original.lock().unwrap().search.0.push(ch);
        notifier.notify_all();
    };

    // This handler is fired when use press SEARCH_SH_KEY
    // this will move the curson to the searchbar from which user can start to type the query
    let activate_search = || {
        let mut state = state_original.lock().unwrap();
        state.active = ui::Window::Searchbar;
        notifier.notify_all();
    };

    // This handler will be fired when user hits UP_ARROW or DOWN_ARROW key
    // UP_ARROW will set the direction to PREV and DOWN_ARROW to NEXT
    // for now, these key will only handle the moving of list
    // So, depending on the window which is currently active, this closure will call
    // the respective handler which will advance the corersponding list
    let handle_up_down = |direction: HeadTo| {
        let state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Sidebar => drop_and_call!(state, advance_sidebar, direction),
            ui::Window::Musicbar => drop_and_call!(state, advance_music_list, direction),
            ui::Window::Playlistbar => drop_and_call!(state, advance_playlist_list, direction),
            ui::Window::Artistbar => drop_and_call!(state, advance_artist_list, direction),
            _ => match direction {
                HeadTo::Next => drop_and_call!(state, moveto_next_window),
                HeadTo::Prev => drop_and_call!(state, moveto_prev_window),
                _ => unreachable!(),
            },
        }
    };

    let start_search = || {
        let mut state = state_original.lock().unwrap();
        let search_term = state.search.0.trim();

        // return instantly if query is empty
        if search_term.is_empty() {
            return;
        }
        // When prefiexed by the string as defined in config only show the specific result type
        // respectively
        else if let Some(0) = search_term.find(&CONFIG.constants.search_by_type[0]) {
            let search_term =
                search_term[CONFIG.constants.search_by_type[0].len() - 1..].to_string();
            state.fetched_page[0] = Some(0);
            state.filled_source.0 = ui::MusicbarSource::Search(search_term);
        } else if let Some(0) = search_term.find(&CONFIG.constants.search_by_type[1]) {
            let search_term =
                search_term[CONFIG.constants.search_by_type[1].len() - 1..].to_string();
            state.fetched_page[1] = Some(0);
            state.filled_source.1 = ui::PlaylistbarSource::Search(search_term);
        } else if let Some(0) = search_term.find(&CONFIG.constants.search_by_type[2]) {
            let search_term =
                search_term[&CONFIG.constants.search_by_type[2].len() - 1..].to_string();
            state.fetched_page[2] = Some(0);
            state.filled_source.2 = ui::ArtistbarSource::Search(search_term);
        }
        // If nothing of the prefix is defined then search for all type
        else {
            let search_term = search_term.to_string();
            state.fetched_page = [Some(0); 3];
            state.filled_source.0 = ui::MusicbarSource::Search(search_term.clone());
            state.filled_source.1 = ui::PlaylistbarSource::Search(search_term.clone());
            state.filled_source.2 = ui::ArtistbarSource::Search(search_term);
        }
        notifier.notify_all();
    };

    let fill_trending_music = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        state.fetched_page[MIDDLE_MUSIC_INDEX] =
            Some(get_page(&state.fetched_page[MIDDLE_MUSIC_INDEX], direction));
        state.filled_source.0 = ui::MusicbarSource::Trending;
        notifier.notify_all();
    };

    let fill_community_source = || {
        let mut state = state_original.lock().unwrap();
        state.artistbar.0 = youtube_community_channels.clone();
        state.active = ui::Window::Artistbar;
        notifier.notify_all();
    };

    let fill_fav_music = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        state.filled_source.0 = ui::MusicbarSource::Favourates;
        let page = get_page(&state.fetched_page[MIDDLE_MUSIC_INDEX], direction);
        state.fetched_page[MIDDLE_MUSIC_INDEX] = Some(page);
        notifier.notify_all();
    };

    let fill_fav_playlist = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        state.filled_source.1 = ui::PlaylistbarSource::Favourates;
        let page = get_page(&state.fetched_page[MIDDLE_PLAYLIST_INDEX], direction);
        state.fetched_page[MIDDLE_PLAYLIST_INDEX] = Some(page);
        notifier.notify_all();
    };

    let fill_fav_artist = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        state.filled_source.2 = ui::ArtistbarSource::Favourates;
        let page = get_page(&state.fetched_page[MIDDLE_ARTIST_INDEX], direction);
        state.fetched_page[MIDDLE_ARTIST_INDEX] = Some(page);
        notifier.notify_all();
    };

    let fill_music_from_playlist = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        if let ui::MusicbarSource::Playlist(playlist_id) = &state.filled_source.0 {
            state.filled_source.0 = ui::MusicbarSource::Playlist(playlist_id.to_string());
            state.fetched_page[MIDDLE_MUSIC_INDEX] =
                Some(get_page(&state.fetched_page[MIDDLE_MUSIC_INDEX], direction));
            notifier.notify_all();
        }
    };

    let fill_music_from_artist = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        if let ui::MusicbarSource::Artist(artist_id) = &state.filled_source.0 {
            state.filled_source.0 = ui::MusicbarSource::Artist(artist_id.to_string());
            state.fetched_page[MIDDLE_MUSIC_INDEX] =
                Some(get_page(&state.fetched_page[MIDDLE_MUSIC_INDEX], direction));
            notifier.notify_all();
        }
    };

    let fill_playlist_from_artist = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        if let ui::PlaylistbarSource::Artist(artist_id) = &state.filled_source.1 {
            state.filled_source.1 = ui::PlaylistbarSource::Artist(artist_id.to_string());
            state.fetched_page[MIDDLE_PLAYLIST_INDEX] = Some(get_page(
                &state.fetched_page[MIDDLE_PLAYLIST_INDEX],
                direction,
            ));
            notifier.notify_all();
        }
    };

    // play next/previous song from queue
    let change_track = |direction: HeadTo| match direction {
        HeadTo::Next => state_original.lock().unwrap().player.play_next(),
        HeadTo::Prev => state_original.lock().unwrap().player.play_prev(),
        HeadTo::Initial => unreachable!(),
    };

    // navigating page is just changing to fetched_page value to next/prev value
    // or changing the prev/next track
    let handle_nav = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let target_index: usize;
        match state.active {
            ui::Window::Musicbar => target_index = MIDDLE_MUSIC_INDEX,
            ui::Window::Playlistbar => target_index = MIDDLE_PLAYLIST_INDEX,
            ui::Window::Artistbar => target_index = MIDDLE_ARTIST_INDEX,
            ui::Window::BottomControl => {
                // On reciving next/prev event when active window is bottom control
                // It implied to change the track
                return drop_and_call!(state, change_track, direction);
            }
            ui::Window::Searchbar | ui::Window::Sidebar | ui::Window::Popup(..) => {
                // If none of above windows are active then nothing to navigate.
                // Early return instead of initilizing `target_index`
                return;
            }
            ui::Window::None => unreachable!(),
        }
        let page = get_page(&state.fetched_page[target_index], direction);
        state.fetched_page[target_index] = Some(page);
        notifier.notify_all();
    };

    let seek_forward = || {
        state_original
            .lock()
            .unwrap()
            .player
            .seek_forward(CONFIG.constants.seek_forward_secs as f64)
            .ok();
        notifier.notify_all();
    };

    let seek_backward = || {
        state_original
            .lock()
            .unwrap()
            .player
            .seek_backward(CONFIG.constants.seek_backward_secs as f64)
            .ok();
        notifier.notify_all();
    };

    let handle_repeat = || {
        let mut state = state_original.lock().unwrap();
        state.player.repeat_nothing();
        if state.playback_behaviour.repeat {
            state.player.repeat_one();
        } else {
            state.player.repeat_playlist();
        }
        state.playback_behaviour.repeat = !state.playback_behaviour.repeat;
        notifier.notify_all();
    };

    let toggle_shuffle = || {
        let mut state = state_original.lock().unwrap();
        if state.playback_behaviour.shuffle {
            state.player.unshuffle();
        } else {
            state.player.shuffle();
        }
        state.playback_behaviour.shuffle = !state.playback_behaviour.shuffle;
        notifier.notify_all();
    };

    let toggle_play = || {
        state_original.lock().unwrap().toggle_pause();
        notifier.notify_all();
    };

    let handle_download = || async {
        let mut state = state_original.lock().unwrap();

        // TODO: Ask for conformation before downloading
        let mut command = tokio::process::Command::new("youtube-dl");
        let download_url;
        if let Some(focused_index) = state.musicbar.1.selected() {
            let music_id = &state.musicbar.0[focused_index].id;
            download_url = format!("https://www.youtube.com/watch?v={}", music_id);
        } else if let Some(focused_index) = state.playlistbar.1.selected() {
            let playlist_id = &state.playlistbar.0[focused_index].id;
            download_url = format!("https://www.youtube.com/playlist?list={}", playlist_id);
        } else {
            return;
        }

        state.status = "Download started..";
        state.active = ui::Window::Popup(
            "Downloading...",
            format!(
                "Download of {} have an eye on your Music folder",
                download_url
            ),
        );
        command.arg(download_url);

        command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .args(&["--extract-audio", "--audio-format", &CONFIG.download.format])
            .current_dir(&CONFIG.download.path)
            .kill_on_drop(false);

        std::mem::drop(state);

        *download_counter.lock().unwrap() += 1;
        let counter_clone = Arc::clone(&download_counter);

        // Wait for 5 second just to make sure that command has finished executing.
        // It usually donot take all those 5 seconds
        // Anyway, download won't finish before 5 seconds
        // Then just wait for command to finish by waiting for exit status
        // decrease the download queue count
        tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            command.status().await.unwrap();
            *counter_clone.lock().unwrap() -= 1;
        });
    };

    // If play is true it means also play the playlist
    // if is false then only expand the playlist and show url but do not play it
    let select_playlist = |play: bool| {
        let mut state = state_original.lock().unwrap();
        if let Some(selected_index) = state.playlistbar.1.selected() {
            let playlist_id = state.playlistbar.0[selected_index].id.clone();
            if play {
                state.activate_playlist(&playlist_id);
            } else {
                let message = format!(
                    "Playlist url: https://youtu.be/playlist?list={}",
                    playlist_id
                );
                state.active = ui::Window::Popup("Info!", message);
            }
            state.filled_source.0 = ui::MusicbarSource::Playlist(playlist_id);
            drop_and_call!(state, fill_music_from_playlist, HeadTo::Initial);
        }
    };

    let select_music = |play: bool| {
        let mut state = state_original.lock().unwrap();
        if let Some(selected_index) = state.musicbar.1.selected() {
            let music_id = &state.musicbar.0[selected_index].id;
            if play {
                let music_id = music_id.clone();
                state.play_music(&music_id);
            } else {
                let message = format!("Music url: https://youtu.be/{}", music_id);
                state.active = ui::Window::Popup("Info!", message);
                notifier.notify_all();
            }
        }
    };

    let change_volume = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();

        let increase_by = match direction {
            HeadTo::Next => CONFIG.constants.volume_step,
            HeadTo::Prev => CONFIG.constants.volume_step * -1,
            HeadTo::Initial => 0,
        };

        let res = state.player.change_volume(increase_by);

        match res {
            Some(vol) => {
                state.playback_behaviour.volume = vol;
            }
            None => {
                state.status = "Volume error..";
            }
        };

        notifier.notify_all();
    };

    let handle_view = || {
        let state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Playlistbar => {
                drop_and_call!(state, select_playlist, false);
            }
            ui::Window::Musicbar => {
                drop_and_call!(state, select_music, false);
            }
            ui::Window::Artistbar => {}
            _ => {}
        }
    };

    let handle_enter = || {
        let mut state = state_original.lock().unwrap();
        let active_window = state.active.clone();
        match active_window {
            ui::Window::Sidebar => {
                let side_select =
                    ui::SidebarOption::try_from(state.sidebar.selected().unwrap()).unwrap();

                match side_select {
                    ui::SidebarOption::Trending => {
                        drop_and_call!(state, fill_trending_music, HeadTo::Initial);
                    }
                    ui::SidebarOption::YoutubeCommunity => {
                        drop_and_call!(state, fill_community_source);
                    }
                    ui::SidebarOption::Liked => {
                        drop_and_call!(state, fill_fav_music, HeadTo::Initial);
                    }
                    ui::SidebarOption::Saved => {
                        drop_and_call!(state, fill_fav_playlist, HeadTo::Initial);
                    }
                    ui::SidebarOption::Following => {
                        drop_and_call!(state, fill_fav_artist, HeadTo::Initial);
                    }
                    ui::SidebarOption::Search => drop_and_call!(state, activate_search),
                }
            }
            ui::Window::Searchbar => {
                drop_and_call!(state, start_search);
            }

            // On enter play the music
            ui::Window::Musicbar => drop_and_call!(state, select_music, true),

            // On enter selection view the playlist content as well as play it
            ui::Window::Playlistbar => drop_and_call!(state, select_playlist, true),

            ui::Window::Artistbar => {
                if let Some(selected_index) = state.artistbar.1.selected() {
                    let artist_id = state.artistbar.0[selected_index].id.clone();
                    state.filled_source.0 = ui::MusicbarSource::Artist(artist_id.clone());
                    state.filled_source.1 = ui::PlaylistbarSource::Artist(artist_id);
                    std::mem::drop(state);
                    fill_music_from_artist(HeadTo::Initial);
                    fill_playlist_from_artist(HeadTo::Initial);
                }
            }
            ui::Window::None | ui::Window::BottomControl | ui::Window::Popup(..) => {}
        }
    };

    let handle_favourates = |add: bool| {
        let mut state = state_original.lock().unwrap();

        state.status = "Processing..";

        match state.active {
            ui::Window::Musicbar => {
                if let Some(selected_index) = state.musicbar.1.selected() {
                    // Why clone? Used unsafe to bypass borrow as immutable and mutable at same time error.
                    // yeah. I am sure this is fine (up until now).
                    let selected_music =
                        &state.musicbar.0[selected_index] as *const fetcher::MusicUnit;
                    if add {
                        state.add_music_to_favourates(unsafe { &*selected_music });
                    } else {
                        state.remove_music_from_favourates(unsafe { &*selected_music });
                    }
                } else {
                    state.status = "Nothing selected..";
                }
            }

            ui::Window::Playlistbar => {
                if let Some(selected_index) = state.playlistbar.1.selected() {
                    let selected_playlist =
                        &state.playlistbar.0[selected_index] as *const fetcher::PlaylistUnit;
                    if add {
                        state.add_playlist_to_favourates(unsafe { &*selected_playlist });
                    } else {
                        state.remove_playlist_from_favourates(unsafe { &*selected_playlist });
                    }
                } else {
                    state.status = "Nothing selected..";
                }
            }

            ui::Window::Artistbar => {
                if let Some(selected_index) = state.artistbar.1.selected() {
                    let selected_artist =
                        &state.artistbar.0[selected_index] as *const fetcher::ArtistUnit;
                    if add {
                        state.add_artist_to_favourates(unsafe { &*selected_artist });
                    } else {
                        state.remove_artist_from_favourates(unsafe { &*selected_artist });
                    }
                } else {
                    state.status = "Nothing selected..";
                }
            }
            _ => {}
        }

        notifier.notify_all();
    };

    'listener_loop: loop {
        if event::poll(Duration::from_millis(CONFIG.constants.refresh_rate)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key) => {
                    let is_with_control = key.modifiers.contains(KeyModifiers::CONTROL);

                    match key.code {
                        KeyCode::Down | KeyCode::PageDown => {
                            handle_up_down(HeadTo::Next);
                        }
                        KeyCode::Up | KeyCode::PageUp => {
                            handle_up_down(HeadTo::Prev);
                        }
                        KeyCode::Right | KeyCode::Tab => {
                            moveto_next_window();
                        }
                        KeyCode::Left | KeyCode::BackTab => {
                            moveto_prev_window();
                        }
                        KeyCode::Esc => {
                            handle_esc();
                        }
                        KeyCode::Enter => {
                            handle_enter();
                        }
                        KeyCode::Backspace | KeyCode::Delete => {
                            handle_backspace();
                        }
                        KeyCode::Char(ch) => {
                            /* If searchbar is active register every char key as input term */
                            if state_original.lock().unwrap().active == ui::Window::Searchbar {
                                handle_search_input(ch);
                            }
                            // Now as this is not the input, call the shortcuts action if this key
                            // is defined in shortcuts
                            else if ch == CONFIG.shortcut_keys.start_search {
                                activate_search();
                            } else if ch == CONFIG.shortcut_keys.toggle_play {
                                toggle_play();
                            } else if ch == CONFIG.shortcut_keys.repeat {
                                handle_repeat();
                            } else if ch == CONFIG.shortcut_keys.suffle {
                                toggle_shuffle();
                            } else if ch == CONFIG.shortcut_keys.forward {
                                seek_forward();
                            } else if ch == CONFIG.shortcut_keys.backward {
                                seek_backward();
                            } else if ch == CONFIG.shortcut_keys.view {
                                handle_view();
                            } else if ch == CONFIG.shortcut_keys.favourates_add {
                                handle_favourates(true);
                            } else if ch == CONFIG.shortcut_keys.favourates_remove {
                                handle_favourates(false);
                            } else if ch == CONFIG.shortcut_keys.prev {
                                if is_with_control {
                                    change_track(HeadTo::Prev);
                                } else {
                                    handle_nav(HeadTo::Prev);
                                }
                            } else if ch == CONFIG.shortcut_keys.next {
                                if is_with_control {
                                    change_track(HeadTo::Next);
                                } else {
                                    handle_nav(HeadTo::Next);
                                }
                            } else if ch == CONFIG.shortcut_keys.download && is_with_control {
                                handle_download().await;
                            } else if ch == CONFIG.shortcut_keys.vol_increase {
                                change_volume(HeadTo::Next);
                            } else if ch == CONFIG.shortcut_keys.vol_decrease {
                                change_volume(HeadTo::Prev);
                            } else if ch == CONFIG.shortcut_keys.quit && is_with_control {
                                let force_quit = key.modifiers.contains(KeyModifiers::ALT);
                                if quit(force_quit) {
                                    break 'listener_loop;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Resize(..) => {
                    // just update the layout
                    notifier.notify_all();
                }
                Event::Mouse(..) => {}
            }
        } else {
            notifier.notify_all();
        }
    }
}
