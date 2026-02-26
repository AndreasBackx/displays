use std::collections::{BTreeMap, BTreeSet};

use display::{Display, LogicalDisplay, PhysicalDisplay};
use display_identifier::DisplayIdentifier;
use display_update::{DisplayUpdate, LogicalDisplayUpdateContent, PhysicalDisplayUpdateContent};
use displays_lib::{self as lib};
use pyo3::{exceptions::PyException, prelude::*};
use tracing_subscriber::util::SubscriberInitExt;

use crate::display::{Orientation, Point};

mod display;
mod display_identifier;
mod display_update;

#[tracing::instrument]
#[pyfunction]
fn get(ids: BTreeSet<DisplayIdentifier>) -> PyResult<BTreeMap<DisplayIdentifier, Display>> {
    let displays = lib::manager::DisplayManager::get(ids.into_iter().map(|id| id.into()).collect())
        .map_err(|err| PyException::new_err(format!("{err:#?}")))?
        .into_iter()
        .map(|(id, display)| (id.into(), display.into()))
        .collect::<BTreeMap<_, _>>();

    Ok(displays)
}

#[tracing::instrument]
#[pyfunction]
fn query() -> PyResult<Vec<Display>> {
    let displays = lib::manager::DisplayManager::query()
        .map_err(|err| PyException::new_err(format!("{err:#?}")))?
        .into_iter()
        .map(|display| display.into())
        .collect::<Vec<_>>();

    Ok(displays)
}

#[tracing::instrument]
#[pyfunction]
fn _apply(updates: Vec<DisplayUpdate>, validate: bool) -> PyResult<Vec<DisplayUpdate>> {
    let displays = lib::manager::DisplayManager::apply(
        updates.into_iter().map(|update| update.into()).collect(),
        validate,
    )
    .map_err(|err| PyException::new_err(format!("{err:#?}")))?
    .into_iter()
    .map(|display| display.into())
    .collect::<Vec<_>>();

    Ok(displays)
}

#[pyfunction]
fn apply(updates: Vec<DisplayUpdate>) -> PyResult<Vec<DisplayUpdate>> {
    _apply(updates, false)
}

#[pyfunction]
fn validate(updates: Vec<DisplayUpdate>) -> PyResult<Vec<DisplayUpdate>> {
    _apply(updates, true)
}

#[pyfunction]
pub fn initialize_tracing(py_impl: Bound<'_, PyAny>) {
    let _ = py_impl;
    tracing_subscriber::registry().init();
}

#[pymodule]
fn displays(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(initialize_tracing, m)?)?;

    m.add_function(wrap_pyfunction!(apply, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_class::<DisplayIdentifier>()?;
    m.add_class::<DisplayUpdate>()?;
    m.add_class::<LogicalDisplayUpdateContent>()?;
    m.add_class::<PhysicalDisplayUpdateContent>()?;
    m.add_class::<Display>()?;
    m.add_class::<LogicalDisplay>()?;
    m.add_class::<PhysicalDisplay>()?;
    m.add_class::<Orientation>()?;
    m.add_class::<Point>()?;
    Ok(())
}
