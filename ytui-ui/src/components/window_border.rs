use ratatui::{
    style::Color,
    style::Style,
    widgets::{BorderType, WidgetRef},
};

pub struct WindowBorder;

impl WidgetRef for WindowBorder {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        ratatui::widgets::Block::bordered()
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Reset))
            .render_ref(area, buf);
    }
}
