impl ytui_ui::DataGetter for super::DataSink {
    fn get_playlist_list(&self) -> Vec<[String; 2]> {
        match self.playlist_list {
            Err(ref error_message) => {
                vec![[
                    String::from("Error: ") + error_message.as_str(),
                    String::from("@sudipghimire533"),
                ]]
            }
            Ok(ref playlist_search_list) => playlist_search_list
                .iter()
                .map(|search_result| {
                    [
                        search_result
                            .title
                            .clone()
                            .unwrap_or(String::from("ERR: NO TITLE")),
                        search_result
                            .author_name
                            .clone()
                            .unwrap_or(String::from("ERR: NO AUTHOR NAME")),
                    ]
                })
                .collect(),
        }
    }

    fn get_artist_list(&self) -> Vec<String> {
        match self.artist_list {
            Err(ref error_message) => {
                vec![String::from("Error: ") + error_message.as_str()]
            }
            Ok(ref artist_search_list) => artist_search_list
                .iter()
                .map(|search_result| {
                    search_result
                        .name
                        .clone()
                        .unwrap_or(String::from("ERR: NO NAME"))
                })
                .collect(),
        }
    }

    fn get_music_list(&self) -> Vec<[String; 3]> {
        match self.music_list {
            Err(ref error_message) => {
                vec![[
                    String::from("Error: ") + error_message.as_str(),
                    String::from("@sudipghimire533"),
                    String::from("NaN / NaN"),
                ]]
            }
            Ok(ref music_list) => music_list
                .iter()
                .map(|music_unit| {
                    let title = music_unit
                        .title
                        .clone()
                        .unwrap_or(String::from("ERR: NO TITLE"));
                    let author = music_unit
                        .author_name
                        .clone()
                        .unwrap_or(String::from("@sudipghimire533"));
                    let duration = music_unit
                        .length
                        .map(|duration| {
                            let seconds = duration.as_secs();
                            format!("{:02}: {:02}", seconds / 60, seconds % 60)
                        })
                        .unwrap_or(String::from("NaN/ NaN"));

                    [title, author, duration]
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

    fn startup_overlay_announcement(&self) -> Option<(String, String)> {
        super::app_announcement::AnnouncementInfo::fetch_startup_announcement()
            .map(|announcement| announcement.get_title_and_body())
    }
}
