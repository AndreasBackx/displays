use displays_core::{self as lib};
use pyo3::prelude::*;

use crate::{display::Display, display_identifier::DisplayIdentifier};

#[pyclass(str, from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayMatch {
    #[pyo3(get, set)]
    requested_id: DisplayIdentifier,
    #[pyo3(get, set)]
    matched_id: DisplayIdentifier,
    #[pyo3(get, set)]
    display: Display,
}

#[pymethods]
impl DisplayMatch {
    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for DisplayMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DisplayMatch(requested_id={requested_id}, matched_id={matched_id}, display={display})",
            requested_id = self.requested_id,
            matched_id = self.matched_id,
            display = self.display,
        )
    }
}

impl From<lib::types::DisplayMatch> for DisplayMatch {
    fn from(value: lib::types::DisplayMatch) -> Self {
        Self {
            requested_id: value.requested_id.into(),
            matched_id: value.matched_id.into(),
            display: value.display.into(),
        }
    }
}
