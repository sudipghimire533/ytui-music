use super::DataSink;
use std::sync::Arc;
use ytui_audio::libmpv::{LoadFileArg, MpvCommand};

impl super::DataSource {
    const MUSIC_URL_PREFIX: &str = "https://youtube.com/watch?v=";

    pub(super) async fn play_from_music_pane(
        &self,
        music_index: usize,
        data_sink: Arc<tokio::sync::Mutex<DataSink>>,
    ) {
        Self::with_unlocked_mutex(data_sink, |data_sink| {
            data_sink
                .music_list
                .as_ref()
                .map(|music_list| {
                    let selected_index = std::cmp::min(music_index, music_list.len());

                    // play selected item
                    let selected_stream =
                        Self::make_streamable_url_from_id(&music_list[selected_index].id);
                    self.player
                        .execute_command(MpvCommand::LoadFile {
                            stream: selected_stream.as_str(),
                            kind: LoadFileArg::Replace,
                        })
                        .unwrap();

                    // put next 5 to queue
                    std::iter::repeat_with(|| fastrand::usize(0..music_list.len()))
                        .take(5)
                        .for_each(|checked_index| {
                            let stream_id = music_list[checked_index].id.as_str();
                            let stream_title = music_list[checked_index]
                                .title
                                .as_deref()
                                .unwrap_or(stream_id);
                            self.player
                                .execute_command(MpvCommand::LoadFile {
                                    stream: Self::make_streamable_url_from_id(stream_id).as_str(),
                                    kind: LoadFileArg::Append {
                                        force_media_title: Some(stream_title),
                                    },
                                })
                                .unwrap();
                        });
                })
                .ok()
        })
        .await;
    }

    pub(super) async fn toggle_pause_status(&self) {
        self.player.cycle_pause_status().unwrap();
    }

    #[inline(always)]
    fn make_streamable_url_from_id(id: &str) -> String {
        String::from(Self::MUSIC_URL_PREFIX) + id
    }
}
