use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::{ListDirection, ListState, TableState},
};
use std::{
    sync::{self, Arc, Condvar, Mutex},
    time::Duration,
};

#[derive(Clone, Copy)]
pub enum Pane {
    SearchBar,
    StatusBar,
    NavigationList,
    QueueList,
    StateBadge,
    Gauge,
    Overlay,
    Music,
    Playlist,
    Artist,
}

pub enum EventAction {
    Quit,
    SearchQuery(String),

    NewSearchInput(Option<String>),
    NewPaneSelected(Option<Pane>),
    NewMovement {
        pane: Pane,
        direction: ListDirection,
    },
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
            Pane::Music => Pane::Artist,
            Pane::Playlist => Pane::Music,
            Pane::Artist => Pane::Playlist,
        }
    }
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
        }
    }

    pub fn start_event_listener_loop(
        locked_state: Arc<Mutex<AppState>>,
        event_sender: Arc<sync::Condvar>,
    ) {
        loop {
            if matches!(event::poll(Duration::from_millis(500)), Ok(true)) {
                let event = event::read().unwrap();
                let (should_quit, should_notify) = Self::handle_event(locked_state.clone(), event);
                if should_quit {
                    event_sender.notify_all();
                    break;
                } else if should_notify {
                    event_sender.notify_all();
                }
            }
        }
    }

    fn handle_event(locked_state: Arc<Mutex<Self>>, event: Event) -> (bool, bool) {
        let mut should_notify = false;
        let mut notify = || should_notify = true;

        match event {
            Event::Resize(w, h) => {
                Self::with_unlocked_state(locked_state, |state| {
                    state.current_size = (w, h);
                    notify();
                });
            }

            Event::FocusGained => {
                Self::with_unlocked_state(locked_state, |state| {
                    state.select_new_pane(Some(
                        state
                            .previously_selected_pane
                            .unwrap_or(Pane::NavigationList),
                    ));
                    notify();
                });
            }

            Event::FocusLost => {
                Self::with_unlocked_state(locked_state, |state| {
                    state.select_new_pane(None);
                    notify();
                });
            }

            Event::Paste(pasted_text) => {
                Self::with_unlocked_state(locked_state, move |state| {
                    match state.search_query.as_mut() {
                        Some(query) => query.push_str(pasted_text.as_str()),
                        None => state.search_query = Some(pasted_text.clone()),
                    }
                    notify()
                });
            }

            Event::Key(key_event) => {
                return Self::with_unlocked_state(locked_state, |state| {
                    state.new_key_event(key_event)
                })
            }

            Event::Mouse(mouse_event) => {}
        }

        (false, should_notify)
    }

    fn new_key_event(&mut self, key_event: KeyEvent) -> (bool, bool) {
        let is_key_release = matches!(key_event.kind, KeyEventKind::Release);
        let is_key_repeat = matches!(key_event.kind, KeyEventKind::Repeat);
        let is_key_press = matches!(key_event.kind, KeyEventKind::Press);
        let with_shift_modifier = key_event.modifiers.contains(KeyModifiers::SHIFT);
        let with_ctrl_modifier = key_event.modifiers.contains(KeyModifiers::CONTROL);
        let search_is_active = self.search_is_active();

        let mut should_notify = false;
        let mut should_quit = false;
        let mut notify = || should_notify = true;
        let mut quit = || should_quit = true;

        match key_event.code {
            event::KeyCode::Char('c') if with_ctrl_modifier => {
                self.quit_ui = true;
                notify();
                quit();
            }
            event::KeyCode::Char('/') if !search_is_active => {
                self.select_new_pane(Some(Pane::SearchBar));
                notify();
            }

            event::KeyCode::Char(c) if search_is_active && !with_ctrl_modifier => {
                match self.search_query.as_mut() {
                    Some(query) => query.push(c),
                    None => self.search_query = Some(String::from(c)),
                }
                notify()
            }

            event::KeyCode::Backspace => {
                if self.search_is_active() {
                    self.search_query.as_mut().map(String::pop);
                } else {
                    self.move_to_prev_pane();
                }
                notify()
            }
            event::KeyCode::Esc | event::KeyCode::Left => {
                if self.selected_pane.is_some() {
                    self.move_to_next_pane();
                }
                notify()
            }
            event::KeyCode::Tab => {
                if with_shift_modifier {
                    self.move_to_prev_pane();
                } else {
                    self.move_to_next_pane();
                }
                notify()
            }

            event::KeyCode::Enter => {
                if self.search_is_active() {
                    let search_query = self.search_query.clone().unwrap_or_default();
                    let trimmed_query = search_query.trim();
                    if !trimmed_query.is_empty() {
                        self.move_to_next_pane();
                        notify()
                    }
                }
            }
            event::KeyCode::Down => match self.selected_pane {
                None
                | Some(Pane::Gauge)
                | Some(Pane::StatusBar)
                | Some(Pane::SearchBar)
                | Some(Pane::StateBadge) => {}

                Some(Pane::NavigationList) => {
                    Self::circular_select_list_state(
                        &mut self.navigation_list_state,
                        ListDirection::TopToBottom,
                    );
                    notify()
                }

                _ => {}
            },

            event::KeyCode::Up => match self.selected_pane {
                None
                | Some(Pane::Gauge)
                | Some(Pane::StatusBar)
                | Some(Pane::SearchBar)
                | Some(Pane::StateBadge) => {}

                Some(Pane::NavigationList) => {
                    Self::circular_select_list_state(
                        &mut self.navigation_list_state,
                        ListDirection::BottomToTop,
                    );

                    notify()
                }

                _ => {}
            },

            event::KeyCode::Right
            | event::KeyCode::Home
            | event::KeyCode::End
            | event::KeyCode::PageUp
            | event::KeyCode::PageDown
            | event::KeyCode::BackTab
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

        (should_quit, should_notify)
    }

    fn with_unlocked_state<T>(
        locked_state: Arc<Mutex<Self>>,
        mut action: impl FnMut(&mut AppState) -> T,
    ) -> T {
        let mut unlocked_state = locked_state.lock().unwrap();
        action(&mut unlocked_state)
    }
}

impl AppState {
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
        self.previously_selected_pane = self.selected_pane;
        self.selected_pane = new_pane;
    }

    fn circular_select_list_state(list_state: &mut ListState, direction: ListDirection) {
        let previous_selection = list_state.selected();
        match direction {
            ListDirection::TopToBottom => {
                list_state.select_next();
                let new_selection = list_state.selected();
                if previous_selection == new_selection {
                    list_state.select_first();
                }
            }
            ListDirection::BottomToTop => {
                list_state.select_previous();
                let new_selection = list_state.selected();
                if previous_selection == new_selection {
                    list_state.select_last();
                }
            }
        }
    }
}
