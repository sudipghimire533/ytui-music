mod artist_pane;
mod music_pane;
mod navigation_list;
mod overlay;
mod playlist_pane;
mod progressbar;
mod queue_list;
mod searchbar;
mod state_badge;
mod statusbar;
pub mod window_border;

pub use component_collection::ComponentsCollection;
pub use component_collection::ComponentsDataCollection;
pub mod component_collection {
    use super::*;
    use crate::{
        state::{AppState, Pane},
        PlayerStats,
    };
    use ratatui::style::Color;
    use ytui_audio::libmpv::LibmpvPlayer;

    pub struct ComponentsCollection<'a> {
        pub searchbar: searchbar::SearchBar<'a>,
        pub state_badge: state_badge::StateBadge<'a>,
        pub statusbar: statusbar::StatusBar<'a>,
        pub navigation_list: navigation_list::NavigationList<'a>,
        pub queue_list: queue_list::QueueList<'a>,
        pub music_pane: music_pane::MusicPane<'a>,
        pub playlist_pane: playlist_pane::PlaylistPane<'a>,
        pub artist_pane: artist_pane::ArtistPane<'a>,
        pub progressbar: progressbar::ProgressBar<'a>,
        pub overlay: Option<overlay::Overlay<'a>>,
    }

    #[derive(Default, Debug)]
    pub struct ComponentsDataCollection {
        pub music_list: Vec<[String; 3]>,
        pub player_stats: PlayerStats,
    }

    impl ComponentsDataCollection {
        pub fn refresh_from_data_provider<DataProvider>(&mut self, data_provider: &DataProvider)
        where
            DataProvider: crate::DataGetter + std::marker::Send + 'static,
        {
            self.music_list = data_provider.get_music_list().collect();
        }

        pub fn refresh_from_player(&mut self, player: &LibmpvPlayer) {
            self.player_stats = PlayerStats::from_libmpv_player(player)
        }
    }

    impl<'a> ComponentsCollection<'a> {
        pub fn create_progress_bar(
            data_collection: &ComponentsDataCollection,
        ) -> progressbar::ProgressBar<'a> {
            let progress_bar_attrs = progressbar::ProgressBarUiAttrs {
                foreground: Color::Green,
                background: Color::Reset,
            };

            progressbar::ProgressBar::create_widget(&progress_bar_attrs)
                .with_player_stats(&data_collection.player_stats)
        }

        pub fn create_all_components(
            app_state: &AppState,
            data_collection: &ComponentsDataCollection,
        ) -> ComponentsCollection<'a> {
            let progressbar = Self::create_progress_bar(&data_collection);
            let searchbar_attrs = searchbar::SearchBarUiAttrs {
                text_color: Color::Red,
                show_border: true,
                show_only_bottom_border: false,
                is_active: matches!(app_state.selected_pane, Some(Pane::SearchBar)),
            };
            let searchbar = searchbar::SearchBar::create_widget(&searchbar_attrs).with_query(
                app_state
                    .search_query
                    .as_deref()
                    .unwrap_or("Listen to something new today"),
            );

            let status_bar_attrs = statusbar::StatusBarUiAttrs {
                show_border: true,
                repeat_char: "󰑖",
                shuffle_char: "󰒝",
                resume_char: "󰏤",
                volume: 100,
            };
            let statusbar = statusbar::StatusBar::create_widget(&status_bar_attrs);

            let queue_list_attrs = queue_list::QueueListUiAttrs {
                text_color: Color::Green,
                highlight_color: Color::Red,
            };
            let queue_list = queue_list::QueueList::create_widget(&queue_list_attrs).with_list(
                [
                    "Lose control by Teddy Swims",
                    "Greedy by Tate McRae",
                    "Beautiful Things by Benson Boone",
                    "Espresso by Sabrina Carpenter",
                    "Come and take your love by unknwon",
                ]
                .repeat(5)
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            );

            let navigation_list_attrs = navigation_list::NavigationListUiAttrs {
                text_color: Color::Green,
                highlight_color: Color::White,
                is_active: matches!(app_state.selected_pane, Some(Pane::NavigationList)),
            };
            let navigation_list =
                navigation_list::NavigationList::create_widget(&navigation_list_attrs).with_list(
                    [
                        "Trending",
                        "Youtube Community",
                        "Liked Songs",
                        "Saved playlist",
                        "Following",
                        "Search",
                    ]
                    .into_iter()
                    .map(ToString::to_string)
                    .collect(),
                );

            let state_badge_attrs = state_badge::StateBadgeUiAttrs {
                text_color: Color::Yellow,
            };
            let state_badge = state_badge::StateBadge::create_widget(&state_badge_attrs)
                .with_msg("@sudipghimire533");

            let music_pane_attrs = music_pane::MusicPaneUiAttrs {
                title_color: Color::LightCyan,
                text_color: Color::Green,
                highlight_color: Color::White,
                is_active: matches!(app_state.selected_pane, Some(Pane::Music)),
            };
            let music_pane = music_pane::MusicPane::create_widget(
                &music_pane_attrs,
                &data_collection.music_list,
            );

            let playlist_pane_attrs = playlist_pane::PlaylistPaneUiAttrs {
                title_color: Color::LightCyan,
                text_color: Color::Yellow,
                highlight_color: Color::DarkGray,
            };
            let playlist_pane = playlist_pane::PlaylistPane::create_widget(
                &playlist_pane_attrs,
                [[
                    "Smoothing sound and something stupid like that",
                    "mighty nepal",
                ]]
                .repeat(20)
                .into_iter()
                .map(|[a, b]| [a.to_string(), b.to_string()])
                .collect(),
            );

            let artist_pane_attrs = artist_pane::ArtistPaneUiAttrs {
                title_color: Color::LightCyan,
                text_color: Color::White,
                highlight_color: Color::Gray,
            };
            let artist_pane = artist_pane::ArtistPane::create_widget(
                &artist_pane_attrs,
                ["Bartika Eam Rai"]
                    .repeat(20)
                    .into_iter()
                    .map(ToString::to_string)
                    .collect(),
            );

            let mut overlay = None;
            if matches!(app_state.selected_pane, Some(Pane::Overlay)) {
                let overlay_attrs = overlay::OverlayUiAttrs {
                    show_borders: true,
                    title: "Release notes".to_string(),
                };
                let new_overlay = overlay:: Overlay::construct_widget(&overlay_attrs).with_announcement("Installation
NOTE: since the dependency libmpv seems not to be maintained anymore,

you will probably need to build from source in any platform. See section Build From Source below.

Download latest binary from release page. If binary is not available for your platform, head on to build from source

Give it executable permission and from downloaded directory, in shell:

ytui_music run
You may need to jump to Usage Guide".to_string());

                overlay = Some(new_overlay);
            }

            Self {
                searchbar,
                state_badge,
                navigation_list,
                queue_list,
                music_pane,
                playlist_pane,
                artist_pane,
                progressbar,
                overlay,
                statusbar,
            }
        }
    }
}
