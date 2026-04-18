use displays_core::{self as lib};
use pyo3::prelude::*;

#[pyclass(str, from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifier {
    #[pyo3(get, set)]
    pub name: Option<String>,
    #[pyo3(get, set)]
    pub serial_number: Option<String>,
}

#[pymethods]
impl DisplayIdentifier {
    #[new]
    #[pyo3(signature = (*, name=None, serial_number=None))]
    pub fn new(name: Option<String>, serial_number: Option<String>) -> Self {
        Self {
            name,
            serial_number,
        }
    }

    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl From<DisplayIdentifier> for lib::types::DisplayIdentifier {
    fn from(value: DisplayIdentifier) -> Self {
        lib::types::DisplayIdentifier {
            name: value.name,
            serial_number: value.serial_number,
        }
    }
}

impl From<lib::types::DisplayIdentifier> for DisplayIdentifier {
    fn from(value: lib::types::DisplayIdentifier) -> Self {
        DisplayIdentifier {
            name: value.name,
            serial_number: value.serial_number,
        }
    }
}

impl std::fmt::Display for DisplayIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DisplayIdentifier(name={name}, serial_number={serial_number})",
            name = self
                .name
                .as_ref()
                .map(|name| format!("{name:?}"))
                .unwrap_or("None".to_owned()),
            serial_number = self
                .serial_number
                .as_ref()
                .map(|serial_number| format!("{serial_number:?}"))
                .unwrap_or("None".to_owned())
        )
    }
}
