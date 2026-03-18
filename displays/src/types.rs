use std::str::FromStr;

/// Platform pixel format identifiers used by logical display state.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum PixelFormat {
    BPP8 = 1,
    BPP16 = 2,
    BPP24 = 3,
    BPP32 = 4,
    NONGDI = 5,
}

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

/// A 2D point used for logical display positioning.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Point {
    /// Horizontal position.
    pub x: i32,
    /// Vertical position.
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
