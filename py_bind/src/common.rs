use eatmud::{Datelike, NaiveDate};
use numpy::PyArray1;
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
pub(crate) fn rsdates_to_pyarr<'py>(
    py: Python<'py>,
    dates: &[NaiveDate],
) -> PyResult<Bound<'py, PyAny>> {
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let days: Vec<i64> = dates.iter().map(|d| (*d - epoch).num_days()).collect();
    let arr = PyArray1::from_vec(py, days);
    arr.call_method1("view", ("datetime64[D]",))
}

#[inline]
pub(crate) fn map_err<T>(e: T) -> PyErr
where
    T: std::fmt::Display,
{
    PyRuntimeError::new_err(e.to_string())
}
