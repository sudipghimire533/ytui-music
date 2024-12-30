pub mod types;
pub mod web_client;

use crate::ensure;
use requests::*;

pub mod requests {
    use super::types;

    pub type RequestStats<'a> = _ApiRequestst<'a, types::api_info::UsageStats>;
    pub type RequestVideoById<'a> = _ApiRequestst<'a, types::video::VideoInfo>;
    pub type RequestTrending<'a> = _ApiRequestst<'a, types::video::TrendingVideos>;
    pub type RequestSearch<'a> = _ApiRequestst<'a, types::common::SearchResults>;

    pub enum InvidiousApiQuery<'a> {
        Stats,
        VideoById {
            video_id: &'a str,
        },
        Trending {
            region: types::region::IsoRegion,
        },
        Search {
            query: String,
            find_playlist: bool,
            find_artist: bool,
            find_music: bool,
        },
    }

    pub struct _ApiRequestst<'a, ExpectedOutResponse> {
        param: InvidiousApiQuery<'a>,

        _out_phantom: std::marker::PhantomData<ExpectedOutResponse>,
        // todo: global options
    }
    impl<'a, Res> _ApiRequestst<'a, Res> {
        pub fn new(param: InvidiousApiQuery<'a>) -> Self {
            Self {
                param,

                _out_phantom: std::marker::PhantomData,
            }
        }

        pub(super) fn get_endpoint_path(&self, mut base_url: String) -> String {
            match self.param {
                InvidiousApiQuery::Stats => {
                    base_url.push_str("/api/v1/stats");
                }
                InvidiousApiQuery::VideoById { video_id } => {
                    base_url.push_str(format!("/api/v1/videos/{video_id}").as_str());
                }
                InvidiousApiQuery::Trending { region } => {
                    base_url.push_str(format!("/trending?region={}", region.as_str()).as_str());
                }

                InvidiousApiQuery::Search {
                    ref query,
                    find_playlist,
                    find_artist,
                    find_music,
                } => {
                    let result_type = if !find_music && find_artist && find_playlist {
                        "&type=all"
                    } else if find_music {
                        "&type=music"
                    } else if find_playlist {
                        "&type=playlist"
                    } else if find_artist {
                        "&type=channel"
                    } else {
                        "&type=all"
                    };

                    base_url.push_str("/search?");
                    base_url.push_str("q=");
                    base_url.push_str(query);
                    base_url.push_str(result_type);
                }

                _ => todo!(),
            };

            base_url
        }
    }
}

pub struct InvidiousBackend {
    pub(crate) base_url: String,
}

impl InvidiousBackend {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn fetch_endpoint<WebC: web_client::WebClient, ExpectedResponse>(
        &self,
        web_client: WebC,
        request: _ApiRequestst<'_, ExpectedResponse>,
    ) -> Result<ExpectedResponse, EndpointFetchError<WebC::WebError>>
    where
        ExpectedResponse: serde::de::DeserializeOwned,
    {
        let endpoint_path = request.get_endpoint_path(self.base_url.clone());
        let web_response = web_client
            .request_binary(endpoint_path.as_str())
            .await
            .map_err(EndpointFetchError::WebClientError)?;

        ensure!(
            (200..300).contains(&web_response.status_code),
            EndpointFetchError::NonOkWebResponse
        );

        // if response is deserialized into SimpleError ( have serde::deny_unknwon_fields )
        // it's a error from api
        if let Ok(error_response) =
            serde_json::from_slice::<types::common::SimpleError>(web_response.body.as_slice())
        {
            Err(EndpointFetchError::ApiError(error_response.error))?
        }

        let response = serde_json::from_slice::<ExpectedResponse>(web_response.body.as_slice())
            .map_err(|err| EndpointFetchError::ResponseDeserilizationError {
                err,
                content: web_response.body,
            })?;

        Ok(response)
    }
}

#[derive(Debug)]
pub enum EndpointFetchError<WebE> {
    NonOkWebResponse,
    WebClientError(WebE),
    ApiError(String),
    InvalidJsonResponse {
        content: Vec<u8>,
    },
    ResponseDeserilizationError {
        err: serde_json::error::Error,
        content: Vec<u8>,
    },
}
