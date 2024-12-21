use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, WidgetRef, Wrap},
};

pub struct OverlayUiAttrs {
    pub show_borders: bool,
    pub title: String,
}

pub struct Overlay<'a> {
    block: Block<'a>,
    widget: Paragraph<'a>,
}

impl Overlay<'_> {
    pub fn construct_widget(style_options: &OverlayUiAttrs) -> Self {
        let block = Block::new()
            .borders(if style_options.show_borders {
                Borders::ALL
            } else {
                Borders::NONE
            })
            .border_type(BorderType::Thick)
            .style(Style::default().bg(Color::Black))
            .padding(Padding::uniform(1))
            .title(style_options.title.clone());

        Self {
            widget: Paragraph::default(),
            block,
        }
    }

    pub fn with_announcement(self, announcement: String) -> Self {
        Self {
            widget: Paragraph::new(announcement)
                .block(self.block)
                .wrap(Wrap { trim: true }),
            block: Default::default(),
        }
    }
}

impl WidgetRef for Overlay<'_> {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Clear.render_ref(area, buf);
        self.widget.render_ref(area, buf);
    }
}
