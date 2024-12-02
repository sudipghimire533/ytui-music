use std::future::Future;

pub struct WebResponse {
    pub(super) status_code: u16,
    pub(super) body: Vec<u8>,
}

pub trait WebClient {
    type WebError;

    fn request_binary(
        &self,
        url: &str,
    ) -> impl Future<Output = Result<WebResponse, Self::WebError>>;
}

#[cfg(feature = "web-reqwest")]
mod reqwest_impl {
    use super::{WebClient, WebResponse};
    use reqwest::header::HeaderName;

    impl WebClient for &reqwest::Client {
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
}
