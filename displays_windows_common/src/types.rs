use std::str::FromStr;

use windows::Win32::{
    Devices::Display::{
        DISPLAYCONFIG_PIXELFORMAT, DISPLAYCONFIG_PIXELFORMAT_16BPP,
        DISPLAYCONFIG_PIXELFORMAT_24BPP, DISPLAYCONFIG_PIXELFORMAT_32BPP,
        DISPLAYCONFIG_PIXELFORMAT_8BPP, DISPLAYCONFIG_PIXELFORMAT_NONGDI, DISPLAYCONFIG_ROTATION,
        DISPLAYCONFIG_ROTATION_IDENTITY, DISPLAYCONFIG_ROTATION_ROTATE180,
        DISPLAYCONFIG_ROTATION_ROTATE270, DISPLAYCONFIG_ROTATION_ROTATE90,
    },
    Foundation::POINTL,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifier {
    pub name: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifierInner {
    pub outer: DisplayIdentifier,
    pub path: Option<String>,
    pub gdi_device_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Brightness(u8);

impl Brightness {
    pub const fn new(value: u8) -> Self {
        if value > 100 {
            panic!("Brightness needs to be between 0 and 100");
        }
        Self(value)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum PixelFormat {
    BPP8 = 1,
    BPP16 = 2,
    BPP24 = 3,
    BPP32 = 4,
    NONGDI = 5,
}

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

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

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

impl From<&POINTL> for Point {
    fn from(value: &POINTL) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<&Point> for POINTL {
    fn from(value: &Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl FromStr for Point {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = value.split(',').collect();
        if items.len() != 2 {
            return Err(format!("`{value}` needs to be of the format `x,y`."));
        }

        let numbers: Vec<i32> = items
            .into_iter()
            .map(|item| {
                item.parse::<i32>()
                    .map_err(|_| format!("`{item}` is not a valid signed number"))
            })
            .collect::<Result<_, Self::Err>>()?;

        Ok(Point {
            x: numbers[0],
            y: numbers[1],
        })
    }
}
