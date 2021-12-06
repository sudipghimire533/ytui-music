use config::initilize::CONFIG;
use reqwest;
use serde::{self, Deserialize};
use tokio;

#[derive(Default)]
pub struct Options {
    exec_name: String,
    sub_command: String,
    arguments: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct AssetOuter {
    url: String,
    name: String,
    content_type: String,
}

#[derive(Deserialize, Debug)]
struct ReleaseInfoResponse {
    assets: Vec<AssetOuter>,
}

impl Options {
    pub fn create_from_args(mut arguments: std::env::Args) -> Result<Self, &'static str> {
        let mut result = Options::default();

        result.exec_name = "ytui_music".to_string();
        if let Some(exec_name) = arguments.next() {
            result.exec_name = exec_name;
        }
        if let Some(cmd) = arguments.next() {
            result.sub_command = cmd;
        }

        result.arguments = arguments.collect::<Vec<String>>();

        Ok(result)
    }

    pub fn evaluate(self) -> bool {
        let ascii_art = r##"
__   ___         _                           _
\ \ / / |_ _   _(_)      _ __ ___  _   _ ___(_) ___
 \ V /| __| | | | |_____| '_ ` _ \| | | / __| |/ __|
  | | | |_| |_| | |_____| | | | | | |_| \__ \ | (__ 
  |_|  \__|\__,_|_|     |_| |_| |_|\__,_|___/_|\___|
"##;
        let author = env!(
            "CARGO_PKG_AUTHORS",
            "Sudip Ghimire <sudipghimire533@gmail.com>"
        );
        println!("{}\nAuthor(s): {}\n", ascii_art, author);

        let mut should_continue = false;
        match self.sub_command.trim() {
            "run" => {
                self.initialize_globals();
                should_continue = true;
            }
            "update" => self.update(),

            "help" => self.show_help(),

            "delete" => match &self.arguments.first() {
                Some(ref arg) if *arg == &String::from("config") => self.delete_config(),
                Some(ref arg) if *arg == &String::from("db") => self.delete_db(),
                _ => self.show_help(),
            },

            "info" => match &self.arguments.first() {
                Some(arg) => match arg.as_str() {
                    "version" => self.show_version(),
                    "ytui" | "about" => self.show_about(),
                    "config" => self.info_config(),
                    "shortcuts" | "keys" => self.describe_keys(),
                    _ => self.show_help(),
                },
                _ => self.show_help(),
            },

            _ => self.show_help(),
        }

        should_continue
    }

    pub fn show_version(self) {
        let prog_name = env!("CARGO_PKG_NAME", "ytui_music");
        let version = env!("CARGO_PKG_VERSION", "undefined");
        println!("{} v{}", prog_name, version);
    }

    pub fn show_about(self) {
        self.show_version();
        println!(include_str!("about.txt"));
    }

    pub fn info_config(self) {
        println!(
            include_str!("info_config.txt"),
            config_dir = "$HOME/.config"
        );
    }

    pub fn describe_keys(self) {
        self.initialize_globals();
        let keys = &CONFIG.shortcut_keys;
        println!(
            include_str!("help_keys.txt"),
            toggle = keys.toggle_play,
            next = keys.next,
            prev = keys.prev,
            suffle = keys.suffle,
            repeat = keys.repeat,
            f_add = keys.favourates_add,
            f_rm = keys.favourates_remove,
            search = keys.start_search,
            view = keys.view,
            backward = keys.backward,
            forward = keys.forward,
            download = keys.download,
            quit = keys.quit
        );
    }

    pub fn show_help(self) {
        println!(include_str!("help_message.txt"));
    }

    pub fn initialize_globals(&self) {
        lazy_static::initialize(&config::initilize::INIT);
    }

    pub fn update(self) {
        println!("Updating ytui-music to latest version....");

        let mut download_path = String::new();
        while download_path.is_empty() {
            println!("Enter path to store updated binary");
            std::io::stdin()
                .read_line(&mut download_path)
                .expect("Couldnot accept input");
            download_path = download_path.trim().to_string();
        }

        let mut download_path = std::path::PathBuf::from(download_path);
        if download_path.is_dir() {
            download_path = download_path.join("ytui_music");
        }
        let download_path = download_path.as_path();

        println!(
            "Latest version of ytui-music will be store in {}.",
            match download_path.to_str() {
                None => {
                    download_path.to_string_lossy().into_owned()
                }
                Some(val) => val.to_string(),
            }
        );
        println!("Downloading binary. Please wait...");

        let os = std::env::consts::OS;
        let mut arch = std::env::consts::ARCH;
        if arch == "x86_64" {
            arch = "amd64";
        }

        let binary_name = format!("ytui_music-{}-{}", os, arch,);

        let after_download = |response: reqwest::Response| async move {
            std::fs::write(
                download_path,
                response
                    .bytes()
                    .await
                    .expect("Couldn't extract bytes from recived binary.."),
            )
            .unwrap();

            return true;
        };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Cannot build tokio runtime to call download_binary")
            .block_on(async move {
                let response = self.download_binary(&binary_name).await;
                match response {
                    None => {
                        eprintln!("Cannot update ytui-music to latest version du to previous error.");
                        eprintln!("To report this problem, you can file this issue in https://github.com/sudipghimire533/ytui-music/issues/");
                    }
                    Some(ren) => {
                        let sucess = after_download(ren).await;
                        if !sucess {
                            eprintln!("Cannot write ytui-music to destination. Update failed..");
                            eprintln!("To report this problem, you can file this issue in https://github.com/sudipghimire533/ytui-music/issues/");
                        } else {
                            println!("Update sucess. Set executable permission if needed.");
                        }
                    }
                }
            });
    }

    pub async fn download_binary(self, binary_name: &str) -> Option<reqwest::Response> {
        use reqwest::header;

        let assest_api_url =
            format!("https://api.github.com/repos/sudipghimire533/ytui-music/releases/latest");

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("ytui-music/update-request"),
        );

        let client_json = reqwest::ClientBuilder::new()
            .default_headers(headers.clone())
            .build()
            .expect("Cannot build json accepting reqwest client.");

        let octets_array = match client_json.get(&assest_api_url).send().await {
            Err(err) => {
                eprintln!(
                    "Cannot get the release info from {url}.\n Erro: {err}",
                    url = assest_api_url,
                    err = err,
                );
                return None;
            }
            Ok(res) => {
                // println!("{:#?}", res.json::<ReleaseInfoResponse>().await);
                // std::process::exit(1);
                match res.json::<ReleaseInfoResponse>().await {
                    Err(err) => {
                        eprintln!("Unexpected response found. Error: {err}", err = err);
                        return None;
                    }
                    Ok(obj) => obj,
                }
            }
        };

        let mut asset = None;
        for ast in octets_array.assets {
            if ast.name == binary_name {
                asset = Some(ast);
                break;
            }
        }
        let asset = asset.expect("Empty asset array recived from github api");

        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_bytes(asset.content_type.as_bytes())
                .expect("Invalid content type recived from github api"),
        );
        let client_octet = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("Cannot build octet accepting reqwest client.");

        // As of now binary is only of <= 5Mib so if binary happens to grow over time (which is not
        // expected as of now), then instead of holding binary in memory, it should written to file
        // peridocially
        let binary = match client_octet.get(&asset.url).send().await {
            Err(err) => {
                eprintln!(
                    "Cannot fetch octet content from {url}. Error: {err}",
                    url = &asset.url,
                    err = err
                );
                return None;
            }
            Ok(res) => res,
        };

        Some(binary)
    }

    pub fn delete_config(self) {
        eprintln!("This function is currently unimplented.");
        eprintln!("You may try to manually delete config.json and mpv.conf file under ytui_music directory in config directory");
    }

    pub fn delete_db(self) {
        eprintln!("This function is currently unimplented.");
        eprintln!("You may try to manually delete storage.db3 file under ytui_music directory in config directory");
    }
}
