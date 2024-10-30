use displays_lib as lib;
use pyo3::{exceptions::PyException, prelude::*, types::PyIterator};

#[pyclass]
#[derive(FromPyObject)]
struct DisplayConfig {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub path: Option<String>,
    #[pyo3(get, set)]
    pub enabled: bool,
}

impl From<&DisplayConfig> for lib::display_config::DisplayConfig {
    fn from(value: &DisplayConfig) -> Self {
        lib::display_config::DisplayConfig {
            name: value.name.clone(),
            path: value.path.clone(),
            enabled: value.enabled,
        }
    }
}

#[pymethods]
impl DisplayConfig {
    #[new]
    #[pyo3(signature = (*, name, path, enabled))]
    fn new(name: String, path: Option<String>, enabled: bool) -> Self {
        Self {
            name,
            path,
            enabled,
        }
    }
}

#[pyclass]
struct State {
    state: lib::state::State,
}

#[pymethods]
impl State {
    #[staticmethod]
    fn query() -> PyResult<State> {
        Ok(State {
            state: lib::state::State::query()
                .map_err(|err| PyException::new_err(err.to_string()))?,
        })
    }

    fn update(&mut self, setups: Vec<PyRef<DisplayConfig>>) -> PyResult<()> {
        let displays = setups
            .into_iter()
            .map(|py_display_config| (&*py_display_config).into())
            .collect();
        let display_configs = lib::display_config::DisplayConfigs { displays };
        self.state
            .update(display_configs)
            .map_err(|err| PyException::new_err(err.to_string()))
    }

    fn _apply(&mut self, validate: bool) -> PyResult<()> {
        self.state
            .apply(validate)
            .map_err(|err| PyException::new_err(err.to_string()))
    }

    fn apply(&mut self) -> PyResult<()> {
        self._apply(false)
    }

    fn validate(&mut self) -> PyResult<()> {
        self._apply(true)
    }

    fn __repr__(&self) -> String {
        format!("State({:?})", std::ptr::addr_of!(self))
    }

    fn __str__(&self) -> String {
        format!("{}", self.state)
    }
}

#[pymodule]
fn displays(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<State>()?;
    m.add_class::<DisplayConfig>()?;
    Ok(())
}
