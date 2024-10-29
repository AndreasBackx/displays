use displays_lib as lib;
use pyo3::{exceptions::PyException, prelude::*};

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

    fn __repr__(&self) -> String {
        format!("{}", self.state)
    }
}

#[pymodule]
fn displays_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<State>()?;
    Ok(())
}
