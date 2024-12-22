use std::{
    borrow::{Borrow, Cow},
    path::Path,
};

#[derive(Debug)]
pub enum YtuiMvpAudioError {
    MpvErr {
        action: &'static str,
        err: libmpv2::Error,
    },
    InvalidConfigPath,
    InvalidSourcePath,
}
impl YtuiMvpAudioError {
    pub fn mpv(action: &'static str, err: libmpv2::Error) -> Self {
        Self::MpvErr { action, err }
    }
}

pub enum MpvCommand<'a> {
    /// stream given list of files
    StreamFiles(&'a [&'a Path]),

    /// Stream given list of remote urls
    StreamUrls(&'a [&'a str]),
}

impl<'a> MpvCommand<'a> {
    pub fn get_raw_command(self) -> Result<(&'static str, Cow<'a, [&'a str]>), YtuiMvpAudioError> {
        match self {
            MpvCommand::StreamFiles(file_paths) => {
                let args = file_paths
                    .iter()
                    .map(|file_path| {
                        file_path
                            .to_str()
                            .ok_or(YtuiMvpAudioError::InvalidSourcePath)
                    })
                    .collect::<Result<Vec<&str>, YtuiMvpAudioError>>()?;

                Ok(("loadfile", Cow::Owned(args)))
            }

            MpvCommand::StreamUrls(network_paths) => Ok(("loadfile", Cow::Borrowed(network_paths))),
        }
    }
}

pub struct LibmpvPlayer {
    mpv_handle: libmpv2::Mpv,
}

impl LibmpvPlayer {
    pub fn new() -> Result<Self, YtuiMvpAudioError> {
        let mpv_handle =
            libmpv2::Mpv::new().map_err(|e| YtuiMvpAudioError::mpv("calling mpv::new", e))?;
        Ok(Self { mpv_handle })
    }

    pub fn load_config(&mut self, config_path: &Path) -> Result<(), YtuiMvpAudioError> {
        let config_path_str = config_path
            .to_str()
            .ok_or(YtuiMvpAudioError::InvalidConfigPath)?;
        self.mpv_handle
            .load_config(config_path_str)
            .map_err(|e| YtuiMvpAudioError::mpv("loading config", e))?;

        Ok(())
    }

    pub fn load_uri(&self, uri: &str) -> Result<(), YtuiMvpAudioError> {
        self.execute_command(MpvCommand::StreamUrls(&[uri]))
    }

    pub fn load_file(&self, file_path: &Path) -> Result<(), YtuiMvpAudioError> {
        self.execute_command(MpvCommand::StreamFiles(&[file_path]))
    }

    pub fn execute_command(&self, command: MpvCommand) -> Result<(), YtuiMvpAudioError> {
        let (command, args) = command.get_raw_command()?;
        self.mpv_handle
            .command(command, args.borrow())
            .map_err(|e| YtuiMvpAudioError::mpv("executing command", e))
    }
}

impl LibmpvPlayer {
    pub async fn load_uri_async(&self, uri: &str) -> Result<(), YtuiMvpAudioError> {
        self.load_uri(uri)
    }

    pub async fn load_file_async(&self, file_path: &Path) -> Result<(), YtuiMvpAudioError> {
        self.load_file(file_path)
    }

    pub async fn execute_command_async<'a>(
        &mut self,
        command: MpvCommand<'a>,
    ) -> Result<(), YtuiMvpAudioError> {
        self.execute_command(command)
    }
}
