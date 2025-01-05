use std::sync::Arc;
use ytui_audio::libmpv::LibmpvPlayer;
use ytui_invidious::invidious::{
    self,
    requests::{self, InvidiousApiQuery},
    types::{
        channel::SearchChannelUnit,
        common::SearchResult,
        playlists::{PlaylistVideoUnit, SearchPlaylistUnit},
        region,
        video::{SearchVideoUnit, TrendingVideos},
    },
    web_client::reqwest_impl::reqwest,
    InvidiousBackend,
};

pub struct DataSink {
    pub(crate) player: Arc<LibmpvPlayer>,
    music_list: MusicList,
    playlist_list: PlaylistList,
    artist_list: ArtistList,

    has_new_data: bool,
}

impl Default for DataSink {
    fn default() -> Self {
        let mpv_player = LibmpvPlayer::new().unwrap();
        // by-default only stream audio
        mpv_player.disable_video().unwrap();

        Self {
            player: Arc::new(mpv_player),
            music_list: MusicList::SearchResult(Default::default()),
            playlist_list: PlaylistList::SearchResult(Default::default()),
            artist_list: ArtistList::SearchResult(Default::default()),
            has_new_data: false,
        }
    }
}

pub struct DataSource {
    player: Arc<LibmpvPlayer>,
    invidious: Arc<InvidiousBackend>,

    reqwest: Arc<reqwest::Client>,

    pub(super) source_action_queue: Arc<tokio::sync::Mutex<SourceAction>>,
    // handles that should be drooped if another task of same kind is spawned.
    // example: if user input new search term, we can abort processing previous search term
    search_invidious_handle: tokio::task::JoinHandle<()>,
    fetch_playlist_handle: tokio::task::JoinHandle<()>,
    fetch_artist_handle: tokio::task::JoinHandle<()>,
    fetch_trending_handle: tokio::task::JoinHandle<()>,
}

#[derive(Default)]
pub struct SourceAction {
    /// if true, exit fetcher loop
    pub(crate) should_quit: bool,

    /// .0: query to execute search on
    /// .1: include music result
    /// .2: include playlist results
    /// .3: include artists results
    search_query: Option<(String, bool, bool, bool)>,

    /// .0: play music of this index in music results
    music_play_index: Option<usize>,
    /// .0: fetch all the music from this playlist
    playlist_fetch_index: Option<usize>,
    /// .0: fetch all playlist and music from this artist
    artist_fetch_index: Option<usize>,
    /// Fetch trending music
    fetch_trending_music: Option<()>,

    /// .0: weather to pause or to resume
    pause_playback_toggle: Option<()>,
}

impl DataSource {
    pub async fn new(mpv_player: Arc<LibmpvPlayer>) -> Self {
        let invidious_backend =
            InvidiousBackend::new("https://invidious.jing.rocks/api/v1".to_string());
        let source_action = SourceAction::default();
        let reqwest_client = reqwest::Client::new();
        let noop_tokio_task = || tokio::task::spawn(async {});

        Self {
            player: mpv_player,
            reqwest: Arc::new(reqwest_client),
            invidious: Arc::new(invidious_backend),
            source_action_queue: Arc::new(tokio::sync::Mutex::new(source_action)),
            search_invidious_handle: noop_tokio_task(),
            fetch_playlist_handle: noop_tokio_task(),
            fetch_artist_handle: noop_tokio_task(),
            fetch_trending_handle: noop_tokio_task(),
        }
    }
}

impl DataSource {
    pub async fn start_data_sourcer_loop(
        mut self,
        source_listener: Arc<tokio::sync::Notify>,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        ui_renderer_notifier: Arc<std::sync::Condvar>,
    ) {
        loop {
            source_listener.notified().await;

            // extract action needed to be performed,
            // self.source_action_queue is not locked afterwards ( while performing the actual
            // action as it will involve network requests )
            //
            // swap with SourceAction::default() as all of this action should be performed only
            // once and can be aborted if new action is requested
            let mut queued_action = SourceAction::default();
            Self::with_unlocked_mutex(self.source_action_queue.clone(), |source_action| {
                std::mem::swap(&mut queued_action, source_action);
            })
            .await;

            if queued_action.should_quit {
                self.abort_all_task().await;
                break;
            }

            self.finish_pending_tasks(
                queued_action,
                data_dump.clone(),
                ui_renderer_notifier.clone(),
            )
            .await;
        }
    }

    pub async fn finish_pending_tasks(
        &mut self,
        source_action: SourceAction,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        ui_renderer_notifier: Arc<std::sync::Condvar>,
    ) {
        if let Some(music_index) = source_action.music_play_index {
            self.play_from_music_pane(music_index, Arc::clone(&data_dump))
                .await;
            ui_renderer_notifier.notify_one();
        } else if let Some(()) = source_action.pause_playback_toggle {
            self.toggle_pause_status().await;
            ui_renderer_notifier.notify_one();
        }

        if let Some(playlist_index) = source_action.playlist_fetch_index {
            self.apply_playlist_query(data_dump, playlist_index, ui_renderer_notifier)
                .await;
        } else if let Some(artist_index) = source_action.artist_fetch_index {
            self.apply_artist_query(data_dump, artist_index, ui_renderer_notifier)
                .await;
        } else if let Some(search_query) = source_action.search_query {
            self.apply_search_query(data_dump, search_query, ui_renderer_notifier)
                .await;
        } else if let Some(()) = source_action.fetch_trending_music {
            self.apply_trending_query(data_dump, region::IsoRegion::NP, ui_renderer_notifier)
                .await;
        }
    }

    async fn apply_trending_query(
        &mut self,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        region: region::IsoRegion,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let invidious = Arc::clone(&self.invidious);
        let reqwest = Arc::clone(&self.reqwest);
        let mut new_fetch_future = tokio::task::spawn(async move {
            let fetch_query =
                requests::RequestTrending::new(InvidiousApiQuery::Trending { region });
            let trending_results = invidious
                .fetch_endpoint(reqwest.as_ref(), fetch_query)
                .await
                .map_err(|e| e.as_error_string())
                .map(MusicList::Trending)
                .unwrap_or_else(MusicList::Error);

            Self::with_unlocked_mutex(data_dump, |data_dump| {
                data_dump.music_list = trending_results;
                data_dump.mark_new_data_arrival();
            })
            .await;

            ui_notifier.notify_one();
        });

        std::mem::swap(&mut self.fetch_trending_handle, &mut new_fetch_future);
        new_fetch_future.abort();
    }

    async fn apply_playlist_query(
        &mut self,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        playlist_index: usize,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let invidious = Arc::clone(&self.invidious);
        let reqwest = Arc::clone(&self.reqwest);
        let mut new_fetch_future = tokio::task::spawn(async move {
            let playlist_id_to_fetch = Self::with_unlocked_mutex(data_dump.clone(), |data_dump| {
                data_dump
                    .playlist_id_at_index_or_last(playlist_index)
                    .map(str::to_string)
            })
            .await;

            let Some(playlist_id) = playlist_id_to_fetch else {
                return;
            };

            let fetch_query = requests::RequestPlaylistById::new(InvidiousApiQuery::PlaylistById {
                playlist_id: playlist_id.as_str(),
            });
            let playlist_result = invidious
                .fetch_endpoint(reqwest.as_ref(), fetch_query)
                .await
                .map_err(|e| e.as_error_string())
                .map(|fetched_res| MusicList::FetchedFromPlaylist(fetched_res.videos))
                .unwrap_or_else(MusicList::Error);

            Self::with_unlocked_mutex(data_dump, |data_dump| {
                data_dump.music_list = playlist_result;
                data_dump.mark_new_data_arrival();
            })
            .await;

            ui_notifier.notify_one();
        });

        std::mem::swap(&mut self.fetch_playlist_handle, &mut new_fetch_future);
        new_fetch_future.abort();
    }

    async fn apply_artist_query(
        &mut self,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        artist_index: usize,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
    }

    async fn apply_search_query(
        &mut self,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        search_query: (String, bool, bool, bool),
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let invidious = Arc::clone(&self.invidious);
        let reqwest = Arc::clone(&self.reqwest);
        let mut new_search_future = tokio::task::spawn(async move {
            // todo: remove current search result from database
            // to hide the result of outdated query?

            // make request
            let search_query = requests::RequestSearch::new(InvidiousApiQuery::Search {
                query: search_query.0,
                find_playlist: search_query.1,
                find_artist: search_query.2,
                find_music: search_query.3,
            });
            let search_results = invidious
                .fetch_endpoint(reqwest.as_ref(), search_query)
                .await
                .map_err(|e| e.as_error_string())
                .map(|search_results| {
                    // classify search results into music/ channel and artist
                    let mut music_results = Vec::new();
                    let mut playlist_results = Vec::new();
                    let mut artist_results = Vec::new();
                    for search_result in search_results {
                        match search_result {
                            SearchResult::Video(search_video_unit) => {
                                music_results.push(search_video_unit);
                            }
                            SearchResult::Playlist(search_playlist_unit) => {
                                playlist_results.push(search_playlist_unit);
                            }
                            SearchResult::Channel(search_channel_unit) => {
                                artist_results.push(search_channel_unit);
                            }
                        }
                    }

                    (
                        MusicList::SearchResult(music_results),
                        PlaylistList::SearchResult(playlist_results),
                        ArtistList::SearchResult(artist_results),
                    )
                })
                .unwrap_or_else(|error_as_str| {
                    (
                        MusicList::Error(error_as_str.clone()),
                        PlaylistList::Error(error_as_str.clone()),
                        ArtistList::Error(error_as_str),
                    )
                });

            // fill result
            Self::with_unlocked_mutex(data_dump, move |data_dump| {
                data_dump.music_list = search_results.0;
                data_dump.playlist_list = search_results.1;
                data_dump.artist_list = search_results.2;
                data_dump.mark_new_data_arrival();
            })
            .await;

            ui_notifier.notify_one();
        });

        std::mem::swap(&mut self.search_invidious_handle, &mut new_search_future);
        new_search_future.abort();
    }

    async fn play_from_music_pane(
        &self,
        music_index: usize,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
    ) {
        const MUSIC_ID_PREFIX: &str = "https://youtube.com/watch?v=";

        let music_id_to_stream = Self::with_unlocked_mutex(data_dump, |data_dump| {
            data_dump
                .music_id_at_index_or_last(music_index)
                .map(str::to_string)
        })
        .await;

        let Some(music_id) = music_id_to_stream else {
            return;
        };
        let music_url = String::from(MUSIC_ID_PREFIX) + music_id.as_str();
        self.player.load_uri(music_url.as_str()).unwrap();
    }

    async fn toggle_pause_status(&self) {
        self.player.cycle_pause_status().unwrap();
    }

    async fn abort_all_task(&self) {
        self.search_invidious_handle.abort();
        self.fetch_playlist_handle.abort();
        self.fetch_artist_handle.abort();
        self.fetch_trending_handle.abort();
    }

    async fn with_unlocked_mutex<LockedData, ReturnData>(
        locked_data: Arc<tokio::sync::Mutex<LockedData>>,
        action: impl FnOnce(&mut LockedData) -> ReturnData,
    ) -> ReturnData {
        let mut unlocked_data = locked_data.lock().await;
        action(&mut unlocked_data)
    }
}

/// Allow ui side to request data
impl ytui_ui::DataRequester for SourceAction {
    fn search_new_term(
        &mut self,
        term: String,
        search_for_music: bool,
        search_for_playlist: bool,
        search_for_artist: bool,
    ) {
        self.search_query = Some((
            term,
            search_for_music,
            search_for_playlist,
            search_for_artist,
        ))
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn play_from_music_pane(&mut self, selected_index: usize) {
        self.music_play_index = Some(selected_index);
    }

    fn fetch_from_artist_pane(&mut self, selected_index: usize) {
        self.playlist_fetch_index = Some(selected_index);
    }

    fn fetch_from_playlist_pane(&mut self, selected_index: usize) {
        self.playlist_fetch_index = Some(selected_index)
    }

    fn fetch_trending_music(&mut self) {
        self.fetch_trending_music = Some(());
    }

    fn toggle_pause_playback(&mut self) {
        self.pause_playback_toggle = Some(());
    }
}

/// Allow UI side to retrive data
impl ytui_ui::DataGetter for DataSink {
    fn get_playlist_list(&self) -> Vec<[String; 2]> {
        match self.playlist_list {
            PlaylistList::Error(ref error_message) => {
                vec![[
                    String::from("Error: ") + error_message.as_str(),
                    String::from("@sudipghimire533"),
                ]]
            }
            PlaylistList::SearchResult(ref playlist_search_list) => playlist_search_list
                .iter()
                .map(|search_result| [search_result.title.clone(), search_result.author.clone()])
                .collect(),
        }
    }

    fn get_artist_list(&self) -> Vec<String> {
        match self.artist_list {
            ArtistList::Error(ref error_message) => {
                vec![String::from("Error: ") + error_message.as_str()]
            }
            ArtistList::SearchResult(ref artist_search_list) => artist_search_list
                .iter()
                .map(|search_result| search_result.author.clone())
                .collect(),
        }
    }

    fn get_music_list(&self) -> Vec<[String; 3]> {
        let format_second_text = |seconds: i32| format!("{:02}:{:02}", seconds / 60, seconds % 60);

        match self.music_list {
            MusicList::Error(ref error_message) => {
                vec![[
                    String::from("Error: ") + error_message.as_str(),
                    String::from("@sudipghimire533"),
                    String::from("NaN / NaN"),
                ]]
            }
            MusicList::SearchResult(ref music_list) => music_list
                .iter()
                .map(|music_unit| {
                    [
                        music_unit.title.clone(),
                        music_unit.author.clone(),
                        format_second_text(music_unit.length_seconds),
                    ]
                })
                .collect(),
            MusicList::FetchedFromPlaylist(ref music_list) => music_list
                .iter()
                .map(|music_unit| {
                    [
                        music_unit.title.clone(),
                        music_unit.author.clone(),
                        format_second_text(music_unit.length_seconds),
                    ]
                })
                .collect(),
            MusicList::Trending(ref music_list) => music_list
                .iter()
                .map(|music_unit| {
                    [
                        music_unit.title.clone(),
                        music_unit.author.clone(),
                        format_second_text(music_unit.length_seconds),
                    ]
                })
                .collect(),
        }
    }

    fn mark_consumed_new_data(&mut self) {
        self.has_new_data = false;
    }

    fn has_new_data(&self) -> bool {
        self.has_new_data
    }
}

impl DataSink {
    fn mark_new_data_arrival(&mut self) {
        self.has_new_data = true;
    }

    fn music_id_at_index_or_last(&self, index: usize) -> Option<&str> {
        match &self.music_list {
            MusicList::Error(_error_message) => None,

            MusicList::SearchResult(music_list) => music_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(music_list.last())
                .map(|music| music.video_id.as_str()),
            MusicList::FetchedFromPlaylist(music_list) => music_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(music_list.last())
                .map(|music| music.video_id.as_str()),
            MusicList::Trending(music_list) => music_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(music_list.last())
                .map(|music| music.video_id.as_str()),
        }
    }

    fn playlist_id_at_index_or_last(&self, index: usize) -> Option<&str> {
        match &self.playlist_list {
            PlaylistList::SearchResult(playlist_list) => playlist_list
                .get(index)
                .map(Option::Some)
                .unwrap_or(playlist_list.last())
                .map(|playlist| playlist.playlist_id.as_str()),
            PlaylistList::Error(_error_message) => None,
        }
    }
}

#[derive(Debug)]
pub enum MusicList {
    Error(String),
    SearchResult(Vec<SearchVideoUnit>),
    FetchedFromPlaylist(Vec<PlaylistVideoUnit>),
    Trending(TrendingVideos),
}

pub enum PlaylistList {
    Error(String),
    SearchResult(Vec<SearchPlaylistUnit>),
}

pub enum ArtistList {
    Error(String),
    SearchResult(Vec<SearchChannelUnit>),
}
