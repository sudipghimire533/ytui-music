use std::time::Duration;

use event_handler::YtuiEvent;
use ratatui::{style::Color, widgets::StatefulWidgetRef, widgets::WidgetRef, Frame};
use ytui_ui::components::navigation_list::{NavigationList, NavigationListUiAttrs};
use ytui_ui::components::progressbar::{ProgressBar, ProgressBarUiAttrs};
use ytui_ui::components::queue_list::{QueueList, QueueListUiAttrs};
use ytui_ui::components::searchbar::{SearchBar, SearchBarUiAttrs};
use ytui_ui::components::state_badge::{StateBadge, StateBadgeUiAttrs};
use ytui_ui::components::statusbar::{StatusBar, StatusBarUiAttrs};
use ytui_ui::components::window_border::WindowBorder;
use ytui_ui::dimension::DimensionArgs;
use ytui_ui::ratatui;
use ytui_ui::ratatui::style::Style;
use ytui_ui::ratatui::widgets::{Block, ListState};

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

    // draw a black background in all of sorrounding area ( if terminal size is too big )
    Block::default()
        .style(Style::new().bg(Color::Black))
        .render_ref(frame.area(), frame.buffer_mut());

    // draw a border around containing all the components render afterwards
    let window_border = WindowBorder;
    window_border.render_ref(dimensions.window_border, frame.buffer_mut());

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

    let queue_list_attrs = QueueListUiAttrs {
        text_color: Color::Green,
        highlight_color: Color::Red,
    };
    let queue_list = QueueList::create_widget(&queue_list_attrs).with_list(
        [
            "Lose control by Teddy Swims",
            "Greedy by Tate McRae",
            "Beautiful Things by Benson Boone",
            "Espresso by Sabrina Carpenter",
            "Come and take your love by unknwon",
        ]
        .repeat(5)
        .into_iter()
        .map(ToString::to_string)
        .collect(),
    );

    let navigation_list_attrs = NavigationListUiAttrs {
        text_color: Color::Green,
        highlight_color: Color::White,
    };
    let navigation_list = NavigationList::create_widget(&navigation_list_attrs).with_list(
        [
            "Trending",
            "Youtube Community",
            "Liked Songs",
            "Saved playlist",
            "Following",
            "Search",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect(),
    );

    let state_badge_attrs = StateBadgeUiAttrs {
        text_color: Color::Yellow,
    };
    let state_badge = StateBadge::create_widget(&state_badge_attrs).with_msg("@sudipghimire533");

    searchbar.render_ref(dimensions.searchbar, frame.buffer_mut());
    statusbar.render_all(dimensions.statusbar, frame.buffer_mut());
    progressbar.render_ref(dimensions.progressbar, frame.buffer_mut());
    navigation_list.render_ref(
        dimensions.navigation_list,
        frame.buffer_mut(),
        &mut ListState::default().with_offset(1).with_selected(Some(2)),
    );
    queue_list.render_ref(
        dimensions.queue_list,
        frame.buffer_mut(),
        &mut ListState::default().with_offset(1).with_selected(Some(4)),
    );
    state_badge.render_ref(dimensions.state_badge, frame.buffer_mut());
}
