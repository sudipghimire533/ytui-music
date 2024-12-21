use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Block, BorderType, Borders, WidgetRef},
};

pub struct StatusBarUiAttrs {
    pub show_border: bool,

    pub repeat_char: &'static str,
    pub shuffle_char: &'static str,
    pub resume_char: &'static str,
    pub volume: u8,
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
    pub fn create_widget(style_options: &StatusBarUiAttrs) -> Self {
        let borders = if style_options.show_border {
            Borders::all()
        } else {
            Borders::NONE
        };

        let block = Block::new()
            .borders(borders)
            .border_type(BorderType::Rounded);

        Self {
            wrapper_block: block,
            repeat: Line::from(style_options.repeat_char).centered(),
            shuffle: Line::from(style_options.shuffle_char).centered(),
            resume: Line::from(style_options.resume_char).centered(),
            volume: Line::from(style_options.volume.to_string() + " ").centered(),
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

