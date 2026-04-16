use displays_core::{self as lib};
use pyo3::prelude::*;

use crate::display_identifier::DisplayIdentifier;

#[pyclass(str, eq, frozen, immutable_type, ord)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Display {
    #[pyo3(get)]
    id: DisplayIdentifier,
    #[pyo3(get)]
    logical: LogicalDisplay,
    #[pyo3(get)]
    physical: Option<PhysicalDisplay>,
}

#[pyclass(str, eq, frozen, immutable_type, ord)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalDisplay {
    #[pyo3(get)]
    is_enabled: bool,
    #[pyo3(get)]
    orientation: Orientation,
    #[pyo3(get)]
    width: Option<u32>,
    #[pyo3(get)]
    height: Option<u32>,
    #[pyo3(get)]
    position: Option<Point>,
}

#[pyclass(
    str,
    eq,
    eq_int,
    frozen,
    immutable_type,
    ord,
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Orientation {
    Landscape = 0,          // 0° (normal)
    Portrait = 90,          // 90° clockwise
    LandscapeFlipped = 180, // 180°
    PortraitFlipped = 270,  // 270° clockwise
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Orientation({name}, angle={angle})",
            name = self.name(),
            angle = self
        )
    }
}

impl From<lib::types::Orientation> for Orientation {
    fn from(value: lib::types::Orientation) -> Self {
        match value {
            lib::types::Orientation::Landscape => Self::Landscape,
            lib::types::Orientation::Portrait => Self::Portrait,
            lib::types::Orientation::LandscapeFlipped => Self::LandscapeFlipped,
            lib::types::Orientation::PortraitFlipped => Self::PortraitFlipped,
        }
    }
}

#[pymethods]
impl Orientation {
    #[getter]
    pub fn name(&self) -> &str {
        match self {
            Orientation::Landscape => "Landscape",
            Orientation::Portrait => "Portrait",
            Orientation::LandscapeFlipped => "Landscape Flipped",
            Orientation::PortraitFlipped => "Portrait Flipped",
        }
    }

    #[getter]
    pub fn value(&self) -> i32 {
        self.clone() as i32
    }
}

#[pyclass(str, eq, frozen, immutable_type, ord)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    #[pyo3(get)]
    x: i32,
    #[pyo3(get)]
    y: i32,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point(x={x}, y={y})", x = self.x, y = self.y,)
    }
}

#[pymethods]
impl Point {
    #[new]
    #[pyo3(signature = (*, x, y))]
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl From<Point> for lib::types::Point {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<lib::types::Point> for Point {
    fn from(value: lib::types::Point) -> Self {
        Point {
            x: value.x,
            y: value.y,
        }
    }
}

#[pyclass(str, eq, frozen, immutable_type, ord)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplay {
    #[pyo3(get)]
    brightness: u8,
    #[pyo3(get)]
    scale_factor: i32,
}

#[pymethods]
impl Display {
    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Display(id={id}, logical={logical}, physical={physical})",
            id = self.id,
            logical = self.logical,
            physical = self
                .physical
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or("None".to_string()),
        )
    }
}

impl From<lib::display::Display> for Display {
    fn from(value: lib::display::Display) -> Self {
        Display {
            id: value.id().outer.into(),
            logical: LogicalDisplay {
                is_enabled: value.logical.state.is_enabled,
                orientation: value.logical.state.orientation.into(),
                height: value.logical.state.height,
                width: value.logical.state.width,
                position: value.logical.state.position.map(|point| point.into()),
            },
            physical: value.physical.map(|physical| PhysicalDisplay {
                brightness: physical.state.brightness.value(),
                scale_factor: physical.state.scale_factor,
            }),
        }
    }
}

#[pymethods]
impl LogicalDisplay {
    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for LogicalDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LogicalDisplay(is_enabled={is_enabled})",
            is_enabled = self.is_enabled,
        )
    }
}

#[pymethods]
impl PhysicalDisplay {
    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for PhysicalDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PhysicalDisplay(brightness={brightness}, scale_factor={scale_factor})",
            brightness = self.brightness,
            scale_factor = self.scale_factor,
        )
    }
}
