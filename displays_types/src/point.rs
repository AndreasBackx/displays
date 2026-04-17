use std::str::FromStr;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::POINTL;

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

#[cfg(target_os = "windows")]
impl From<&POINTL> for Point {
    fn from(value: &POINTL) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[cfg(target_os = "windows")]
impl From<&Point> for POINTL {
    fn from(value: &Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
