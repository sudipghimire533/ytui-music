use super::{ArtistList, MusicList, PlaylistList};

impl ytui_ui::DataGetter for super::DataSink {
    fn get_playlist_list(&self) -> Vec<[String; 2]> {
        match self.playlist_list {
            PlaylistList::Error(ref error_message) => {
                vec![[
                    String::from("Error: ") + error_message.as_str(),
                    String::from("@sudipghimire533"),
                ]]
            }
            PlaylistList::SearchResult(ref playlist_search_list) => playlist_search_list
                .iter()
                .map(|search_result| [search_result.title.clone(), search_result.author.clone()])
                .collect(),
        }
    }

    fn get_artist_list(&self) -> Vec<String> {
        match self.artist_list {
            ArtistList::Error(ref error_message) => {
                vec![String::from("Error: ") + error_message.as_str()]
            }
            ArtistList::SearchResult(ref artist_search_list) => artist_search_list
                .iter()
                .map(|search_result| search_result.author.clone())
                .collect(),
        }
    }

    fn get_music_list(&self) -> Vec<[String; 3]> {
        let format_second_text = |seconds: i32| format!("{:02}:{:02}", seconds / 60, seconds % 60);

        match self.music_list {
            MusicList::Error(ref error_message) => {
                vec![[
                    String::from("Error: ") + error_message.as_str(),
                    String::from("@sudipghimire533"),
                    String::from("NaN / NaN"),
                ]]
            }
            MusicList::SearchResult(ref music_list) => music_list
                .iter()
                .map(|music_unit| {
                    [
                        music_unit.title.clone(),
                        music_unit.author.clone(),
                        format_second_text(music_unit.length_seconds),
                    ]
                })
                .collect(),
            MusicList::FetchedFromPlaylist(ref music_list) => music_list
                .iter()
                .map(|music_unit| {
                    [
                        music_unit.title.clone(),
                        music_unit.author.clone(),
                        format_second_text(music_unit.length_seconds),
                    ]
                })
                .collect(),
            MusicList::Trending(ref music_list) => music_list
                .iter()
                .map(|music_unit| {
                    [
                        music_unit.title.clone(),
                        music_unit.author.clone(),
                        format_second_text(music_unit.length_seconds),
                    ]
                })
                .collect(),
        }
    }

    fn mark_consumed_new_data(&mut self) {
        self.has_new_data = false;
    }

    fn has_new_data(&self) -> bool {
        self.has_new_data
    }
}
