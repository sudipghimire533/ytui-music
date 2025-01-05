use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Cell, Padding, Row, StatefulWidgetRef, Table, TableState},
};

pub struct ArtistPaneUiAttrs {
    pub title_color: Color,
    pub text_color: Color,
    pub highlight_color: Color,
    pub is_active: bool,
}

pub struct ArtistPane<'a> {
    widget: Table<'a>,
}

impl ArtistPane<'_> {
    pub fn create_widget(style_options: &ArtistPaneUiAttrs, items: &[String]) -> Self {
        let rows = items
            .iter()
            .map(|d| Row::new([d.clone()]))
            .collect::<Vec<Row>>();

        let headers = ["Name"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::default().fg(style_options.title_color))
            .height(1);

        let widget = Table::new(rows, [Constraint::Fill(1)])
            .header(headers)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::left(1))
                    .border_style(Style::default().fg(if style_options.is_active {
                        Color::Cyan
                    } else {
                        Color::White
                    })),
            )
            .style(Style::default().fg(style_options.text_color))
            .row_highlight_style(Style::default().fg(style_options.highlight_color));

        Self { widget }
    }
}

impl<'a> StatefulWidgetRef for ArtistPane<'a> {
    type State = <Table<'a> as StatefulWidgetRef>::State;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut TableState) {
        self.widget.render_ref(area, buf, state);
    }
}
