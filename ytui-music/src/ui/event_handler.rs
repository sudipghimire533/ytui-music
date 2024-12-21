use crossterm::event::{self, Event};

pub enum YtuiEvent {
    Quit,
}

pub fn start_event_listener_loop(ytui_event_sender: std::sync::mpsc::Sender<YtuiEvent>) {
    'event_listener_loop: loop {
        let backend_event = event::read().unwrap();

        let is_some_key_event = matches!(backend_event, Event::Key(_));
        if is_some_key_event {
            ytui_event_sender.send(YtuiEvent::Quit).unwrap();
            break 'event_listener_loop;
        }
    }
}
