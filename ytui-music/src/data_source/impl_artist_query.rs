use super::DataSink;
use std::sync::Arc;

impl super::DataSource {
    pub(super) async fn apply_artist_query(
        &mut self,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
        artist_index: usize,
        ui_notifier: Arc<std::sync::Condvar>,
    ) {
    }
}
