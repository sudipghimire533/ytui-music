mod ui;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
pub mod test_backend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(ui::State::default()));
    let cvar = Arc::new(Condvar::new());

    let handler;
    {
        let mut state_for_painter = Arc::clone(&state);
        let mut state_for_handler = Arc::clone(&state);
        let mut cvar_for_painter = Arc::clone(&cvar);
        let mut cvar_for_handler = Arc::clone(&cvar);

        handler = thread::spawn(move || {
            ui::event::event_sender(&mut state_for_handler, &mut cvar_for_handler)
        });
        ui::draw_ui(&mut state_for_painter, &mut cvar_for_painter);
    }

    handler.join().unwrap();
    Ok(println!())
}
