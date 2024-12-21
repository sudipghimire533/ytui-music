use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Span, Text},
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
    widget: Paragraph<'a>,
    text: Span<'a>,
    block: Block<'a>,
}

impl SearchBar<'_> {
    pub fn create_widget(style_options: &SearchBarUiAttrs) -> Self {
        let block = Block::new()
            .borders(style_options.get_block_borders())
            .border_type(BorderType::Rounded);
        let text = Span::default().style(Style::default().fg(style_options.text_color));

        Self {
            text,
            block,
            widget: Default::default(),
        }
    }

    pub fn with_query(self, query: impl ToString) -> Self {
        let Self {
            widget: _default_widget,
            text,
            block,
        } = self;

        let text = text.content(query.to_string());
        let widget = Paragraph::new(text).block(block);

        Self {
            widget,
            text: Default::default(),
            block: Default::default(),
        }
    }
}

impl WidgetRef for SearchBar<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.widget.render_ref(area, buf);
    }
}
