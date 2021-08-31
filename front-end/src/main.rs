use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
use tokio;
mod communicator;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(ui::State::default()));
    let cvar = Arc::new(Condvar::new());

    let (handler, communicate);
    {
        let mut state_for_painter = Arc::clone(&state);
        let mut state_for_handler = Arc::clone(&state);
        let mut state_for_communicator = Arc::clone(&state);
        let mut cvar_for_painter = Arc::clone(&cvar);
        let mut cvar_for_handler = Arc::clone(&cvar);
        let mut cvar_for_communicator = Arc::clone(&cvar);

        handler = thread::spawn(move || {
            ui::event::event_sender(&mut state_for_handler, &mut cvar_for_handler)
        });
        communicate = thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    communicator::communicator(
                        &mut state_for_communicator,
                        &mut cvar_for_communicator,
                    )
                    .await;
                });
        });

        ui::draw_ui(&mut state_for_painter, &mut cvar_for_painter);
    }

    handler.join().unwrap();
    communicate.join().unwrap();
    Ok(println!())
}
