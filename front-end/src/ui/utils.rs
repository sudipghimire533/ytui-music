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
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
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
                state.search.as_str(),
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
            .column_spacing(1)
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
                .map(|artist| ListItem::new(Span::styled(&artist.name, Style::list_idle())))
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

    pub fn get_controller(state: &'parent ui::State) -> Gauge<'parent> {
        let title = if let Some((title, _)) = &state.bottom.playing {
            title
        } else {
            ">> Play some Music <<"
        };

        let block = Block::new(format!(
            " {} / {} ",
            state.bottom.music_elapse.to_string(),
            &state.bottom.music_duration.to_string()
        ))
        .title_alignment(Alignment::Center);
        let mut ratio =
            state.bottom.music_elapse.as_secs_f64() / state.bottom.music_duration.as_secs_f64();
        ratio = if ratio > 1.0 {
            1.0
        } else if ratio.is_nan() {
            0.0
        } else {
            ratio
        };

        Gauge::default()
            .ratio(ratio)
            .gauge_style(Style::default().fg(Color::DarkGray))
            .label(Span::styled(
                &title[0..std::cmp::min(title.len(), CURRENT_TITLE_LEN)],
                Style::default().fg(Color::White),
            ))
            .block(block)
    }
}

pub trait ExtendBlock<'a> {
    fn new(title: String) -> Self;
    fn active(title: String) -> Self;
}
pub trait ExtendStyle {
    fn list_hilight() -> Style;
    fn block_title() -> Style;
    fn list_idle() -> Style;
}

impl ExtendStyle for Style {
    fn list_hilight() -> Style {
        Style::default().fg(Color::White)
    }
    fn block_title() -> Style {
        Style {
            fg: Some(Color::LightMagenta),
            bg: None,
            add_modifier: Modifier::BOLD | Modifier::ITALIC,
            sub_modifier: Modifier::empty(),
        }
    }
    fn list_idle() -> Style {
        Style {
            fg: Some(Color::Yellow),
            bg: None,
            add_modifier: Modifier::BOLD,
            sub_modifier: Modifier::empty(),
        }
    }
}

impl<'a> ExtendBlock<'a> for Block<'_> {
    fn new(title: String) -> Self {
        Block::default()
            .title(Span::styled(title, Style::block_title()))
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
    }
    fn active(title: String) -> Self {
        Block::default()
            .title(Span::styled(title, Style::block_title().fg(Color::Cyan)))
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .borders(Borders::ALL)
    }
}

impl ui::Position {
    pub fn caclulate(screen_rect: &Rect) -> Self {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Length(10),
                ]
                .as_ref(),
            )
            .split(*screen_rect);

        let (top_section, main_section, bottom_section) = (
            ui::TopLayout::new(layout[0]),
            ui::MainLayout::new(layout[1]),
            ui::BottomLayout::new(layout[2]),
        );
        let sidebar = main_section.sidebar;
        let middle_section = main_section.middle_section;
        let middle_bottom = middle_section.bottom;

        ui::Position {
            search: top_section.layout[0],
            help: top_section.layout[1],
            shortcut: sidebar.layout,
            music: middle_section.layout,
            playlist: middle_bottom.layout[0],
            artist: middle_bottom.layout[1],
            controllers: bottom_section.layout,
        }
    }
}

impl Default for ui::State<'_> {
    fn default() -> Self {
        let mpv = libmpv::Mpv::new().unwrap();
        mpv.set_property("video", "no").unwrap();
        mpv.set_property("cache", "yes").unwrap();
        mpv.set_property("cache-secs", 10).unwrap();
        mpv.set_property("cache-pause-wait", 5).unwrap();
        mpv.set_property("demuxer-readahead-secs", 10).unwrap();
        mpv.set_property("hwdec", "yes").unwrap();
        mpv.set_property("cache-pause-wait", 10).unwrap();
        mpv.set_property("hwdec", "yes").unwrap();
        mpv.set_property("demuxer-cache-wait", "no").unwrap();
        mpv.set_property("cache-on-disk", "yes").unwrap();
        mpv.set_property("ytdl-format", "opus").unwrap();
        mpv.set_property("script-opts", "ytdl_hook-try_ytdl_first=yes")
            .unwrap();

        let mut sidebar_list_state = ListState::default();
        sidebar_list_state.select(Some(0));
        ui::State {
            help: "Press ?",
            sidebar: sidebar_list_state,
            musicbar: VecDeque::new(),
            playlistbar: VecDeque::new(),
            artistbar: VecDeque::new(),
            search: String::new(),
            active: ui::Window::Sidebar,
            fetched_page: [None; 3],
            bottom: ui::BottomState {
                playing: None,
                music_duration: Duration::new(0, 0),
                music_elapse: Duration::new(0, 0),
            },
            player: mpv,
            fetcher: fetcher::Fetcher::new(),
        }
    }
}

impl ui::State<'_> {
    pub fn play_first_of_musicbar(&mut self, notifier: &Arc<std::sync::Condvar>) {
        if let Some(music) = self.musicbar.front() {
            self.help = "Starting..";
            self.bottom.music_duration = Duration::from_secs(0);
            self.bottom.music_elapse = Duration::from_secs(0);
            self.bottom.playing = None;
            notifier.notify_one();

            match self
                .player
                .command("loadfile", [music.path.as_str()].as_ref())
            {
                Ok(_) => {
                    self.bottom.playing = Some((music.name.clone(), true));
                    self.bottom.music_duration = Duration::from_string(music.duration.as_str());
                    self.help = "Press ?";
                }
                Err(_) => {
                    self.help = "Error..";
                    self.bottom.playing = None;
                    self.bottom.music_elapse = Duration::from_secs(0);
                    self.bottom.music_duration = Duration::from_secs(0);
                }
            }
            notifier.notify_one();
        }
    }
    pub fn refresh_time_elapsed(&mut self) -> bool {
        // It may be better to use wait event method from mpv
        // but for that we need tp spawn seperate thread/task
        // and also we are updating the ui anway so it may also be affordable to just query mpv in
        // ui updating loop
        if let Some((_, true)) = self.bottom.playing {
            match self.player.get_property::<i64>("audio-pts") {
                Ok(time) => {
                    self.bottom.music_elapse = Duration::from_secs(time as u64);
                    return true;
                }
                Err(_e) => {
                    // This error is generally expected to be -10 (property exist but not available
                    // at the moment)
                    // which means that the mpv has not yet loaded the file
                    // this depends on the condition of network and amount of task needed to done
                    // by mpv (ususally depends on the length of stream)
                    // eprintln!("Err: {}", e);
                }
            }
        }
        false
    }

    pub fn toggle_pause(&mut self, notifier: &Arc<std::sync::Condvar>) {
        if let Some((music_title, is_playing)) = &self.bottom.playing {
            if *is_playing {
                self.player.pause().unwrap();
            } else {
                self.player.unpause().unwrap();
            }
            // TODO: music_title.clone() feels like heavy and unneeded
            // Fight back the compiler
            self.bottom.playing = Some((music_title.to_string(), !is_playing));
            notifier.notify_one();
        }
    }
}

impl ui::Window {
    /* Any components of top bar and bottombar are not focusable instead directly controlled by the shortcut keys */
    pub fn next(&self) -> ui::Window {
        match self {
            ui::Window::Sidebar => ui::Window::Musicbar,
            ui::Window::Musicbar => ui::Window::Playlistbar,
            ui::Window::Playlistbar => ui::Window::Artistbar,
            ui::Window::Searchbar | ui::Window::Helpbar | ui::Window::Artistbar => {
                ui::Window::Sidebar
            }
            ui::Window::None => unreachable!(),
        }
    }

    pub fn prev(&self) -> ui::Window {
        match self {
            ui::Window::Artistbar => ui::Window::Playlistbar,
            ui::Window::Playlistbar => ui::Window::Musicbar,
            ui::Window::Musicbar => ui::Window::Sidebar,
            ui::Window::Searchbar | ui::Window::Helpbar | ui::Window::Sidebar => {
                ui::Window::Artistbar
            }
            ui::Window::None => unreachable!(),
        }
    }
}

impl std::convert::TryFrom<usize> for ui::SidebarOption {
    type Error = &'static str;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ui::SidebarOption::Trending),
            1 => Ok(ui::SidebarOption::YoutubeCommunity),
            2 => Ok(ui::SidebarOption::RecentlyPlayed),
            3 => Ok(ui::SidebarOption::Followings),
            4 => Ok(ui::SidebarOption::Favourates),
            5 => Ok(ui::SidebarOption::MyPlalist),
            6 => Ok(ui::SidebarOption::Search),
            _ => Err("No sidebar option found corresponding to this usize"),
        }
    }
}
