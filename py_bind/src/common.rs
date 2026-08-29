use eatmud::{Datelike, NaiveDate};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDate;
use pyo3::types::PyDateAccess;

#[inline]
pub(crate) fn pydate_to_rsdate(obj: &Bound<'_, PyAny>) -> PyResult<NaiveDate> {
    if let Ok(date) = obj.cast::<PyDate>() {
        NaiveDate::from_ymd_opt(
            date.get_year(),
            date.get_month() as u32,
            date.get_day() as u32,
        )
        .ok_or_else(|| PyRuntimeError::new_err("Invalid date value"))
    } else if let Ok(s) = obj.extract::<String>() {
        NaiveDate::parse_from_str(&s, "%Y%m%d")
            .or_else(|_| NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
            .map_err(|_| PyRuntimeError::new_err(format!("Invalid date string: {}", s)))
    } else {
        Err(PyRuntimeError::new_err("Expected date or string"))
    }
}

#[inline]
pub(crate) fn rsdate_to_pydate(py: Python<'_>, d: NaiveDate) -> PyResult<Bound<'_, PyDate>> {
    PyDate::new(py, d.year(), d.month() as u8, d.day() as u8)
}

#[inline]
pub(crate) fn map_err<T>(e: T) -> PyErr
where
    T: std::fmt::Display,
{
    PyRuntimeError::new_err(e.to_string())
}
