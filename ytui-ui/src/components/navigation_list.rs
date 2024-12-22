use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, List, ListState, Padding, StatefulWidgetRef},
};

pub struct NavigationListUiAttrs {
    pub text_color: Color,
    pub highlight_color: Color,
}

pub struct NavigationList<'a> {
    widget: List<'a>,
}

impl NavigationList<'_> {
    pub fn create_widget(style_options: &NavigationListUiAttrs) -> Self {
        let widget = List::default()
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::left(1)),
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

impl<'a> StatefulWidgetRef for NavigationList<'a> {
    type State = <List<'a> as StatefulWidgetRef>::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        self.widget.render_ref(area, buf, state);
    }
}
