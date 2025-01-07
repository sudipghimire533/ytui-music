use std::{borrow::Cow, path::Path};

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

pub enum LoadFileArg<'a> {
    Replace,
    Append { force_media_title: Option<&'a str> },
}

#[derive(strum::IntoStaticStr)]
pub enum MpvCommand<'a> {
    LoadFile {
        stream: &'a str,
        kind: LoadFileArg<'a>,
    },
}

pub enum MpvPropertyGet {
    Duration,
    MediaTitle,
    TimePos,
    PauseStatus,
    Volume,
    PercentPos,
    PlaylistCount,
    NthPlaylistItemTitle(usize),
}
impl MpvPropertyGet {
    fn prop_key(self) -> Cow<'static, str> {
        match self {
            Self::Duration => Cow::Borrowed("duration"),
            Self::MediaTitle => Cow::Borrowed("media-title"),
            Self::TimePos => Cow::Borrowed("time-pos"),
            Self::PauseStatus => Cow::Borrowed("pause"),
            Self::Volume => Cow::Borrowed("volume"),
            Self::PercentPos => Cow::Borrowed("percent-pos"),
            Self::PlaylistCount => Cow::Borrowed("playlist-count"),
            Self::NthPlaylistItemTitle(n) => {
                Cow::Owned(String::from("playlist/") + n.to_string().as_str() + "/filename")
            }
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

    pub fn cycle_pause_status(&self) -> Result<(), YtuiMvpAudioError> {
        let current_status = self
            .get_property::<bool>(MpvPropertyGet::PauseStatus)?
            .unwrap();
        self.set_property(
            MpvPropertyGet::PauseStatus.prop_key().as_ref(),
            !current_status,
        )
    }

    pub fn execute_command(&self, command: MpvCommand) -> Result<(), YtuiMvpAudioError> {
        let mpv_cmd = |c, args| {
            self.mpv_handle
                .command(c, args)
                .map_err(|e| YtuiMvpAudioError::mpv("executing command", e))
        };
        match command {
            MpvCommand::LoadFile { stream, kind } => match kind {
                LoadFileArg::Replace => mpv_cmd("loadfile", &[stream, "replace"]),

                LoadFileArg::Append { force_media_title } => {
                    mpv_cmd("loadfile", &[stream, "append"])
                }
            },
        }
    }

    pub fn enable_video(&self) -> Result<(), YtuiMvpAudioError> {
        self.set_property("video", "auto")
    }

    pub fn disable_video(&self) -> Result<(), YtuiMvpAudioError> {
        self.set_property("video", "no")
    }

    pub fn get_property<PropertyValueType: libmpv2::GetData + std::fmt::Debug>(
        &self,
        property: MpvPropertyGet,
    ) -> Result<Option<PropertyValueType>, YtuiMvpAudioError> {
        let property_key = property.prop_key();
        match self
            .mpv_handle
            .get_property::<PropertyValueType>(property_key.as_ref())
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
