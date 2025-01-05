use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Gauge, WidgetRef},
};

use crate::PlayerStats;

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

    pub fn with_player_stats(self, player_stats: &PlayerStats) -> Self {
        let elapsed_duration = player_stats.elabsed_duration.unwrap_or_default();
        let total_duration = player_stats.total_duration.unwrap_or_default();

        let duration_text = format!(
            " {:02}:{:02} | {:02}:{:02} ",
            elapsed_duration / 60,
            elapsed_duration % 60,
            total_duration / 60,
            total_duration % 60
        );

        let block = self
            .block
            .expect("is always created by create_widget")
            .title(duration_text)
            .title_alignment(ratatui::layout::Alignment::Center);
        let percent = (elapsed_duration * 100) / player_stats.total_duration.unwrap_or(1);
        let gauge = self.gauge.block(block).percent(percent as u16);

        Self { gauge, block: None }
    }
}

impl WidgetRef for ProgressBar<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.gauge.render_ref(area, buf);
    }
}
