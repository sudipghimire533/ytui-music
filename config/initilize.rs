use lazy_static::lazy_static;
use super::{Config, ConfigContainer};

lazy_static! {
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
}
