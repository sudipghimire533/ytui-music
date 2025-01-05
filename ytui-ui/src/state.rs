use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::{ListDirection, ListState, TableState},
};
use std::{
    sync::{self, Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Copy)]
pub enum Pane {
    SearchBar,
    StatusBar,
    StateBadge,
    NavigationList,
    Gauge,
    QueueList,
    Overlay,
    Music,
    Playlist,
    Artist,
}

impl Pane {
    fn next_pane_to_select(self) -> Self {
        match self {
            Pane::StatusBar | Pane::StateBadge | Pane::Overlay | Pane::Gauge => {
                Pane::NavigationList
            }
            Pane::SearchBar => Pane::Music,
            Pane::NavigationList => Pane::QueueList,
            Pane::QueueList => Pane::Music,
            Pane::Music => Pane::Playlist,
            Pane::Playlist => Pane::Artist,
            Pane::Artist => Pane::Music,
        }
    }

    fn previous_pane_to_select(self) -> Self {
        match self {
            Pane::StatusBar | Pane::StateBadge | Pane::Overlay | Pane::Gauge => Pane::SearchBar,
            Pane::SearchBar => Pane::Artist,
            Pane::NavigationList => Pane::SearchBar,
            Pane::QueueList => Pane::NavigationList,
            Pane::Music => Pane::NavigationList,
            Pane::Playlist => Pane::Music,
            Pane::Artist => Pane::Playlist,
        }
    }
}

pub struct EventHandleResponse {
    should_quit: bool,
    should_notify_renderer: bool,
    should_notifiy_sourcer: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub(super) quit_ui: bool,

    current_size: (u16, u16),
    previously_selected_pane: Option<Pane>,

    pub(super) selected_pane: Option<Pane>,
    pub(super) search_query: Option<String>,
    pub(super) queue_list_state: ListState,
    pub(super) navigation_list_state: ListState,
    pub(super) music_pane_state: TableState,
    pub(super) playlist_pane_state: TableState,
    pub(super) artist_pane_state: TableState,

    pub(super) app_state_changed: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            quit_ui: false,
            previously_selected_pane: None,
            selected_pane: Some(Pane::Overlay),
            search_query: None,
            current_size: (0, 0),

            queue_list_state: Default::default(),
            navigation_list_state: Default::default(),
            music_pane_state: Default::default(),
            playlist_pane_state: Default::default(),
            artist_pane_state: Default::default(),

            app_state_changed: true,
        }
    }

    pub fn start_event_listener_loop<SourceAction: super::DataRequester>(
        // state and update condvar for ui
        locked_state: Arc<Mutex<AppState>>,
        input_event_sender: Arc<sync::Condvar>,

        // state and update condvar for fetching result + playing audio
        locked_source_action_queue: Arc<tokio::sync::Mutex<SourceAction>>,
        source_event_notifier: Arc<tokio::sync::Notify>,
    ) {
        loop {
            if matches!(event::poll(Duration::from_millis(500)), Ok(true)) {
                let source_response = Self::handle_event(
                    locked_state.clone(),
                    event::read().unwrap(),
                    locked_source_action_queue.clone(),
                );

                if source_response.should_quit {
                    input_event_sender.notify_one();
                    source_event_notifier.notify_one();
                    break;
                }

                if source_response.should_notify_renderer {
                    input_event_sender.notify_one();
                }
                if source_response.should_notifiy_sourcer {
                    source_event_notifier.notify_one();
                }
            }
        }
    }

    fn handle_event<SourceAction: super::DataRequester>(
        locked_state: Arc<Mutex<Self>>,
        event: Event,
        locked_source_action_queue: Arc<tokio::sync::Mutex<SourceAction>>,
    ) -> EventHandleResponse {
        let mut event_handle_response = EventHandleResponse {
            should_quit: false,
            should_notifiy_sourcer: false,
            should_notify_renderer: false,
        };
        let mut notify_ui = || event_handle_response.should_notify_renderer = true;

        match event {
            Event::Resize(w, h) => {
                Self::with_unlocked_state(locked_state, |state| {
                    state.mark_state_change();
                    state.current_size = (w, h);
                });
                notify_ui();
            }

            Event::FocusGained => {
                Self::with_unlocked_state(locked_state, |state| {
                    state.select_new_pane(Some(
                        state
                            .previously_selected_pane
                            .unwrap_or(Pane::NavigationList),
                    ));
                    notify_ui();
                });
            }

            Event::FocusLost => {
                Self::with_unlocked_state(locked_state, |state| {
                    state.mark_state_change();
                    state.select_new_pane(None);
                    notify_ui();
                });
            }

            Event::Paste(pasted_text) => {
                Self::with_unlocked_state(locked_state, move |state| {
                    state.mark_state_change();
                    match state.search_query.as_mut() {
                        Some(query) => query.push_str(pasted_text.as_str()),
                        None => state.search_query = Some(pasted_text.clone()),
                    }
                    notify_ui()
                });
            }

            Event::Key(key_event) => {
                return Self::with_unlocked_state(locked_state, move |state| {
                    state.new_key_event(key_event, locked_source_action_queue.clone())
                })
            }

            Event::Mouse(mouse_event) => {}
        }

        event_handle_response
    }

    fn new_key_event<SourceAction: super::DataRequester>(
        &mut self,
        key_event: KeyEvent,
        locked_action_queue: Arc<tokio::sync::Mutex<SourceAction>>,
    ) -> EventHandleResponse {
        let with_shift_modifier = key_event.modifiers.contains(KeyModifiers::SHIFT);
        let with_ctrl_modifier = key_event.modifiers.contains(KeyModifiers::CONTROL);
        let search_is_active = self.search_is_active();

        let mut event_handle_response = EventHandleResponse {
            should_notifiy_sourcer: false,
            should_quit: false,
            should_notify_renderer: false,
        };
        let mut notify_ui = || event_handle_response.should_notify_renderer = true;
        let mut notify_source = || event_handle_response.should_notifiy_sourcer = true;

        match key_event.code {
            event::KeyCode::Char('c') if with_ctrl_modifier => {
                self.quit_ui = true;
                event_handle_response.should_quit = true;
                Self::with_unlocked_source(locked_action_queue, |source| source.quit());

                self.mark_state_change();
                notify_ui();
                notify_source();
            }
            event::KeyCode::Char('/') if !search_is_active => {
                self.select_new_pane(Some(Pane::SearchBar));
                notify_ui();
            }

            event::KeyCode::Char(c) if search_is_active && !with_ctrl_modifier => {
                match self.search_query.as_mut() {
                    Some(query) => query.push(c),
                    None => self.search_query = Some(String::from(c)),
                }
                self.mark_state_change();
                notify_ui()
            }

            event::KeyCode::Backspace => {
                if self.search_is_active() {
                    self.search_query.as_mut().map(String::pop);
                } else {
                    self.move_to_prev_pane();
                }
                self.mark_state_change();
                notify_ui()
            }
            event::KeyCode::Esc => {
                if search_is_active || matches!(self.selected_pane, Some(Pane::Overlay)) {
                    self.move_to_next_pane();
                    notify_ui()
                }
            }
            event::KeyCode::Tab => {
                self.move_to_next_pane();
                notify_ui();
            }
            event::KeyCode::BackTab => {
                self.move_to_prev_pane();
                notify_ui();
            }

            event::KeyCode::Enter => match self.selected_pane {
                Some(Pane::SearchBar) => {
                    let trimmed_query = self
                        .search_query
                        .as_ref()
                        .map(|s| s.trim())
                        .unwrap_or_default();
                    if !trimmed_query.is_empty() {
                        Self::with_unlocked_source(locked_action_queue, |source| {
                            source.search_new_term(trimmed_query.to_string(), true, true, true);
                        });

                        self.move_to_next_pane();
                        notify_ui();
                        notify_source();
                    }
                }

                Some(Pane::Music) => {
                    if let Some(selected_music_index) = self.music_pane_state.selected() {
                        Self::with_unlocked_source(locked_action_queue, |source| {
                            source.play_from_music_pane(selected_music_index)
                        });
                        notify_source();
                    }
                }

                Some(Pane::Artist) => {
                    if let Some(selected_artist_index) = self.artist_pane_state.selected() {
                        Self::with_unlocked_source(locked_action_queue, |source| {
                            source.fetch_from_artist_pane(selected_artist_index);
                        });
                        self.music_pane_state.select(None);
                        self.playlist_pane_state.select(None);
                        notify_source();
                    }
                }

                Some(Pane::Playlist) => {
                    if let Some(selected_playlist_index) = self.playlist_pane_state.selected() {
                        Self::with_unlocked_source(locked_action_queue, |source| {
                            source.fetch_from_artist_pane(selected_playlist_index);
                        });
                        self.music_pane_state.select(None);
                        notify_source();
                    }
                }

                Some(Pane::NavigationList) => {
                    if let Some(selected_index) = self.navigation_list_state.selected() {
                        if selected_index == 0 {
                            Self::with_unlocked_source(locked_action_queue, |source| {
                                source.fetch_trending_music();
                            });
                            self.music_pane_state.select(None);
                            self.select_new_pane(Some(Pane::Music));
                            notify_source();
                        } else if selected_index == 5 {
                            self.select_new_pane(Some(Pane::SearchBar));
                            notify_ui();
                        }
                    }
                }

                None
                | Some(Pane::StatusBar)
                | Some(Pane::StateBadge)
                | Some(Pane::QueueList)
                | Some(Pane::Overlay)
                | Some(Pane::Gauge) => {}
            },

            event::KeyCode::Down | event::KeyCode::Up => {
                let list_direction = if matches!(key_event.code, event::KeyCode::Down) {
                    ListDirection::TopToBottom
                } else {
                    ListDirection::BottomToTop
                };

                if let Some(active_pane) = self.selected_pane {
                    if let Some(table_state) = self.get_table_state_of(active_pane) {
                        Self::circular_select_table_state(table_state, list_direction);
                        self.mark_state_change();
                        notify_ui();
                    } else if let Some(list_state) = self.get_list_state_of(active_pane) {
                        Self::circular_select_list_state(list_state, list_direction);
                        self.mark_state_change();
                        notify_ui();
                    }
                }
            }

            event::KeyCode::Char(' ') | event::KeyCode::Media(event::MediaKeyCode::PlayPause) => {
                Self::with_unlocked_source(locked_action_queue, |source| {
                    source.toggle_pause_playback();
                });
                self.mark_state_change();
                notify_source();
                notify_ui();
            }

            event::KeyCode::Right
            | event::KeyCode::Left
            | event::KeyCode::Home
            | event::KeyCode::End
            | event::KeyCode::PageUp
            | event::KeyCode::PageDown
            | event::KeyCode::Delete
            | event::KeyCode::Insert
            | event::KeyCode::F(_)
            | event::KeyCode::Char(_)
            | event::KeyCode::Null
            | event::KeyCode::CapsLock
            | event::KeyCode::ScrollLock
            | event::KeyCode::NumLock
            | event::KeyCode::PrintScreen
            | event::KeyCode::Pause
            | event::KeyCode::Menu
            | event::KeyCode::KeypadBegin
            | event::KeyCode::Media(_)
            | event::KeyCode::Modifier(_) => {}
        }

        event_handle_response
    }

    fn with_unlocked_state<T>(
        locked_state: Arc<Mutex<Self>>,
        mut action: impl FnMut(&mut AppState) -> T,
    ) -> T {
        let mut unlocked_state = locked_state.lock().unwrap();
        action(&mut unlocked_state)
    }

    fn with_unlocked_source<T, SourceAction>(
        locked_action_queue: Arc<tokio::sync::Mutex<SourceAction>>,
        mut action: impl FnMut(&mut SourceAction) -> T,
    ) -> T {
        let mut unlocked_source = locked_action_queue.blocking_lock();
        action(&mut unlocked_source)
    }
}

impl AppState {
    pub fn mark_state_change(&mut self) {
        self.app_state_changed = true;
    }

    pub fn search_is_active(&self) -> bool {
        self.selected_pane
            .map(|pane| matches!(pane, Pane::SearchBar))
            .unwrap_or_default()
    }

    pub fn move_to_next_pane(&mut self) {
        if let Some(selected_pane) = self.selected_pane {
            self.select_new_pane(Some(selected_pane.next_pane_to_select()));
        }
    }

    pub fn move_to_prev_pane(&mut self) {
        if let Some(selected_pane) = self.selected_pane {
            self.select_new_pane(Some(selected_pane.previous_pane_to_select()));
        }
    }

    pub fn select_new_pane(&mut self, new_pane: Option<Pane>) {
        self.mark_state_change();
        self.previously_selected_pane = self.selected_pane;
        self.selected_pane = new_pane;
    }

    fn circular_select_table_state(table_state: &mut TableState, direction: ListDirection) {
        match direction {
            ListDirection::TopToBottom => {
                table_state.select_next();
            }
            ListDirection::BottomToTop => {
                if table_state.selected() == Some(0) {
                    table_state.select_last();
                } else {
                    table_state.select_previous();
                }
            }
        }
    }

    fn circular_select_list_state(list_state: &mut ListState, direction: ListDirection) {
        match direction {
            ListDirection::TopToBottom => {
                list_state.select_next();
            }
            ListDirection::BottomToTop => {
                if list_state.selected() == Some(0) {
                    list_state.select_last();
                } else {
                    list_state.select_previous();
                }
            }
        }
    }

    fn get_table_state_of(&mut self, pane: Pane) -> Option<&mut TableState> {
        let ts = match pane {
            Pane::Music => &mut self.music_pane_state,
            Pane::Playlist => &mut self.playlist_pane_state,
            Pane::Artist => &mut self.artist_pane_state,

            _ => return None,
        };
        Some(ts)
    }

    fn get_list_state_of(&mut self, pane: Pane) -> Option<&mut ListState> {
        let ls = match pane {
            Pane::QueueList => &mut self.queue_list_state,
            Pane::NavigationList => &mut self.navigation_list_state,

            _ => return None,
        };
        Some(ls)
    }
}
