use std::time::Duration;

use event_handler::YtuiEvent;
use ratatui::{style::Color, widgets::WidgetRef, Frame};
use ytui_ui::components::progressbar::{ProgressBar, ProgressBarUiAttrs};
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

        if matches!(input_reactor.try_recv(), Ok(YtuiEvent::Quit)) {
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
    let searchbar =
        SearchBar::create_widget(&searchbar_attrs).with_query("searching for something cool");

    let status_bar_attrs = StatusBarUiAttrs {
        show_border: true,
        repeat_char: "󰑖",
        shuffle_char: "󰒝",
        resume_char: "󰏤",
        volume: 100,
    };
    let statusbar = StatusBar::create_widget(&status_bar_attrs);

    let progress_bar_attrs = ProgressBarUiAttrs {
        foreground: Color::Green,
        background: Color::Reset,
    };
    let progressbar = ProgressBar::create_widget(&progress_bar_attrs)
        .with_duration(Duration::from_secs(200), Duration::from_secs(450));

    searchbar.render_ref(dimensions.searchbar, frame.buffer_mut());
    statusbar.render_all(dimensions.statusbar, frame.buffer_mut());
    progressbar.render_ref(dimensions.progressbar, frame.buffer_mut());
}
