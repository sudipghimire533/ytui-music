use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path;
use std::time::Duration;

pub const CONF_DIR_NAME: &str = "ytui_music";
pub const CONFIG_FILE_NAME: &str = "config.json";
pub const MPV_OPTION_FILE_NAME: &str = "mpv.conf";

// A stupid as fu#k shit logic to get either true or false.
// Create instant clock.
// execute some useless statements (which may even removed by compiler optimizer)
// get the time elapsed in nano second
// if it is even number return true else return false
fn get_random_bool() -> bool {
    let now = std::time::Instant::now();
    let _ = vec![1; 3];
    let elapsed = now.elapsed().as_nanos();
    elapsed % 2 == 0
}

trait Random {
    #[must_use]
    fn suffle(&self, timeout: Duration) -> Self;
}
impl<T> Random for Vec<T>
where
    T: std::cmp::PartialEq + Clone,
{
    fn suffle(&self, timeout: Duration) -> Self {
        let length = self.len();
        let mut new_vector = Vec::with_capacity(length);

        let now = std::time::Instant::now();
        let mut current = 0;
        while new_vector.len() != length {
            // When timwout occurs push all the reamining vector directly
            if now.elapsed() > timeout {
                for val in self {
                    if !new_vector.contains(val) {
                        new_vector.push(val.clone());
                    }
                }
            } else if get_random_bool() {
                new_vector.push(self[current].clone());
            }
            current = (current + 1) % length;
        }

        new_vector
    }
}

type Color = (u8, u8, u8);

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ShortcutsKeys {
    pub toggle_play: char,
    pub next: char,
    pub prev: char,
    pub start_search: char,
    pub download: char,
    pub quit: char,
    pub forward: char,
    pub backward: char,
    pub suffle: char,
    pub repeat: char,
}

impl Default for ShortcutsKeys {
    fn default() -> Self {
        ShortcutsKeys {
            // This key will pause the playpack if it is currently playing
            // and unpause the playback if is currently paused
            toggle_play: ' ',

            // When pressed this key over musicbar/artistbar/playlistbar, it will try to fetch more item and add to the list
            // When pressed this with CTRL key it will play the next track from playlist
            // When pressed from bottom music control, it will play the next track from playlist
            next: 'n',

            // Same of p but instead of fetching next data or playing next track it try to fetch
            // previous data or play previous track
            prev: 'p',

            // This will move the cursor to the search box
            start_search: '/',

            // This will downlaod the item currently focused from playlistbar/musicbar.
            // if an item from musicbar is focused, download that music
            // if an item from playlistbat is focused, download all content from that playlist
            // otherwise do nothing
            download: 'd',

            // When this key is pressed with CTRL, it will quit the application after checking
            // weather if there are any ongoing downloads
            // When pressed with CTRL and SHIFT it will force quit the app without ongoing downloads
            // check
            quit: 'c',

            // Seek the playback forward by time specified in config
            forward: '>',

            // Same as forward but instead seek backward
            backward: '<',

            // Turn suffle on if already is off and vice-versa
            // Suffle on: play the playlist in random order
            // Suffle off: play the playlist in as is order
            suffle: 's',

            // Turn repeat on if already is off and vice-versa
            // Repeat on: Play all the items from playlist. If last item ends play first
            // Repeat off: If currenlt playing item ends play same item again. i.e repeat one
            repeat: 'r',
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Theme {
    pub border_idle: Color,
    pub border_highlight: Color,
    pub list_idle: Color,
    pub list_hilight: Color,
    pub sidebar_list: Color,
    pub block_title: Color,
    pub gauge_fill: Color,
    pub color_primary: Color,
    pub color_secondary: Color,
    pub status_text: Color,
}

impl Default for Theme {
    fn default() -> Self {
        // TODO: Set actual colors
        Theme {
            // Apply this on the border of blocks when that window is not active
            border_idle: (255, 255, 255),

            // Apply this on the border of blocks when that windows is active
            border_highlight: (10, 150, 150),

            // Apply to the list items that are idle
            list_idle: (200, 160, 0),

            // Apply to the list item that is currently under cursor
            list_hilight: (255, 255, 255),

            // Applies to the text in top status bar
            status_text: (175, 125, 115),

            // Applies to the progress bar of bottom bar
            gauge_fill: (85, 85, 85),

            // Applies to the sidebar list item when idle
            sidebar_list: (100, 250, 20),

            // Applies to the title (top-left corner of border) of the block
            block_title: (175, 125, 115),

            // Color_(promary/secondary/tertiary) are for everything else other than above.
            // Instead of relying on terminal color, using this will bring more consistency in the ui
            color_primary: (100, 250, 20),
            color_secondary: (250, 230, 70),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Constants {
    pub item_per_list: u8,
    pub server_time_out: u32,
    pub refresh_rate: u64,
    pub seek_forward_secs: u32,
    pub seek_backward_secs: u32,
}

impl Default for Constants {
    fn default() -> Self {
        Constants {
            item_per_list: 10,
            server_time_out: 30_000,
            refresh_rate: 900,
            seek_forward_secs: 10,
            seek_backward_secs: 10,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Servers {
    pub list: Vec<String>,
}

impl Default for Servers {
    fn default() -> Self {
        let content = include_str!("invidious_servers.list");
        let mut list: Vec<String> = Vec::new();

        for mut server in content.lines() {
            server = server.trim();
            if !server.is_empty() {
                list.push(server.to_string());
            }
        }

        Servers { list }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Downloads {
    pub path: String,
    pub format: String,
}

impl Default for Downloads {
    fn default() -> Self {
        let audio_folder = dirs::audio_dir()
            .expect("Cound not find audio directory")
            .as_path()
            .to_string_lossy()
            .to_string();

        Downloads {
            path: audio_folder,
            format: "mp3".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MpvOptions {
    config_path: String,
}

impl Default for MpvOptions {
    fn default() -> Self {
        MpvOptions {
            config_path: ConfigContainer::get_config_dir()
                .unwrap()
                .as_path()
                .to_string_lossy()
                .to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct Config {
    #[serde(default, alias = "ShortcutKeys")]
    pub shortcut_keys: ShortcutsKeys,
    #[serde(default, alias = "Colors")]
    pub theme: Theme,
    #[serde(default, alias = "Servers")]
    pub servers: Servers,
    #[serde(default, alias = "Constants")]
    pub constants: Constants,
    #[serde(default, alias = "MpvOptions")]
    pub mpv: MpvOptions,
    #[serde(default, alias = "Downloads")]
    pub download: Downloads,
}

impl Config {
    pub fn get_string(&self) -> Option<String> {
        match serde_json::ser::to_string_pretty(self) {
            Ok(val) => Some(val),
            Err(err) => {
                eprintln!(
                    "Error while decoding the config file content. Erro: {}",
                    err
                );
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct ConfigContainer {
    pub config: Config,
    file_path: path::PathBuf,
}

impl Default for ConfigContainer {
    fn default() -> Self {
        ConfigContainer {
            config: Config::default(),
            file_path: Self::get_config_path().unwrap(),
        }
    }
}

impl ConfigContainer {
    fn from_file(file_path: &path::Path) -> Option<Self> {
        let file = match File::open(file_path) {
            Ok(val) => val,
            Err(err) => {
                eprintln!(
                    "Unable to open config file from {path}. Error: {err}",
                    path = file_path.to_string_lossy(),
                    err = err
                );
                return None;
            }
        };

        let reader = BufReader::new(file);
        let mut config: Config = match serde_json::from_reader(reader) {
            Ok(val) => val,
            Err(err) => {
                eprintln!(
                    "Invalid format of config file. Deserialize message: {}",
                    err
                );
                return None;
            }
        };

        // Most of invidious server do not expect this much of api calls.
        // So be sure we dont kill a single server instead distribute the load.
        // Here, it is achived by rearrenging the server list in random order
        // so that first server don't always have to be first to send request
        config.servers.list = config.servers.list.suffle(Duration::from_secs(4));

        Some(Self {
            config,
            file_path: file_path.to_path_buf(),
        })
    }

    fn flush(&self) -> Option<()> {
        let content = match self.config.get_string() {
            Some(val) => val,
            None => return None,
        };

        let mut file_handle = match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.file_path)
        {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Unable to open config file for write. Error: {}", err);
                return None;
            }
        };

        match file_handle.write(content.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("unable to write config to file. Error: {}", err);
                return None;
            }
        }

        Some(())
    }

    pub fn get_config_dir() -> Option<path::PathBuf> {
        match dirs::preference_dir() {
            Some(mut config_dir) => {
                config_dir = config_dir.join(CONF_DIR_NAME);

                match std::fs::DirBuilder::new()
                    .recursive(true)
                    .create(&config_dir)
                {
                    Ok(_) => Some(config_dir),
                    Err(err) => {
                        eprintln!(
                            "Cannot create app folder in your config folder as {path}. Error: {err}",
                            path = &config_dir.as_path().to_string_lossy(),
                            err = err
                        );
                        None
                    }
                }
            }
            None => {
                eprintln!("Cannot get your os user config directory..");
                None
            }
        }
    }

    fn get_config_path() -> Option<path::PathBuf> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join(CONFIG_FILE_NAME);

        Some(config_path)
    }

    fn default_config_to_file() -> Option<ConfigContainer> {
        let config_container = ConfigContainer::default();
        config_container.flush();

        Some(config_container)
    }

    pub fn give_me_config() -> Option<Self> {
        let config_path = ConfigContainer::get_config_path()?;
        let mpv_conf_file = ConfigContainer::get_config_dir()?.join(MPV_OPTION_FILE_NAME);

        if !mpv_conf_file.exists() {
            let mpv_options = include_str!("./default_mpv_options.conf");
            match std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&mpv_conf_file)
            {
                Ok(mut mpv_conf_file) => {
                    match mpv_conf_file.write_all(mpv_options.as_bytes()) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("Cannot write default mpv options. Error: {err}", err = err);
                        }
                    };
                }
                Err(err) => {
                    eprintln!(
                        "Cannot open file {path} to write mpv default options. Error: {err}",
                        path = mpv_conf_file.as_path().to_string_lossy(),
                        err = err
                    );
                    return None;
                }
            }
        }

        let config_container;

        if config_path.exists() {
            config_container = ConfigContainer::from_file(&config_path)?;
        } else {
            config_container = ConfigContainer::default_config_to_file()?;
        }

        Some(config_container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_config_path() -> path::PathBuf {
        path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("sample_config.json")
    }

    #[test]
    fn display_config_path() {
        let path = ConfigContainer::get_config_path().unwrap();
        eprintln!("Config path: {}", path.as_path().to_string_lossy());
    }

    #[test]
    fn inspect_server_list() {
        let path = get_test_config_path();
        for _ in 0..4 {
            let servers = ConfigContainer::from_file(&path)
                .unwrap()
                .config
                .servers
                .list;
            eprintln!("{:#?}", servers);
        }
    }
}
