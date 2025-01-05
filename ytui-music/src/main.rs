use std::sync::Arc;
mod data_sink;
mod data_source;

fn main() {
    let default_data_sink = data_sink::DataSink::default();
    let mpv_player_for_source = default_data_sink.make_player_copy();
    let mpv_player_for_ui = default_data_sink.make_player_copy();

    let locked_data_sink = tokio::sync::Mutex::new(default_data_sink);
    let data_dump_for_producer = Arc::new(locked_data_sink);
    let data_dump_for_ui = Arc::clone(&data_dump_for_producer);

    let data_source = tiny_tokio_runtime()
        .block_on(async { data_source::DataSource::new(mpv_player_for_source).await });
    let data_source_action = Arc::clone(&data_source.source_action_queue);

    let ytui_ui =
        ytui_ui::renderer::YtuiUi::new(data_source_action, data_dump_for_ui, mpv_player_for_ui);

    let ui_notifier_for_data_source = ytui_ui.get_ui_notifier_copy();
    let source_request_listener = ytui_ui.get_source_notifier_copy();

    let ui_thread_handle = ytui_ui.app_start();

    let sourcer_handle = std::thread::spawn(move || {
        full_tokio_runtime().block_on(async {
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

fn tiny_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}

fn full_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}
