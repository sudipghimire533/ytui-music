use super::DataSink;
use std::{borrow::Borrow, sync::Arc};
use stream_mandu::{region, StreamMandu};

impl super::DataSource {
    pub(super) async fn apply_trending_query(
        &mut self,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        region: region::IsoRegion,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let remote_source = Arc::clone(&self.remote_source);
        let mut new_fetch_future = tokio::task::spawn(async move {
            Self::perform_trending_inner(
                data_sink,
                remote_source.borrow(),
                ui_notifier.borrow(),
                region,
            )
            .await;
        });

        std::mem::swap(&mut self.fetch_trending_handle, &mut new_fetch_future);
        new_fetch_future.abort();
    }

    async fn perform_trending_inner(
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        remote_source: &dyn StreamMandu,
        ui_notifier: &std::sync::Condvar,
        region: region::IsoRegion,
    ) {
        let trending_results = remote_source
            .fetch_trending_music(Some(region))
            .await
            .map_err(|err| format!("Fetch Trending: {err:?}"));

        Self::with_unlocked_mutex(data_sink, |data_sink| {
            data_sink.music_list = trending_results;
            data_sink.mark_new_data_arrival();
        })
        .await;

        ui_notifier.notify_one();
    }
}
