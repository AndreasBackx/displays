use std::collections::BTreeSet;

use displays_lib as lib;
use pyo3::{exceptions::PyException, prelude::*};

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DisplayConfig {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub path: Option<String>,
    #[pyo3(get, set)]
    pub is_enabled: bool,
}

impl From<DisplayConfig> for lib::manager::DisplayConfig {
    fn from(value: DisplayConfig) -> Self {
        lib::manager::DisplayConfig {
            name: value.name,
            path: value.path,
            is_enabled: value.is_enabled,
        }
    }
}

#[pymethods]
impl DisplayConfig {
    #[new]
    #[pyo3(signature = (*, name, path=None, is_enabled=true))]
    fn new(name: String, path: Option<String>, is_enabled: bool) -> Self {
        Self {
            name,
            path,
            is_enabled,
        }
    }

    fn __str__(&self) -> String {
        let Self {
            name,
            path,
            is_enabled,
        } = self;
        format!(
            "DisplayConfig(name={name:?}, path={path}, is_enabled={is_enabled})",
            path = path
                .as_ref()
                .map(|path| format!("{path:?}"))
                .unwrap_or("None".to_owned())
        )
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

fn try_new_state() -> PyResult<lib::state::State> {
    lib::state::State::try_new().map_err(|err| PyException::new_err(err.to_string()))
}

fn _apply(configs: BTreeSet<DisplayConfig>, validate: bool) -> PyResult<()> {
    let displays = configs
        .into_iter()
        .map(|py_display_config| py_display_config.into())
        .collect();
    let display_configs = lib::manager::DisplayConfigs { displays };

    let mut state = try_new_state()?;

    state
        .update(display_configs)
        .map_err(|err| PyException::new_err(err.to_string()))?;
    state
        .apply(validate)
        .map_err(|err| PyException::new_err(err.to_string()))
}

#[pyfunction]
fn apply(configs: BTreeSet<DisplayConfig>) -> PyResult<()> {
    _apply(configs, false)
}

#[pyfunction]
fn validate(configs: BTreeSet<DisplayConfig>) -> PyResult<()> {
    _apply(configs, true)
}

#[pyfunction]
fn query() -> PyResult<BTreeSet<DisplayConfig>> {
    let state = try_new_state()?;
    let configs = state
        .query()
        .map_err(|err| PyException::new_err(err.to_string()))?;

    Ok(configs
        .into_iter()
        .map(|config| DisplayConfig {
            name: config.name,
            path: config.path,
            is_enabled: config.is_enabled,
        })
        .collect())
}

#[pymodule]
fn displays(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(apply, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_class::<DisplayConfig>()?;
    Ok(())
}
