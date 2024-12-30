use std::{borrow::Cow, task::Wake};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Cell, Padding, Row, StatefulWidgetRef, Table, TableState},
};

pub struct MusicPaneUiAttrs {
    pub title_color: Color,
    pub text_color: Color,
    pub highlight_color: Color,
}

pub struct MusicPane<'a> {
    widget: Table<'a>,
}

impl<'a> MusicPane<'a> {
    pub fn create_widget(
        style_options: &MusicPaneUiAttrs,
        items: impl Iterator<Item = [Cow<'a, str>; 3]>,
    ) -> Self {
        let rows = items
            .into_iter()
            .map(|row| row.into_iter().map(Cell::from).collect())
            .collect::<Vec<Row>>();

        let headers = ["Title", "Artist", "Length"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::default().fg(style_options.title_color))
            .height(1);

        let widget = Table::new(
            rows,
            [
                Constraint::Fill(3),
                Constraint::Fill(2),
                Constraint::Fill(1),
            ],
        )
        .header(headers)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::left(1))
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(style_options.text_color))
        .row_highlight_style(Style::default().fg(style_options.highlight_color));

        Self { widget }
    }
}

impl<'a> StatefulWidgetRef for MusicPane<'a> {
    type State = <Table<'a> as StatefulWidgetRef>::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut TableState) {
        self.widget.render_ref(area, buf, state);
    }
}
