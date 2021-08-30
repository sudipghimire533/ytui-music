use crate::test_backend;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::collections::VecDeque;
use std::{
    convert::TryFrom,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

pub const MIDDLE_MUSIC_INDEX: usize = 0;
pub const MIDDLE_PLAYLIST_INDEX: usize = 0;
pub const MIDDLE_ARTIST_INDEX: usize = 0;
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

pub fn event_sender(state_original: &mut Arc<Mutex<ui::State>>, notifier: &mut Arc<Condvar>) {
    let advance_sidebar = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let current = state.sidebar.0.selected().unwrap_or_default();
        state.sidebar.0.select(Some(advance_index(
            current,
            ui::utils::SIDEBAR_LIST_COUNT,
            direction,
        )));
        notifier.notify_all();
    };
    let advance_music_list = |move_down: HeadTo| {
        if advance_list(&mut state_original.lock().unwrap().musicbar, move_down) {
            notifier.notify_all();
        }
    };
    let advance_artist_list = |move_down: HeadTo| {
        if advance_list(&mut state_original.lock().unwrap().artistbar, move_down) {
            notifier.notify_all();
        }
    };
    let advance_playlist_list = |move_down: HeadTo| {
        if advance_list(&mut state_original.lock().unwrap().playlistbar, move_down) {
            notifier.notify_all();
        }
    };
    let quit = || {
        // setting active window to None is to quit
        state_original.lock().unwrap().active = ui::Window::None;
        notifier.notify_all();
    };
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
    let handle_esc = || {
        let mut state = state_original.lock().unwrap();
        if state.active == ui::Window::Searchbar {
            state.search.0.clear();
            drop_and_call!(state, moveto_next_window);
        }
    };
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
    let handle_search_input = |ch| {
        state_original.lock().unwrap().search.0.push(ch);
        notifier.notify_all();
    };
    let activate_search = || {
        let mut state = state_original.lock().unwrap();
        state.active = ui::Window::Searchbar;
        // Mark search option to be real active
        // this bring state to same state weather
        // activated from shortcut key or from sidebar
        state.sidebar.1 = ui::SidebarOption::Search;
        notifier.notify_all();
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

    let fill_search_music = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let page = get_page(&state.fetched_page[MIDDLE_MUSIC_INDEX], direction);
        state.to_fetch = ui::FillFetch::Search(state.search.1.clone(), [Some(page), None, None]);
        state.help = "Searching..";
        notifier.notify_all();
    };
    let fill_search_playlist = |direction: HeadTo| {};
    let fill_search_artist = |direction: HeadTo| {};

    let fill_trending_music = |direction: HeadTo| {
        let mut state = state_original.lock().unwrap();
        let page = get_page(&state.fetched_page[MIDDLE_MUSIC_INDEX], direction);
        state.to_fetch = ui::FillFetch::Trending(page);
        state.help = "Fetching..";
        notifier.notify_all();
    };
    let fill_community_music = |direction: HeadTo| {
        //   fill!("community music", direction, state_original, notifier);
    };
    let fill_recents_music = |direction: HeadTo| {
        // fill!("recents music", direction, state_original, notifier);
    };
    let fill_favourates_music = |direction: HeadTo| {
        // fill!("favourates music", direction, state_original, notifier);
    };
    let fill_following_artist = |direction: HeadTo| {
        // fill!("following artist", direction, state_original, notifier);
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
        let active_sidebar = &state.sidebar.1;
        match state.active {
            ui::Window::Musicbar => match active_sidebar {
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
                ui::SidebarOption::Search => {
                    drop_and_call!(state, fill_search_music, direction);
                }
                _ => {}
            },
            ui::Window::Artistbar => match active_sidebar {
                ui::SidebarOption::Followings => {
                    drop_and_call!(state, fill_following_artist, direction);
                }
                _ => {}
            },
            _ => {}
        }
    };
    let handle_enter = || {
        let mut state = state_original.lock().unwrap();
        let active_window = state.active.clone();
        match active_window {
            ui::Window::Sidebar => {
                let side_select =
                    ui::SidebarOption::try_from(state.sidebar.0.selected().unwrap()).unwrap();
                std::mem::drop(state);

                match side_select {
                    ui::SidebarOption::Trending => fill_trending_music(HeadTo::Initial),
                    ui::SidebarOption::YoutubeCommunity => fill_community_music(HeadTo::Initial),
                    ui::SidebarOption::Favourates => fill_favourates_music(HeadTo::Initial),
                    ui::SidebarOption::Followings => fill_following_artist(HeadTo::Initial),
                    ui::SidebarOption::RecentlyPlayed => fill_recents_music(HeadTo::Initial),
                    ui::SidebarOption::Search => activate_search(),
                    _ => {}
                }
                state_original.lock().unwrap().sidebar.1 = side_select;
            }
            ui::Window::Musicbar => {
                let music_under_selection = !state.musicbar.is_empty();
                if music_under_selection {
                    state.play_first_of_musicbar(&notifier);
                }
            }
            ui::Window::Searchbar => {
                state.search.1 = state.search.0.trim().to_string();
                std::mem::drop(state);
                fill_search_playlist(HeadTo::Initial);
                fill_search_artist(HeadTo::Initial);
                fill_search_music(HeadTo::Initial);
            }
            _ => {}
        }
    };

    'listener_loop: loop {
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
                            break 'listener_loop;
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
                    notifier.notify_all();
                }
                Event::Mouse(..) => {}
            }
        } else {
            if state_original.lock().unwrap().refresh_time_elapsed() {
                notifier.notify_all();
            }
        }
    }
}
