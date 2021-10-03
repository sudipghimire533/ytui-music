use serde::{Deserialize, Serialize};
use serde_json;
use std::default::Default;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path;

// A stupid as fu#k shit logic to get either true or false.
// Create instant clock.
// execute some useless statements (which may even removed by compiler optimizer)
// get the time elapsed in nano second
// if it is even number return true else return false
fn get_random_bool() -> bool {
    let now = std::time::Instant::now();
    let _ = vec![1; 3];
    let elapsed = now.elapsed().as_nanos();
    let res = elapsed % 2 == 0;
    res
}

trait Random {
    #[must_use]
    fn suffle(&self, timeout: u64) -> Self;
}
impl<T> Random for Vec<T>
where
    T: std::cmp::PartialEq + Clone,
{
    fn suffle(&self, timeout: u64) -> Self {
        let length = self.len();
        let mut new_vector = Vec::with_capacity(length);

        let now = std::time::Instant::now();
        let mut current = 0;
        while new_vector.len() != length {
            // When timwout occurs push all the reamining vector directly
            if now.elapsed().as_secs() > timeout {
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

type Color = [u8; 3];

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ShortcutsKeys {
    pub toggle_play: char,
    pub next: char,
    pub prev: char,
    pub start_search: char,
    pub download: char,
    pub help: char,
    pub quit: char,
    pub forward: char,
    pub backward: char,
    pub suffle: char,
    pub repeat: char,
}

impl Default for ShortcutsKeys {
    fn default() -> Self {
        ShortcutsKeys {
            toggle_play: ' ',
            next: 'n',
            prev: 'p',
            start_search: '/',
            download: 'd',
            help: '?',
            quit: 'c',
            forward: '>',
            backward: '<',
            suffle: 's',
            repeat: 'r',
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Theme {
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
pub struct Constants {
    pub item_per_list: u8,
    pub server_time_out: u32,
    pub refresh_rate: u64,
}

impl Default for Constants {
    fn default() -> Self {
        Constants {
            item_per_list: 10,
            server_time_out: 30_000,
            refresh_rate: 900,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Servers {
    pub list: Vec<String>,
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
pub struct Config {
    #[serde(default, alias = "ShortcutKeys")]
    pub shortcut_keys: ShortcutsKeys,
    #[serde(default, alias = "Colors")]
    pub theme: Theme,
    #[serde(default, alias = "Servers")]
    pub servers: Servers,
    #[serde(default, alias = "Constants")]
    pub constants: Constants,
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
        config.servers.list = config.servers.list.suffle(4);

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

    fn get_config_path() -> Option<path::PathBuf> {
        let config_dir = match dirs::preference_dir() {
            Some(path) => path,
            None => {
                eprintln!("Cannot get your os user config directory...");
                return None;
            }
        };

        let mut path = config_dir.join("ytui_music");

        match std::fs::DirBuilder::new().recursive(true).create(&path) {
            Ok(_) => {}
            Err(err) => {
                eprintln!(
                    "Cannot create app folder in your config folder as {path}. Error: {err}",
                    path = &path.as_path().to_string_lossy(),
                    err = err
                );
                return None;
            }
        }

        path = path.join("config.json");

        Some(path)
    }

    fn write_defult_config(config_path: &path::Path) -> Option<ConfigContainer> {
        let default_config = Config::default();
        let config_container = ConfigContainer {
            config: default_config,
            file_path: config_path.into(),
        };

        config_container.flush();

        Some(config_container)
    }

    pub fn give_me_config() -> Option<Self> {
        let config_path = ConfigContainer::get_config_path()?;
        let config_container: ConfigContainer;
        if config_path.exists() {
            config_container = ConfigContainer::from_file(&config_path)?;
        } else {
            config_container = ConfigContainer::write_defult_config(&config_path)?;
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
    fn default_and_file_are_eq() {
        let file_path = get_test_config_path();
        let config_from_file = ConfigContainer::from_file(&file_path).unwrap().config;
        let default_config = Config::default();

        assert_eq!(config_from_file.constants, default_config.constants);
        assert_eq!(config_from_file.theme, default_config.theme);
        assert_eq!(config_from_file.servers, default_config.servers);
        assert_eq!(config_from_file.shortcut_keys, default_config.shortcut_keys);
    }

    #[test]
    fn display_config_path() {
        let path = ConfigContainer::get_config_path().unwrap();
        eprintln!("Config path: {}", path.as_path().to_string_lossy());
    }

    #[test]
    #[ignore]
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
