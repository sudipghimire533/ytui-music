use super::DataSink;
use std::sync::Arc;

impl super::DataSource {
    const MUSIC_URL_PREFIX: &str = "https://youtube.com/watch?v=";

    pub(super) async fn play_from_music_pane(
        &self,
        music_index: usize,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
    ) {
        let Some(music_id) = Self::with_unlocked_mutex(data_sink, |data_sink| {
            data_sink
                .music_list
                .as_ref()
                .map(|music_list| {
                    DataSink::at_or_last(music_list, music_index).map(|music| music.id.clone())
                })
                .ok()
                .flatten()
        })
        .await
        else {
            return;
        };

        let music_url = String::from(Self::MUSIC_URL_PREFIX) + music_id.as_str();
        self.player.load_uri(music_url.as_str()).unwrap();
    }

    pub(super) async fn toggle_pause_status(&self) {
        self.player.cycle_pause_status().unwrap();
    }
}
