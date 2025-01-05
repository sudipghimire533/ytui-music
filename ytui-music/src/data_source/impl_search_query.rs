use crate::data_sink::{ArtistList, DataSink, MusicList, PlaylistList};
use std::borrow::Borrow;
use std::sync::Arc;
use ytui_invidious::invidious::{
    requests::{self, InvidiousApiQuery},
    types::common::{SearchResult, SearchResults},
    web_client::reqwest_impl::reqwest,
    InvidiousBackend,
};

impl super::DataSource {
    pub(super) async fn apply_search_query(
        &mut self,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        search_query: (String, bool, bool, bool),
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let invidious = Arc::clone(&self.invidious);
        let reqwest = Arc::clone(&self.reqwest);

        let mut new_search_future = tokio::task::spawn(async move {
            Self::perform_search_inner(
                search_query,
                invidious.borrow(),
                reqwest.borrow(),
                ui_notifier.borrow(),
                data_sink,
            )
            .await
        });

        std::mem::swap(&mut self.search_invidious_handle, &mut new_search_future);
        new_search_future.abort();
    }

    async fn perform_search_inner(
        search_query: (String, bool, bool, bool),
        invidious: &InvidiousBackend,
        reqwest: &reqwest::Client,
        ui_notifier: &std::sync::Condvar,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
    ) {
        let search_query = requests::RequestSearch::new(InvidiousApiQuery::Search {
            query: search_query.0,
            find_playlist: search_query.1,
            find_artist: search_query.2,
            find_music: search_query.3,
        });
        let search_results = invidious
            .fetch_endpoint(reqwest, search_query)
            .await
            .map(Self::classify_search_result)
            .unwrap_or_else(|err| {
                let error_as_str = err.as_error_string();
                (
                    MusicList::Error(error_as_str.clone()),
                    PlaylistList::Error(error_as_str.clone()),
                    ArtistList::Error(error_as_str),
                )
            });

        Self::with_unlocked_mutex(data_sink, move |data_sink| {
            data_sink.music_list = search_results.0;
            data_sink.playlist_list = search_results.1;
            data_sink.artist_list = search_results.2;
            data_sink.mark_new_data_arrival();
        })
        .await;

        ui_notifier.notify_one();
    }

    fn classify_search_result(
        search_results: SearchResults,
    ) -> (MusicList, PlaylistList, ArtistList) {
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
    }
}
