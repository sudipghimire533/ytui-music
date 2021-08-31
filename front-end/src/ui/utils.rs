use crate::ui;
use fetcher::ExtendDuration;
use libmpv;
use ui::shared_import::*;

const HEART_FILLED: &str = "\u{2665}";
const HEART_OUTLINE: &str = "\u{2661}";
const CURRENT_TITLE_LEN: usize = 70;
pub const SIDEBAR_LIST_COUNT: usize = 7;
pub const SIDEBAR_LIST_ITEMS: [&str; SIDEBAR_LIST_COUNT] = [
    "Trending",
    "Youtube Communinty",
    "Recently Played",
    "Followings",
    "Favourates",
    "My Playlist",
    "Search",
];

impl<'parent> ui::TopLayout {
    pub fn new(parent: Rect) -> Self {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(parent);

        ui::TopLayout {
            layout: [layout[0], layout[1]],
        }
    }
    pub fn get_helpbox(state: &'parent ui::State) -> Paragraph<'parent> {
        Paragraph::new(Span::styled(
            state.help,
            Style::default()
                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                .fg(Color::Yellow),
        ))
        .block(Block::new("Help".to_owned()))
        .block(Block::new("Help".to_owned()))
    }
    pub fn get_searchbox(state: &'parent ui::State) -> Paragraph<'parent> {
        let mut cursor_style = Style::default().fg(Color::White);

        let block = match state.active {
            ui::Window::Searchbar => {
                cursor_style = cursor_style.add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK);
                Block::active("Search ".to_owned())
            }
            _ => {
                cursor_style = cursor_style.add_modifier(Modifier::HIDDEN);
                Block::new("Search ".to_owned())
            }
        };
        let text = text::Spans::from(vec![
            Span::styled(
                state.search.0.as_str(),
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled("/", cursor_style),
        ]);
        Paragraph::new(text).block(block)
    }
}

impl<'parent> ui::MainLayout {
    pub fn new(parent: Rect) -> Self {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(parent);

        ui::MainLayout {
            sidebar: ui::SideBar::new(layout[0]),
            middle_section: ui::MiddleLayout::new(layout[1]),
        }
    }
}

impl<'parent> ui::MiddleLayout {
    pub fn new(parent: Rect) -> Self {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(parent);

        ui::MiddleLayout {
            layout: layout[0],
            bottom: ui::MiddleBottom::new(layout[1]),
        }
    }

    pub fn get_music_container(state: &'parent ui::State) -> (Table<'parent>, TableState) {
        let mut tb_state = TableState::default();
        let block = match state.active {
            ui::Window::Musicbar => {
                tb_state.select(Some(0));
                Block::active("Music ".to_owned())
            }
            _ => {
                tb_state.select(None);
                Block::new("Music ".to_owned())
            }
        };

        let data_list = &state.musicbar;
        let items: Vec<Row> = data_list
            .iter()
            .map(|music| {
                Row::new(vec![
                    match music.liked {
                        true => HEART_FILLED,
                        false => HEART_OUTLINE,
                    },
                    &music.name,
                    &music.artist,
                    &music.duration,
                ])
            })
            .collect();
        let table = Table::new(items)
            .header(
                Row::new(vec!["", "Title", "Artist", "Length"])
                    .style(Style::default().fg(Color::LightYellow)),
            )
            .widths(
                [
                    Constraint::Percentage(5),
                    Constraint::Percentage(50),
                    Constraint::Percentage(30),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .column_spacing(2)
            .style(Style::list_idle())
            .highlight_style(Style::list_hilight())
            .block(block);

        (table, tb_state)
    }
}

impl<'parent> ui::MiddleBottom {
    pub fn new(parent: Rect) -> Self {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(parent);

        ui::MiddleBottom {
            layout: [layout[0], layout[1]],
        }
    }

    pub fn get_playlist_container(state: &'parent ui::State) -> (List<'parent>, ListState) {
        let mut list_state = ListState::default();
        let block = match state.active {
            ui::Window::Playlistbar => {
                list_state.select(Some(0));
                Block::active("Playlist ".to_owned())
            }
            _ => {
                list_state.select(None);
                Block::new("Playlist ".to_owned())
            }
        };
        let data_list = &state.playlistbar;
        let artist_list = List::new(
            data_list
                .iter()
                .map(|playlist| {
                    let text = Spans::from(vec![
                        Span::styled(
                            format!("[{}] ", &playlist.video_count),
                            Style::list_idle().fg(Color::LightYellow),
                        ),
                        Span::styled(&playlist.name, Style::list_idle()),
                        Span::styled(" by ", Style::list_hilight()),
                        Span::styled(&playlist.author, Style::list_idle().fg(Color::LightYellow)),
                    ]);
                    ListItem::new(text)
                })
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(Style::list_hilight())
        .block(block);

        (artist_list, list_state)
    }

    pub fn get_artist_container(state: &'parent ui::State) -> (List<'parent>, ListState) {
        let mut list_state = ListState::default();
        let block = match state.active {
            ui::Window::Artistbar => {
                list_state.select(Some(0));
                Block::active("Artist ".to_owned())
            }
            _ => {
                list_state.select(None);
                Block::new("Artist ".to_owned())
            }
        };
        let data_list = &state.artistbar;
        let playlist_list = List::new(
            data_list
                .iter()
                .map(|playlist| ListItem::new(Span::styled(&playlist.name, Style::list_idle())))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(Style::list_hilight())
        .block(block);

        (playlist_list, list_state)
    }
}

impl<'parent> ui::SideBar {
    pub fn new(parent: Rect) -> Self {
        let layout = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .split(parent);

        ui::SideBar { layout: layout[0] }
    }

    pub fn get_shortcuts(state: &ui::State) -> List<'parent> {
        let block = match state.active {
            ui::Window::Sidebar => Block::active("Shortcut ".to_owned()),
            _ => Block::new("Shortcut ".to_owned()),
        };
        List::new(
            SIDEBAR_LIST_ITEMS
                .iter()
                .map(|v| ListItem::new(Span::styled(*v, Style::list_idle().fg(Color::LightGreen))))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(Style::list_hilight())
        .block(block)
    }
}

impl<'parent> ui::BottomLayout {
    pub fn new(parent: Rect) -> Self {
        let layout = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .split(parent)[0];

        ui::BottomLayout { layout }
    }

    pub fn get_controller(state: &'pare