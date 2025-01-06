use super::{WebClient, WebResponse};
pub use reqwest;
use reqwest::header::HeaderName;

#[async_trait::async_trait]
impl WebClient for reqwest::Client {
    type WebError = reqwest::Error;

    async fn request_binary(&self, url: &str) -> Result<WebResponse, Self::WebError> {
        let req = self
            .get(url)
            .header(HeaderName::from_static("content-type"), "application/json")
            .build()?;

        let response = self.execute(req).await?;

        Ok(WebResponse {
            status_code: response.status().as_u16(),
            body: response.bytes().await?.to_vec(),
        })
    }
}
