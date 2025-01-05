use super::dimension::DimensionArgs;
use crate::{
    components::{self, ComponentsCollection},
    dimension::Dimension,
    state::{self, AppState},
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
    player: Arc<ytui_audio::libmpv::LibmpvPlayer>,

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
        player: Arc<ytui_audio::libmpv::LibmpvPlayer>,
    ) -> Self {
        Self {
            player,
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

    pub fn app_start(self) -> std::thread::JoinHandle<()> {
        let mut terminal = ratatui::try_init().unwrap();
        let mut dimension_size = Size::new(0, 0);
        let mut dimensions = DimensionArgs.calculate_dimension(Rect::default());
        let mut data_collection = components::ComponentsDataCollection::default();
        let mut components =
            components::ComponentsCollection::create_all_components(&self.state, &data_collection);

        let mut paint_ui = move |app_state: &mut state::AppState, trigerred_by_timeout: bool| {
            terminal.draw(|frame| {
                let trigerred_by_ui = std::mem::take(&mut app_state.app_state_changed);
                let mut trigerred_by_source = false;

                if trigerred_by_timeout {
                    data_collection.refresh_from_player(&self.player);
                    components.progressbar =
                        components::ComponentsCollection::create_progress_bar(&data_collection);
                } else {
                    let mut data_provider = self.request_available_data.blocking_lock();
                    if data_provider.has_new_data() {
                        data_collection.refresh_from_data_provider(&*data_provider);
                        data_provider.mark_consumed_new_data();
                        trigerred_by_source = true;
                    }
                }

                if trigerred_by_ui {
                    data_collection.refresh_from_player(&self.player);

                    // re-calcualate dimensions only when screen size changes
                    let frame_size = frame.area().as_size();
                    if frame_size != dimension_size {
                        dimension_size = frame_size;
                        dimensions = DimensionArgs.calculate_dimension(frame.area());
                        Self::draw_window_broder_and_background(frame, &dimensions);
                    }
                }

                if trigerred_by_ui || trigerred_by_source {
                    components = components::ComponentsCollection::create_all_components(
                        app_state,
                        &data_collection,
                    );
                }

                // every frame should have complete drawing of all components
                Self::draw_all_components(frame.buffer_mut(), &dimensions, &components, app_state);
            })?;

            Ok::<(), std::io::Error>(())
        };
        paint_ui(&mut AppState::new(), false).unwrap();

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

        let ui_render_handler = std::thread::spawn(move || {
            let progressbar_update_latency = Duration::from_millis(900);

            loop {
                let (mut app_state, timeout_res) = self
                    .ui_renderer_notifier
                    .wait_timeout(
                        state_for_renderer.lock().unwrap(),
                        progressbar_update_latency,
                    )
                    .unwrap();

                if app_state.quit_ui {
                    ratatui::restore();
                    input_listener_handle.join().unwrap();
                    break;
                }
                paint_ui(&mut app_state, timeout_res.timed_out()).ok();
            }
        });

        ui_render_handler
    }

    fn draw_window_broder_and_background(frame: &mut ratatui::Frame, dimensions: &Dimension) {
        // draw a black background in all of sorrounding area ( if terminal size is too big )
        Block::default()
            .style(Style::new().bg(Color::Black))
            .render_ref(frame.area(), frame.buffer_mut());

        // draw a border around containing all the components render afterwards
        let window_border = components::window_border::WindowBorder;
        window_border.render_ref(dimensions.window_border, frame.buffer_mut());
    }

    fn draw_all_components(
        screen: &mut ratatui::buffer::Buffer,
        dimensions: &Dimension,
        components: &ComponentsCollection,
        app_state: &mut AppState,
    ) {
        components
            .searchbar
            .render_ref(dimensions.searchbar, screen);
        components
            .state_badge
            .render_ref(dimensions.state_badge, screen);
        components.navigation_list.render_ref(
            dimensions.navigation_list,
            screen,
            &mut app_state.navigation_list_state,
        );
        components.queue_list.render_ref(
            dimensions.queue_list,
            screen,
            &mut app_state.queue_list_state,
        );
        components.music_pane.render_ref(
            dimensions.music_pane,
            screen,
            &mut app_state.music_pane_state,
        );
        components.playlist_pane.render_ref(
            dimensions.playlist_pane,
            screen,
            &mut app_state.playlist_pane_state,
        );
        components.artist_pane.render_ref(
            dimensions.artist_pane,
            screen,
            &mut app_state.artist_pane_state,
        );
        components
            .progressbar
            .render_ref(dimensions.progressbar, screen);
        components
            .statusbar
            .render_all(dimensions.statusbar, screen);

        if let Some(ref overlay) = components.overlay {
            overlay.render_ref(dimensions.overlay, screen);
        }
    }
}
