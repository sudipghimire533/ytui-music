use config::initilize::CONFIG;

#[derive(Default)]
pub struct Options {
    exec_name: String,
    sub_command: String,
    arguments: Vec<String>,
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
        eprintln!("Upadting is currently not fully implemented.");
        eprintln!("Download manually for https://github.com/sudipghimire533/ytui-music/releases");
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
