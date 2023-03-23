use rand;
use rusqlite;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path;
use std::time::Duration;
pub mod initilize;

pub const CONF_DIR_NAME: &str = "ytui_music";
pub const CONFIG_FILE_NAME: &str = "config.json";
pub const MPV_OPTION_FILE_NAME: &str = "mpv.conf";
pub const SQLITE_DB_NAME: &str = "storage.db3";
pub const AUDIO_DIR_VAR_KEY: &str = "YTUI_MUSIC_DIR";
pub const YTUI_CONFIG_DIR_VAR_KEY: &str = "YTUI_CONFIG_DIR";

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
            } else if rand::random() {
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
    pub view: char,
    pub favourates_add: char,
    pub favourates_remove: char,
    pub vol_increase: char,
    pub vol_decrease: char,
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

            // This key + CTRL will downlaod the item currently focused from playlistbar/musicbar.
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

            // This key will expand the content of playlist but do not play it
            // Also will show the selection url
            view: 'v',

            // Add the current selection to the favourates list
            favourates_add: 'f',

            // Remove the current selection from the favourates lits. Adding and removing from
            // favourates list are not done by same key because toggeling means first the exsistance of
            // given selection should be checked in database and then again query another INSERT/REMOVE
            // statement. However, if sepearte keys are used, only single INSERT/REMOVE query is to be
            // executed.
            favourates_remove: 'u',

            // Key to increase the volume of playback
            vol_increase: '+',

            // Same as vol_increase but decrease the volume
            vol_decrease: '-',
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
    pub item_per_list: usize,
    pub server_time_out: u32,
    pub refresh_rate: u64,
    pub seek_forward_secs: u32,
    pub seek_backward_secs: u32,
    pub region: String,

    // Amount to increase/decrease by
    pub volume_step: i8,

    // When any search query is prefixed by these strings in search query,
    // it will only show the result music/playlist/artist
    // prefixed by [0] => only music search and so on
    // If it is intended to not use this feature then just set these string to some random characters
    // that you would probably never type in search query.
    pub search_by_type: [String; 3],
}

impl Default for Constants {
    fn default() -> Self {
        Constants {
            item_per_list: 10,
            server_time_out: 30_000,
            refresh_rate: 900,
            seek_forward_secs: 10,
            seek_backward_secs: 10,
            region: String::from("NP"),
            volume_step: 10,
            search_by_type: [
                String::from("music:"),
                String::from("playlist:"),
                String::from("artist:"),
            ],
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
        // Get Audio directory to which music will be downladed to.
        // This will first prirotise `crate::AUDIO_DIR_VAR_KEY` and move to get from
        // dirs crate and panic if still can't get required directory
        let audio_folder = {
            match std::env::var(AUDIO_DIR_VAR_KEY) {
                Ok(audio_dir) => audio_dir,

                Err(_) => match dirs::audio_dir() {
                    Some(audio_dir) => audio_dir
                        .to_str()
                        .expect("While reading audio_dir. Non-utf8 character is not supported")
                        .to_string(),

                    None => {
                        eprintln!(
                            "Ytui was unable to find the audio directory.\
                            You can try re-run by setting {} variable or downloading the template config file.",
                            AUDIO_DIR_VAR_KEY
                        );
                        std::process::exit(1);
                    }
                },
            }
        };

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
    #[serde(default, rename = "ShortcutKeys")]
    pub shortcut_keys: ShortcutsKeys,
    #[serde(default, rename = "Colors")]
    pub theme: Theme,
    #[serde(default, rename = "Servers")]
    pub servers: Servers,
    #[serde(default, rename = "Constants")]
    pub constants: Constants,
    #[serde(default, rename = "MpvOptions")]
    pub mpv: MpvOptions,
    #[serde(default, rename = "Downloads")]
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

        // @dir: a path string
        // @returns: An option returning None is the path does not exists or path is not dir or
        //          A pathbuf that points to the real direcotry(after reading symbolinc link)
        let validate_dir = |dir: &str| -> Option<path::PathBuf> {
            let mut dir_path = path::PathBuf::from(dir);
            if let Ok(pth) = dir_path.read_link() {
                dir_path = pth
            }

            if !dir_path.is_dir() {
                None
            } else {
                Some(dir_path)
            }
        };

        // Validate mpv config path dir
        // Any error on unable to locate mpv dir or file should be hard error
        if let Some(new_path) = validate_dir(&config.mpv.config_path) {
            let mut mpv_config = new_path.join(MPV_OPTION_FILE_NAME);
            if let Ok(pth) = mpv_config.read_link() {
                mpv_config = pth;
            }

            if !mpv_config.is_file() {
                eprintln!("Config Error: Mpv config path defined in `MpvOptions{{ config_path }}` should point to valid file path");
                return None;
            }
            config.mpv.config_path = mpv_config.to_string_lossy().to_string();
        } else {
            eprintln!(
                "The directory directory defined in config to locate mpv option cannot be found"
            );
            eprintln!(
                "Note: `config_path` must be a path to directory where mpv.conf file is located"
            );
            return None;
        }

        // Check and update the download directory
        // If dir is not valid, program should continue as it not hard error
        if let Some(dow_path) = validate_dir(&config.download.path) {
            config.download.path = dow_path.to_string_lossy().to_string();
        } else {
            eprintln!("Music download path as defined in `Downloads{{path}}` must point to valid directory");
            eprintln!(
                "On download you may not save the downloaded file or mat crash with some error."
            );
            eprintln!("Continue...")
        }

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
        // If $YTUI_MUSIC_CONFIG_DIR env is set. Use it
        if let Ok(val) = std::env::var("YTUI_MUSIC_CONFIG_DIR") {
            eprintln!("YTUI_MUSIC_CONFIG_DIR environment variable is set. Using it...");
            let config_dir = path::PathBuf::from(val);
            return Some(config_dir);
        }

        let config_dir = {
            if let Ok(config_dir) = std::env::var(YTUI_CONFIG_DIR_VAR_KEY) {
                path::PathBuf::from(config_dir)
            } else {
                if let Some(mut config_dir) = dirs::preference_dir() {
                    config_dir = config_dir.join(CONF_DIR_NAME);
                    config_dir
                } else {
                    eprintln!("Cannot get you configuration directory to where to config of Ytui will be stored.");

                    return None;
                }
            }
        };

        std::fs::DirBuilder::new()
            .recursive(true) // create parent directory as needed
            .create(&config_dir) // Create if directory do not already exists
            .map(|_| config_dir) // put directory value inside Ok
            .ok() // convert to Option
    }

    fn get_db_path() -> Option<path::PathBuf> {
        let config_dir = Self::get_config_dir()?;
        let db_path = config_dir.join(SQLITE_DB_NAME);

        Some(db_path)
    }

    pub fn give_me_storage() -> Option<rusqlite::Connection> {
        let db_path = Self::get_db_path()?;

        let connection = match rusqlite::Connection::open(&db_path) {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!(
                    "Cannot create connection to storage db. Error: {err}",
                    err = err
                );
                return None;
            }
        };

        // All the types are are decleared as text.
        // The destination types fetcher::{MusicUnit, Playlistunit, ArtistUnit}
        // fiels are all decleared in string format. So on retriving with SELECT query
        // it makes easy to fetch columns without any conversion method
        let create_favourates_table = format!(
            "
                CREATE TABLE IF NOT EXISTS {tb_music} (
                    id          TEXT    NOT NULL    PRIMARY KEY,
                    title       TEXT    NOT NULL,
                    author      TEXT    NOT NULL,
                    duration    TEXT     NOT NULL
                );

                CREATE TABLE IF NOT EXISTS {tb_playlist} (
                    id      TEXT    NOT NULL    PRIMARY KEY,
                    name    TEXT    NOT NULL,
                    author  TEXT    NOT NULL,
                    count   TEXT     NOT NULL
                );

                CREATE TABLE IF NOT EXISTS {tb_artist} (
                    id      TEXT    NOT NULL    PRIMARY KEY,
                    name    TEXT    NOT NULL,
                    count   TEXT    NOT NULL
                );
           ",
            tb_music = initilize::TB_FAVOURATES_MUSIC,
            tb_playlist = initilize::TB_FAVOURATES_PLAYLIST,
            tb_artist = initilize::TB_FAVOURATES_ARTIST
        );

        let res = connection.execute_batch(&create_favourates_table);

        if let Err(err) = res {
            eprintln!(
                "Cannot initlize required table in newly created database. Error: {err}",
                err = err
            );
            return None;
        }

        Some(connection)
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
