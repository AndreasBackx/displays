use displays_lib::{self as lib};
use pyo3::prelude::*;

use crate::display_identifier::DisplayIdentifier;

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Display {
    #[pyo3(get)]
    id: DisplayIdentifier,
    #[pyo3(get)]
    logical: LogicalDisplay,
    #[pyo3(get)]
    physical: PhysicalDisplay,
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalDisplay {
    #[pyo3(get)]
    is_enabled: bool,
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplay {
    #[pyo3(get)]
    brightness: u8,
}

#[pymethods]
impl Display {
    pub fn __str__(&self) -> String {
        format!("{self}",)
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Display(id={id}, logical={logical}, physical={physical})",
            id = self.id,
            logical = self.logical,
            physical = self.physical,
        )
    }
}

impl From<lib::display::Display> for Display {
    fn from(value: lib::display::Display) -> Self {
        Display {
            id: value.id().outer.into(),
            logical: LogicalDisplay {
                is_enabled: value.logical.is_enabled,
            },
            physical: PhysicalDisplay {
                // brightness: value.physical.brightness,
                brightness: 0,
            },
        }
    }
}

#[pymethods]
impl LogicalDisplay {
    pub fn __str__(&self) -> String {
        format!("{self}",)
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
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
    pub fn __str__(&self) -> String {
        format!("{self}",)
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl std::fmt::Display for PhysicalDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PhysicalDisplay(brightness={brightness})",
            brightness = self.brightness,
        )
    }
}
