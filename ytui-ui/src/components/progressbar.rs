use std::{process::abort, time::Duration};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Gauge, WidgetRef},
};

pub struct ProgressBarUiAttrs {
    pub foreground: ratatui::style::Color,
    pub background: ratatui::style::Color,
}

pub struct ProgressBar<'a> {
    block: Option<Block<'a>>,
    gauge: Gauge<'a>,
}

impl ProgressBar<'_> {
    pub fn create_widget(style_options: &ProgressBarUiAttrs) -> Self {
        let gauge = Gauge::default()
            .percent(10)
            .style(
                Style::default()
                    .fg(style_options.foreground)
                    .bg(style_options.background),
            )
            .label("")
            .use_unicode(true);

        let block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        ProgressBar {
            gauge,
            block: Some(block),
        }
    }

    pub fn with_duration(self, played: Duration, remaining: Duration) -> Self {
        let played_sec = played.as_secs();
        let total_sec = played_sec + remaining.as_secs();

        let duration_text = format!(
            " {:02}:{:02} | {:02}:{:02} ",
            played_sec / 60,
            played_sec % 60,
            total_sec / 60,
            total_sec % 60
        );

        let block = self
            .block
            .expect("is always created by create_widget")
            .title(duration_text)
            .title_alignment(ratatui::layout::Alignment::Center);
        let percent = (played_sec * 100) / total_sec;
        let gauge = self.gauge.block(block).percent(percent as u16);

        Self { gauge, block: None }
    }
}

impl WidgetRef for ProgressBar<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.gauge.render_ref(area, buf);
    }
}
