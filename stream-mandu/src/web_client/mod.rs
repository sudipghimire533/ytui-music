#[cfg(feature = "web-reqwest")]
pub mod async_reqwest_impl;

pub struct WebResponse {
    pub(super) status_code: u16,
    pub(super) body: Vec<u8>,
}

#[async_trait::async_trait]
pub trait WebClient {
    type WebError: std::error::Error;

    async fn request_binary(&self, url: &str) -> Result<WebResponse, Self::WebError>;
}
