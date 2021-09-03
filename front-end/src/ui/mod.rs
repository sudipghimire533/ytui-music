pub mod event;
mod utils;
use std::sync::Condvar;
use tui::{backend::CrosstermBackend, Terminal};
mod shared_import {
    pub use fetcher;
    pub use libmpv;
    pub use serde::{Deserialize, Serialize};
    pub use std::convert::{From, Into, TryFrom, TryInto};
    pub use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
        time::Duration,
    };
    pub use tui::{
        backend::Backend,
        layout::{self, Alignment, Constraint, Direction, Layout, Rect},
        style::{self, Color, Modifier, Style},
        text::{self, Span, Spans, Text},
        widgets::{
            self, Block, BorderType, Borders, Cell, Gauge, List, ListItem, ListState, Paragraph,
            Row, Table, TableState, Tabs, Widget,
        },
    };
}
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use shared_import::*;

pub struct TopLayout {
    layout: [Rect; 2],
}

pub struct MainLayout {
    sidebar: SideBar,
    middle_section: MiddleLayout,
}

pub struct SideBar {
    layout: Rect,
}

pub struct MiddleLayout {
    layout: Rect,
    bottom: MiddleBottom,
}

pub struct MiddleBottom {
    layout: [Rect; 2],
}

pub struct BottomLayout {
    layout: Rect,
}

#[derive(Default)]
pub struct Position {
    pub search: Rect,
    pub help: Rect,
    pub shortcut: Rect,
    pub music: Rect,
    pub playlist: Rect,
    pub artist: Rect,
    pub controllers: Rect,
}

pub fn draw_ui(state: &mut Arc<Mutex<State>>, cvar: &mut Arc<Condvar>) {
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");
    terminal::enable_raw_mode().expect("Faild to enable raw mode");

    let backed = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backed).expect("Failed to create terminal from backend");

    terminal
        .clear()
        .unwrap_or_else(|_| eprintln!("Failed to clear the terminal"));
    terminal
        .hide_cursor()
        .unwrap_or_else(|_| eprintln!("Failed to hide cursor"));

    let mut paint_ui = || {
        terminal
            .draw(|screen| {
                let mut state_unlocked = state.lock().unwrap();

                let position = Position::caclulate(&screen.size());

                screen.render_widget(TopLayout::get_helpbox(&state_unlocked), position.help);
                screen.render_widget(TopLayout::get_searchbox(&state_unlocked), position.search);
                screen.render_stateful_widget(
                    SideBar::get_shortcuts(&state_unlocked),
                    position.shortcut,
                    &mut state_unlocked.sidebar,
                );

                let (music_table, mut music_table_state) =
                    MiddleLayout::get_music_container(&state_unlocked);
                screen.render_stateful_widget(music_table, position.music, &mut music_table_state);

                let (playlist_list, mut playlist_list_state) =
                    MiddleBottom::get_playlist_container(&state_unlocked);
                screen.render_stateful_widget(
                    playlist_list,
                    position.playlist,
                    &mut playlist_list_state,
                );

                let (artist_list, mut artist_list_state) =
                    MiddleBottom::get_artist_container(&state_unlocked);
                screen.render_stateful_widget(artist_list, position.artist, &mut artist_list_state);

                state_unlocked.refresh_time_elapsed();
                screen.render_widget(
                    BottomLayout::get_controller(&state_unlocked),
                    position.controllers,
                );
            })
            .unwrap();
    };
    paint_ui();

    'reactor: loop {
        // Use if instead of match because if will drop the mutex while going to else branch
        // but match keeps locking the mutex until match expression finished
        if cvar.wait(state.lock().unwrap()).unwrap().active == Window::None {
            break 'reactor;
        } else {
            paint_ui();
        }
    }

    crossterm::terminal::disable_raw_mode().unwrap_or_else(|_| {
        eprintln!("Failed to leave raw mode. You may need to restart the terminal")
    });
    execute!(std::io::stdout(), LeaveAlternateScreen).unwrap_or_else(|_| {
        eprintln!("Failed to leave alternate mode. You may need to restart the terminal")
    });
    terminal
        .show_cursor()
        .unwrap_or_else(|_| eprintln!("Failed to show cursor. Try: stty sane"));
}

#[derive(Clone)]
pub enum SidebarOption {
    Trending = 0,
    YoutubeCommunity = 1,
    RecentlyPlayed = 2,
    Favourates = 3,
    Search = 4,
    None = 5,
}

#[derive(PartialEq, Clone)]
pub enum Window {
    Searchbar,
    Helpbar,
    Sidebar,
    Musicbar,
    Playlistbar,
    Artistbar,
    None,
}

pub struct BottomState {
    music_duration: Duration,
    music_elapse: Duration,
    // String inside the Some is the title of song being played
    // true in Some means that music is currently playing
    // false in Some means music is paused
    // None means playing nothing. eg: At the start of program
    playing: Option<(String, bool)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicbarSource {
    Search(String, usize),
    Trending(usize),
    YoutubeCommunity,
    RecentlyPlayed,
    Favourates,
    Playlist(String, usize),
    Artist(String, usize),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PlaylistbarSource {
    Search(String, usize),
    RecentlyPlayed,
    Favourates,
    Artist(String, usize),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArtistbarSource {
    Search(String, usize),
    RecentlyPlayed,
    Favourates,
}

pub struct State<'p> {
    pub help: &'p str,
    // First is state of the sidebar list itself
    // And second is the state that is actually active.
    // which remains same even selected() of ListState is changed.
    // second memeber of tuple is only changed when user press ENTER on given SidebarOption
    sidebar: ListState,
    pub musicbar: VecDeque<fetcher::MusicUnit>,
    pub playlistbar: VecDeque<fetcher::PlaylistUnit>,
    pub artistbar: VecDeque<fetcher::ArtistUnit>,
    pub filled_source: (MusicbarSource, PlaylistbarSource, ArtistbarSource),
    bottom: BottomState,
    // First string is the actual string being typed on searchbar (to actually render)
    // If (musicbar or playlistbar or artistbar) is filled with search result
    // second memebr is Some(result_of_this_query) (to send to fetcher)
    // second member is the string of searchbar when use pressed ENTER last time in searchbar
    pub search: (String, String),
    pub active: Window,
    pub fetched_page: [Option<usize>; 3],
    player: libmpv::Mpv,
}
