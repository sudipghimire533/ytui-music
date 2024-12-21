use super::common;
use serde::Deserialize;

/*
* ImageAttachment
{
    "type": "image",
    "imageThumbnails": ImageObject[]
}
*/
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAttachment {
    #[serde(rename = "image")]
    o_type: String,
    image_thumbnails: Vec<common::ImageObject>,
}
/*
* MultiImageAttachment
*{
    "type": "multiImage",
    "images": ImageObject[][]
}
*/
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiImageAttachment {
    #[serde(rename = "type")]
    o_type: String,
    images: Vec<Vec<common::ImageObject>>,
}

/*
* PoolAttachment
*{
    "type": "poll",
    "totalVotes": Number,
    "choices": {
        "text": String,
        "image?": ImageObject[]
    }[]

}
*/
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollAttachment {
    #[serde(rename = "type")]
    o_type: String,
    total_votes: i64,
    choices: Vec<PoolChoice>,
}

/*
PoolChoice
    {
        "text": String,
        "image?": ImageObject[]
    }
*/
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolChoice {
    pub text: String,
    pub image: Option<Vec<common::ImageObject>>,
}

/*
* Unknown
* {
*   type: "unknown",
*   "error": String
* }
*/
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownAttachment {
    #[serde(rename = "type")]
    pub o_type: String,
    pub error: String,
}

pub type VideoAttachment = common::VideoObject;
pub type PlaylistAttachment = common::PlaylistObject;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Attachment {
    VideoAttachment(VideoAttachment),
    ImageAttachment(ImageAttachment),
    MultiImageAttachment(MultiImageAttachment),
    PollAttachment(PollAttachment),
    PlaylistAttachment(PlaylistAttachment),
    UnknownAttachment(UnknownAttachment),
}
