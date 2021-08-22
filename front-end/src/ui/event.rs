use crate::test_backend;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::collections::VecDeque;
use std::{
    convert::TryFrom,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};
use tokio;

const MIDDLE_MUSIC_INDEX: usize = 0;
const MIDDLE_PLAYLIST_INDEX: usize = 0;
const MIDDLE_ARTIST_INDEX: usize = 0;
const SEARCH_SH_KEY: char = '/';
const HELP_SH_KEY: char = '?';
const NEXT_SH_KEY: char = 'n';
const PREV_SH_KEY: char = 'p';
const QUIT_SH_KEY: char = 'q';
const SEEK_F_KEY: char = '>';
const SEEK_B_KEY: char = '<';
const TOGGLE_PAUSE_KEY: char = ' ';
const REFRESH_RATE: u64 = 950;

enum HeadTo {
    Initial,
    Next,
    Prev,
}
fn advance_index(current: usize, limit: usize, direction: HeadTo) -> usize {
    match direction {
        HeadTo::Next => (current + 1) % limit,
        HeadTo::Prev => current.checked_sub(1).unwrap_or(limit - 1) % limit,
        HeadTo::Initial => current,
    }
}

fn advance_list<T>(list: &mut VecDeque<T>, direction: HeadTo) -> bool {
    if list.is_empty() {
        return false;
    }
    match direction {
        HeadTo::Next => list.rotate_left(1),
        HeadTo::Prev => list.rotate_right(1),
        HeadTo::Initial => return false,
    }
    true
}
macro_rules! drop_and_call {
    ($state: expr, $callback: expr) => {{
        std::mem::drop($state);
        $callback()
    }};
    ($state: expr, $callback: expr, $($args: expr)*) => {{
        std::mem::drop($state);
        $callback( $($args)* )
    }};
}

macro_rules! fill_core {
    ("@internal-update-music", $res: expr, $state: expr, $notifier: expr) => {
        let mut state = $state.lock().unwrap();
        state.musicbar = $res;
        state.active = ui::Window::Musicbar;
        state.help = "Press ?";
        $notifier.notify_one();
    };
    ("@internal-update-artist", $res: expr, $state: expr, $notifier: expr) => {
        let mut state = $state.lock().unwrap();
        state.artistbar = $res;
        state.active = ui::Window::Artistbar;
        state.help = "Press ?";
        $notifier.notify_one();
    };
    ($direction: expr, $st_index: expr, $callback: path, $state: expr, $notifier: expr) => {{
        let mut state = $state.lock().unwrap();
        state.help = "Loading...";
        $notifier.notify_one();

        let fetch_page = match state.fetched_page[$st_index] {
            None => 0,
            Some(prev) => match $direction {
                HeadTo::Next => prev + 1,
                HeadTo::Prev if prev > 0 => prev - 1,
                _ => 0,
            },
        };
        let res = match $callback(fetch_page) {
            Some(data) => {
                state.fetched_page[$st_index] = Some(fetch_page);
                ::std::collections::VecDeque::from(data)
            }
            None => {
                state.fetched_page[$st_index] = None;
                ::std::collections::VecDeque::new()
            }
        };
        res
    }};
}

macro_rules! fill {
    ("trending music", $direction: expr, $state: expr, $notifier: expr) => {{
        let res = fill_core!(
            $direction,
            MIDDLE_MUSIC_INDEX,
            test_backend::get_trending_music,
            $state,
            $notifier
        );
        fill_core!("@internal-update-music", res, $state, $notifier);
    }};
    ("community music", $direction: expr, $state: expr, $notifier: expr) => {{
        let res = fill_core!(
            $direction,
            MIDDLE_MUSIC_INDEX,
            test_backend::get_community_music,
            $state,
            $notifier
        );
        fill_core!("@internal-update-music", res, $state, $notifier);
    }};
    ("recents music", $direction: expr, $state: expr, $notifier: expr) => {{
        let res = fill_core!(
            $direction,
            MIDDLE_MUSIC_INDEX,
            test_backend::get_recents_music,
            $state,
            $notifier
        );
        fill_core!("@internal-update-music", res, $state, $notifier);
    }};
    ("favourates music", $direction: expr, $state: expr, $notifier: expr) => {
        let res = fill_core!(
            $direction,
            MIDDLE_MUSIC_INDEX,
            test_backend::get_favourates_music,
            $state,
            $notifier
        );
        fill_core!("@internal-update-music", res, $state, $notifier);
    };
    ("following artist", $direction: expr, $state: expr, $notifier: expr) => {
        let res = fill_core!(
            $direction,
            MIDDLE_ARTIST_INDEX,
            test_backend::get_following_artist,
            $state,
            $notifier
        );
        fill_core!("@internal-update-artist", res, $state, $notifier);
    };
}

pub fn event_sender(state_original: &mut Arc<Mutex<ui::State>>, notifier: &mut Arc<Condvar>) {
    let advance_sidebar = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let current = state.sidebar.selected().unwrap_or_default();
        state.sidebar.select(Some(advance_index(
            current,
            ui::utils::SIDEBAR_LIST_COUNT,
            direction,
        )));
        notifier.notify_one();
    };
    let advance_music_list = |move_down: HeadTo| {
        if advance_list(&mut state_original.lock().unwrap().musicbar, move_down) {
            notifier.notify_one();
        }
    };
    let advance_artist_list = |move_down: HeadTo| {
        if advance_list(&mut state_original.lock().unwrap().artistbar, move_down) {
            notifier.notify_one();
        }
    };
    let advance_playlist_list = |move_down: HeadTo| {
        if advance_list(&mut state_original.lock().unwrap().playlistbar, move_down) {
            notifier.notify_one();
        }
    };
    let quit = || {
        // setting active window to None is to quit
        state_original.lock().unwrap().active = ui::Window::None;
        notifier.notify_one();
    };
    let moveto_next_window = || {
        let mut state = state_original.lock().unwrap();
        state.active = state.active.next();
        notifier.notify_one();
    };
    let moveto_prev_window = || {
        let mut state = state_original.lock().unwrap();
        state.active = state.active.prev();
        notifier.notify_one();
    };
    let handle_esc = || {
        let mut state = state_original.lock().unwrap();
        if state.active == ui::Window::Searchbar {
            state.search.clear();
            drop_and_call!(state, moveto_next_window);
        }
    };
    let handle_backspace = || {
        let mut state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Searchbar => {
                state.search.pop();
                notifier.notify_one();
            }
            _ => drop_and_call!(state, moveto_prev_window),
        }
    };
    let handle_search_input = |ch| {
        state_original.lock().unwrap().search.push(ch);
        notifier.notify_one();
    };
    let activate_search = || {
        state_original.lock().unwrap().active = ui::Window::Searchbar;
        notifier.notify_one();
    };
    let show_help = || {
        todo!();
    };
    let handle_up = || {
        let state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Sidebar => drop_and_call!(state, advance_sidebar, HeadTo::Prev),
            ui::Window::Musicbar => drop_and_call!(state, advance_music_list, HeadTo::Prev),
            ui::Window::Playlistbar => drop_and_call!(state, advance_playlist_list, HeadTo::Prev),
            ui::Window::Artistbar => drop_and_call!(state, advance_artist_list, HeadTo::Prev),
            _ => drop_and_call!(state, moveto_prev_window),
        }
    };
    let handle_down = || {
        let state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Sidebar => drop_and_call!(state, advance_sidebar, HeadTo::Next),
            ui::Window::Musicbar => drop_and_call!(state, advance_music_list, HeadTo::Next),
            ui::Window::Playlistbar => drop_and_call!(state, advance_playlist_list, HeadTo::Next),
            ui::Window::Artistbar => drop_and_call!(state, advance_artist_list, HeadTo::Next),
            _ => drop_and_call!(state, moveto_next_window),
        }
    };

    let fill_trending_music = |direction: HeadTo| {
        fill!("trending music", direction, state_original, notifier);
    };
    let fill_community_music = |direction: HeadTo| {
        fill!("community music", direction, state_original, notifier);
    };
    let fill_recents_music = |direction: HeadTo| {
        fill!("recents music", direction, state_original, notifier);
    };
    let fill_favourates_music = |direction: HeadTo| {
        fill!("favourates music", direction, state_original, notifier);
    };
    let fill_following_artist = |direction: HeadTo| {
        fill!("following artist", direction, state_original, notifier);
    };
    let handle_play_advance = |direction: HeadTo| {
        advance_music_list(direction);
        state_original
            .lock()
            .unwrap()
            .play_first_of_musicbar(notifier);
    };
    let handle_page_nav = |direction: HeadTo| {
        let state = state_original.lock().unwrap();
        match state.active {
            ui::Window::Musicbar => {
                match ui::SidebarOption::try_from(state.sidebar.selected().unwrap()).unwrap() {
                    ui::SidebarOption::Trending => {
                        drop_and_call!(state, fill_trending_music, direction);
                    }
                    ui::SidebarOption::YoutubeCommunity => {
                        drop_and_call!(state, fill_community_music, direction);
                    }
                    ui::SidebarOption::RecentlyPlayed => {
                        drop_and_call!(state, fill_recents_music, direction);
                    }
                    ui::SidebarOption::Favourates => {
                        drop_and_call!(state, fill_favourates_music, direction);
                    }
                    _ => {}
                }
            }
            ui::Window::Artistbar => {
                match ui::SidebarOption::try_from(state.sidebar.selected().unwrap()).unwrap() {
                    ui::SidebarOption::Followings => {
                        drop_and_call!(state, fill_following_artist, direction);
                    }
                    _ => {}
                }
            }
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
                std::mem::drop(state);
                match side_select {
                    ui::SidebarOption::Trending => fill_trending_music(HeadTo::Initial),
                    ui::SidebarOption::YoutubeCommunity => fill_community_music(HeadTo::Initial),
                    ui::SidebarOption::Favourates => fill_favourates_music(HeadTo::Initial),
                    ui::SidebarOption::Followings => fill_following_artist(HeadTo::Initial),
                    ui::SidebarOption::RecentlyPlayed => fill_recents_music(HeadTo::Initial),
                    _ => {}
                }
            }
            ui::Window::Musicbar => {
                let music_under_selection = !state.musicbar.is_empty();
                if music_under_selection {
                    state.play_first_of_musicbar(&notifier);
                }
            }
            _ => {}
        }
    };

    let listener_looop = || loop {
        async {
            if event::poll(Duration::from_millis(REFRESH_RATE)).unwrap() {
                match event::read().unwrap() {
                    Event::Key(key) => match key.code {
                        KeyCode::Down | KeyCode::PageDown => {
                            handle_down();
                        }
                        KeyCode::Up | KeyCode::PageUp => {
                            handle_up();
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
                            /* Handle single character key shortcut as it is not in input */
                            else if ch == SEARCH_SH_KEY {
                                activate_search();
                            } else if ch == HELP_SH_KEY {
                                show_help();
                            } else if ch == QUIT_SH_KEY {
                                quit();
                                return;
                            } else if ch == NEXT_SH_KEY {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    handle_play_advance(HeadTo::Next);
                                } else {
                                    handle_page_nav(HeadTo::Next);
                                }
                            } else if ch == PREV_SH_KEY {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    handle_play_advance(HeadTo::Prev);
                                } else {
                                    handle_page_nav(HeadTo::Prev);
                                }
                            } else if ch == SEEK_F_KEY {
                            } else if ch == SEEK_B_KEY {
                            } else if ch == TOGGLE_PAUSE_KEY {
                                state_original.lock().unwrap().toggle_pause(notifier);
                            }
                        }
                        _ => {}
                    },
                    Event::Resize(..) => {
                        // just update the layout
                        notifier.notify_one();
                    }
                    Event::Mouse(..) => {}
                }
            } else {
                if state_original.lock().unwrap().refresh_time_elapsed() {
                    notifier.notify_one();
                }
            }
        }
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            listener_looop().await;
        });
}
