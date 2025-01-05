pub use super::data_sink::DataSink;
use std::sync::Arc;
use ytui_audio::libmpv::LibmpvPlayer;
use ytui_invidious::invidious::{self, web_client::reqwest_impl::reqwest};

mod impl_artist_query;
mod impl_audio_request;
mod impl_data_requester;
mod impl_playlist_query;
mod impl_search_query;
mod impl_trending_query;

pub struct DataSource {
    player: Arc<LibmpvPlayer>,
    invidious: Arc<invidious::InvidiousBackend>,

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
            invidious::InvidiousBackend::new("https://invidious.jing.rocks/api/v1".to_string());
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
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
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
                data_sink.clone(),
                ui_renderer_notifier.clone(),
            )
            .await;
        }
    }

    pub async fn finish_pending_tasks(
        &mut self,
        source_action: SourceAction,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        ui_renderer_notifier: Arc<std::sync::Condvar>,
    ) {
        if let Some(music_index) = source_action.music_play_index {
            self.play_from_music_pane(music_index, Arc::clone(&data_sink))
                .await;
            ui_renderer_notifier.notify_one();
        } else if let Some(()) = source_action.pause_playback_toggle {
            self.toggle_pause_status().await;
            ui_renderer_notifier.notify_one();
        }

        if let Some(playlist_index) = source_action.playlist_fetch_index {
            self.apply_playlist_query(data_sink, playlist_index, ui_renderer_notifier)
                .await;
        } else if let Some(artist_index) = source_action.artist_fetch_index {
            self.apply_artist_query(data_sink, artist_index, ui_renderer_notifier)
                .await;
        } else if let Some(search_query) = source_action.search_query {
            self.apply_search_query(data_sink, search_query, ui_renderer_notifier)
                .await;
        } else if let Some(()) = source_action.fetch_trending_music {
            self.apply_trending_query(
                data_sink,
                invidious::types::region::IsoRegion::NP,
                ui_renderer_notifier,
            )
            .await;
        }
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
