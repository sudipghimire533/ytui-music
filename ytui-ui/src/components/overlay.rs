use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, WidgetRef, Wrap},
};

pub struct OverlayUiAttrs {
    pub show_borders: bool,
}

pub struct Overlay<'a> {
    widget: Paragraph<'a>,
}

impl Overlay<'_> {
    pub fn construct_widget(
        style_options: &OverlayUiAttrs,
        announcement_title: String,
        announcement_text: String,
    ) -> Self {
        let block = Block::new()
            .borders(if style_options.show_borders {
                Borders::ALL
            } else {
                Borders::NONE
            })
            .border_type(BorderType::Thick)
            .style(Style::default().bg(Color::Black))
            .padding(Padding::uniform(1))
            .title(announcement_title);

        let paragraph = Paragraph::new(announcement_text)
            .block(block)
            .wrap(Wrap { trim: true });

        Self { widget: paragraph }
    }
}

impl WidgetRef for Overlay<'_> {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Clear.render_ref(area, buf);
        self.widget.render_ref(area, buf);
    }
}
