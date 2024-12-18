use super::common;
use serde::Deserialize;

/*
{
    type: "playlist",
    title: String,
    playlistId: String,
    playlistThumbnail: String,
    author: String,
    authorId: String,
    authorUrl: String,
    authorVerified: Boolean,

    videoCount: Int32,
    videos: [
      {
        title: String,
        videoId: String,
        lengthSeconds: Int32,
        videoThumbnails: [
          {
            quality: String,
            url: String,
            width: Int32,
            height: Int32
          }
        ]
      }
    ]
  }
*/
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPlaylistUnit {
    #[serde(rename = "type")]
    pub o_type: String,
    pub title: String,
    pub playlist_id: String,
    pub playlist_thumbnail: String,
    pub author: String,
    pub author_id: String,
    pub author_url: String,
    pub author_verified: bool,

    pub video_count: i32,
    pub videos: Vec<common::PlaylistVideoObject>,
}

/*
{
    "title": String,
    "playlistId": String,

    "author": String,
    "authorId": String,
    "authorThumbnails": [
        {
            "url": String,
            "width": String,
            "height": String
        }
    ],
    "description": String,
    "descriptionHtml": String,

    "videoCount": Int32,
    "viewCount": Int64,
    "updated": Int64,

    "videos": [
        {
          "title": String,
          "videoId": String,
          "author": String,
          "authorId": String,
          "authorUrl": String,

          "videoThumbnails": [
              {
                  "quality": String,
                  "url": String,
                  "width": Int32,
                  "height": Int32
              }
          ],
          "index": Int32,
          "lengthSeconds": Int32
        }
    ]
}
*/

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoUnit {
    pub title: String,
    pub video_id: String,
    pub author: String,
    pub author_id: String,
    pub author_url: String,

    pub video_thumbnails: Vec<common::ThumbnailObject>,
    pub index: i32,
    pub length_seconds: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    #[serde(rename = "type")]
    pub o_type: String,

    pub title: String,
    pub playlist_id: String,
    pub playlist_thumbnail: String,

    pub author: String,
    pub author_id: String,
    pub author_url: String,
    pub subtitle: Option<String>,
    pub author_thumbnails: Vec<common::ImageObject>,

    pub description: String,
    pub description_html: String,

    pub video_count: i32,
    pub view_count: i64,
    pub updated: i64,

    pub is_listed: bool,
    pub videos: Vec<PlaylistVideoUnit>,
}
