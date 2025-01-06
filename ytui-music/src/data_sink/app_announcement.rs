use std::cmp::Ordering;
use stream_mandu::web_client::async_reqwest_impl::reqwest;

const ANNOUNCEMENT_PATH: &str = "https://raw.githubusercontent.com/sudipghimire533/ytui-music/refs/heads/new.ytui-music/announcements/announcement-latest.json";
const ANNOUNCEMENT_FETCH_ERROR: &str = r#"Can not check github for aany announcement text.

This application is currently in heavy development. There might be interesting changes and new features available. You can manually check on:

- Visit github repo: https://github.com/sudipghimire533/ytui-music

Caused by:
"#;

#[derive(serde::Deserialize)]
pub struct AnnouncementInfo {
    title: String,
    announcement_text: String,

    target_min_version: (u8, u8, u8),
    target_max_version: (u8, u8, u8),
    exclude_version: Vec<(u8, u8, u8)>,
}

impl AnnouncementInfo {
    pub fn get_title_and_body(self) -> (String, String) {
        (self.title, self.announcement_text)
    }

    pub fn fetch_startup_announcement() -> Option<AnnouncementInfo> {
        match reqwest::blocking::get(ANNOUNCEMENT_PATH) {
            Ok(response) => match response.text() {
                Ok(response_text) => {
                    match serde_json::from_str::<AnnouncementInfo>(response_text.as_str()) {
                        Ok(announcement) => announcement
                            .should_show_in_this_version()
                            .then_some(announcement),

                        Err(_err) => Some(AnnouncementInfo {
                            title: String::from("RAW ANNOUNCEMENT"),
                            announcement_text: format!(
                                "SOURCE: {ANNOUNCEMENT_PATH}\n\n{response_text}"
                            ),
                            ..AnnouncementInfo::default()
                        }),
                    }
                }

                Err(err) => {
                    let mut err_annoncement = AnnouncementInfo::default();
                    err_annoncement.announcement_text += format!("{err:#?}").as_str();
                    Some(err_annoncement)
                }
            },

            Err(err) => {
                let mut err_annoncement = AnnouncementInfo::default();
                err_annoncement.announcement_text += format!("{err:#?}").as_str();
                Some(err_annoncement)
            }
        }
    }

    fn should_show_in_this_version(&self) -> bool {
        let current_app_version = [
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
        ]
        .iter()
        .map(|env| env.parse().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .map(|[a, b, c]: [u8; 3]| (a, b, c))
        .unwrap();

        Self::semver_compare(current_app_version, self.target_min_version).is_ge()
            && Self::semver_compare(current_app_version, self.target_max_version).is_le()
            && !self
                .exclude_version
                .iter()
                .any(|&exclude_version| exclude_version == current_app_version)
    }

    fn semver_compare(
        (version_major, version_min, version_patch): (u8, u8, u8),
        (other_major, other_min, other_patch): (u8, u8, u8),
    ) -> Ordering {
        match version_major.cmp(&other_major) {
            Ordering::Equal => match version_min.cmp(&other_min) {
                Ordering::Equal => version_patch.cmp(&other_patch),
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl Default for AnnouncementInfo {
    fn default() -> Self {
        Self {
            title: String::from("Announcement Error:"),
            announcement_text: String::from(ANNOUNCEMENT_FETCH_ERROR),
            target_min_version: (u8::MIN, u8::MIN, u8::MIN),
            target_max_version: (u8::MAX, u8::MAX, u8::MAX),
            exclude_version: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AnnouncementInfo;
    use std::{ffi::OsStr, fs::read_dir, fs::read_to_string};

    #[test]
    fn latest_announcement_can_be_parsed() {
        let announcement_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../announcements/");

        let announcements = read_dir(announcement_dir)
            .unwrap()
            .map(|dir_entry| dir_entry.unwrap().path())
            .filter(|file_path| file_path.extension() == Some(OsStr::new("json")));
        for announcement_file in announcements {
            let file_content = read_to_string(announcement_file).unwrap();
            let _announcement =
                serde_json::from_str::<AnnouncementInfo>(file_content.as_str()).unwrap();
        }
    }
}
