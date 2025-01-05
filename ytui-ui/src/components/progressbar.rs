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
    gauge: Gauge<'a>,
}

impl ProgressBar<'_> {
    pub fn create_widget(style_options: &ProgressBarUiAttrs, player_stats: &PlayerStats) -> Self {
        let elapsed_duration = player_stats.elabsed_duration.unwrap_or_default();
        let total_duration = player_stats.total_duration.unwrap_or_default();

        let duration_text = format!(
            " {:02}:{:02} | {:02}:{:02} ",
            elapsed_duration / 60,
            elapsed_duration % 60,
            total_duration / 60,
            total_duration % 60
        );

        let block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .title(duration_text)
            .title_alignment(ratatui::layout::Alignment::Center);

        let gauge = Gauge::default()
            .percent(10)
            .style(
                Style::default()
                    .fg(style_options.foreground)
                    .bg(style_options.background),
            )
            .label(player_stats.media_title.clone().unwrap_or_default())
            .use_unicode(true)
            .block(block)
            .percent(player_stats.playback_percent.unwrap_or_default() as u16);

        ProgressBar { gauge }
    }
}

impl WidgetRef for ProgressBar<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.gauge.render_ref(area, buf);
    }
}
