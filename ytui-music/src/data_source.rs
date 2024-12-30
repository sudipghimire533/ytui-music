use std::{
    borrow::Cow,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Duration,
};

use ytui_audio::libmpv::LibmpvPlayer;
use ytui_invidious::invidious::{
    requests::{self, InvidiousApiQuery},
    types::{
        channel::SearchChannelUnit,
        common::{SearchResult, SearchResults},
        playlists::SearchPlaylistUnit,
        video::SearchVideoUnit,
    },
    web_client::reqwest_impl::reqwest,
    InvidiousBackend,
};

#[derive(Default)]
pub struct DataSink {
    music_search_list: Vec<SearchVideoUnit>,
    playlist_search_list: Vec<SearchPlaylistUnit>,
    artist_search_list: Vec<SearchChannelUnit>,
}

pub struct DataSource {
    player: LibmpvPlayer,
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
}

impl DataSource {
    pub async fn new() -> Self {
        Self {
            player: LibmpvPlayer::new().unwrap(),
            invidious: Arc::new(InvidiousBackend::new(
                "https://invidious.jing.rocks/api/v1".to_string(),
            )),
            source_action_queue: Arc::new(tokio::sync::Mutex::new(SourceAction::default())),
            search_invidious_handle: tokio::task::spawn(async {}),
            reqwest: Arc::new(reqwest::Client::new()),
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
            let mut queued_action = SourceAction::default();
            {
                let mut unlocked_queued_action = self.source_action_queue.lock().await;
                std::mem::swap(&mut queued_action, &mut *unlocked_queued_action);
            }

            if queued_action.should_quit {
                self.abort_all_task().await;
                break;
            }

            self.finish_pending_task(
                queued_action,
                data_dump.clone(),
                ui_renderer_notifier.clone(),
            )
            .await;
        }
    }

    pub async fn finish_pending_task(
        &mut self,
        source_action: SourceAction,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
        ui_renderer_notifier: Arc<std::sync::Condvar>,
    ) {
        if let Some(search_query) = source_action.search_query {
            self.apply_search_query(data_dump, search_query, Arc::clone(&ui_renderer_notifier))
                .await;
        }

        if let Some(playlist_query) = source_action.fetch_playlist {
            self.apply_playlist_query(data_dump, playlist_query, Arc::clone(&ui_renderer_notifier))
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
        let (invidious, reqwest, data_dump) = self.get_invidious(data_dump);
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
                .unwrap();

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
            Self::with_unlocked_data_dump(data_dump, move |data_dump| {
                std::mem::swap(&mut data_dump.music_search_list, &mut music_results);
                std::mem::swap(&mut data_dump.playlist_search_list, &mut playlist_results);
                std::mem::swap(&mut data_dump.artist_search_list, &mut artist_results);
            })
            .await;

            // render newly fetch results
            ui_notifier.notify_one();
        });

        std::mem::swap(&mut self.search_invidious_handle, &mut new_search_future);
        new_search_future.abort();
    }

    fn get_invidious(
        &self,
        data_dump: Arc<tokio::sync::Mutex<DataSink>>,
    ) -> (
        Arc<InvidiousBackend>,
        Arc<reqwest::Client>,
        Arc<tokio::sync::Mutex<DataSink>>,
    ) {
        (
            Arc::clone(&self.invidious),
            Arc::clone(&self.reqwest),
            Arc::clone(&data_dump),
        )
    }

    async fn abort_all_task(&self) {
        self.search_invidious_handle.abort();
    }

    async fn with_unlocked_data_dump<R>(
        locked_data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        mut action: impl FnMut(&mut DataSink) -> R,
    ) -> R {
        let mut unlocked_data_sink = locked_data_sink.lock().await;
        action(&mut unlocked_data_sink)
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
}

/// Allow UI side to retrive data
impl ytui_ui::DataGetter for DataSink {
    fn get_playlist_list(&self) -> &[&str; 2] {
        panic!()
    }

    fn get_artist_list(&self) -> &[&str] {
        panic!()
    }

    fn get_music_list(&self) -> impl Iterator<Item = [std::borrow::Cow<'_, str>; 3]> {
        self.music_search_list.iter().map(|search_result| {
            [
                Cow::Borrowed(search_result.title.as_str()),
                Cow::Borrowed(search_result.author.as_str()),
                Cow::Owned(format!(
                    "{:02}:{:02}",
                    search_result.length_seconds / 60,
                    search_result.length_seconds % 60
                )),
            ]
        })
    }
}
