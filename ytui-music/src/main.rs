fn main() {
    let ytui_ui = ytui_ui::renderer::YtuiUi::new();
    let ui_thread_handle = ytui_ui.app_start();

    ui_thread_handle.join().unwrap()
}
