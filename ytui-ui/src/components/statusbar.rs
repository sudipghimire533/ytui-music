use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, BorderType, Borders, WidgetRef},
};

use crate::PlayerStats;

pub struct StatusBarUiAttrs {
    pub show_border: bool,

    pub repeat_char: &'static str,
    pub shuffle_char: &'static str,
    pub resume_char: &'static str,
    pub pause_char: &'static str,
}

impl StatusBarUiAttrs {}

pub struct StatusBar<'a> {
    wrapper_block: Block<'a>,
    repeat: Line<'a>,
    shuffle: Line<'a>,
    resume: Line<'a>,
    volume: Line<'a>,
}

impl StatusBar<'_> {
    pub fn create_widget(style_options: &StatusBarUiAttrs, player_stats: &PlayerStats) -> Self {
        let borders = if style_options.show_border {
            Borders::all()
        } else {
            Borders::NONE
        };

        let play_icon = if player_stats.paused.unwrap_or(true) {
            style_options.pause_char
        } else {
            style_options.resume_char
        };

        let block = Block::new()
            .borders(borders)
            .border_type(BorderType::Rounded)
            .title("Controls: ");

        let volume_str = player_stats
            .volume
            .map(|v| v.to_string() + " ")
            .unwrap_or(String::from("E "));

        Self {
            wrapper_block: block,
            repeat: Line::from(style_options.repeat_char).centered(),
            shuffle: Line::from(style_options.shuffle_char).centered(),
            resume: Line::from(play_icon).centered(),
            volume: Line::from(volume_str).centered(),
        }
    }
}

impl StatusBar<'_> {
    pub fn render_all(
        &self,
        (rect, [repeat_rect, resume_rect, shuffle_rect, volume_rect]): (Rect, [Rect; 4]),
        buf: &mut Buffer,
    ) {
        self.wrapper_block.render_ref(rect, buf);
        self.repeat.render_ref(repeat_rect, buf);
        self.resume.render_ref(resume_rect, buf);
        self.shuffle.render_ref(shuffle_rect, buf);
        self.volume.render_ref(volume_rect, buf);
    }
}
