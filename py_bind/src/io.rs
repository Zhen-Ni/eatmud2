use crate::data::PyStock;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Read stock data from GuoTaiAn's txt output file.
#[pyfunction]
pub fn read_tdx(path: &str) -> PyResult<PyStock> {
    let stock = eatmud::io::read_tdx(path).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyStock { inner: stock })
}

