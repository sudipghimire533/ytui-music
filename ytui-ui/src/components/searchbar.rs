use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, WidgetRef},
};

pub struct SearchBarUiAttrs {
    pub text_color: ratatui::style::Color,
    pub show_border: bool,
    pub show_only_bottom_border: bool,
    pub is_active: bool,
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
    block: Block<'a>,
}

impl SearchBar<'_> {
    pub fn create_widget(style_options: &SearchBarUiAttrs) -> Self {
        let block = Block::new()
            .borders(style_options.get_block_borders())
            .border_style(Style::default().fg(if style_options.is_active {
                Color::Cyan
            } else {
                Color::Gray
            }))
            .border_type(BorderType::Rounded)
            .title(
                Span::default()
                    .content("Search: ")
                    .italic()
                    .style(Style::default().fg(if style_options.is_active {
                        Color::Cyan
                    } else {
                        Color::Magenta
                    })),
            );

        Self {
            block,
            widget: Default::default(),
        }
    }

    pub fn with_query(self, query: impl ToString) -> Self {
        let widget = Paragraph::new(query.to_string()).block(self.block);

        Self {
            widget,
            block: Default::default(),
        }
    }
}

impl WidgetRef for SearchBar<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.widget.render_ref(area, buf);
    }
}
