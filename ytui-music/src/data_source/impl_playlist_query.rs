use crate::data_sink::DataSink;
use std::{borrow::Borrow, sync::Arc};
use stream_mandu::StreamMandu;

impl super::DataSource {
    pub(super) async fn apply_playlist_query(
        &mut self,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        playlist_index: usize,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let remote_source = Arc::clone(&self.remote_source);
        let mut new_fetch_future = tokio::task::spawn(async move {
            Self::playlist_query_inner(
                data_sink,
                remote_source.borrow(),
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
        remote_source: &dyn StreamMandu,
        ui_notifier: &std::sync::Condvar,
        playlist_index: usize,
    ) {
        let Some(playlist_id) = Self::with_unlocked_mutex(data_sink.clone(), |data_sink| {
            data_sink
                .playlist_list
                .as_ref()
                .map(|playlist_list| {
                    let selected_playlist = DataSink::at_or_last(playlist_list, playlist_index)?;
                    Some(selected_playlist.id.clone())
                })
                .ok()
                .flatten()
        })
        .await
        else {
            return;
        };

        let music_items = remote_source
            .get_playlist_info(playlist_id.as_str())
            .await
            .map_err(|err| format!("Id: {playlist_id}. Error: {err}"))
            .map(|playlist| playlist.item_list);

        Self::with_unlocked_mutex(data_sink, |data_dump| {
            data_dump.music_list = music_items;
            data_dump.mark_new_data_arrival();
        })
        .await;

        ui_notifier.notify_one();
    }
}
