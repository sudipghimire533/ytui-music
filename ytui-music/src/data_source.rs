use std::{borrow::Cow, sync::Arc};
use ytui_audio::libmpv::LibmpvPlayer;
use ytui_invidious::invidious::{
    requests::{self, InvidiousApiQuery},
    types::{
        channel::SearchChannelUnit, common::SearchResult, playlists::SearchPlaylistUnit,
        video::SearchVideoUnit,
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

    /// .0: playlist id to fetch music from
    fetch_playlist: Option<String>,

    /// .0: artist id to fetch from
    /// .1: if true, fetch music of this artist
    /// .2: if true, fetch playlist from this artists
    fetch_artist: Option<(String, bool, bool)>,

    /// .0: play music of this index in music results
    music_play_index: Option<usize>,
}

impl DataSource {
    pub async fn new(mpv_player: Arc<LibmpvPlayer>) -> Self {
        let invidious_backend =
            InvidiousBackend::new("https://invidious.jing.rocks/api/v1".to_string());
        let source_action = SourceAction::default();
        let reqwest_client = reqwest::Client::new();
        let noop_tokio_task = tokio::task::spawn(async {});

        Self {
            player: mpv_player,
            //player: mpv_player,
            invidious: Arc::new(invidious_backend),
            source_action_queue: Arc::new(tokio::sync::Mutex::new(source_action)),
            search_invidious_handle: noop_tokio_task,
            reqwest: Arc::new(reqwest_client),
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
        }

        if let Some(search_query) = source_action.search_query {
            self.apply_search_query(
                Arc::clone(&data_dump),
                search_query,
                Arc::clone(&ui_renderer_notifier),
            )
            .await;
        }

        if let Some(playlist_query) = source_action.fetch_playlist {
            self.apply_playlist_query(
                Arc::clone(&data_dump),
                playlist_query,
                Arc::clone(&ui_renderer_notifier),
            )
            .await;
        }

        if let Some(artist_query) = source_action.fetch_artist {
            self.apply_artist_query(data_dump, artist_query, Arc::clone(&ui_renderer_notifier))
                .await;
        }
    }

    async fn apply_playlist_query(
        &mut self,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        playlist_query: String,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
    }

    async fn apply_artist_query(
        &mut self,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        artist_query: (String, bool, bool),
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
            //let search_query = requests::RequestSearch::new(InvidiousApiQuery::Search {
            //    query: search_query.0,
            //    find_playlist: search_query.1,
            //    find_artist: search_query.2,
            //    find_music: search_query.3,
            //});
            //let search_results = invidious
            //    .fetch_endpoint(reqwest.as_ref(), search_query)
            //    .await
            //    .unwrap();
            let search_results = vec![SearchResult::Video(SearchVideoUnit {
                title: "something".to_string(),
                video_id: "NKQQJnBClAg".to_string(),
                author: "sudip".to_string(),
                author_id: Default::default(),
                author_url: Default::default(),
                video_thumbnails: Default::default(),
                description: Default::default(),
                description_html: Default::default(),
                view_count: Default::default(),
                view_count_text: Default::default(),
                published: Default::default(),
                published_text: Default::default(),
                length_seconds: 300,
                live_now: Default::default(),
                paid: Default::default(),
                premium: Default::default(),
            })];

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

            // fill result
            Self::with_unlocked_mutex(data_dump, move |data_dump| {
                data_dump.music_list = MusicList::SearchResult(music_results);
                data_dump.playlist_list = PlaylistList::SearchResult(playlist_results);
                data_dump.artist_list = ArtistList::SearchResult(artist_results);

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

        let mut music_id_to_stream = None;
        Self::with_unlocked_mutex(data_dump, |data_dump| {
            music_id_to_stream = match data_dump.music_list {
                MusicList::SearchResult(ref music_list) => {
                    music_list.get(music_index).map(|m| m.video_id.clone())
                }
            };
        })
        .await;

        let Some(music_id) = music_id_to_stream else {
            return;
        };
        let music_url = String::from(MUSIC_ID_PREFIX) + music_id.as_str();
        self.player.load_uri(music_url.as_str()).unwrap();
    }

    async fn abort_all_task(&self) {
        self.search_invidious_handle.abort();
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
}

/// Allow UI side to retrive data
impl ytui_ui::DataGetter for DataSink {
    fn get_playlist_list(&self) -> &[&str; 2] {
        panic!()
    }

    fn get_artist_list(&self) -> &[&str] {
        panic!()
    }

    fn get_music_list(&self) -> impl Iterator<Item = [String; 3]> {
        match self.music_list {
            MusicList::SearchResult(ref music_search_list) => {
                music_search_list.iter().map(|search_result| {
                    [
                        search_result.title.clone(),
                        search_result.author.clone(),
                        format!(
                            "{:02}:{:02}",
                            search_result.length_seconds / 60,
                            search_result.length_seconds % 60
                        ),
                    ]
                })
            }
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
}

#[derive(Debug)]
pub enum MusicList {
    SearchResult(Vec<SearchVideoUnit>),
}

pub enum PlaylistList {
    SearchResult(Vec<SearchPlaylistUnit>),
}

pub enum ArtistList {
    SearchResult(Vec<SearchChannelUnit>),
}
