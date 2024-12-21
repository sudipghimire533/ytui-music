use event_handler::YtuiEvent;
use ratatui::{layout::Rect, style::Color, widgets::WidgetRef, Frame};
use ytui_ui::components::searchbar::{SearchBar, SearchBarUiAttrs};
use ytui_ui::components::statusbar::{StatusBar, StatusBarUiAttrs};
use ytui_ui::dimension::DimensionArgs;
use ytui_ui::ratatui;

mod event_handler;

pub(crate) fn start_ui_render_loop() {
    let mut terminal = ratatui::try_init().unwrap();

    let (input_listener, input_reactor) = std::sync::mpsc::channel::<YtuiEvent>();

    let input_listener_handle =
        std::thread::spawn(move || event_handler::start_event_listener_loop(input_listener));

    'ui_loop: loop {
        terminal
            .draw(|frame| {
                let dimension_args = DimensionArgs;
                draw_ui_in_frame(frame, &dimension_args)
            })
            .unwrap();

        if matches!(input_reactor.recv().unwrap(), YtuiEvent::Quit) {
            break 'ui_loop;
        }
    }

    ratatui::restore();
    input_listener_handle.join().unwrap();
}

fn draw_ui_in_frame(frame: &mut Frame, dimenstion_args: &DimensionArgs) {
    let dimensions = dimenstion_args.calculate_dimension(frame.area());

    let searchbar_attrs = SearchBarUiAttrs {
        text_color: Color::Red,
        show_border: true,
        show_only_bottom_border: false,
    };
    let searchbar = SearchBar::create_widget(&searchbar_attrs);

    let status_bar_attrs = StatusBarUiAttrs {
        show_border: true,
        repeat_char: "󰑖",
        shuffle_char: "󰒝",
        resume_char: "󰏤",
        volume: 100,
    };
    let statusbar = StatusBar::create_widget(&status_bar_attrs);

    searchbar.render_ref(dimensions.searchbar, frame.buffer_mut());
    statusbar.render_all(dimensions.statusbar, frame.buffer_mut());
}
