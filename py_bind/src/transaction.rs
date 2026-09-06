use crate::chrono::PyWeekday;
use crate::common::{map_err, pydate_to_rsdate, rsdate_to_pydate};
use crate::data::PyFund;
use crate::record::{ConciseRecordSource, DetailedRecordSource, PyConciseRecord, PyDetailedRecord};
use eatmud::Fund as CoreFund;
use eatmud::transaction::{
    Transaction as CoreTransaction, TransactionIterator as CoreTransactionIterator,
};
use numpy::{PyArray1, PyArray2, ToPyArray};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyAnyMethods, PyDate, PySlice, PyTuple};
use std::ptr::NonNull;
use std::sync::{Arc, OnceLock};

#[pyclass(name = "Transaction")]
pub struct PyTransaction {
    pub inner: Arc<CoreTransaction>,
    navs_cache: Arc<OnceLock<Py<PyArray2<f64>>>>,
}

#[pymethods]
impl PyTransaction {
    #[new]
    #[pyo3(signature = (funds, start_date=None, end_date=None))]
    fn new(
        funds: Vec<Bound<'_, PyFund>>,
        start_date: Option<&Bound<'_, PyAny>>,
        end_date: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let py_refs: Vec<PyRef<PyFund>> = funds.iter().map(|f| f.borrow()).collect();
        let funds_refs: Vec<&CoreFund> = py_refs.iter().map(|f| &f.inner).collect();
        let start = start_date.map(pydate_to_rsdate).transpose()?;
        let end = end_date.map(pydate_to_rsdate).transpose()?;
        let core_trans = CoreTransaction::new(&funds_refs, start, end);
        Ok(PyTransaction {
            inner: Arc::new(core_trans),
            navs_cache: Arc::new(OnceLock::new()),
        })
    }

    #[staticmethod]
    fn from_funds(funds: Vec<Bound<'_, PyFund>>) -> Self {
        let py_refs: Vec<PyRef<PyFund>> = funds.iter().map(|f| f.borrow()).collect();
        let funds_refs: Vec<&CoreFund> = py_refs.iter().map(|f| &f.inner).collect();
        let core_trans = CoreTransaction::from_funds(&funds_refs);
        PyTransaction {
            inner: Arc::new(core_trans),
            navs_cache: Arc::new(OnceLock::new()),
        }
    }

    #[getter]
    fn names(&self) -> Vec<String> {
        self.inner.names().to_vec()
    }

    #[getter]
    fn codes(&self) -> Vec<String> {
        self.inner.codes().to_vec()
    }

    #[getter]
    fn ndays(&self) -> usize {
        self.inner.ndays()
    }

    #[getter]
    fn nfunds(&self) -> usize {
        self.inner.nfunds()
    }

    #[getter]
    fn start_date<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDate>> {
        rsdate_to_pydate(this.py(), this.borrow().inner.start_date())
    }

    #[getter]
    fn end_date<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDate>> {
        rsdate_to_pydate(this.py(), this.borrow().inner.end_date())
    }

    fn date<'py>(this: &Bound<'py, Self>) -> PyResult<Vec<Bound<'py, PyDate>>> {
        this.borrow()
            .inner
            .date()
            .iter()
            .map(|d| rsdate_to_pydate(this.py(), *d))
            .collect()
    }

    fn navs<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let py = this.py();
        let this_ref = this.borrow();
        let arr = this_ref
            .navs_cache
            .get_or_init(|| this_ref.inner.navs().to_pyarray(py).unbind());
        Ok(arr.bind(py).clone())
    }

    #[pyo3(signature = (save_log=true, save_record=true))]
    fn iter(&self, save_log: bool, save_record: bool) -> PyResult<PyTransactionIterator> {
        // Lifetime Hack: Cast the reference to 'static, because Arc guarantees the data stays alive
        let trans_ref: &'static CoreTransaction =
            unsafe { &*(&*self.inner as *const CoreTransaction) };
        let core_iter = trans_ref.iter(save_log, save_record);
        Ok(PyTransactionIterator {
            trans: self.inner.clone(),
            inner: core_iter,
            navs_cache: self.navs_cache.clone(),
        })
    }
}

#[pyclass(name = "TransactionIterator")]
pub struct PyTransactionIterator {
    trans: Arc<CoreTransaction>,
    inner: CoreTransactionIterator<'static>,
    navs_cache: Arc<OnceLock<Py<PyArray2<f64>>>>,
}

impl PyTransactionIterator {
    pub fn inner(&self) -> &CoreTransactionIterator<'static> {
        &self.inner
    }
    pub fn inner_mut(&mut self) -> &mut CoreTransactionIterator<'static> {
        &mut self.inner
    }
}

#[pymethods]
impl PyTransactionIterator {
    #[getter]
    fn nfunds(&self) -> usize {
        self.inner().nfunds()
    }

    #[getter]
    fn ndays(&self) -> usize {
        self.inner().ndays()
    }

    fn today<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDate>> {
        rsdate_to_pydate(this.py(), this.borrow().inner().today())
    }

    fn cash(&self) -> f64 {
        self.inner().cash()
    }

    fn share(&self, idx: usize) -> f64 {
        self.inner().share(idx)
    }

    fn fund_asset(&self, idx: usize) -> f64 {
        self.inner().fund_asset(idx)
    }

    fn asset(&self) -> f64 {
        self.inner().asset()
    }

    fn dates<'py>(this: &Bound<'py, Self>) -> PyResult<Vec<Bound<'py, PyDate>>> {
        this.borrow()
            .inner()
            .dates()
            .iter()
            .map(|d| rsdate_to_pydate(this.py(), *d))
            .collect()
    }

    fn navs<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        let py = this.py();
        let this_ref = this.borrow();

        let arr_ptr = this_ref
            .navs_cache
            .get_or_init(|| this_ref.trans.navs().to_pyarray(py).unbind());
        let full_arr_bound = arr_ptr.bind(py).clone();

        let idx = this_ref.inner().dates().len() as isize;

        let py_slice = py.get_type::<PySlice>();
        let index = py_slice.call1((idx,))?;

        let key = PyTuple::new(py, [index])?;

        let view = full_arr_bound.into_any().get_item(key)?;
        Ok(view)
    }

    fn cash_log(&self) -> Option<Vec<f64>> {
        self.inner().cash_log().map(|v| v.to_vec())
    }

    fn share_log(&self, idx: usize) -> Option<Vec<f64>> {
        self.inner().share_log(idx).map(|v| v.to_vec())
    }

    fn fund_asset_log(&self, idx: usize) -> Option<Vec<f64>> {
        self.inner().fund_asset_log(idx).map(|v| v.to_vec())
    }

    fn asset_log(&self) -> Option<Vec<f64>> {
        self.inner().asset_log().map(|v| v.to_vec())
    }

    #[pyo3(signature = (amount, comment=None))]
    fn inflow(&mut self, amount: f64, comment: Option<&str>) -> PyResult<()> {
        match comment {
            Some(c) => self.inner_mut().inflow_comment(amount, c),
            None => self.inner_mut().inflow(amount),
        }
        .map(|_| ())
        .map_err(map_err)
    }

    #[pyo3(signature = (fundid, investment, fee=0.0, comment=None))]
    fn buy(
        &mut self,
        fundid: usize,
        investment: f64,
        fee: f64,
        comment: Option<&str>,
    ) -> PyResult<()> {
        match comment {
            Some(c) => self.inner_mut().buy_comment(fundid, investment, fee, c),
            None => self.inner_mut().buy(fundid, investment, fee),
        }
        .map(|_| ())
        .map_err(map_err)
    }

    #[pyo3(signature = (fundid, share, fee=0.0, comment=None))]
    fn sell(&mut self, fundid: usize, share: f64, fee: f64, comment: Option<&str>) -> PyResult<()> {
        match comment {
            Some(c) => self.inner_mut().sell_comment(fundid, share, fee, c),
            None => self.inner_mut().sell(fundid, share, fee),
        }
        .map(|_| ())
        .map_err(map_err)
    }

    #[pyo3(signature = (fundid, position, fee=0.0, comment=None, perfect_position=false))]
    fn position(
        &mut self,
        fundid: usize,
        position: f64,
        fee: f64,
        comment: Option<&str>,
        perfect_position: bool,
    ) -> PyResult<()> {
        match comment {
            Some(c) => {
                self.inner_mut()
                    .position_comment(fundid, position, fee, perfect_position, c)
            }
            None => self
                .inner_mut()
                .position(fundid, position, fee, perfect_position),
        }
        .map(|_| ())
        .map_err(map_err)
    }

    fn next_day(&mut self) -> bool {
        self.inner_mut().next_day().is_some()
    }

    fn next_weekday(&mut self, weekday: Option<PyWeekday>) -> bool {
        let w = weekday.map(|w| w.inner);
        self.inner_mut().next_weekday(w).is_some()
    }

    fn goto(&mut self, date: &Bound<'_, PyAny>) -> PyResult<bool> {
        let naive = pydate_to_rsdate(date)?;
        Ok(self.inner_mut().goto(naive).is_some())
    }

    fn next_month(&mut self, day: Option<u32>) -> bool {
        self.inner_mut().next_month(day).is_some()
    }

    fn cash_record(this: &Bound<'_, Self>) -> Option<PyConciseRecord> {
        this.borrow()
            .inner()
            .cash_record()
            .map(|r| PyConciseRecord {
                inner: ConciseRecordSource::Borrowed {
                    ptr: NonNull::from(r),
                    parent: this.clone().into_any().unbind(),
                },
            })
    }

    fn fund_record(this: Bound<'_, Self>, idx: usize) -> Option<PyDetailedRecord> {
        this.borrow()
            .inner()
            .fund_record(idx)
            .map(|r| PyDetailedRecord {
                inner: DetailedRecordSource::Borrowed {
                    ptr: NonNull::from(r),
                    parent: this.clone().into_any().unbind(),
                },
            })
    }

    fn record(this: &Bound<'_, Self>) -> Option<PyConciseRecord> {
        this.borrow().inner().record().map(|r| PyConciseRecord {
            inner: ConciseRecordSource::Owned(r),
        })
    }
}

#[pyclass(name = "HistoryView")]
pub struct PyHistoryView {
    trans: Arc<CoreTransaction>,
    ref_data: Py<PyArray1<f64>>,
}

#[pymethods]
impl PyHistoryView {
    #[staticmethod]
    fn from_vec<'py>(trans: &Bound<'py, PyTransaction>, ref_data: Vec<f64>) -> PyResult<Self> {
        let trans_ptr = trans.borrow().inner.clone();
        if trans_ptr.date().len() != ref_data.len() {
            return Err(PyValueError::new_err(
                "Size of ref_data must match the length of trans",
            ));
        }

        let ref_data = PyArray1::from_vec(trans.py(), ref_data).unbind();

        Ok(PyHistoryView {
            trans: trans_ptr,
            ref_data: ref_data,
        })
    }

    #[staticmethod]
    fn from_arr<'py>(
        trans: &Bound<'py, PyTransaction>,
        ref_data: &Bound<'py, PyArray1<f64>>,
    ) -> PyResult<Self> {
        let trans_ptr = trans.borrow().inner.clone();
        if trans_ptr.date().len() != ref_data.len()? {
            return Err(PyValueError::new_err(
                "Size of ref_data must match the length of trans",
            ));
        }

        Ok(PyHistoryView {
            trans: trans_ptr,
            ref_data: ref_data.clone().unbind(),
        })
    }

    pub fn get<'py>(
        this: &Bound<'py, Self>,
        it: &PyTransactionIterator,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        if Arc::ptr_eq(&this.borrow().trans, &it.trans) {
            let py = this.py();
            let this_ref = this.borrow();
            let idx = it.inner().index();
            let full_data = this_ref.ref_data.bind(py);
            let slice = PySlice::new(py, 0, idx as isize, 1);
            let sliced = full_data.get_item(slice)?;
            let sliced_arr = sliced.cast::<PyArray1<f64>>()?;
            Ok(sliced_arr.to_owned())
        } else {
            Err(PyRuntimeError::new_err(
                "Given iterator is not from the same transaction instance it created from",
            ))
        }
    }
}
