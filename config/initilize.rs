use super::{Config, ConfigContainer};
use lazy_static;
use lazy_static::lazy_static as compute_static;
use rusqlite::{self, Connection};
use std::sync::Mutex;

pub const TB_FAVOURATES_MUSIC: &str = "favourates_music";
pub const TB_FAVOURATES_PLAYLIST: &str = "favourates_playlist";
pub const TB_FAVOURATES_ARTIST: &str = "favourates_artist";

compute_static! {
    pub static ref CONFIG: Config = {
        match ConfigContainer::give_me_config() {
            Some(config_container) => config_container.config,

            None => {
                let mut response = String::new();
                eprintln!("Cannot get config file. Use default config? [yes/no]");
                std::io::stdin().read_line(&mut response).unwrap();

                match response.trim().to_ascii_lowercase().as_str() {
                    "yes" | "y" | "yeah" | "yep" => Config::default(),

                    _ => {
                        eprintln!("A valid config is required for startup. Exiting..");
                        std::process::exit(1);
                    }
                }
            }
        }
    };

    pub static ref STORAGE: Mutex<Connection> = {
        match ConfigContainer::give_me_storage() {
            Some(conn) => Mutex::new(conn),
            None => {
                eprintln!("A valid storage is required for startup. Exiting..");
                std::process::exit(1);
            }
        }
    };

    pub static ref INIT: () = {
        lazy_static::initialize(&CONFIG);
        lazy_static::initialize(&STORAGE);
    };
}
