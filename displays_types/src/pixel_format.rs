#[cfg(target_os = "windows")]
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_PIXELFORMAT, DISPLAYCONFIG_PIXELFORMAT_16BPP, DISPLAYCONFIG_PIXELFORMAT_24BPP,
    DISPLAYCONFIG_PIXELFORMAT_32BPP, DISPLAYCONFIG_PIXELFORMAT_8BPP,
    DISPLAYCONFIG_PIXELFORMAT_NONGDI,
};

/// Platform pixel format identifiers used by logical display state.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum PixelFormat {
    BPP8 = 1,
    BPP16 = 2,
    BPP24 = 3,
    BPP32 = 4,
    NONGDI = 5,
}

#[cfg(target_os = "windows")]
impl From<&DISPLAYCONFIG_PIXELFORMAT> for PixelFormat {
    fn from(value: &DISPLAYCONFIG_PIXELFORMAT) -> Self {
        match *value {
            DISPLAYCONFIG_PIXELFORMAT_8BPP => PixelFormat::BPP8,
            DISPLAYCONFIG_PIXELFORMAT_16BPP => PixelFormat::BPP16,
            DISPLAYCONFIG_PIXELFORMAT_24BPP => PixelFormat::BPP24,
            DISPLAYCONFIG_PIXELFORMAT_32BPP => PixelFormat::BPP32,
            DISPLAYCONFIG_PIXELFORMAT_NONGDI => PixelFormat::NONGDI,
            _ => unimplemented!("Nonexistent pixel format."),
        }
    }
}

#[cfg(target_os = "windows")]
impl From<&PixelFormat> for DISPLAYCONFIG_PIXELFORMAT {
    fn from(value: &PixelFormat) -> Self {
        match *value {
            PixelFormat::BPP8 => DISPLAYCONFIG_PIXELFORMAT_8BPP,
            PixelFormat::BPP16 => DISPLAYCONFIG_PIXELFORMAT_16BPP,
            PixelFormat::BPP24 => DISPLAYCONFIG_PIXELFORMAT_24BPP,
            PixelFormat::BPP32 => DISPLAYCONFIG_PIXELFORMAT_32BPP,
            PixelFormat::NONGDI => DISPLAYCONFIG_PIXELFORMAT_NONGDI,
        }
    }
}
