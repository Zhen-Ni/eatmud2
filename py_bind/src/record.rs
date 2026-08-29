use pyo3::prelude::*;
use pyo3::types::{PyDate, PyTuple, PyType};

use eatmud::record::{
    ConciseRecordSlice as CoreConciseRecordSlice, DetailedRecordSlice as CoreDetailedRecordSlice,
};
use eatmud::{ConciseRecord as CoreConciseRecord, DetailedRecord as CoreDetailedRecord};

use crate::common::{pydate_to_rsdate, rsdate_to_pydate};

#[pyclass(name = "ConciseRecordSlice")]
pub struct PyConciseRecordSlice {
    pub inner: CoreConciseRecordSlice,
}

#[pymethods]
impl PyConciseRecordSlice {
    #[getter]
    fn date<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDate>> {
        let py = this.py();
        rsdate_to_pydate(py, this.borrow().inner.date())
    }

    #[getter]
    fn investment(&self) -> f64 {
        self.inner.investment()
    }

    #[getter]
    fn present_value(&self) -> f64 {
        self.inner.present_value()
    }

    #[getter]
    fn comment(&self) -> &str {
        self.inner.comment()
    }

    #[getter]
    fn total_investment(&self) -> f64 {
        self.inner.total_investment()
    }

    #[getter]
    fn profit(&self) -> f64 {
        self.inner.profit()
    }
}

#[pyclass(name = "DetailedRecordSlice")]
pub struct PyDetailedRecordSlice {
    pub inner: CoreDetailedRecordSlice,
}

#[pymethods]
impl PyDetailedRecordSlice {
    #[getter]
    fn date<'py>(this: Bound<'py, Self>) -> PyResult<Bound<'py, PyDate>> {
        let py = this.py();
        rsdate_to_pydate(py, this.borrow().inner.date())
    }

    #[getter]
    fn investment(&self) -> f64 {
        self.inner.investment()
    }

    #[getter]
    fn nav(&self) -> f64 {
        self.inner.nav()
    }

    #[getter]
    fn share(&self) -> f64 {
        self.inner.share()
    }

    #[getter]
    fn comment(&self) -> &str {
        self.inner.comment()
    }

    #[getter]
    fn fee(&self) -> f64 {
        self.inner.fee()
    }

    #[getter]
    fn total_investment(&self) -> f64 {
        self.inner.total_investment()
    }

    #[getter]
    fn total_share(&self) -> f64 {
        self.inner.total_share()
    }

    #[getter]
    fn present_value(&self) -> f64 {
        self.inner.present_value()
    }

    #[getter]
    fn profit(&self) -> f64 {
        self.inner.profit()
    }
}

#[pyclass(name = "ConciseRecord")]
pub struct PyConciseRecord {
    pub inner: CoreConciseRecord,
}

#[pymethods]
impl PyConciseRecord {
    #[new]
    #[pyo3(signature=(name, code, comment=None))]
    fn new(name: &str, code: &str, comment: Option<&str>) -> Self {
        if let Some(comment) = comment {
            PyConciseRecord {
                inner: CoreConciseRecord::new_comment(name, code, comment),
            }
        } else {
            PyConciseRecord {
                inner: CoreConciseRecord::new(name, code),
            }
        }
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn code(&self) -> &str {
        self.inner.code()
    }

    #[getter]
    fn comment(&self) -> &str {
        self.inner.comment()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __getitem__(&self, idx: isize) -> PyResult<PyConciseRecordSlice> {
        let len = self.inner.len() as isize;
        let actual_idx = if idx < 0 { len + idx } else { idx };
        if actual_idx < 0 || actual_idx >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err("index out of range"));
        }
        Ok(PyConciseRecordSlice {
            inner: self.inner[actual_idx as usize].clone(),
        })
    }

    #[classmethod]
    fn from_detailed(_cls: &Bound<'_, PyType>, detailed: &PyDetailedRecord) -> PyConciseRecord {
        PyConciseRecord {
            inner: CoreConciseRecord::from(&detailed.inner),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn irr_direct(
        &self,
        start_date: &Bound<'_, PyAny>,
        end_date: &Bound<'_, PyAny>,
        start_value: f64,
        end_value: f64,
        start_idx: usize,
        end_idx: usize,
        x0: f64,
    ) -> PyResult<f64> {
        Ok(self.inner.irr_direct(
            pydate_to_rsdate(start_date)?,
            pydate_to_rsdate(end_date)?,
            start_value,
            end_value,
            start_idx,
            end_idx,
            x0,
        ))
    }

    #[pyo3(signature = (start_date=None, end_date=None, start_value=None, end_value=None, start_index=None, end_index=None, x0=None))]
    fn irr(
        &self,
        start_date: Option<&Bound<'_, PyAny>>,
        end_date: Option<&Bound<'_, PyAny>>,
        start_value: Option<f64>,
        end_value: Option<f64>,
        start_index: Option<usize>,
        end_index: Option<usize>,
        x0: Option<f64>,
    ) -> PyResult<f64> {
        let start_date = start_date.map(pydate_to_rsdate).transpose()?;
        let end_date = end_date.map(pydate_to_rsdate).transpose()?;
        Ok(self.inner.irr(
            start_date,
            end_date,
            start_value,
            end_value,
            start_index,
            end_index,
            x0,
        ))
    }

    fn irr_naive(&self) -> f64 {
        self.inner.irr_naive()
    }

    fn append(
        &mut self,
        date: &Bound<'_, PyAny>,
        investment: f64,
        present_value: f64,
        comment: &str,
    ) -> PyResult<()> {
        let d = pydate_to_rsdate(date)?;
        self.inner.append(d, investment, present_value, comment);
        Ok(())
    }
}

#[pyclass(name = "DetailedRecord")]
pub struct PyDetailedRecord {
    pub inner: CoreDetailedRecord,
}

#[pymethods]
impl PyDetailedRecord {
    #[new]
    #[pyo3(signature=(name, code, comment=None))]
    fn new(name: &str, code: &str, comment: Option<&str>) -> Self {
        if let Some(comment) = comment {
            PyDetailedRecord {
                inner: CoreDetailedRecord::new_comment(name, code, comment),
            }
        } else {
            PyDetailedRecord {
                inner: CoreDetailedRecord::new(name, code),
            }
        }
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn code(&self) -> &str {
        self.inner.code()
    }

    #[getter]
    fn comment(&self) -> &str {
        self.inner.comment()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __getitem__(&self, idx: isize) -> PyResult<PyDetailedRecordSlice> {
        let len = self.inner.len() as isize;
        let actual_idx = if idx < 0 { len + idx } else { idx };
        if actual_idx < 0 || actual_idx >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err("index out of range"));
        }
        Ok(PyDetailedRecordSlice {
            inner: self.inner[actual_idx as usize].clone(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn irr_direct(
        &self,
        start_date: &Bound<'_, PyAny>,
        end_date: &Bound<'_, PyAny>,
        start_value: f64,
        end_value: f64,
        start_idx: usize,
        end_idx: usize,
        x0: f64,
    ) -> PyResult<f64> {
        Ok(self.inner.irr_direct(
            pydate_to_rsdate(start_date)?,
            pydate_to_rsdate(end_date)?,
            start_value,
            end_value,
            start_idx,
            end_idx,
            x0,
        ))
    }

    #[pyo3(signature = (start_date=None, end_date=None, start_value=None, end_value=None, start_index=None, end_index=None, x0=None))]
    fn irr(
        &self,
        start_date: Option<&Bound<'_, PyAny>>,
        end_date: Option<&Bound<'_, PyAny>>,
        start_value: Option<f64>,
        end_value: Option<f64>,
        start_index: Option<usize>,
        end_index: Option<usize>,
        x0: Option<f64>,
    ) -> PyResult<f64> {
        let start_date = start_date.map(pydate_to_rsdate).transpose()?;
        let end_date = end_date.map(pydate_to_rsdate).transpose()?;
        Ok(self.inner.irr(
            start_date,
            end_date,
            start_value,
            end_value,
            start_index,
            end_index,
            x0,
        ))
    }

    fn irr_naive(&self) -> f64 {
        self.inner.irr_naive()
    }

    fn append(
        &mut self,
        date: &Bound<'_, PyAny>,
        investment: f64,
        nav: f64,
        share: f64,
        comment: &str,
    ) -> PyResult<()> {
        let d = pydate_to_rsdate(date)?;
        self.inner.append(d, investment, nav, share, comment);
        Ok(())
    }
}

#[pyfunction]
#[pyo3(signature = (record, duration=None))]
pub fn get_irrs(record: &Bound<'_, PyAny>, duration: Option<f64>) -> PyResult<Vec<f64>> {
    if let Ok(c) = record.extract::<PyRef<PyConciseRecord>>() {
        Ok(eatmud::record::get_irrs(&c.inner, duration))
    } else if let Ok(d) = record.extract::<PyRef<PyDetailedRecord>>() {
        Ok(eatmud::record::get_irrs(&d.inner, duration))
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected ConciseRecord or DetailedRecord",
        ))
    }
}

#[pyfunction]
#[pyo3(signature = (*records))]
pub fn merge_records(records: &Bound<'_, PyTuple>) -> PyResult<PyConciseRecord> {
    if records.is_empty() {
        return Ok(PyConciseRecord {
            inner: CoreConciseRecord::new("", ""),
        });
    }

    let mut core_records: Vec<CoreConciseRecord> = Vec::new();
    for r in records.iter() {
        if let Ok(c) = r.extract::<PyRef<PyConciseRecord>>() {
            core_records.push(c.inner.clone());
        } else if let Ok(d) = r.extract::<PyRef<PyDetailedRecord>>() {
            core_records.push(CoreConciseRecord::from(&d.inner));
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Expected ConciseRecord or DetailedRecord",
            ));
        }
    }

    let mut merged = core_records.remove(0);
    for r in core_records {
        merged = eatmud::merge_records!(&merged, &r);
    }

    Ok(PyConciseRecord { inner: merged })
}

