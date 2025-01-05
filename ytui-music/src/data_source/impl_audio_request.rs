use super::DataSink;
use std::sync::Arc;

impl super::DataSource {
    const MUSIC_URL_PREFIX: &str = "https://youtube.com/watch?v=";

    pub(super) async fn play_from_music_pane(
        &self,
        music_index: usize,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
    ) {
        let music_id_to_stream = Self::with_unlocked_mutex(data_sink, |data_sink| {
            data_sink
                .music_id_at_index_or_last(music_index)
                .map(str::to_string)
        })
        .await;

        let Some(music_id) = music_id_to_stream else {
            return;
        };
        let music_url = String::from(Self::MUSIC_URL_PREFIX) + music_id.as_str();
        self.player.load_uri(music_url.as_str()).unwrap();
    }

    pub(super) async fn toggle_pause_status(&self) {
        self.player.cycle_pause_status().unwrap();
    }
}
