use std::collections::{BTreeMap, BTreeSet};

use display::Display;
use display_identifier::DisplayIdentifier;
use display_update::DisplayUpdate;
use displays_lib::{self as lib};
use pyo3::{exceptions::PyException, prelude::*};

mod display;
mod display_identifier;
mod display_update;

#[pyfunction]
fn get(ids: BTreeSet<DisplayIdentifier>) -> PyResult<BTreeMap<DisplayIdentifier, Display>> {
    let displays = lib::manager::DisplayManager::get(ids.into_iter().map(|id| id.into()).collect())
        .map_err(|err| PyException::new_err(err.to_string()))?
        .into_iter()
        .map(|(id, display)| (id.into(), display.into()))
        .collect::<BTreeMap<_, _>>();

    Ok(displays)
}

#[pyfunction]
fn query() -> PyResult<Vec<Display>> {
    let displays = lib::manager::DisplayManager::query()
        .map_err(|err| PyException::new_err(err.to_string()))?
        .into_iter()
        .map(|display| display.into())
        .collect::<Vec<_>>();

    Ok(displays)
}

#[pyfunction]
fn _apply(updates: Vec<DisplayUpdate>, validate: bool) -> PyResult<Vec<DisplayUpdate>> {
    let displays = lib::manager::DisplayManager::apply(
        updates.into_iter().map(|update| update.into()).collect(),
        validate,
    )
    .map_err(|err| PyException::new_err(err.to_string()))?
    .into_iter()
    .map(|display| display.into())
    .collect::<Vec<_>>();

    Ok(displays)
}

#[pymodule]
fn displays(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(apply, m)?)?;
    // m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(get, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_class::<DisplayIdentifier>()?;
    Ok(())
}
