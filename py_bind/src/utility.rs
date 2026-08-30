use pyo3::exceptions::PyValueError;
use pyo3::{PyResult, pyfunction};

/// Calaulate internal rate of return.
///
/// An gradient-based iteration method is used for solving internal
/// rate of return (IRR). The IRR is represented as:
/// ```ignore
///     end_value = sum(investment_i * (1 + x) ** t_i)
/// ```
/// where investment_i is the items in `investment` and `t_i` is the
/// duration of the date of `investment_i` to `end_value` in years,
/// and `x` is the IRR value to be solved. `x` is in the interval of
/// (-1, +∞), its initial value is given by `x0`.
#[pyfunction]
pub fn irr(
    days_array: Vec<f64>,
    investment_array: Vec<f64>,
    end_value: f64,
    x0: f64,
) -> PyResult<f64> {
    eatmud::irr(&days_array, &investment_array, end_value, x0).ok_or_else(|| {
        PyValueError::new_err("Failed to calculate IRR: calculation did not converge.")
    })
}

/// Calculate the maximum relative drawdown of a price sequence.
///
/// Maximum drawdown is defined as::
///
///     (peak_value - trough_value) / peak_value
///
/// The result lies in the range ``[0.0, 1.0]``. This function returns the
/// drawdown value together with the indices of its peak and trough points.
///
/// Parameters
/// ----------
/// prices : sequence of float
///     1‑dimensional price series. An empty input raises `ValueError`.
///
/// Returns
/// -------
/// max_drawdown : float
///     Maximum relative drawdown.
/// peak_index : int
///     Index of the peak preceding the maximum drawdown.
/// trough_index : int
///     Index of the trough of the maximum drawdown.
///
/// Raises
/// ------
/// ValueError
///     If the input ``prices`` is an empty sequence.
#[pyfunction]
pub fn max_drawdown(prices: Vec<f64>) -> PyResult<(f64, usize, usize)> {
    eatmud::max_drawdown(&prices).ok_or_else(|| {
        PyValueError::new_err("Failed to calculate max drawdown: got empty sequence.")
    })
}

/// Calculate the moving average (MA) of a data sequence.
///
/// Parameters
/// ----------
/// data : sequence of float
///     1‑dimensional input data sequence.
/// period : int
///     The window size (number of periods) for the moving average.
///
/// Returns
/// -------
/// sequence of float
///     The simple moving average sequence.
#[pyfunction]
pub fn moving_average(data: Vec<f64>, period: usize) -> Vec<f64> {
    eatmud::moving_average(&data, period)
}

/// Calculate the exponential moving average (EMA) of a data sequence.
///
/// Parameters
/// ----------
/// data : sequence of float
///     1‑dimensional input data sequence.
/// period : int
///     The number of periods for the exponential moving average.
///
/// Returns
/// -------
/// sequence of float
///     The exponential moving average sequence.
#[pyfunction]
pub fn exponential_moving_average(data: Vec<f64>, period: usize) -> Vec<f64> {
    eatmud::exponential_moving_average(&data, period)
}
