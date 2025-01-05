use crate::data_sink::{DataSink, MusicList};
use std::{borrow::Borrow, sync::Arc};
use ytui_invidious::invidious::{
    requests::{self, InvidiousApiQuery},
    web_client::reqwest_impl::reqwest,
    InvidiousBackend,
};

impl super::DataSource {
    pub(super) async fn apply_playlist_query(
        &mut self,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        playlist_index: usize,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let invidious = Arc::clone(&self.invidious);
        let reqwest = Arc::clone(&self.reqwest);
        let mut new_fetch_future = tokio::task::spawn(async move {
            Self::playlist_query_inner(
                data_sink,
                invidious.borrow(),
                reqwest.borrow(),
                ui_notifier.borrow(),
                playlist_index,
            )
            .await
        });

        std::mem::swap(&mut self.fetch_playlist_handle, &mut new_fetch_future);
        new_fetch_future.abort();
    }

    async fn playlist_query_inner(
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        invidious: &InvidiousBackend,
        reqwest: &reqwest::Client,
        ui_notifier: &std::sync::Condvar,
        playlist_index: usize,
    ) {
        let playlist_id_to_fetch = Self::with_unlocked_mutex(data_sink.clone(), |data_sink| {
            data_sink
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
            .fetch_endpoint(reqwest, fetch_query)
            .await
            .map_err(|e| e.as_error_string())
            .map(|fetched_res| MusicList::FetchedFromPlaylist(fetched_res.videos))
            .unwrap_or_else(MusicList::Error);

        Self::with_unlocked_mutex(data_sink, |data_dump| {
            data_dump.music_list = playlist_result;
            data_dump.mark_new_data_arrival();
        })
        .await;

        ui_notifier.notify_one();
    }
}
