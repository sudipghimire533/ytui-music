use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, List, ListState, StatefulWidgetRef},
};

pub struct QueueListUiAttrs {
    pub text_color: Color,
    pub highlight_color: Color,
    pub is_active: bool,
}

pub struct QueueList<'a> {
    widget: List<'a>,
}

impl QueueList<'_> {
    pub fn create_widget(style_options: &QueueListUiAttrs) -> Self {
        let widget = List::default()
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(Span::from(" Next in Queue ").style(Style::default().fg(Color::Cyan)))
                    .style(Style::default().fg(if style_options.is_active {
                        Color::Cyan
                    } else {
                        Color::Green
                    }))
                    .title_alignment(ratatui::layout::Alignment::Center),
            )
            .direction(ratatui::widgets::ListDirection::TopToBottom)
            .style(Style::default().fg(style_options.text_color))
            .highlight_style(Style::default().fg(style_options.highlight_color));

        Self { widget }
    }

    pub fn with_list(self, items: Vec<String>) -> Self {
        Self {
            widget: self.widget.items(items),
        }
    }
}

impl<'a> StatefulWidgetRef for QueueList<'a> {
    type State = <List<'a> as StatefulWidgetRef>::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        self.widget.render_ref(area, buf, state);
    }
}
