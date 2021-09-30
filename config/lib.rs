use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json;
use std::default::Default;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path;

type Color = [u8; 3];

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct ShortcutsKeys {
    toggle_play: char,
    play_next: char,
    play_prev: char,
    start_search: char,
    download: char,
}

impl Default for ShortcutsKeys {
    fn default() -> Self {
        ShortcutsKeys {
            toggle_play: ' ',
            play_next: 'n',
            play_prev: 'p',
            start_search: '/',
            download: 'd',
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Theme {
    border_idle: Color,
    border_hilight: Color,
    list_title: Color,
    list_hilight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        // TODO: Set actual colors
        Theme {
            border_idle: [100, 200, 100],
            border_hilight: [100, 100, 100],
            list_title: [100, 200, 255],
            list_hilight: [100, 250, 200],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Constants {
    item_per_list: u8,
    server_time_out: u32,
}

impl Default for Constants {
    fn default() -> Self {
        Constants {
            item_per_list: 10,
            server_time_out: 30_000,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Servers {
    list: Vec<String>,
}

impl Default for Servers {
    fn default() -> Self {
        Servers {
            list: vec![
                "https://ytprivate.com/api/v1".to_string(),
                "https://vid.puffyan.us/api/v1".to_string(),
                "https://invidious.snopyta.org/api/v1".to_string(),
                "https://ytb.trom.tf/api/v1".to_string(),
                "https://invidious.namazso.eu/api/v1".to_string(),
                "https://invidious.hub.ne.kr/api/v1".to_string(),
            ],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
struct Config {
    #[serde(default, alias = "ShortcutKeys")]
    shortcut_keys: ShortcutsKeys,
    #[serde(default, alias = "Colors")]
    theme: Theme,
    #[serde(default, alias = "Servers")]
    servers: Servers,
    #[serde(default, alias = "Constants")]
    constants: Constants,
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
struct ConfigContainer {
    config: Config,
    file_path: path::PathBuf,
}

impl ConfigContainer {
    pub fn from_file(file_path: &path::Path) -> Option<Self> {
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
        let config: Config = match serde_json::from_reader(reader) {
            Ok(val) => val,
            Err(err) => {
                eprintln!(
                    "Invalid format of config file. Deserialize message: {}",
                    err
                );
                return None;
            }
        };

        Some(Self {
            config,
            file_path: file_path.to_path_buf(),
        })
    }

    pub fn flush(&self) -> Option<()> {
        let content = match self.config.get_string() {
            Some(val) => val,
            None => return None,
        };

        let mut file_handle = match std::fs::OpenOptions::new()
            .write(true)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_file_are_eq() {
        let file_path = path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("sample_config.json");
        let config_from_file = ConfigContainer::from_file(&file_path).unwrap().config;
        let default_config = Config::default();

        assert_eq!(config_from_file.constants, default_config.constants);
        assert_eq!(config_from_file.theme, default_config.theme);
        assert_eq!(config_from_file.servers, default_config.servers);
        assert_eq!(config_from_file.shortcut_keys, default_config.shortcut_keys);
    }
}
