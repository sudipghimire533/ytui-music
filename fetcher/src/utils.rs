use crate::{Fetcher, ReturnAction};
use config::initilize::{
    CONFIG, STORAGE, TB_FAVOURATES_ARTIST, TB_FAVOURATES_MUSIC, TB_FAVOURATES_PLAYLIST,
};
use reqwest;
use std::iter::DoubleEndedIterator;
use std::time::Duration;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.131 Safari/537.36";
const FIELDS: [&str; 3] = [
    "videoId,title,author,lengthSeconds",
    "title,playlistId,author,videoCount",
    "author,authorId,videoCount",
];
const FILTER_TYPE: [&str; 3] = ["music", "playlist", "channel"];

impl crate::ExtendDuration for Duration {
    fn to_string(self) -> String {
        let (hr, min, sec) = {
            let mut remaining_seconds = self.as_secs();
            let hr = remaining_seconds / 3600;
            remaining_seconds = remaining_seconds % 3600;
            let min = remaining_seconds / 60;
            let sec = remaining_seconds % 60;

            (hr, min, sec)
        };

        if hr > 0 {
            format!("{}:{:02}:{:02}", hr, min, sec)
        } else {
            format!("{:02}:{:02}", min, sec)
        }
    }

    // This function assumes that the string is alwayd formatted in "min:secs"
    fn from_string(inp: &str) -> Duration {
        let mut time_components = inp.split(':');

        let seconds = time_components
            .next_back()
            .unwrap_or("0")
            .trim()
            .parse::<u64>()
            .unwrap();
        let minutes = time_components
            .next_back()
            .unwrap_or("0")
            .trim()
            .parse::<u64>()
            .unwrap();
        let hours = time_components
            .next_back()
            .unwrap_or("0")
            .trim()
            .parse::<u64>()
            .unwrap();

        let total_secs = seconds + (minutes * 60) + (hours * 60 * 60);
        Duration::from_secs(total_secs)
    }
}

impl Default for Fetcher {
    fn default() -> Self {
        super::Fetcher {
            trending_now: None,
            playlist_content: super::PlaylistRes::default(),
            artist_content: super::ArtistRes::default(),
            search_res: super::SearchRes::default(),
            servers: &CONFIG.servers.list,
            client: reqwest::ClientBuilder::default()
                .user_agent(USER_AGENT)
                .gzip(true)
                .build()
                .unwrap(),
            active_server_index: 0,
            region: &CONFIG.constants.region,
            item_per_page: CONFIG.constants.item_per_list,
        }
    }
}

macro_rules! search {
    ("music", $fetcher: expr, $query: expr, $page: expr) => {
        search!(
            "@internal-core",
            $fetcher,
            $query,
            $page,
            $fetcher.search_res.music,
            0,
            super::MusicUnit
        )
    };
    ("playlist", $fetcher: expr, $query: expr, $page: expr) => {
        search!(
            "@internal-core",
            $fetcher,
            $query,
            $page,
            $fetcher.search_res.playlist,
            1,
            super::PlaylistUnit
        )
    };
    ("artist", $fetcher: expr, $query: expr, $page: expr) => {
        search!(
            "@internal-core",
            $fetcher,
            $query,
            $page,
            $fetcher.search_res.artist,
            2,
            super::ArtistUnit
        )
    };

    ("@internal-core", $fetcher: expr, $query: expr, $page: expr, $store_target: expr, $filter_index: expr, $unit_type: ty) => {{
        let suffix = format!(
            "/search?q={query}&type={s_type}&{region}&page={page}&fields={fields}",
            query = $query,
            s_type = FILTER_TYPE[$filter_index],
            region = $fetcher.region,
            fields = FIELDS[$filter_index],
            page = $page
        );
        let lower_limit = $page * $fetcher.item_per_page;
        let mut upper_limit =
            std::cmp::min($store_target.len(), lower_limit + $fetcher.item_per_page);

        let is_new_query = *$query != $fetcher.search_res.query;
        let is_new_type = $fetcher.search_res.last_fetched != $filter_index;
        let insufficient_data =
            upper_limit.checked_sub(lower_limit).unwrap_or(0) < $fetcher.item_per_page;

        $fetcher.search_res.last_fetched = $filter_index;
        if is_new_query || insufficient_data || is_new_type {
            let obj = $fetcher.send_request::<Vec<$unit_type>>(&suffix, 1).await;
            if is_new_query || is_new_type {
                $store_target.clear();
            }
            match obj {
                Ok(data) => {
                    $fetcher.search_res.query = $query.to_string();
                    $store_target.extend_from_slice(data.as_slice());
                    upper_limit =
                        std::cmp::min($store_target.len(), lower_limit + $fetcher.item_per_page);
                }
                Err(e) => return Err(e),
            }
        }

        if upper_limit > lower_limit {
            Ok($store_target[lower_limit..upper_limit].to_vec())
        } else {
            Err(ReturnAction::EOR)
        }
    }};
}

impl Fetcher {
    pub fn change_server(&mut self) {
        self.active_server_index = (self.active_server_index + 1) % self.servers.len();
    }

    // All the request should be send from this function
    async fn send_request<'de, Res>(
        &mut self,
        path: &str,
        retry_for: i32,
    ) -> Result<Res, ReturnAction>
    where
        Res: serde::de::DeserializeOwned,
    {
        self.change_server();

        let url = self.servers[self.active_server_index].to_string() + path;
        let res = self.client.get(url).send().await;

        match res {
            Ok(response) => {
                if let Ok(obj) = response.json::<Res>().await {
                    Ok(obj)
                } else {
                    Err(ReturnAction::Failed)
                }
            }
            Err(_) if retry_for > 0 => {
                self.change_server();
                Err(ReturnAction::Retry)
            }
            Err(_) => Err(ReturnAction::Failed),
        }
    }

    pub async fn get_trending_music(
        &mut self,
        page: usize,
    ) -> Result<Vec<super::MusicUnit>, ReturnAction> {
        let lower_limit = self.item_per_page * page;

        if self.trending_now.is_none() {
            let suffix = format!(
                "/trending?type=Music&region={region}&fields={music_field}",
                region = self.region,
                music_field = FIELDS[0]
            );

            let obj = self.send_request::<Vec<super::MusicUnit>>(&suffix, 2).await;
            match obj {
                Ok(mut res) => {
                    res.shrink_to_fit();
                    self.trending_now = Some(res);
                }
                Err(e) => return Err(e),
            }
        }

        let trending_now = self.trending_now.as_ref().unwrap();
        let upper_limit = std::cmp::min(trending_now.len(), lower_limit + self.item_per_page);

        if lower_limit >= upper_limit {
            Err(ReturnAction::EOR)
        } else {
            Ok(trending_now[lower_limit..upper_limit].to_vec())
        }
    }

    pub async fn get_playlist_content(
        &mut self,
        playlist_id: &str,
        page: usize,
    ) -> Result<Vec<super::MusicUnit>, ReturnAction> {
        let lower_limit = page * self.item_per_page;

        let is_new_id = *playlist_id != self.playlist_content.id;
        if is_new_id {
            self.playlist_content.id = playlist_id.to_string();
            let suffix = format!(
                "/playlists/{playlist_id}?fields=videos({music_field})",
                playlist_id = playlist_id,
                music_field = FIELDS[0]
            );

            let obj = self
                .send_request::<super::FetchPlaylistContentRes>(&suffix, 1)
                .await;
            match obj {
                Ok(mut data) => {
                    data.videos.shrink_to_fit();
                    self.playlist_content.music = data.videos;
                }
                Err(e) => return Err(e),
            }
        }

        let upper_limit = std::cmp::min(
            self.playlist_content.music.len(),
            lower_limit + self.item_per_page,
        );
        if lower_limit >= upper_limit {
            Err(ReturnAction::EOR)
        } else {
            let mut res = self.playlist_content.music[lower_limit..upper_limit].to_vec();
            res.shrink_to_fit();
            Ok(res)
        }
    }

    pub async fn get_playlist_of_channel(
        &mut self,
        channel_id: &str,
        page: usize,
    ) -> Result<Vec<super::PlaylistUnit>, ReturnAction> {
        let lower_limit = page * self.item_per_page;

        let is_new_id = *channel_id != self.artist_content.playlist.0;
        if is_new_id || self.artist_content.playlist.1.is_empty() {
            self.artist_content.playlist.0 = channel_id.to_string();
            let suffix = format!(
                "/channels/{channel_id}/playlists?fields=playlists({channel_fields})",
                channel_id = channel_id,
                channel_fields = FIELDS[1],
            );

            let obj = self
                .send_request::<super::FetchArtistPlaylist>(&suffix, 1)
                .await;
            match obj {
                Ok(mut data) => {
                    data.playlists.shrink_to_fit();
                    self.artist_content.playlist.1 = data.playlists;
                }
                Err(e) => return Err(e),
            }
        }

        let upper_limit = std::cmp::min(
            self.artist_content.playlist.1.len(),
            lower_limit + self.item_per_page,
        );
        if lower_limit >= upper_limit {
            Err(ReturnAction::EOR)
        } else {
            let mut res = self.artist_content.playlist.1[lower_limit..upper_limit].to_vec();
            res.shrink_to_fit();
            Ok(res)
        }
    }

    pub async fn get_videos_of_channel(
        &mut self,
        channel_id: &str,
        page: usize,
    ) -> Result<Vec<super::MusicUnit>, ReturnAction> {
        let lower_limit = page * self.item_per_page;

        let is_new_id = *channel_id != self.artist_content.music.0;
        if is_new_id || self.artist_content.music.1.is_empty() {
            self.artist_content.music.0 = channel_id.to_string();
            let suffix = format!(
                "/channels/{channel_id}/videos&fields={music_field}",
                channel_id = channel_id,
                music_field = FIELDS[0]
            );

            let obj = self.send_request::<Vec<super::MusicUnit>>(&suffix, 1).await;
            match obj {
                Ok(mut data) => {
                    data.shrink_to_fit();
                    self.artist_content.music.1 = data;
                }
                Err(e) => return Err(e),
            }
        }

        let upper_limit = std::cmp::min(
            self.artist_content.music.1.len(),
            lower_limit + self.item_per_page,
        );
        if lower_limit >= upper_limit {
            Err(ReturnAction::EOR)
        } else {
            let mut res = self.artist_content.music.1[lower_limit..upper_limit].to_vec();
            res.shrink_to_fit();
            Ok(res)
        }
    }

    pub async fn get_favourates_music(
        &mut self,
        page: usize,
    ) -> Result<Vec<super::MusicUnit>, ReturnAction> {
        let lower_limit = page * self.item_per_page;
        let conn = STORAGE.lock().unwrap();

        let query = format!(
            "
            SELECT
            id, title, author, duration
            FROM {tb_name}
            LIMIT {from}, {count}
        ",
            tb_name = TB_FAVOURATES_MUSIC,
            from = lower_limit,
            count = self.item_per_page,
        );

        let mut stmt = match conn.prepare(&query) {
            Ok(val) => val,
            Err(err) => {
                eprintln!(
                    "Error preparing select statement for favourates music. Error: {err}",
                    err = err
                );
                return Err(ReturnAction::Failed);
            }
        };

        let results = stmt.query_map([], |row| {
            Ok(super::MusicUnit {
                id: row.get(0).unwrap_or_default(),
                name: row.get(1).unwrap_or("SQL_ERROR".into()),
                artist: row.get(2).unwrap_or("SQL_ERROR".into()),
                duration: row.get(3).unwrap_or("3:0".into()),
            })
        });

        let res = match results {
            Err(err) => {
                eprintln!(
                    "Cannot get results of favourates music. Error: {err}",
                    err = err
                );
                return Err(ReturnAction::Failed);
            }
            Ok(results) => {
                let mut return_res: Vec<super::MusicUnit> = Vec::with_capacity(self.item_per_page);
                for music in results {
                    return_res.push(music.unwrap());
                }

                return_res
            }
        };

        if res.is_empty() {
            return Err(ReturnAction::EOR);
        }

        Ok(res)
    }

    pub async fn get_favourates_playlist(
        &mut self,
        page: usize,
    ) -> Result<Vec<super::PlaylistUnit>, ReturnAction> {
        let lower_limit = page * self.item_per_page;
        let conn = STORAGE.lock().unwrap();

        let query = format!(
            "
            SELECT
            id, name, author, count
            FROM {tb_name}
            LIMIT {from}, {count}
        ",
            tb_name = TB_FAVOURATES_PLAYLIST,
            from = lower_limit,
            count = self.item_per_page,
        );

        let mut stmt = match conn.prepare(&query) {
            Ok(val) => val,
            Err(err) => {
                eprintln!(
                    "Error preparing select statement for favourates playlist. Error: {err}",
                    err = err
                );
                return Err(ReturnAction::Failed);
            }
        };

        let results = stmt.query_map([], |row| {
            Ok(super::PlaylistUnit {
                id: row.get(0).unwrap_or_default(),
                name: row.get(1).unwrap_or("SQL_ERROR".into()),
                author: row.get(2).unwrap_or("SQL_ERROR".into()),
                video_count: row.get(3).unwrap_or("NaN".into()),
            })
        });

        let res = match results {
            Err(err) => {
                eprintln!(
                    "Cannot get results of favourates music. Error: {err}",
                    err = err
                );
                return Err(ReturnAction::Failed);
            }
            Ok(results) => {
                let mut return_res: Vec<super::PlaylistUnit> =
                    Vec::with_capacity(self.item_per_page);
                for playlist in results {
                    return_res.push(playlist.unwrap());
                }

                return_res
            }
        };

        if res.is_empty() {
            return Err(ReturnAction::EOR);
        }

        Ok(res)
    }

    pub async fn get_favourates_artist(
        &mut self,
        page: usize,
    ) -> Result<Vec<super::ArtistUnit>, ReturnAction> {
        let lower_limit = page * self.item_per_page;
        let conn = STORAGE.lock().unwrap();

        let query = format!(
            "
            SELECT
            id, name, count
            FROM {tb_name}
            LIMIT {from}, {count}
        ",
            tb_name = TB_FAVOURATES_ARTIST,
            from = lower_limit,
            count = self.item_per_page,
        );

        let mut stmt = match conn.prepare(&query) {
            Ok(val) => val,
            Err(err) => {
                eprintln!(
                    "Error preparing select statement for favourates artist. Error: {err}",
                    err = err
                );
                return Err(ReturnAction::Failed);
            }
        };

        let results = stmt.query_map([], |row| {
            Ok(super::ArtistUnit {
                id: row.get(0).unwrap_or_default(),
                name: row.get(1).unwrap_or("SQL_ERROR".into()),
                video_count: row.get(2).unwrap_or("NaN".into()),
            })
        });

        let res = match results {
            Err(err) => {
                eprintln!(
                    "Cannot get results of favourates artist. Error: {err}",
                    err = err
                );
                return Err(ReturnAction::Failed);
            }
            Ok(results) => {
                let mut return_res: Vec<super::ArtistUnit> = Vec::with_capacity(self.item_per_page);
                for artist in results {
                    return_res.push(artist.unwrap());
                }

                return_res
            }
        };

        if res.is_empty() {
            return Err(ReturnAction::EOR);
        }

        Ok(res)
    }

    pub async fn search_music(
        &mut self,
        query: &str,
        page: usize,
    ) -> Result<Vec<super::MusicUnit>, ReturnAction> {
        search!("music", self, query, page)
    }

    pub async fn search_playlist(
        &mut self,
        query: &str,
        page: usize,
    ) -> Result<Vec<super::PlaylistUnit>, ReturnAction> {
        search!("playlist", self, query, page)
    }

    pub async fn search_artist(
        &mut self,
        query: &str,
        page: usize,
    ) -> Result<Vec<super::ArtistUnit>, ReturnAction> {
        search!("artist", self, query, page)
    }
}
