use crate::data_sink::{DataSink, MusicList};
use std::{borrow::Borrow, sync::Arc};
use ytui_invidious::invidious::{
    requests::{self, InvidiousApiQuery},
    types::region,
    web_client::reqwest_impl::reqwest,
    InvidiousBackend,
};

impl super::DataSource {
    pub(super) async fn apply_trending_query(
        &mut self,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        region: region::IsoRegion,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
        let invidious = Arc::clone(&self.invidious);
        let reqwest = Arc::clone(&self.reqwest);
        let mut new_fetch_future = tokio::task::spawn(async move {
            Self::perform_trending_inner(
                data_sink,
                invidious.borrow(),
                reqwest.borrow(),
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
        invidious: &InvidiousBackend,
        reqwest: &reqwest::Client,
        ui_notifier: &std::sync::Condvar,
        region: region::IsoRegion,
    ) {
        let fetch_query = requests::RequestTrending::new(InvidiousApiQuery::Trending { region });
        let trending_results = invidious
            .fetch_endpoint(reqwest, fetch_query)
            .await
            .map_err(|e| e.as_error_string())
            .map(MusicList::Trending)
            .unwrap_or_else(MusicList::Error);

        Self::with_unlocked_mutex(data_sink, |data_sink| {
            data_sink.music_list = trending_results;
            data_sink.mark_new_data_arrival();
        })
        .await;

        ui_notifier.notify_one();
    }
}
