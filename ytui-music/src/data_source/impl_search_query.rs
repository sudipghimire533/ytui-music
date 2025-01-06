use super::DataSink;
use std::borrow::Borrow;
use std::sync::Arc;
use stream_mandu::StreamMandu;

impl super::DataSource {
    pub(super) async fn apply_search_query(
        &mut self,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        search_query: (String, bool, bool, bool),
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let remote_source = Arc::clone(&self.remote_source);

        let mut new_search_future = tokio::task::spawn(async move {
            Self::perform_search_inner(
                search_query,
                remote_source.borrow(),
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
        remote_source: &dyn StreamMandu,
        ui_notifier: &std::sync::Condvar,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
    ) {
        let search_results = remote_source
            .search_query(
                search_query.0.as_str(),
                search_query.1,
                search_query.2,
                search_query.3,
            )
            .await
            .map_err(|err| format!("Search error: {err}"));

        Self::with_unlocked_mutex(data_sink, move |data_sink| {
            match search_results {
                Ok(search_results) => {
                    data_sink.music_list = Ok(search_results.musics);
                    data_sink.playlist_list = Ok(search_results.playlists);
                    data_sink.artist_list = Ok(search_results.artists);
                }
                Err(error_message) => {
                    data_sink.music_list = Err(error_message.clone());
                    data_sink.playlist_list = Err(error_message.clone());
                    data_sink.artist_list = Err(error_message);
                }
            }

            data_sink.mark_new_data_arrival();
        })
        .await;

        ui_notifier.notify_one();
    }
}
