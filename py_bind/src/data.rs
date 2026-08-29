use eatmud::prelude::*;
use eatmud::{
    Fund as CoreFund, FundSlice as CoreFundSlice, Stock as CoreStock, StockSlice as CoreStockSlice,
};

use pyo3::prelude::*;
use pyo3::types::{PyDate, PyType};

use crate::common::{pydate_to_rsdate, rsdate_to_pydate};

#[pyclass(name = "FundSlice")]
pub struct PyFundSlice {
    pub inner: CoreFundSlice,
}

#[pymethods]
impl PyFundSlice {
    #[getter]
    fn date<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDate>> {
        let py = this.py();
        let date = this.borrow().inner.date();
        rsdate_to_pydate(py, date)
    }

    #[getter]
    fn value(&self) -> f64 {
        self.inner.value()
    }
}

#[pyclass(name = "StockSlice")]
pub struct PyStockSlice {
    pub inner: CoreStockSlice,
}

#[pymethods]
impl PyStockSlice {
    #[getter]
    fn date<'py>(this: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDate>> {
        let py = this.py();
        let date = this.borrow().inner.date();
        rsdate_to_pydate(py, date)
    }

    #[getter]
    fn value(&self) -> f64 {
        self.inner.value()
    }

    #[getter]
    fn open(&self) -> f64 {
        self.inner.open()
    }

    #[getter]
    fn high(&self) -> f64 {
        self.inner.high()
    }

    #[getter]
    fn low(&self) -> f64 {
        self.inner.low()
    }
    #[getter]
    fn close(&self) -> f64 {
        self.inner.close()
    }

    #[getter]
    fn volume(&self) -> f64 {
        self.inner.volume()
    }
}

#[pyclass(name = "Fund")]
pub struct PyFund {
    pub inner: CoreFund,
}

#[pymethods]
impl PyFund {
    #[new]
    fn new(name: &str, code: &str) -> Self {
        PyFund {
            inner: CoreFund::new(name, code),
        }
    }

    #[getter]
    fn get_name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn get_code(&self) -> &str {
        self.inner.code()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn append(&mut self, date: &Bound<'_, PyAny>, value: f64) -> PyResult<()> {
        let d = pydate_to_rsdate(date)?;
        self.inner.append(d, value);
        Ok(())
    }

    #[pyo3(signature = (start_date=None, end_date=None))]
    fn truncate(
        &mut self,
        start_date: Option<&Bound<'_, PyAny>>,
        end_date: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let start = start_date.map(pydate_to_rsdate).transpose()?;
        let end = end_date.map(pydate_to_rsdate).transpose()?;
        self.inner.truncate(start, end);
        Ok(())
    }

    fn __getitem__(&mut self, index: usize) -> PyResult<PyFundSlice> {
        Ok(PyFundSlice {
            inner: self.inner[index].clone(),
        })
    }

    #[classmethod]
    fn from_stock(_cls: Bound<'_, PyType>, stock: &PyStock) -> PyResult<PyFund> {
        let fund: CoreFund = CoreFund::from(&stock.inner);
        Ok(PyFund { inner: fund })
    }
}

#[pyclass(name = "Stock")]
pub struct PyStock {
    pub inner: CoreStock,
}

#[pymethods]
impl PyStock {
    #[new]
    fn new(name: &str, code: &str) -> Self {
        PyStock {
            inner: CoreStock::new(name, code),
        }
    }

    #[getter]
    fn get_name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn get_code(&self) -> &str {
        self.inner.code()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn append(
        &mut self,
        date: &Bound<'_, PyAny>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> PyResult<()> {
        let d = pydate_to_rsdate(date)?;
        self.inner.append(d, open, high, low, close, volume);
        Ok(())
    }

    #[pyo3(signature = (start_date=None, end_date=None))]
    fn truncate(
        &mut self,
        start_date: Option<&Bound<'_, PyAny>>,
        end_date: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let start = start_date.map(pydate_to_rsdate).transpose()?;
        let end = end_date.map(pydate_to_rsdate).transpose()?;
        self.inner.truncate(start, end);
        Ok(())
    }

    fn __getitem__(&mut self, index: usize) -> PyResult<PyStockSlice> {
        Ok(PyStockSlice {
            inner: self.inner[index].clone(),
        })
    }
}
