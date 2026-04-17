#[cfg(target_os = "windows")]
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_ROTATION, DISPLAYCONFIG_ROTATION_IDENTITY,
    DISPLAYCONFIG_ROTATION_ROTATE180, DISPLAYCONFIG_ROTATION_ROTATE270,
    DISPLAYCONFIG_ROTATION_ROTATE90,
};

/// Display orientation in degrees clockwise.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Orientation {
    Landscape = 0,
    Portrait = 90,
    LandscapeFlipped = 180,
    PortraitFlipped = 270,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Landscape
    }
}

#[cfg(target_os = "windows")]
impl From<&DISPLAYCONFIG_ROTATION> for Orientation {
    fn from(value: &DISPLAYCONFIG_ROTATION) -> Self {
        match *value {
            DISPLAYCONFIG_ROTATION_IDENTITY => Orientation::Landscape,
            DISPLAYCONFIG_ROTATION_ROTATE90 => Orientation::Portrait,
            DISPLAYCONFIG_ROTATION_ROTATE180 => Orientation::LandscapeFlipped,
            DISPLAYCONFIG_ROTATION_ROTATE270 => Orientation::PortraitFlipped,
            _ => unimplemented!("Nonexistent display orientation."),
        }
    }
}

#[cfg(target_os = "windows")]
impl From<&Orientation> for DISPLAYCONFIG_ROTATION {
    fn from(value: &Orientation) -> Self {
        match *value {
            Orientation::Landscape => DISPLAYCONFIG_ROTATION_IDENTITY,
            Orientation::Portrait => DISPLAYCONFIG_ROTATION_ROTATE90,
            Orientation::LandscapeFlipped => DISPLAYCONFIG_ROTATION_ROTATE180,
            Orientation::PortraitFlipped => DISPLAYCONFIG_ROTATION_ROTATE270,
        }
    }
}
