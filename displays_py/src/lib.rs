#![doc = include_str!("../docs/crate.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/python-dev-setup.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/displays-python.md")]

use display::{Display, LogicalDisplay, PhysicalDisplay, Size};
use display_identifier::DisplayIdentifier;
use display_match::DisplayMatch;
use display_update::{DisplayUpdate, LogicalDisplayUpdateContent, PhysicalDisplayUpdateContent};
use display_update_result::{DisplayUpdateResult, FailedDisplayUpdate};
use displays_core::{self as lib};
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tracing_subscriber::util::SubscriberInitExt;

use crate::display::{Orientation, Point};

mod display;
mod display_identifier;
mod display_match;
mod display_update;
mod display_update_result;

fn into_py_runtime_error(err: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

#[tracing::instrument]
#[pyfunction]
fn get(ids: Vec<DisplayIdentifier>) -> PyResult<Vec<DisplayMatch>> {
    let displays = lib::manager::DisplayManager::get(ids.into_iter().map(|id| id.into()).collect())
        .map_err(into_py_runtime_error)?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    Ok(displays)
}

#[tracing::instrument]
#[pyfunction]
fn query() -> PyResult<Vec<Display>> {
    let displays = lib::manager::DisplayManager::query()
        .map_err(into_py_runtime_error)?
        .into_iter()
        .map(|display| display.into())
        .collect::<Vec<_>>();

    Ok(displays)
}

#[tracing::instrument]
#[pyfunction]
fn _apply(updates: Vec<DisplayUpdate>, validate: bool) -> PyResult<Vec<DisplayUpdateResult>> {
    let displays = lib::manager::DisplayManager::apply(
        updates.into_iter().map(|update| update.into()).collect(),
        validate,
    )
    .map_err(into_py_runtime_error)?
    .into_iter()
    .map(Into::into)
    .collect::<Vec<_>>();

    Ok(displays)
}

#[pyfunction]
fn apply(updates: Vec<DisplayUpdate>) -> PyResult<Vec<DisplayUpdateResult>> {
    _apply(updates, false)
}

#[pyfunction]
fn validate(updates: Vec<DisplayUpdate>) -> PyResult<Vec<DisplayUpdateResult>> {
    _apply(updates, true)
}

#[allow(dead_code)]
fn initialize_tracing() {
    tracing_subscriber::registry().init();
}

#[pymodule]
fn displays(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(apply, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_class::<DisplayIdentifier>()?;
    m.add_class::<DisplayMatch>()?;
    m.add_class::<DisplayUpdate>()?;
    m.add_class::<DisplayUpdateResult>()?;
    m.add_class::<FailedDisplayUpdate>()?;
    m.add_class::<LogicalDisplayUpdateContent>()?;
    m.add_class::<PhysicalDisplayUpdateContent>()?;
    m.add_class::<Display>()?;
    m.add_class::<LogicalDisplay>()?;
    m.add_class::<PhysicalDisplay>()?;
    m.add_class::<Orientation>()?;
    m.add_class::<Point>()?;
    m.add_class::<Size>()?;
    Ok(())
}
