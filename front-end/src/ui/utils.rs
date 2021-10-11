use crate::ui;
use fetcher::ExtendDuration;
use ui::shared_import::*;

const CURRENT_TITLE_LEN: usize = 70;
pub const SIDEBAR_LIST_COUNT: usize = 5;
pub const SIDEBAR_LIST_ITEMS: [&str; SIDEBAR_LIST_COUNT] = [
    "Trending",
    "Youtube Communinty",
    "Recently Played",
    "Favourates",
    "Search",
];
use crate::CONFIG;

// A helper macro to decode the tuple with three memebers to tui::style::Color::Rgb value
// enum Example {
//  First(i32, i32, i32) => accepts 3 individual value
//  Second((i32, i32, i32)) => accepts single tuple with 3 memebers
// }
// Rgb in tui is defined in fasion of First in above example
macro_rules! rgb {
    ($tuple: expr) => {
        Color::Rgb($tuple.0, $tuple.1, $tuple.2)
    };
}

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

    pub fn get_statusbox(state: &'parent ui::State) -> Paragraph<'parent> {
        Paragraph::new(Span::styled(
            state.status,
            Style::default()
                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
                .fg(rgb!(CONFIG.theme.color_secondary)),
        ))
        .block(Block::new("status".to_owned()))
        .block(Block::new("status".to_owned()))
    }

    pub fn get_searchbox(state: &'parent ui::State) -> Paragraph<'parent> {
        let mut cursor_style = Style::default().fg(rgb!(CONFIG.theme.color_secondary));

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
                    .fg(rgb!(CONFIG.theme.color_primary))
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

    pub fn get_music_container(state: &'parent mut ui::State) -> Table<'parent> {
        let block = match state.active {
            ui::Window::Musicbar => Block::active("Music ".to_owned()),
            _ => {
                state.musicbar.1.select(None);
                Block::new("Music ".to_owned())
            }
        };

        let data_list = &state.musicbar.0;
        let items: Vec<Row> = data_list
            .iter()
            .map(|music| {
                Row::new(vec![
                    music.name.as_str(),
                    music.artist.as_str(),
                    music.duration.as_str(),
                ])
            })
            .collect();
        let table = Table::new(items)
            .header(Row::new(vec!["Title", "Artist", "Length"]).style(Style::list_title()))
            .widths(
                [
                    Constraint::Percentage(55),
                    Constraint::Percentage(30),
                    Constraint::Percentage(15),
                ]
                .as_ref(),
            )
            .column_spacing(2)
            .style(Style::list_idle())
            .highlight_style(Style::list_highlight())
            .block(block);

        table
    }
}

impl<'parent> ui::MiddleBottom {
    pub fn new(parent: Rect) -> Self {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(parent);

        ui::MiddleBottom {
            layout: [layout[0], layout[1]],
        }
    }

    pub fn get_playlist_container(state: &'parent mut ui::State) -> Table<'parent> {
        let block = match state.active {
            ui::Window::Playlistbar => Block::active("Playlist ".to_owned()),
            _ => {
                state.playlistbar.1.select(None);
                Block::new("Playlist ".to_owned())
            }
        };
        let data_list = &state.playlistbar.0;
        let items: Vec<Row> = data_list
            .iter()
            .map(|playlist| {
                Row::new(vec![
                    playlist.video_count.as_str(),
                    playlist.name.as_str(),
                    playlist.author.as_str(),
                ])
            })
            .collect();
        let table = Table::new(items)
            .header(Row::new(vec!["#", "Name", "Creator"]).style(Style::list_title()))
            .widths(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(60),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .column_spacing(1)
            .style(Style::list_idle())
            .highlight_style(Style::list_highlight())
            .block(block);

        table
    }

    pub fn get_artist_container(state: &'parent mut ui::State) -> Table<'parent> {
        let block;
        if state.active == ui::Window::Artistbar {
            block = Block::active("Artist ".to_string());
        } else {
            block = Block::new("Artist ".to_string());
            state.artistbar.1.select(None);
        }
        let data_list = &state.artistbar;
        let items: Vec<Row> = data_list
            .0
            .iter()
            .map(|artist| Row::new(vec![artist.video_count.as_str(), artist.name.as_str()]))
            .collect();
        let table = Table::new(items)
            .header(Row::new(vec!["#", "Name"]).style(Style::list_title()))
            .widths([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
            .column_spacing(1)
            .style(Style::list_idle())
            .highlight_style(Style::list_highlight())
            .block(block);

        table
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
                .map(|v| {
                    ListItem::new(Span::styled(
                        *v,
                        Style::list_idle().fg(rgb!(CONFIG.theme.color_primary)),
                    ))
                })
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(Style::list_highlight())
        .block(block)
    }
}

impl<'parent> ui::BottomLayout {
    pub fn new(parent: Rect) -> Self {
        // 9 columns to show playback behavious (paused, suffle, repeat) 3 col each
        // and +2 for border = 11
        let progressbar_width = parent.width.checked_sub(11).unwrap_or_default();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(progressbar_width),
                Constraint::Length(11),
            ])
            .split(parent);

        ui::BottomLayout {
            layout: [layout[0], layout[1]],
        }
    }

    pub fn get_status_bar(state: &'parent ui::State) -> Gauge<'parent> {
        let content;
        if let Some((name, _)) = &state.bottom.playing {
            content = name.as_str()
        } else {
            content = ">> Play some Music <<"
        };

        let heading = format!(
            "{} / {}",
            state.bottom.music_elapse.to_string(),
            state.bottom.music_duration.to_string()
        );

        let mut block;
        if state.active == ui::Window::BottomControl {
            block = Block::active(heading);
        } else {
            block = Block::new(heading);
        }
        block = block.title_alignment(Alignment::Center);

        let mut ratio =
            state.bottom.music_elapse.as_secs_f64() / state.bottom.music_duration.as_secs_f64();
        if ratio > 1.0 {
            ratio = 1.0
        } else if ratio.is_nan() {
            ratio = 0.0
        }

        Gauge::default()
            .ratio(ratio)
            .gauge_style(Style::default().fg(rgb!(CONFIG.theme.gauge_fill)))
            .label(Span::styled(
                &content[0..std::cmp::min(content.len(), CURRENT_TITLE_LEN)],
                Style::default().fg(rgb!(CONFIG.theme.color_primary)),
            ))
            .block(block)
    }

    pub fn get_icons_set(state: &'parent ui::State) -> Paragraph<'parent> {
        let block = Block::new(String::new());
        let (paused_status, suffle, repeat);

        if let Some((_, false)) = state.bottom.playing {
            // is paused
            paused_status = Span::styled(" _ ", Style::list_idle());
        } else {
            paused_status = Span::styled(" P ", Style::list_highlight());
        }

        if state.playback_behaviour.repeat {
            repeat = Span::styled(" R ", Style::list_highlight());
        } else {
            repeat = Span::styled(" 1 ", Style::list_idle());
        }

        if state.playback_behaviour.shuffle {
            suffle = Span::styled(" S ", Style::list_highlight());
        } else {
            suffle = Span::styled(" _ ", Style::list_idle());
        }

        let paragraph = Paragraph::new(Spans::from(vec![paused_status, suffle, repeat]))
            .alignment(Alignment::Center)
            .block(block);
        paragraph
    }
}

pub trait ExtendBlock<'a> {
    fn new(title: String) -> Self;
    fn active(title: String) -> Self;
}
pub trait ExtendStyle {
    fn list_highlight() -> Style;
    fn block_title() -> Style;
    fn list_idle() -> Style;
    fn list_title() -> Style;
}

impl ExtendStyle for Style {
    #[inline(always)]
    fn list_highlight() -> Style {
        Style::default().fg(rgb!(CONFIG.theme.list_hilight))
    }

    #[inline(always)]
    fn list_title() -> Style {
        Style::default().fg(rgb!(CONFIG.theme.color_secondary))
    }

    #[inline(always)]
    fn block_title() -> Style {
        Style {
            fg: Some(rgb!(CONFIG.theme.block_title)),
            bg: None,
            add_modifier: Modifier::BOLD | Modifier::ITALIC,
            sub_modifier: Modifier::empty(),
        }
    }

    #[inline(always)]
    fn list_idle() -> Style {
        Style {
            fg: Some(rgb!(CONFIG.theme.list_idle)),
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
            .border_style(Style::default().fg(rgb!(CONFIG.theme.border_idle)))
            .borders(Borders::ALL)
    }
    fn active(title: String) -> Self {
        Block::default()
            .title(Span::styled(
                title,
                Style::block_title().fg(rgb!(CONFIG.theme.border_highlight)),
            ))
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(rgb!(CONFIG.theme.border_highlight)))
            .borders(Borders::ALL)
    }
}

impl ui::Position {
    pub fn caclulate(screen_rect: &Rect) -> Self {
        // 3 line for each bottom and top bar (1 for content and 2 for border)
        // remaining height for middlebar
        let for_middle = screen_rect.height.checked_sub(3 + 3).unwrap_or_default();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(for_middle),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(*screen_rect);

        let top_section = ui::TopLayout::new(main_layout[0]);
        let main_section = ui::MainLayout::new(main_layout[1]);
        let bottom_section = ui::BottomLayout::new(main_layout[2]);
        let sidebar = main_section.sidebar;
        let middle_section = main_section.middle_section;
        let middle_bottom = middle_section.bottom;

        ui::Position {
            search: top_section.layout[0],
            status: top_section.layout[1],
            shortcut: sidebar.layout,
            music: middle_section.layout,
            playlist: middle_bottom.layout[0],
            artist: middle_bottom.layout[1],
            music_info: bottom_section.layout[0],
            bottom_icons: bottom_section.layout[1],
        }
    }
}

impl Default for ui::State<'_> {
    fn default() -> Self {
        let mpv = libmpv::Mpv::new().unwrap();
        mpv.configure_defult();
        mpv.cache_for(10);
        // By default repeat the playlist. Set playlist to repeat
        mpv.repeat_playlist();

        let mut sidebar_list_state = ListState::default();
        sidebar_list_state.select(Some(0));
        ui::State {
            status: "@sudipghimire533",
            sidebar: sidebar_list_state,
            musicbar: (Vec::new(), TableState::default()),
            playlistbar: (Vec::new(), TableState::default()),
            artistbar: (Vec::new(), TableState::default()),
            search: (String::new(), String::new()),
            active: ui::Window::Sidebar,
            fetched_page: [None; 3],
            filled_source: (
                ui::MusicbarSource::RecentlyPlayed,
                ui::PlaylistbarSource::RecentlyPlayed,
                ui::ArtistbarSource::RecentlyPlayed,
            ),
            bottom: ui::BottomState {
                playing: None,
                music_duration: Duration::new(0, 0),
                music_elapse: Duration::new(0, 0),
            },
            player: mpv,
            playback_behaviour: ui::PlaybackBehaviour {
                shuffle: false,
                repeat: true,
            },
        }
    }
}

pub trait ExtendMpv {
    fn configure_defult(&self);
    fn repeat_playlist(&self);
    fn repeat_one(&self);
    fn repeat_nothing(&self);
    fn shuffle(&self);
    fn unshuffle(&self);
    fn cache_for(&self, time: i64);
    fn play_next(&self);
    fn play_prev(&self);
}

impl ExtendMpv for libmpv::Mpv {
    fn configure_defult(&self) {
        let config_dir = config::ConfigContainer::get_config_dir().unwrap();

        self.set_property("config-dir", config_dir.to_str().unwrap())
            .unwrap();
        let mpv_config_path = config_dir.join(config::MPV_OPTION_FILE_NAME);
        self.set_property("include", mpv_config_path.to_str().unwrap())
            .unwrap();

        // Video is always hidden. Override config file
        self.set_property("video", "no").unwrap();
    }

    #[inline(always)]
    fn shuffle(&self) {
        self.command("playlist-shuffle", &[]).ok();
    }

    #[inline(always)]
    fn unshuffle(&self) {
        self.command("playlist-unshuffle", &[]).ok();
    }

    #[inline(always)]
    fn cache_for(&self, time: i64) {
        self.set_property("cache-secs", time).ok();
    }

    #[inline(always)]
    fn repeat_playlist(&self) {
        self.set_property("loop-playlist", "inf").unwrap();
    }

    fn repeat_nothing(&self) {
        self.set_property("loop-playlist", "no").unwrap();
        self.set_property("loop-file", "no").unwrap();
    }

    #[inline(always)]
    fn repeat_one(&self) {
        self.set_property("loop-file", "inf").unwrap();
    }

    #[inline(always)]
    fn play_next(&self) {
        self.playlist_next_weak().unwrap();
    }

    #[inline(always)]
    fn play_prev(&self) {
        self.playlist_previous_weak().unwrap();
    }
}

impl ui::State<'_> {
    pub fn play_music(&mut self, music_id: &str) {
        self.player.unpause().ok();
        match self.player.command(
            "loadfile",
            [format!("https://www.youtube.com/watch?v={}", music_id).as_str()].as_ref(),
        ) {
            Ok(_) => {
                // clear any previous thing from bottombar
                self.bottom.music_duration = Duration::from_secs(0);
                self.bottom.music_elapse = Duration::from_secs(0);

                self.status = "Playing...";
                // set currently playing (unpaused) to ture. no need to set real title as it will
                // be done by refresh_mpv_status() later on
                self.bottom.playing = Some((String::new(), true))
            }
            Err(_) => self.status = "Playback error..",
        }
        // Now as the selection is being played. Add remaining item from musicbar to the play
        // queue.
        for music in self.musicbar.0.iter() {
            // If this is the currently payed song donot add it to prevent having
            // currently played song two time in queue
            if music.id == *music_id {
                continue;
            }
            self.player
                .command(
                    "loadfile",
                    [
                        format!("https://www.youtube.com/watch?v={}", music.id).as_str(),
                        "append",
                    ]
                    .as_ref(),
                )
                .ok();
        }
    }

    // This function is called when user press enter in non-empty list of playlistbar
    pub fn activate_playlist(&mut self, playlist_id: &str) {
        match self.player.command(
            "loadfile",
            [format!("https://www.youtube.com/playlist?list={}", playlist_id).as_str()].as_ref(),
        ) {
            Ok(_) => {
                // send unpause signal
                self.player.unpause().ok();

                // clear any previous thing from bottombar
                self.bottom.music_duration = Duration::from_secs(0);
                self.bottom.music_elapse = Duration::from_secs(0);

                self.status = "Playing..";
                // set currently playing (unpaused) to ture. no need to set real title as it will
                // be done by refresh_mpv_status() later on
                self.bottom.playing = Some((String::new(), true));
            }
            Err(_) => self.status = "Playback error..",
        }
    }

    // This function can also be used to check playing status
    // Returning true means some music is playing which may be paused or unpaused
    pub fn refresh_mpv_status(&mut self) {
        // It may be better to use wait event method from mpv
        // but for that we need tp spawn seperate thread/task
        // and also we are updating the ui anway so it may also be affordable to just query mpv in
        // ui updating loop
        if let Some((_, true)) = self.bottom.playing {
            match self.player.get_property::<i64>("audio-pts") {
                Ok(time) => {
                    self.bottom.music_elapse = Duration::from_secs(time as u64);
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

            // These will update the status from mpv in real time. This will always show to correct
            // title of the music that is being playing even from playlist so we there is no need
            // to listen to mpv event for playlist index change just to change the title and
            // duration of currently playing music.
            let title = self
                .player
                .get_property::<String>("media-title")
                .unwrap_or(">> Play some music <<".to_string());
            let estimated_duration_reply = self
                .player
                .get_property::<i64>("duration")
                .unwrap_or_default();

            self.bottom.playing = Some((title, true)); // at this scope of match playing status is always true
            self.bottom.music_duration =
                Duration::from_secs(estimated_duration_reply.try_into().unwrap_or_default());
        }
    }

    pub fn toggle_pause(&mut self) {
        if let Some((music_title, is_playing)) = &self.bottom.playing {
            if *is_playing {
                self.player.pause().unwrap();
            } else {
                self.player.unpause().unwrap();
            }
            self.bottom.playing = Some((music_title.to_string(), !is_playing));
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
            ui::Window::Searchbar | ui::Window::Artistbar | ui::Window::BottomControl => {
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
            ui::Window::Searchbar | ui::Window::Sidebar | ui::Window::BottomControl => {
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
            3 => Ok(ui::SidebarOption::Favourates),
            4 => Ok(ui::SidebarOption::Search),
            _ => Err("No sidebar option found corresponding to this usize"),
        }
    }
}
