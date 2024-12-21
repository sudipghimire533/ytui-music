use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, WidgetRef},
};

pub struct SearchBarUiAttrs {
    pub text_color: ratatui::style::Color,
    pub show_border: bool,
    pub show_only_bottom_border: bool,
}

impl SearchBarUiAttrs {
    fn get_block_borders(&self) -> Borders {
        if self.show_border {
            if self.show_only_bottom_border {
                Borders::BOTTOM
            } else {
                Borders::all()
            }
        } else {
            Borders::NONE
        }
    }
}

pub struct SearchBar<'a> {
    pub input_text: String,
    pub widget: Paragraph<'a>,
}

impl SearchBar<'_> {
    pub fn create_widget(style_options: &SearchBarUiAttrs) -> Self {
        let block = Block::new()
            .borders(style_options.get_block_borders())
            .border_type(BorderType::Rounded);

        let query_text = Text::style(
            "some query".into(),
            Style::default().fg(style_options.text_color),
        );

        let widget = Paragraph::new(query_text).block(block);

        Self {
            input_text: "some query".to_string(),
            widget,
        }
    }
}

impl WidgetRef for SearchBar<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.widget.render_ref(area, buf);
    }
}

