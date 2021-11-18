use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
mod cli;
mod communicator;
mod ui;

/*
* The role of main function is just to spwan two different loop in each thread and again pass
* control to another loop
* 1) handler is a sync thread for event_listener: This thread will wait for user input and respond
*    See ui/event.rs for implementation
* 2) comminucate is the sync thread for the comminucator which act as the bridge bwteen backend and
*    front-end. It checks for data required, get data from fetcher and also handles the filling of
*    data in respective place
* And the main thread itself will pass the control to `draw_ui` which handles rendering or painting
* to the terminal. This painter function as well as other 2 spawned thread above runs in a loop and
* all those loop and terminated when user hits key to quit the application.
*
* See below files for respective function
* __ui/mod.rs__: Defines structures as well as draw_ui function which render the content. This files
* contains the decleration and abstraction to control the layout, state and related things.
*
* __ui/utils.rs__: This files decleare and implementat all the defination in __ui/mod.rs__. This
* includes building the individual components, defining styles and layout, Initilizing the state
* and other structs.
*
* __ui/event.rs__: The sole purpose of this file is to wait for user event and responds by either
* updating the ui or by asking the comminucator to fill the required data
*
* __communicator.rs__: This file reads the state variable, compares it to previous state and change
* the data to be rendered. This includes calling the fetcher backed, navigating pages and so on.
*
* All the comminucation required are done via a single state variable which stores the state as
* well as presented data. Given state variable is shared via wrapping in condavr so that one thread
* can notify other thread when it bring some change in state
*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let opts = cli::Options::create_from_args(std::env::args());
        match opts {
            Err(err) => {
                eprintln!(
                    "There was an error while prasing cli options.\nError: {err}",
                    err = err
                );
                std::process::exit(1)
            }
            Ok(opts) => {
                let should_continue = opts.evaluate();
                if !should_continue {
                    std::process::exit(0)
                }
            }
        }
    }

    let state = Arc::new(Mutex::new(ui::State::default()));
    let cvar = Arc::new(Condvar::new());

    let (handler, communicate);
    {
        // same state is shared among all thread
        let mut state_for_painter = Arc::clone(&state);
        let mut state_for_handler = Arc::clone(&state);
        let mut state_for_communicator = Arc::clone(&state);
        let mut cvar_for_painter = Arc::clone(&cvar);
        let mut cvar_for_handler = Arc::clone(&cvar);
        let mut cvar_for_communicator = Arc::clone(&cvar);

        handler = thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    ui::event::event_sender(&mut state_for_handler, &mut cvar_for_handler).await;
                });
        });

        communicate = thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
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

    println!();
    Ok(())
}
