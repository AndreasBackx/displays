use displays_lib::{self as lib};
use pyo3::prelude::*;

use crate::display_identifier::DisplayIdentifier;

#[pyclass(str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayUpdate {
    #[pyo3(get, set)]
    id: DisplayIdentifier,
    #[pyo3(get, set)]
    logical: Option<LogicalDisplayUpdateContent>,
    #[pyo3(get, set)]
    physical: Option<PhysicalDisplayUpdateContent>,
}

#[pyclass(str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalDisplayUpdateContent {
    #[pyo3(get, set)]
    is_enabled: Option<bool>,
}

#[pyclass(str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayUpdateContent {
    #[pyo3(get, set)]
    brightness: Option<u32>,
}

#[pymethods]
impl DisplayUpdate {
    #[new]
    #[pyo3(signature = (id, *, logical=None, physical=None))]
    fn new(
        id: DisplayIdentifier,
        logical: Option<LogicalDisplayUpdateContent>,
        physical: Option<PhysicalDisplayUpdateContent>,
    ) -> Self {
        Self {
            id,
            logical,
            physical,
        }
    }

    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for DisplayUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DisplayUpdate(id={id}, logical={logical}, physical={physical})",
            id = self.id,
            logical = self
                .logical
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or("None".to_string()),
            physical = self
                .physical
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or("None".to_string()),
        )
    }
}

#[pymethods]
impl PhysicalDisplayUpdateContent {
    #[new]
    #[pyo3(signature = (*, brightness))]
    fn new(brightness: Option<u32>) -> Self {
        Self { brightness }
    }

    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for PhysicalDisplayUpdateContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PhysicalDisplayUpdateContent(brightness={brightness})",
            brightness = self
                .brightness
                .map(|value| value.to_string())
                .unwrap_or("None".to_string()),
        )
    }
}

#[pymethods]
impl LogicalDisplayUpdateContent {
    #[new]
    #[pyo3(signature = (*, is_enabled))]
    fn new(is_enabled: Option<bool>) -> Self {
        Self { is_enabled }
    }

    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for LogicalDisplayUpdateContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LogicalDisplayUpdateContent(is_enabled={is_enabled})",
            is_enabled = self
                .is_enabled
                .map(|value| if value { "True" } else { "False" })
                .unwrap_or("None"),
        )
    }
}

impl From<lib::display::DisplayUpdate> for DisplayUpdate {
    fn from(value: lib::display::DisplayUpdate) -> Self {
        DisplayUpdate {
            id: value.id.into(),
            logical: value.logical.map(|logical| logical.into()),
            physical: value.physical.map(|physical| physical.into()),
        }
    }
}

impl From<DisplayUpdate> for lib::display::DisplayUpdate {
    fn from(value: DisplayUpdate) -> Self {
        lib::display::DisplayUpdate {
            id: value.id.into(),
            logical: value.logical.map(|logical| logical.into()),
            physical: value.physical.map(|physical| physical.into()),
        }
    }
}

impl From<lib::logical_display::LogicalDisplayUpdateContent> for LogicalDisplayUpdateContent {
    fn from(value: lib::logical_display::LogicalDisplayUpdateContent) -> Self {
        LogicalDisplayUpdateContent {
            is_enabled: value.is_enabled,
        }
    }
}

impl From<LogicalDisplayUpdateContent> for lib::logical_display::LogicalDisplayUpdateContent {
    fn from(value: LogicalDisplayUpdateContent) -> Self {
        lib::logical_display::LogicalDisplayUpdateContent {
            is_enabled: value.is_enabled,
        }
    }
}

impl From<lib::physical_display::PhysicalDisplayUpdateContent> for PhysicalDisplayUpdateContent {
    fn from(value: lib::physical_display::PhysicalDisplayUpdateContent) -> Self {
        PhysicalDisplayUpdateContent {
            brightness: value.brightness,
        }
    }
}
impl From<PhysicalDisplayUpdateContent> for lib::physical_display::PhysicalDisplayUpdateContent {
    fn from(value: PhysicalDisplayUpdateContent) -> Self {
        lib::physical_display::PhysicalDisplayUpdateContent {
            brightness: value.brightness,
        }
    }
}
