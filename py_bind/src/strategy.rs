use crate::chrono::PyWeekday;
use crate::transaction::PyTransactionIterator;
use crate::common::map_err;
use eatmud::strategy::aip_monthly as core_aip_monthly;
use eatmud::strategy::kelly::{
    KellyIndicator as CoreKellyIndicator, kelly_hint as core_kelly_hint,
    kelly_weekly as core_kelly_weekly,
};
use pyo3::prelude::*;

#[pyfunction]
pub fn aip_monthly(
    it: &mut PyTransactionIterator,
    day: u32,
    amounts: Vec<f64>,
    fees: Vec<f64>,
) -> PyResult<()> {
    core_aip_monthly(&mut it.inner, day, &amounts, &fees).map_err(map_err)
}

#[pyclass(name = "KellyIndicator")]
pub struct PyKellyIndicator {
    inner: CoreKellyIndicator,
}

#[pymethods]
impl PyKellyIndicator {
    #[getter]
    fn position(&self) -> f64 {
        self.inner.position
    }
    #[getter]
    fn upper_bound(&self) -> f64 {
        self.inner.upper_bound
    }
    #[getter]
    fn lower_bound(&self) -> f64 {
        self.inner.lower_bound
    }
    #[getter]
    fn upper_risk_bound(&self) -> f64 {
        self.inner.upper_risk_bound
    }
    #[getter]
    fn lower_risk_bound(&self) -> f64 {
        self.inner.lower_risk_bound
    }
}

#[pyfunction]
pub fn kelly_hint(
    it: &PyTransactionIterator,
    fund_index: usize,
    weekday: PyWeekday,
    n: usize,
    inflation: f64,
    risk_bound: f64,
) -> PyResult<PyKellyIndicator> {
    core_kelly_hint(
        &it.inner,
        fund_index,
        weekday.inner,
        n,
        inflation,
        risk_bound,
    )
    .map(|r| PyKellyIndicator { inner: r })
    .map_err(map_err)
}

#[pyfunction]
pub fn kelly_weekly(
    it: &mut PyTransactionIterator,
    weekday: PyWeekday,
    ns: Vec<usize>,
    inflations: Vec<f64>,
    risk_bounds: Vec<f64>,
) -> PyResult<()> {
    core_kelly_weekly(&mut it.inner, weekday.inner, &ns, &inflations, &risk_bounds).map_err(map_err)
}
