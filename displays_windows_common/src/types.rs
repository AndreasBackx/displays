use std::str::FromStr;

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
