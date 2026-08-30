use std::cmp::Ordering;
pub const DAYS_PER_YEAR: f64 = 360.;

pub enum SIDE {
    Left,
    Right,
}

/// Find indices of element in the sorted array by provided key function.
///
/// Assuming that `a` is sorted by `key`:
///
///  ------ | ----------------------------
///  `side` | returned index `i` satisfies
///  ------ | ----------------------------
///  left   | ``key(a[i-1]) < v <= key(a[i])``
///  right  | ``key(a[i-1]) <= v < key(a[i])``
///  ------ | ----------------------------
///
/// Note that if `a` is unsorted, the result may be ambigious.
pub(crate) fn search_sorted<T, U: Ord>(
    a: &[T],
    v: &U,
    key: impl Fn(&T) -> U,
    side: Option<SIDE>,
) -> usize {
    if a.is_empty() {
        return 0;
    }
    let side = side.unwrap_or(SIDE::Left);
    let (mut lo, mut hi) = (0, a.len() - 1);
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        match key(&a[mid]).cmp(v) {
            Ordering::Less => lo = mid,
            Ordering::Greater => hi = mid,
            Ordering::Equal => match side {
                SIDE::Left => hi = mid,
                SIDE::Right => lo = mid,
            },
        }
    }
    let lv = key(&a[lo]);
    let rv = key(&a[hi]);
    // if v in a
    if *v == lv || *v == rv {
        match side {
            SIDE::Left => {
                if *v == lv {
                    lo
                } else {
                    hi
                }
            }

            SIDE::Right => {
                if *v == rv {
                    lo
                } else {
                    hi
                }
            }
        }
    }
    // if v not in a
    else if *v < lv {
        lo
    } else if *v > rv {
        hi + 1
    } else {
        hi
    }
}

/// Calculate internal rate of return.
///
/// An gradient-based iteration method is used for solving internal
/// rate of return (IRR). The IRR is represented as:
/// ```ignore
///     end_value = sum(investment_i * (1 + x) ** t_i)
/// ```
/// where investment_i is the items in `investment` and `t_i` is the
/// duration of the date of `investment_i` to `end_value` in years,
/// and `x` is the IRR value to be solved. `x` is in the interval of
/// (-1, +∞), its initial value is given by `x0`. To make the solution
/// always in this interval, let `x = exp(p) - 1`, thus p ∈ (-∞, ∞):
/// ```ignore
///     end_value = sum(investment_i * exp(p * t_i))
/// ```
/// and its derivative is:
/// ```ignore
///     d(end_value) / d(p) = sum(investment_i * t_i * exp(p * t_i))
/// ```
pub fn irr(days_array: &[f64], investment_array: &[f64], end_value: f64, x0: f64) -> Option<f64> {
    // Avoid sigularity.
    let mut all_zeros = true;
    for i in investment_array {
        if *i != 0. {
            all_zeros = false;
            break;
        }
    }
    if all_zeros {
        return None;
    }

    let t_list = days_array
        .iter()
        .map(|x| x / DAYS_PER_YEAR)
        .collect::<Vec<_>>();

    let f = |p: f64| -> f64 {
        end_value
            - t_list
                .iter()
                .zip(investment_array)
                .map(|(&t, &x)| x * f64::exp(p * t))
                .sum::<f64>()
    };
    let g = |p: f64| {
        t_list
            .iter()
            .zip(investment_array)
            .map(|(&t, &x)| -x * t * f64::exp(p * t))
            .sum()
    };
    let p0 = f64::ln(x0 + 1.);
    let p = newton1d(f, g, p0, 1e-6, 1000)?;
    Some(f64::exp(p) - 1.)
}

/// Find root of function using Newton's method.
///
/// The Newton's method uses the target funciton and its derivation to
/// find the root of the function.
///
/// # Arguments
///
/// * `f` - The target function which takes exactly one argument.
/// * `d` - The derivation of `f`.
/// * `x0` - The initial guess of the root.
/// * `tol` - The absolute tolerance for root finding.
/// * `maxiter` - The maximum number of iterations to find the root.
pub(crate) fn newton1d(
    f: impl Fn(f64) -> f64,
    d: impl Fn(f64) -> f64,
    x0: f64,
    tol: f64,
    maxiter: usize,
) -> Option<f64> {
    let mut x = x0;
    for _ in 0..maxiter {
        let new_x = x - f(x) / d(x);
        if (new_x - x).abs() < tol {
            return Some(new_x);
        }
        x = new_x;
    }
    None
}

/// Calculate the max drawdown of the given price sequence.
///
/// Returns the maximum relative drawdown, together with its peak and trough indexes.
/// Drawdown is computed as `(peak_value - trough_value) / peak_value`,
/// with result range within `[0.0, 1.0]`.
///
/// # Arguments
///
/// * `prices` - 1‑dimensional slice of price values. Empty input returns [`None`].
///
/// # Returns
///
/// * [`Some((max_drawdown, peak_index, trough_index))`] on non‑empty input:
///     - `max_drawdown`: maximum relative drawdown.
///     - `peak_index`: index of the peak before drawdown.
///     - `trough_index`: index of the trough for maximum drawdown.
/// * [`None`] when input slice is empty.
///
/// # Examples
///
/// ```
/// use eatmud::max_drawdown;
/// let prices = &[100.0, 120.0, 80.0];
/// let (dd, peak_idx, trough_idx) = max_drawdown(prices).unwrap();
/// assert!((dd - 0.3333333333333333).abs() < 1e-12);
/// assert_eq!(peak_idx, 1);
/// assert_eq!(trough_idx, 2);
/// ```
pub fn max_drawdown(prices: &[f64]) -> Option<(f64, usize, usize)> {
    if prices.is_empty() {
        return None;
    }
    let mut max_drawdown = 0.0;
    let mut peak_idx = 0;
    let mut running_peak_index = 0;
    let mut trough_idx = 0;
    let mut running_peak_value = prices[0];
    for (i, &p) in prices.iter().enumerate() {
        if p > running_peak_value {
            running_peak_index = i;
            running_peak_value = p;
        }
        let drawdown = (running_peak_value - p) / running_peak_value;
        if drawdown > max_drawdown {
            peak_idx = running_peak_index;
            max_drawdown = drawdown;
            trough_idx = i;
        }
    }
    Some((max_drawdown, peak_idx, trough_idx))
}

/// Computes the moving average (MA) of a data sequence.
///
/// For the first few points where the available samples are fewer than `period`,
/// partial-window averages are returned. Full-window SMA values start when
/// enough samples are accumulated.
///
/// # Arguments
///
/// * `data` - Input data sequence.
/// * `period` - Window size for moving average.
///
/// # Returns
///
/// A `Vec<f64>` with the same length as input `data`, containing moving‑average values.
///
/// # Notes
///
/// If `period` is 0, the output elements become NaN values.
///
/// # Examples
///
/// ```
/// use eatmud::moving_average;
/// let data = &[60.0, 100.0, 80.0, 120.0];
/// let result = moving_average(data, 3);
/// assert_eq!(result, vec![60.0, 80.0, 80.0, 100.0]);
/// ```
pub fn moving_average(data: &[f64], period: usize) -> Vec<f64> {
    let mut sum = 0.0;
    let mut result = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        sum += data[i];
        if i >= period {
            sum -= data[i - period];
        }
        let window_len = usize::min(i + 1, period);
        result.push(sum / window_len as f64);
    }
    result
}

/// Computes the exponential moving average (EMA) of a data sequence.
///
/// EMA applies exponential weighting to recent observations.
/// The smoothing factor is defined as ``alpha = 2 / (period + 1)``.
/// Formula:
/// ```ignore
/// ema[i] = alpha * data[i] + (1 - alpha) * ema[i-1]
/// ```
/// The first output value is initialized with ``data[0]``.
///
/// # Arguments
///
/// * `data` - Input data sequence.
/// * `period` - Window size used to compute the smoothing factor.
///
/// # Returns
///
/// A `Vec<f64>` with the same length as input `data`, containing EMA values.
///
/// # Examples
///
/// ```
/// use eatmud::exponential_moving_average;
/// let data = &[60.0, 100.0, 80.0, 120.0];
/// let result = exponential_moving_average(data, 3);
/// assert_eq!(result.len(), 4)
/// ```
pub fn exponential_moving_average(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());
    let mut prev = data[0];
    for &val in data.iter() {
        let current = alpha * val + (1.0 - alpha) * prev;
        result.push(current);
        prev = current;
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search_sorted() {
        let a = vec![2, 4, 6, 8, 10, 12, 14, 16];
        let idx1 = search_sorted(&a, &9, |&x| x, None);
        let idx2 = search_sorted(&a, &9, |&x| x, Some(SIDE::Right));
        let idx3 = search_sorted(&a, &8, |&x| x, None);
        let idx4 = search_sorted(&a, &8, |&x| x, Some(SIDE::Right));
        let idx5 = search_sorted(&a, &0, |&x| x, None);
        let idx6 = search_sorted(&a, &0, |&x| x, Some(SIDE::Right));
        let idx7 = search_sorted(&a, &20, |&x| x, None);
        let idx8 = search_sorted(&a, &20, |&x| x, Some(SIDE::Right));
        assert!(idx1 == 4);
        assert!(idx2 == 4);
        assert!(idx3 == 3);
        assert!(idx4 == 4);
        assert!(idx5 == 0);
        assert!(idx6 == 0);
        assert!(idx7 == 8);
        assert!(idx8 == 8);
    }

    #[test]
    fn test_newton1d() {
        fn f(x: f64) -> f64 {
            x * x + 2. * x + 1.
        }
        fn g(x: f64) -> f64 {
            2. * x + 2.
        }
        let x0 = newton1d(f, g, 0.0, 1e-6, 100).unwrap();
        assert!((x0 + 1.).abs() < 1e-3);
    }

    #[test]
    fn test_irr() {
        let days_array = [720., 360., 0.];
        let investment_array = [1., 2., 0.];
        let end_value = 8.;
        let x0 = 0.0;
        let res = irr(&days_array, &investment_array, end_value, x0).unwrap();
        assert!((res - 1.).abs() < 1e-2);
    }

    #[test]
    fn test_normal_drawdown() {
        let res = max_drawdown(&[100.0, 120.0, 80.0]);
        let (dd, peak, trough) = res.unwrap();
        assert!((dd - 40.0 / 120.0).abs() < 1e-12);
        assert_eq!(peak, 1);
        assert_eq!(trough, 2);
    }

    #[test]
    fn test_no_drawdown() {
        let res = max_drawdown(&[10.0, 20.0, 30.0]);
        let (dd, _, _) = res.unwrap();
        assert_eq!(dd, 0.0);
    }

    #[test]
    fn test_empty_drawdown() {
        let res = max_drawdown(&[]);
        assert!(res.is_none());
    }

    #[test]
    fn test_moving_average() {
        let data = &[60.0, 100.0, 80.0, 120.0];
        let result = moving_average(data, 3);
        assert_eq!(result, vec![60.0, 80.0, 80.0, 100.0]);
    }

    #[test]
    fn test_moving_average_period_zero() {
        let data = &[10.0, 20.0];
        let res = moving_average(data, 0);
        assert!(res[0].is_nan());
        assert!(res[1].is_nan());
    }

    #[test]
    fn test_exponential_moving_average() {
        let data = &[60.0, 100.0, 80.0, 120.0];
        let res = exponential_moving_average(data, 3);
        assert_eq!(res.len(), 4);
        assert!((res[0] - 60.0).abs() < 1e-12);
    }

    #[test]
    fn test_exponential_moving_average_empty() {
        let res = exponential_moving_average(&[], 5);
        assert!(res.is_empty());
    }
}
