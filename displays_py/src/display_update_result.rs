use displays_core::{self as lib};
use pyo3::prelude::*;

use crate::{display_identifier::DisplayIdentifier, display_update::DisplayUpdate};

#[pyclass(str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FailedDisplayUpdate {
    #[pyo3(get, set)]
    matched_id: DisplayIdentifier,
    #[pyo3(get, set)]
    message: String,
}

#[pyclass(str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayUpdateResult {
    #[pyo3(get, set)]
    requested_update: DisplayUpdate,
    #[pyo3(get, set)]
    applied: Vec<DisplayIdentifier>,
    #[pyo3(get, set)]
    failed: Vec<FailedDisplayUpdate>,
}

#[pymethods]
impl FailedDisplayUpdate {
    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for FailedDisplayUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FailedDisplayUpdate(matched_id={matched_id}, message={message:?})",
            matched_id = self.matched_id,
            message = self.message,
        )
    }
}

#[pymethods]
impl DisplayUpdateResult {
    pub fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl std::fmt::Display for DisplayUpdateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DisplayUpdateResult(requested_update={requested_update}, applied={applied:?}, failed={failed:?})",
            requested_update = self.requested_update,
            applied = self.applied,
            failed = self.failed,
        )
    }
}

impl From<lib::manager::FailedDisplayUpdate> for FailedDisplayUpdate {
    fn from(value: lib::manager::FailedDisplayUpdate) -> Self {
        Self {
            matched_id: value.matched_id.into(),
            message: value.message,
        }
    }
}

impl From<lib::manager::DisplayUpdateResult> for DisplayUpdateResult {
    fn from(value: lib::manager::DisplayUpdateResult) -> Self {
        Self {
            requested_update: value.requested_update.into(),
            applied: value.applied.into_iter().map(Into::into).collect(),
            failed: value.failed.into_iter().map(Into::into).collect(),
        }
    }
}
