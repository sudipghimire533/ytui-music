use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode},
    layout::Direction,
    widgets::{ListDirection, ListState, TableState},
};
use std::sync::{self, Arc, Mutex};

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
    pub fn can_move_to_pane(self) -> bool {
        matches!(
            self,
            Pane::SearchBar
                | Pane::NavigationList
                | Pane::QueueList
                | Pane::Music
                | Pane::Artist
                | Pane::Playlist,
        )
    }

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

    pub fn start_event_listener_loop(&mut self, event_sender: sync::mpsc::Sender<EventAction>) {
        loop {
            let next_event = event::read().unwrap();
            let event_action = self.handle_event(next_event);

            if let Some(event_action) = event_action {
                if matches!(event_action, EventAction::Quit) {
                    event_sender.send(event_action).unwrap();
                    break;
                }
                event_sender.send(event_action).unwrap();
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<EventAction> {
        match event {
            Event::Resize(w, h) => self.current_size = (w, h),

            Event::FocusGained => {
                self.select_new_pane(Some(
                    self.previously_selected_pane
                        .unwrap_or(Pane::NavigationList),
                ));
            }

            Event::FocusLost => {
                self.select_new_pane(None);
            }

            Event::Paste(pasted_text) if self.search_is_active() => {
                match self.search_query.as_mut() {
                    Some(query) => query.push_str(pasted_text.as_str()),
                    None => self.search_query = Some(pasted_text),
                }
            }
            Event::Paste(_) => {}

            Event::Key(key_event) => return self.new_key_event(key_event),

            Event::Mouse(mouse_event) => {}
        }

        None
    }

    fn new_key_event(&mut self, key_event: KeyEvent) -> Option<EventAction> {
        let is_key_release = matches!(key_event.kind, KeyEventKind::Release);
        let is_key_repeat = matches!(key_event.kind, KeyEventKind::Repeat);
        let is_key_press = matches!(key_event.kind, KeyEventKind::Press);
        let with_shift_modifier = key_event.modifiers.contains(KeyModifiers::SHIFT);
        let with_ctrl_modifier = key_event.modifiers.contains(KeyModifiers::CONTROL);
        let search_is_active = self.search_is_active();

        match key_event.code {
            event::KeyCode::Char('c') if with_ctrl_modifier => Some(EventAction::Quit),
            event::KeyCode::Char('k') if !search_is_active => {
                self.select_new_pane(Some(Pane::SearchBar));
                Some(EventAction::NewPaneSelected(self.selected_pane))
            }

            event::KeyCode::Char(c) if search_is_active && !with_ctrl_modifier => {
                match self.search_query.as_mut() {
                    Some(query) => query.push(c),
                    None => self.search_query = Some(String::from(c)),
                }

                Some(EventAction::NewSearchInput(self.search_query.clone()))
            }

            event::KeyCode::Backspace => {
                if self.search_is_active() {
                    self.search_query.as_mut().map(String::pop);
                    Some(EventAction::NewSearchInput(self.search_query.clone()))
                } else {
                    self.move_to_prev_pane();
                    Some(EventAction::NewPaneSelected(self.selected_pane))
                }
            }
            event::KeyCode::Esc | event::KeyCode::Left => {
                if self.selected_pane.is_some() {
                    self.move_to_next_pane();
                    Some(EventAction::NewPaneSelected(self.selected_pane))
                } else {
                    None
                }
            }
            event::KeyCode::Tab => {
                if with_shift_modifier {
                    self.move_to_prev_pane();
                } else {
                    self.move_to_next_pane();
                }
                self.move_to_next_pane();
                Some(EventAction::NewPaneSelected(self.selected_pane))
            }

            event::KeyCode::Enter => {
                if self.search_is_active() {
                    let search_query = self.search_query.clone().unwrap_or_default();
                    let trimmed_query = search_query.trim();
                    if !trimmed_query.is_empty() {
                        self.move_to_next_pane();
                        Some(EventAction::SearchQuery(trimmed_query.to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            event::KeyCode::Right => None,
            event::KeyCode::Up => match self.selected_pane {
                Some(Pane::NavigationList) => {
                    self.navigation_list_state.select_previous();
                    Some(EventAction::NewNavigationListState(
                        self.navigation_list_state.clone(),
                    ))
                }
                Some(Pane::QueueList) => {
                    self.queue_list_state.select_previous();
                    Some(EventAction::NewQueueListState(
                        self.queue_list_state.clone(),
                    ))
                }
                Some(Pane::Music) => {
                    self.music_pane_state.select_previous();
                    Some(EventAction::NewMusicPaneListState(
                        self.music_pane_state.clone(),
                    ))
                }
                Some(Pane::Playlist) => {
                    self.playlist_pane_state.select_previous();
                    Some(EventAction::NewMusicPaneListState(
                        self.music_pane_state.clone(),
                    ))
                }
                Some(Pane::Artist) => None,

                Some(Pane::Overlay) => {
                    // scroll?
                    None
                }

                None
                | Some(Pane::Gauge)
                | Some(Pane::StatusBar)
                | Some(Pane::SearchBar)
                | Some(Pane::StateBadge) => None,
            },
            event::KeyCode::Down => match self.selected_pane {
                Some(Pane::NavigationList) => {
                    self.navigation_list_state.select_next();
                    Some(EventAction::NewNavigationListState(
                        self.navigation_list_state.clone(),
                    ))
                }
                Some(Pane::QueueList) => {
                    self.queue_list_state.select_next();
                    Some(EventAction::NewQueueListState(
                        self.queue_list_state.clone(),
                    ))
                }
                Some(Pane::Music) => {
                    self.music_pane_state.select_next();
                    Some(EventAction::NewMusicPaneListState(
                        self.music_pane_state.clone(),
                    ))
                }
                Some(Pane::Playlist) => {
                    self.playlist_pane_state.select_next();
                    Some(EventAction::NewMusicPaneListState(
                        self.music_pane_state.clone(),
                    ))
                }
                Some(Pane::Artist) => None,

                Some(Pane::Overlay) => {
                    // scroll?
                    None
                }

                None
                | Some(Pane::Gauge)
                | Some(Pane::StatusBar)
                | Some(Pane::SearchBar)
                | Some(Pane::StateBadge) => None,
            },

            event::KeyCode::Home => None,
            event::KeyCode::End => None,
            event::KeyCode::PageUp => None,
            event::KeyCode::PageDown => None,
            event::KeyCode::BackTab => None,
            event::KeyCode::Delete => None,
            event::KeyCode::Insert => None,
            event::KeyCode::F(_) => None,
            event::KeyCode::Char(_) => None,
            event::KeyCode::Null => None,
            event::KeyCode::CapsLock => None,
            event::KeyCode::ScrollLock => None,
            event::KeyCode::NumLock => None,
            event::KeyCode::PrintScreen => None,
            event::KeyCode::Pause => None,
            event::KeyCode::Menu => None,
            event::KeyCode::KeypadBegin => None,
            event::KeyCode::Media(media_key_code) => None,
            event::KeyCode::Modifier(modifier_key_code) => None,
        }
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
}
