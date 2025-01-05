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

pub enum MpvPropertyGet {
    Duration,
    MediaTitle,
    TimePos,
    PauseStatus,
    Volume,
}
impl MpvPropertyGet {
    const fn prop_key(self) -> &'static str {
        match self {
            Self::Duration => "duration",
            Self::MediaTitle => "media-title",
            Self::TimePos => "time-pos",
            Self::PauseStatus => "pause",
            Self::Volume => "volume",
        }
    }
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

    pub fn cycle_pause_status(&self) -> Result<(), YtuiMvpAudioError> {
        let current_status = self
            .get_property::<bool>(MpvPropertyGet::PauseStatus)?
            .unwrap();
        self.set_property(MpvPropertyGet::PauseStatus.prop_key(), !current_status)
    }

    pub fn execute_command(&self, command: MpvCommand) -> Result<(), YtuiMvpAudioError> {
        let (command, args) = command.get_raw_command()?;
        self.mpv_handle
            .command(command, args.borrow())
            .map_err(|e| YtuiMvpAudioError::mpv("executing command", e))
    }

    pub fn enable_video(&self) -> Result<(), YtuiMvpAudioError> {
        self.set_property("video", "auto")
    }

    pub fn disable_video(&self) -> Result<(), YtuiMvpAudioError> {
        self.set_property("video", "no")
    }

    pub fn get_property<PropertyValueType: libmpv2::GetData>(
        &self,
        property_key: MpvPropertyGet,
    ) -> Result<Option<PropertyValueType>, YtuiMvpAudioError> {
        match self
            .mpv_handle
            .get_property::<PropertyValueType>(property_key.prop_key())
        {
            Ok(v) => Ok(Some(v)),
            Err(libmpv2::Error::Raw(libmpv2::mpv_error::PropertyUnavailable)) => Ok(None),
            Err(e) => Err(YtuiMvpAudioError::mpv("getting property", e)),
        }
    }

    fn set_property<PropertyValueType: libmpv2::SetData>(
        &self,
        property_key: &str,
        property_value: PropertyValueType,
    ) -> Result<(), YtuiMvpAudioError> {
        self.mpv_handle
            .set_property(property_key, property_value)
            .map_err(|e| YtuiMvpAudioError::mpv("setting property", e))
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
