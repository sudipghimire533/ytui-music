use super::common;
use serde::Deserialize;

pub type AuthorThumbnail = common::ImageObject;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveFormats {
    pub index: String,
    pub bitrate: String,
    pub init: String,
    pub url: String,
    pub itag: String,
    pub type_: String,
    pub clen: String,
    pub lmt: String,
    pub projection_type: serde_json::Value,
    pub container: String,
    pub encoding: String,
    #[serde(default = "String::new")] // TODO: this field is not present?
    pub quality_label: String,
    #[serde(default = "String::new")] // TODO: this field is not present?
    pub resolution: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatStreams {
    pub url: String,
    pub itag: String,
    pub type_: String,
    pub quality: String,
    pub container: String,
    pub encoding: String,
    pub quality_label: String,
    pub resolution: String,
    pub size: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryBoards {
    pub url: String,
    pub template_url: String,
    pub width: i32,
    pub height: i32,
    pub count: i32,
    pub interval: i32,
    pub storyboard_width: i32,
    pub storyboard_height: i32,
    pub storyboard_count: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedVideo {
    pub video_id: String,
    pub title: String,
    pub video_thumbnails: Vec<common::ThumbnailObject>,
    pub author: String,
    pub length_seconds: i32,
    pub view_count_text: String,
}

/*
VideoInfo

{
  "type": String, // "video"|"published"
  "title": String,
  "videoId": String,
  "videoThumbnails": [
    {
      "quality": String,
      "url": String,
      "width": Int32,
      "height": Int32
    }
  ],
  "storyboards": [
    {
      "url": String,
      "templateUrl": String,
      "width": Int32,
      "height": Int32,
      "count": Int32,
      "interval ": Int32,
      "storyboardWidth": Int32,
      "storyboardHeight": Int32,
      "storyboardCount": Int32
    }
  ],

  "description": String,
  "descriptionHtml": String,
  "published": Int64,
  "publishedText": String,

  "keywords": Array(String),
  "viewCount": Int64,
  "likeCount": Int32,
  "dislikeCount": Int32,

  "paid": Bool,
  "premium": Bool,
  "isFamilyFriendly": Bool,
  "allowedRegions": Array(String),
  "genre": String,
  "genreUrl": String,

  "author": String,
  "authorId": String,
  "authorUrl": String,
  "authorThumbnails": [
    {
      "url": String,
      "width": Int32,
      "height": Int32
    }
  ],

  "subCountText": String,
  "lengthSeconds": Int32,
  "allowRatings": Bool,
  "rating": Float32,
  "isListed": Bool,
  "liveNow": Bool,
  "isPostLiveDvr": Bool,
  "isUpcoming": Bool,
  "dashUrl:" String,
  "premiereTimestamp": Int64?,

  "hlsUrl": String?,
  "adaptiveFormats": [
    {
      "index": String,
      "bitrate": String,
      "init": String,
      "url": String,
      "itag": String,
      "type": String,
      "clen": String,
      "lmt": String,
      "projectionType": Int32,
      "container": String,
      "encoding": String,
      "qualityLabel": String?,
      "resolution": String?
    }
  ],
  "formatStreams": [
    {
      "url": String,
      "itag": String,
      "type": String,
      "quality": String,
      "container": String,
      "encoding": String,
      "qualityLabel": String,
      "resolution": String,
      "size": String
    }
  ],
  "captions": [
    {
      "label": String,
      "languageCode": String,
      "url": String
    }
  ],
  "recommendedVideos": [
    {
      "videoId": String,
      "title": String,
      "videoThumbnails": [
        {
          "quality": String,
          "url": String,
          "width": Int32,
          "height": Int32
        }
      ],
      "author": String,
      "lengthSeconds": Int32,
      "viewCountText": String
    }
  ]
}
*/

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    #[serde(rename = "type")]
    pub o_type: String,
    pub title: String,
    pub video_id: String,
    pub video_thumbnails: Vec<common::ThumbnailObject>,
    pub storyboards: Vec<StoryBoards>,

    pub description: String,
    pub description_html: String,
    pub published: i64,
    pub published_text: String,

    pub keywords: Vec<String>,
    pub view_count: i64,
    pub like_count: i32,
    pub dislike_count: i32,

    pub paid: bool,
    pub premium: bool,
    pub is_family_friendly: bool,
    pub allowed_regions: Vec<String>,
    pub genre: String,
    pub genre_url: String,

    pub author: String,
    pub author_id: String,
    pub author_url: String,
    // ? pub author_verified: bool,
    pub author_thumbnails: Vec<common::ImageObject>,

    pub sub_count_text: String,
    pub length_seconds: i32,
    pub allow_ratings: bool,
    pub rating: f32,
    pub is_listed: bool,
    pub live_now: bool,
    pub is_post_live_dvr: bool,
    pub is_upcoming: bool,
    pub dash_url: String,
    pub premiere_timestamp: Option<i64>,

    pub hls_url: Option<String>,
    pub adaptive_formats: Vec<AdaptiveFormats>,
    pub format_streams: Vec<FormatStreams>,
    pub captions: Vec<common::CaptionUnit>,
    pub recommended_videos: Vec<RecommendedVideo>,
}

/*
* TrendingVideo
[
  {
    "title": String,
    "videoId": String,
    "videoThumbnails": [
      {
        "quality": String,
        "url": String,
        "width": Int32,
        "height": Int32
      }
    ],

    "lengthSeconds": Int32,
    "viewCount": Int64,

    "author": String,
    "authorId": String,
    "authorUrl": String,

    "published": Int64,
    "publishedText": String,
    "description": String,
    "descriptionHtml": String,

    "liveNow": Bool,
    "paid": Bool,
    "premium": Bool
  }
]
*/
pub type TrendingVideos = Vec<TrendingVideo>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingVideo {
    pub title: String,
    pub video_id: String,
    pub video_thumbnails: Vec<common::ThumbnailObject>,

    pub length_seconds: i32,
    pub view_count: i64,

    pub author: String,
    pub author_id: String,
    pub author_url: String,

    pub published: i64,
    pub published_text: String,
    pub description: String,
    pub description_html: String,

    pub live_now: bool,
    pub paid: bool,
    pub premium: bool,
}

/*
* PopularShort
[
    {
        "type": "shortVideo",
        "title": String,
        "videoId": String,
        "videoThumbnails": [
            {
            "quality": String,
            "url": String,
            "width": Int32,
            "height": Int32
            }
        ],

        "lengthSeconds": Int32,
        "viewCount": Int64,

        "author": String,
        "authorId": String,
        "authorUrl": String,

        "published": Int64,
        "publishedText": String
    }
]
*/
pub type PopularVideos = Vec<PopularVideo>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopularVideo {
    #[serde(rename = "type")]
    pub o_type: String,
    pub title: String,
    pub video_id: String,
    pub video_thumbnails: Vec<common::ThumbnailObject>,

    pub length_seconds: i32,
    pub view_count: i64,

    pub author: String,
    pub author_id: String,
    pub author_url: String,

    pub published: i64,
    pub published_text: String,
}

/*
{
    type: "video",
    title: String,
    videoId: String,
    author: String,
    authorId: String,
    authorUrl: String,
    videoThumbnails: [
      {
        quality: String,
        url: String,
        width: Int32,
        height: Int32
      }
    ],
    description: String,
    descriptionHtml: String,
    viewCount: Int64,
    published: Int64,
    publishedText: String,
    lengthSeconds: Int32,
    liveNow: Bool,
    paid: Bool,
    premium: Bool

*/

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchVideoUnit {
    pub title: String,
    pub video_id: String,
    pub author: String,
    pub author_id: String,
    pub author_url: String,
    pub video_thumbnails: Vec<common::ThumbnailObject>,
    pub description: String,
    pub description_html: String,
    pub view_count: i64,
    pub view_count_text: String,
    pub published: i64,
    pub published_text: String,
    pub length_seconds: i32,
    pub live_now: bool,
    #[serde(default)]
    pub paid: bool,
    pub premium: bool,
}

/// Response to get all videos from channel
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelVideos {
    pub videos: Vec<VideoInfo>,
    pub continuation: String,
}
