pub mod types;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[async_trait::async_trait]
pub trait StreamMandu: Send + Sync {
    async fn get_music_info(&self, music_id: &str) -> Result<types::MusicInfo>;

    async fn get_playlist_info(&self, playist_id: &str) -> Result<types::PlaylistInfo>;

    async fn get_artist_info(&self, artist_id: &str) -> Result<types::ArtistInfo>;

    async fn search_query(
        &self,
        query: &str,
        include_music: bool,
        include_playlist: bool,
        include_artists: bool,
    ) -> Result<types::SearchResults>;

    async fn fetch_trending_music(
        &self,
        region: Option<super::region::IsoRegion>,
    ) -> Result<Vec<types::MusicInfo>>;
}
