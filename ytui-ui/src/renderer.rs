use crate::state;

use super::components::{
    artist_pane::{ArtistPane, ArtistPaneUiAttrs},
    music_pane::{MusicPane, MusicPaneUiAttrs},
    navigation_list::{NavigationList, NavigationListUiAttrs},
    overlay::{Overlay, OverlayUiAttrs},
    playlist_pane::{PlaylistPane, PlaylistPaneUiAttrs},
    progressbar::{ProgressBar, ProgressBarUiAttrs},
    queue_list::{QueueList, QueueListUiAttrs},
    searchbar::{SearchBar, SearchBarUiAttrs},
    state_badge::{StateBadge, StateBadgeUiAttrs},
    statusbar::{StatusBar, StatusBarUiAttrs},
    window_border::WindowBorder,
};
use super::dimension::DimensionArgs;
use ratatui::{
    layout::Direction,
    style::{Color, Style},
    widgets::{Block, ListDirection, ListState, StatefulWidgetRef, TableState, WidgetRef as _},
};
use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct YtuiUi {
    state: super::state::AppState,
}

impl YtuiUi {
    pub fn app_start(mut self) {
        let mut terminal = ratatui::try_init().unwrap();

        let (event_sender, event_listener) = std::sync::mpsc::channel::<state::EventAction>();

        let mut state_for_renderer = self.state.clone();

        let mut should_exit_render_loop = false;
        while !should_exit_render_loop {
            terminal
                .draw(|frame| {
                    let dimension_args = DimensionArgs;
                    Self::draw_ui_in_frame(frame, &dimension_args, &mut state_for_renderer);

                    match event_listener.try_recv() {
                        Err(mpsc_err) => {
                            if matches!(mpsc_err, std::sync::mpsc::TryRecvError::Disconnected) {
                                should_exit_render_loop = true;
                            }
                        }

                        Ok(event_action) => match event_action {
                            state::EventAction::Quit => {
                                should_exit_render_loop = true;
                            }
                            state::EventAction::SearchQuery(_) => {}
                            state::EventAction::NewSearchInput(search_query) => {
                                state_for_renderer.search_query = search_query;
                            }
                            state::EventAction::NewPaneSelected(pane) => {
                                state_for_renderer.select_new_pane(pane);
                            }
                            state::EventAction::NewMovement { pane, direction } => match pane {
                                state::Pane::Overlay => match direction {
                                    ListDirection::TopToBottom => todo!(),
                                    ListDirection::BottomToTop => todo!(),
                                },
                                state::Pane::NavigationList => todo!(),
                                state::Pane::QueueList => todo!(),
                                state::Pane::Music => todo!(),
                                state::Pane::Playlist => todo!(),
                                state::Pane::Artist => todo!(),

                                _ => {}
                            },
                        },
                    }
                })
                .unwrap();
        }

        ratatui::restore();
        input_listener_handle.join().unwrap();
    }

    fn draw_ui_in_frame(
        frame: &mut ratatui::Frame,
        dimenstion_args: &DimensionArgs,
        app_state: &mut state::AppState,
    ) {
        let dimensions = dimenstion_args.calculate_dimension(frame.area());

        // draw a black background in all of sorrounding area ( if terminal size is too big )
        Block::default()
            .style(Style::new().bg(Color::Black))
            .render_ref(frame.area(), frame.buffer_mut());

        // draw a border around containing all the components render afterwards
        let window_border = WindowBorder;
        window_border.render_ref(dimensions.window_border, frame.buffer_mut());

        let searchbar_attrs = SearchBarUiAttrs {
            text_color: Color::Red,
            show_border: true,
            show_only_bottom_border: false,
        };
        let searchbar =
            SearchBar::create_widget(&searchbar_attrs).with_query("searching for something cool");

        let status_bar_attrs = StatusBarUiAttrs {
            show_border: true,
            repeat_char: "󰑖",
            shuffle_char: "󰒝",
            resume_char: "󰏤",
            volume: 100,
        };
        let statusbar = StatusBar::create_widget(&status_bar_attrs);

        let progress_bar_attrs = ProgressBarUiAttrs {
            foreground: Color::Green,
            background: Color::Reset,
        };
        let progressbar = ProgressBar::create_widget(&progress_bar_attrs)
            .with_duration(Duration::from_secs(200), Duration::from_secs(450));

        let queue_list_attrs = QueueListUiAttrs {
            text_color: Color::Green,
            highlight_color: Color::Red,
        };
        let queue_list = QueueList::create_widget(&queue_list_attrs).with_list(
            [
                "Lose control by Teddy Swims",
                "Greedy by Tate McRae",
                "Beautiful Things by Benson Boone",
                "Espresso by Sabrina Carpenter",
                "Come and take your love by unknwon",
            ]
            .repeat(5)
            .into_iter()
            .map(ToString::to_string)
            .collect(),
        );

        let navigation_list_attrs = NavigationListUiAttrs {
            text_color: Color::Green,
            highlight_color: Color::White,
        };
        let navigation_list = NavigationList::create_widget(&navigation_list_attrs).with_list(
            [
                "Trending",
                "Youtube Community",
                "Liked Songs",
                "Saved playlist",
                "Following",
                "Search",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
        );

        let state_badge_attrs = StateBadgeUiAttrs {
            text_color: Color::Yellow,
        };
        let state_badge =
            StateBadge::create_widget(&state_badge_attrs).with_msg("@sudipghimire533");

        let music_pane_attrs = MusicPaneUiAttrs {
            title_color: Color::LightCyan,
            text_color: Color::Green,
            highlight_color: Color::White,
        };
        let music_pane = MusicPane::create_widget(
            &music_pane_attrs,
            [["Mero desh birami", "Uniq Poet", "04:04"]]
                .repeat(20)
                .into_iter()
                .map(|[a, b, c]| [a.to_string(), b.to_string(), c.to_string()])
                .collect(),
        );

        let playlist_pane_attrs = PlaylistPaneUiAttrs {
            title_color: Color::LightCyan,
            text_color: Color::Yellow,
            highlight_color: Color::DarkGray,
        };
        let playlist_pane = PlaylistPane::create_widget(
            &playlist_pane_attrs,
            [[
                "Smoothing sound and something stupid like that",
                "mighty nepal",
            ]]
            .repeat(20)
            .into_iter()
            .map(|[a, b]| [a.to_string(), b.to_string()])
            .collect(),
        );

        let artist_pane_attrs = ArtistPaneUiAttrs {
            title_color: Color::LightCyan,
            text_color: Color::White,
            highlight_color: Color::Gray,
        };
        let artist_pane = ArtistPane::create_widget(
            &artist_pane_attrs,
            ["Bartika Eam Rai"]
                .repeat(20)
                .into_iter()
                .map(ToString::to_string)
                .collect(),
        );

        let overlay_attrs = OverlayUiAttrs {
            show_borders: true,
            title: "Release notes".to_string(),
        };
        let overlay = Overlay::construct_widget(&overlay_attrs).with_announcement("Installation
NOTE: since the dependency libmpv seems not to be maintained anymore,

you will probably need to build from source in any platform. See section Build From Source below.

Download latest binary from release page. If binary is not available for your platform, head on to build from source

Give it executable permission and from downloaded directory, in shell:

ytui_music run
You may need to jump to Usage Guide".to_string());

        searchbar.render_ref(dimensions.searchbar, frame.buffer_mut());
        statusbar.render_all(dimensions.statusbar, frame.buffer_mut());
        progressbar.render_ref(dimensions.progressbar, frame.buffer_mut());
        navigation_list.render_ref(
            dimensions.navigation_list,
            frame.buffer_mut(),
            &mut app_state.navigation_list_state,
        );
        queue_list.render_ref(
            dimensions.queue_list,
            frame.buffer_mut(),
            &mut app_state.queue_list_state,
        );
        state_badge.render_ref(dimensions.state_badge, frame.buffer_mut());
        music_pane.render_ref(
            dimensions.music_pane,
            frame.buffer_mut(),
            &mut app_state.music_pane_state,
        );
        playlist_pane.render_ref(
            dimensions.playlist_pane,
            frame.buffer_mut(),
            &mut app_state.playlist_pane_state,
        );
        artist_pane.render_ref(
            dimensions.artist_pane,
            frame.buffer_mut(),
            &mut app_state.artist_pane_state,
        );

        if matches!(app_state.selected_pane, Some(state::Pane::Overlay)) {
            overlay.render_ref(dimensions.overlay, frame.buffer_mut());
        }
    }

    pub fn new() -> Self {
        Self {
            state: super::state::AppState::new(),
        }
    }
}
