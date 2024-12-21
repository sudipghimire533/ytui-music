use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Paragraph, WidgetRef},
};

pub struct StateBadgeUiAttrs {
    pub text_color: Color,
}

pub struct StateBadge<'a> {
    block: Block<'a>,
    span: Span<'a>,
    widget: Paragraph<'a>,
}

impl StateBadge<'_> {
    pub fn create_widget(style_options: &StateBadgeUiAttrs) -> Self {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Status ");
        let paragraph = Span::default().style(Style::default().fg(style_options.text_color));

        Self {
            block,
            span: paragraph,
            widget: Default::default(),
        }
    }

    pub fn with_msg(self, message: impl ToString) -> Self {
        let widget = Paragraph::new(self.span.content(message.to_string()))
            .block(self.block)
            .centered();

        Self {
            widget,
            span: Default::default(),
            block: Default::default(),
        }
    }
}

impl WidgetRef for StateBadge<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.widget.render_ref(area, buf);
    }
}
