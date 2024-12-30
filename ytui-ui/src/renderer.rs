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
use crate::{
    dimension::Dimension,
    state::{self, Pane},
};
use ratatui::{
    layout::{Rect, Size},
    style::{Color, Style},
    widgets::{Block, StatefulWidgetRef, WidgetRef},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct YtuiUi<RequestQueuer, DataProvidier> {
    state: super::state::AppState,

    source_notifier: Arc<tokio::sync::Notify>,
    request_new_data: Arc<tokio::sync::Mutex<RequestQueuer>>,
    request_available_data: Arc<tokio::sync::Mutex<DataProvidier>>,

    ui_renderer_notifier: Arc<std::sync::Condvar>,
}

impl<RequestQueuer, DataProvider> YtuiUi<RequestQueuer, DataProvider>
where
    RequestQueuer: super::DataRequester + std::marker::Send + 'static,
    DataProvider: super::DataGetter + std::marker::Send + 'static,
{
    pub fn new(
        request_new_data: Arc<tokio::sync::Mutex<RequestQueuer>>,
        request_available_data: Arc<tokio::sync::Mutex<DataProvider>>,
    ) -> Self {
        Self {
            request_new_data,
            request_available_data,
            state: super::state::AppState::new(),
            source_notifier: Arc::new(tokio::sync::Notify::new()),
            ui_renderer_notifier: Arc::new(std::sync::Condvar::new()),
        }
    }

    pub fn get_ui_notifier_copy(&self) -> Arc<std::sync::Condvar> {
        Arc::clone(&self.ui_renderer_notifier)
    }

    pub fn get_source_notifier_copy(&self) -> Arc<tokio::sync::Notify> {
        Arc::clone(&self.source_notifier)
    }

    pub fn app_start(mut self) -> std::thread::JoinHandle<()> {
        let mut terminal = ratatui::try_init().unwrap();
        let mut dimension_size = Size::new(0, 0);
        let mut dimensions = DimensionArgs.calculate_dimension(Rect::default());

        let mut paint_ui = move |app_state: &mut state::AppState| {
            terminal
                .draw(|frame| {
                    // re-calcualate dimensions only when screen size changes
                    let frame_size = frame.area().as_size();
                    if frame_size != dimension_size {
                        dimension_size = frame_size;
                        dimensions = DimensionArgs.calculate_dimension(frame.area());
                        Self::draw_window_broder_and_background(frame, &dimensions);
                    }

                    let data_provider = self.request_available_data.blocking_lock();
                    Self::draw_components_in_frame(frame, &dimensions, app_state, &data_provider);
                })
                .unwrap();
        };
        paint_ui(&mut self.state);

        let state_for_renderer = Arc::new(Mutex::new(self.state));
        let state_for_listener = Arc::clone(&state_for_renderer);

        let ui_notifier_for_event_handler = Arc::clone(&self.ui_renderer_notifier);
        let input_listener_handle = std::thread::spawn(move || {
            state::AppState::start_event_listener_loop(
                state_for_listener,
                ui_notifier_for_event_handler,
                self.request_new_data,
                self.source_notifier,
            )
        });

        let ui_render_handler = std::thread::spawn(move || loop {
            let mut app_state = self
                .ui_renderer_notifier
                .wait(state_for_renderer.lock().unwrap())
                .unwrap();
            if app_state.quit_ui {
                ratatui::restore();
                input_listener_handle.join().unwrap();
                break;
            }
            paint_ui(&mut app_state);
        });

        ui_render_handler
    }

    fn draw_window_broder_and_background(frame: &mut ratatui::Frame, dimensions: &Dimension) {
        // draw a black background in all of sorrounding area ( if terminal size is too big )
        Block::default()
            .style(Style::new().bg(Color::Black))
            .render_ref(frame.area(), frame.buffer_mut());

        // draw a border around containing all the components render afterwards
        let window_border = WindowBorder;
        window_border.render_ref(dimensions.window_border, frame.buffer_mut());
    }

    fn draw_components_in_frame(
        frame: &mut ratatui::Frame,
        dimensions: &Dimension,
        app_state: &mut state::AppState,
        data_source: &DataProvider,
    ) {
        let searchbar_attrs = SearchBarUiAttrs {
            text_color: Color::Red,
            show_border: true,
            show_only_bottom_border: false,
            is_active: matches!(app_state.selected_pane, Some(Pane::SearchBar)),
        };
        let searchbar = SearchBar::create_widget(&searchbar_attrs).with_query(
            app_state
                .search_query
                .as_deref()
                .unwrap_or("Listen to something new today"),
        );

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
            is_active: matches!(app_state.selected_pane, Some(Pane::NavigationList)),
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
        let music_pane = MusicPane::create_widget(&music_pane_attrs, data_source.get_music_list());

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
            overlay.render_ref(dimensions.overlay, frame.buffer_mut());
        }
    }
}
