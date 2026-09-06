pub mod chrono;
mod common;
pub mod data;
pub mod io;
pub mod record;
pub mod strategy;
pub mod transaction;
pub mod utility;

use pyo3::prelude::*;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<data::PyFundSlice>()?;
    m.add_class::<data::PyStockSlice>()?;
    m.add_class::<data::PyFund>()?;
    m.add_class::<data::PyStock>()?;
    m.add_class::<data::PyMacdResult>()?;
    m.add_class::<data::PyBollResult>()?;
    m.add_class::<record::PyConciseRecord>()?;
    m.add_class::<record::PyDetailedRecord>()?;
    m.add_class::<record::PyConciseRecordSlice>()?;
    m.add_class::<record::PyDetailedRecordSlice>()?;
    m.add_class::<transaction::PyTransaction>()?;
    m.add_class::<transaction::PyTransactionIterator>()?;
    m.add_class::<transaction::PyHistoryView>()?;
    m.add_class::<chrono::PyWeekday>()?;

    m.add_function(pyo3::wrap_pyfunction!(record::get_irrs, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(record::merge_records, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(utility::irr, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(utility::max_drawdown, m)?)?;

    let io_module = PyModule::new(m.py(), "io")?;
    io_module.add_function(pyo3::wrap_pyfunction!(io::read_tdx, &io_module)?)?;
    m.add_submodule(&io_module)?;

    let strategy_module = PyModule::new(m.py(), "strategy")?;
    strategy_module.add_class::<strategy::PyKellyIndicator>()?;
    strategy_module.add_function(wrap_pyfunction!(strategy::aip_monthly, m)?)?;
    strategy_module.add_function(wrap_pyfunction!(strategy::kelly_weekly, m)?)?;
    strategy_module.add_function(wrap_pyfunction!(strategy::kelly_hint, m)?)?;
    m.add_submodule(&strategy_module)?;

    let io_module = PyModule::new(m.py(), "io")?;
    io_module.add_function(pyo3::wrap_pyfunction!(io::read_tdx, &io_module)?)?;
    m.add_submodule(&io_module)?;

    Ok(())
}
