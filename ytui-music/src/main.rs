use std::sync::{Arc, Mutex};
mod data_source;
use data_source::DataSource;

fn main() {
    let data_source = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async { DataSource::new().await });
    let data_source_action = Arc::clone(&data_source.source_action_queue);

    let data_dump_for_producer =
        Arc::new(tokio::sync::Mutex::new(data_source::DataSink::default()));
    let data_dump_for_ui = Arc::clone(&data_dump_for_producer);

    let ytui_ui = ytui_ui::renderer::YtuiUi::new(data_source_action, data_dump_for_ui);

    let ui_notifier_for_data_source = ytui_ui.get_ui_notifier_copy();
    let source_request_listener = ytui_ui.get_source_notifier_copy();

    let ui_thread_handle = ytui_ui.app_start();

    let sourcer_handle = std::thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                data_source
                    .start_data_sourcer_loop(
                        source_request_listener,
                        data_dump_for_producer,
                        ui_notifier_for_data_source,
                    )
                    .await;
            });
    });

    ui_thread_handle.join().unwrap();
    sourcer_handle.join().unwrap();
}
