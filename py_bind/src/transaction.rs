use crate::chrono::PyWeekday;
use crate::common::{map_err, pydate_to_rsdate, rsdate_to_pydate, rsdates_to_pyarr};
use crate::data::PyFund;
use crate::record::{ConciseRecordSource, DetailedRecordSource, PyConciseRecord, PyDetailedRecord};
use eatmud::transaction::{
    Transaction as CoreTransaction, TransactionIterator as CoreTransactionIterator,
};
use eatmud::{Fund as CoreFund, HistoryView as CoreHistoryView};
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDate};
use std::ptr::NonNull;
use std::sync::Arc;

#[pyclass(name = "Transaction")]
pub struct PyTransaction {
    pub inner: Arc<CoreTransaction>,
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
        })
    }

    #[staticmethod]
    fn from_funds(funds: Vec<Bound<'_, PyFund>>) -> Self {
        let py_refs: Vec<PyRef<PyFund>> = funds.iter().map(|f| f.borrow()).collect();
        let funds_refs: Vec<&CoreFund> = py_refs.iter().map(|f| &f.inner).collect();
        let core_trans = CoreTransaction::from_funds(&funds_refs);
        PyTransaction {
            inner: Arc::new(core_trans),
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

    fn date<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        let py = this.py();
        let binding = this.borrow();
        let dates = binding.inner.date();
        rsdates_to_pyarr(py, dates)
    }

    fn navs<'py>(this: &Bound<'py, Self>) -> Bound<'py, PyArray2<f64>> {
        let this_ref = &*this.borrow();
        let core_array = this_ref.inner.navs();
        let container = this.clone().into_any();
        let pyarr = unsafe { PyArray2::borrow_from_array(core_array, container) };
        let _ro = pyarr.readwrite().make_nonwriteable();
        pyarr
    }

    #[pyo3(signature = (save_log=true, save_record=true))]
    fn iter(&self, save_log: bool, save_record: bool) -> PyTransactionIterator {
        // Lifetime Hack: Cast the reference to 'static, because Arc guarantees the data stays alive
        let trans_ref: &'static CoreTransaction =
            unsafe { &*(&*self.inner as *const CoreTransaction) };
        let core_iter = trans_ref.iter(save_log, save_record);
        PyTransactionIterator {
            _trans: self.inner.clone(),
            inner: core_iter,
        }
    }
}

#[pyclass(name = "TransactionIterator")]
pub struct PyTransactionIterator {
    // Must reserve _trans here due to the lifetime hack.
    _trans: Arc<CoreTransaction>,
    inner: CoreTransactionIterator<'static>,
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

    fn dates<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        let py = this.py();
        let binding = this.borrow();
        let dates = binding.inner().dates();
        rsdates_to_pyarr(py, dates)
    }

    fn navs<'py>(this: &Bound<'py, Self>) -> Bound<'py, PyArray2<f64>> {
        let this_ref = &*this.borrow();
        let core_array = this_ref.inner.navs();
        let container = this.clone().into_any();
        let pyarr = unsafe { PyArray2::borrow_from_array(&core_array, container) };
        let _ro = pyarr.readwrite().make_nonwriteable();
        pyarr
    }

    fn cash_log<'py>(this: &Bound<'py, Self>) -> Option<Bound<'py, PyArray1<f64>>> {
        let binding = this.borrow();
        let rsarr = binding.inner().cash_log()?;
        let container = this.clone().into_any();
        let pyarr = unsafe { PyArray1::borrow_from_array(&rsarr, container) };
        let _ro = pyarr.readwrite().make_nonwriteable();
        Some(pyarr)
    }

    fn share_log<'py>(this: &Bound<'py, Self>, idx: usize) -> Option<Bound<'py, PyArray1<f64>>> {
        let binding = this.borrow();
        let rsarr = binding.inner().share_log(idx)?;
        let container = this.clone().into_any();
        let pyarr = unsafe { PyArray1::borrow_from_array(&rsarr, container) };
        let _ro = pyarr.readwrite().make_nonwriteable();
        Some(pyarr)
    }

    fn fund_asset_log<'py>(
        this: &Bound<'py, Self>,
        idx: usize,
    ) -> Option<Bound<'py, PyArray1<f64>>> {
        let binding = this.borrow();
        let rsarr = binding.inner().fund_asset_log(idx)?;
        let pyarr = PyArray1::from_owned_array(this.py(), rsarr);
        let _ro = pyarr.readwrite().make_nonwriteable();
        Some(pyarr)
    }

    fn asset_log<'py>(this: &Bound<'py, Self>) -> Option<Bound<'py, PyArray1<f64>>> {
        let binding = this.borrow();
        let rsarr = binding.inner().asset_log()?;
        let pyarr = PyArray1::from_owned_array(this.py(), rsarr);
        let _ro = pyarr.readwrite().make_nonwriteable();
        Some(pyarr)
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
    _trans: Arc<CoreTransaction>,
    inner: Arc<CoreHistoryView<'static, f64>>,
}

#[pymethods]
impl PyHistoryView {
    #[staticmethod]
    fn from_vec<'py>(trans: &Bound<'py, PyTransaction>, ref_data: Vec<f64>) -> PyResult<Self> {
        let core_trans: &'static CoreTransaction =
            unsafe { &*(&*trans.borrow().inner as *const CoreTransaction) };
        let inner = CoreHistoryView::from_vec(core_trans, ref_data).map_err(map_err)?;

        Ok(PyHistoryView {
            _trans: trans.borrow().inner.clone(),
            inner: Arc::new(inner),
        })
    }

    #[staticmethod]
    fn from_arr<'py>(
        trans: &Bound<'py, PyTransaction>,
        ref_data: &Bound<'py, PyArray1<f64>>,
    ) -> PyResult<Self> {
        let core_trans: &'static CoreTransaction =
            unsafe { &*(&*trans.borrow().inner as *const CoreTransaction) };
        let inner =
            CoreHistoryView::from_arr(core_trans, ref_data.to_owned_array()).map_err(map_err)?;

        Ok(PyHistoryView {
            _trans: trans.borrow().inner.clone(),
            inner: Arc::new(inner),
        })
    }

    pub fn get<'py>(
        this: &Bound<'py, Self>,
        it: &PyTransactionIterator,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let core_view = &*this.borrow().inner;
        let core_it = &it.inner;
        let core_arr = core_view.get(core_it).map_err(map_err)?;
        let container = this.clone().into_any();
        let pyarr = unsafe { PyArray1::borrow_from_array(&core_arr, container) };
        let _ro = pyarr.readwrite().make_nonwriteable();
        Ok(pyarr)
    }
}
