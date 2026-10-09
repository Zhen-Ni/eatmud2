use crate::DAYS_PER_YEAR;
use crate::{TransactionIterator, Weekday};
use chrono::Datelike;
use ndarray::{Array, s};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct KellyError(&'static str);

impl std::fmt::Display for KellyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "KellyError: {}", self.0)
    }
}

/// Find maximal and minimal values in an iterable container simutanuously.
///
/// The original code may look like (ignore NAN):
/// ```ignore
/// let max = *iterable
///     .iter()
///     .max_by(|&x, &y| f64::partial_cmp(x, y).unwrap())
///     .unwrap();
/// let min = *iterable
///     .iter()
///     .min_by(|&x, &y| f64::partial_cmp(x, y).unwrap())
///     .unwrap();
/// ```
/// Now it can be replaced as:
/// ```ignore
/// let (max, min) = maxmin!(y);
/// ```
macro_rules! maxmin {
    ($iterable: ident) => {{
        let mut it = $iterable.iter();
        let mut max = *it
            .next()
            .expect("fail to find max and min as array is empty");
        let mut min = max;
        for &v in it {
            if max < v {
                max = v;
            }
            if v < min {
                min = v;
            }
        }
        (max, min)
    }};
}

impl std::error::Error for KellyError {}

pub struct KellyIndicator {
    pub position: f64,
    pub upper_bound: f64,
    pub lower_bound: f64,
    pub upper_risk_bound: f64,
    pub lower_risk_bound: f64,
}

/// Get the position, upper and lower bounds using Kelly strategy.
///
/// This function provides a detailed inspect into the Kelly strategy
/// during iteration. For a TransactionIterator object, giving the
/// index of the selected fund and other parameters, it returns the
/// current position, the limits where kelly strategy holds a position
/// within 0% to 100% and the risk bounds.
pub fn kelly_hint(
    it: &TransactionIterator,
    fund_index: usize,
    weekday: Weekday,
    n: usize,
    inflation: f64,
    risk_bound: f64,
) -> Result<KellyIndicator, Box<dyn std::error::Error>> {
    if it.navs().shape()[0] < n {
        return Err(Box::new(KellyError("n too large for kelly strategy")));
    }
    let inflation_array = Array::linspace((n - 1) as f64, 0., n);
    let inflation_array = inflation_array.mapv(|x| (1. + inflation).powf(x / DAYS_PER_YEAR));
    let navs = it.navs();
    // Net asset value of the last n days.
    let y0 = navs.slice(s![-(n as isize).., fund_index as isize]);
    let y = &y0 * &inflation_array;
    // Get winning rate.
    let mut y_weekly_iter = y
        .iter()
        .zip(&it.dates()[it.dates().len() - n..])
        .filter(|&(_yi, &di)| di.weekday() == weekday)
        .map(|(&yi, _di)| yi);
    let mut win_count = 0usize;
    let mut total_count = 0usize;
    let mut y_weekly_prev = y_weekly_iter
        .next()
        .ok_or(KellyError("cannot calculate winning rate"))?;
    for y_weekly_curr in y_weekly_iter {
        if y_weekly_curr > y_weekly_prev {
            win_count += 1;
        }
        total_count += 1;
        y_weekly_prev = y_weekly_curr;
    }
    if total_count == 0 {
        return Err(Box::new(KellyError("cannot calculate winning rate")));
    }
    let p = win_count as f64 / total_count as f64;
    let q = 1. - p;
    let (y_max, y_min) = maxmin!(y);
    let (y0_max, y0_min) = maxmin!(y0);

    // Kelly.
    let position = get_kelly_position(*y.last().unwrap(), y_max, y_min, p);
    // Risk control.
    let position = risk_control(position, *y0.last().unwrap(), y0_max, y0_min, risk_bound);

    let upper_bound = y_max * p + y_min * q;
    let lower_bound = y_max * y_min / (y_max * q + y_min * p);
    let bound_width = (y0_max - y0_min) * risk_bound;
    let upper_risk_bound = y0_max - bound_width;
    let lower_risk_bound = y0_min + bound_width;

    Ok(KellyIndicator {
        position,
        upper_bound,
        lower_bound,
        upper_risk_bound,
        lower_risk_bound,
    })
}

/// The Kelly strategy transacts weekly.
///
/// This implementation maintains the sliding-window extrema and the
/// winning rate incrementally, so each week costs only O(log n)
/// amortized instead of O(n) per fund.
pub fn kelly_weekly(
    it: &mut TransactionIterator,
    weekday: Weekday,
    ns: &[usize],
    inflations: &[f64],
    risk_bounds: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    if it.navs().shape()[0] < *ns.iter().max().unwrap() {
        return Err(Box::new(KellyError(
            "ns too large for transaction simulation",
        )));
    }
    let nfunds = it.nfunds();
    let mut states: Vec<_> = (0..nfunds)
        .map(|j| KellyFundState::new(ns[j], inflations[j], risk_bounds[j]))
        .collect();
    // Indices (into the transaction dates) of the iterated days
    // matching `weekday`.
    let mut samples: Vec<usize> = Vec::new();
    // Number of days that have been pushed into `states`.
    let mut pushed = 0usize;

    while it.next_weekday(Some(weekday)).is_some() {
        push_new_days(&*it, weekday, &mut states, &mut samples, &mut pushed);
        for j in 0..nfunds {
            let f = states[j].position(it, j, &samples)?;
            // Adjust position
            let position = f / nfunds as f64;
            let comment = &format!("position = {:.2}%", 100. * f);
            it.position_comment(j, position, 0.0, true, comment)?;
        }
    }
    Ok(())
}

/// Push newly iterated days into the per-fund states.
fn push_new_days(
    it: &TransactionIterator,
    weekday: Weekday,
    states: &mut [KellyFundState],
    samples: &mut Vec<usize>,
    pushed: &mut usize,
) {
    let navs = it.navs();
    let dates = it.dates();
    for i in *pushed..it.index() {
        let is_sample = dates[i].weekday() == weekday;
        if is_sample {
            samples.push(i);
        }
        for (j, st) in states.iter_mut().enumerate() {
            st.push_day(i, navs[[i, j]], is_sample);
        }
    }
    *pushed = it.index();
}

/// Incremental state of one fund for the Kelly strategy.
///
/// The state is updated once per day and queried once per week, so
/// the total cost is O(ndays) instead of O(ndays * ns).
struct KellyFundState {
    /// Window length.
    n: usize,
    inflation: f64,
    risk_bound: f64,
    /// Monotonic deques of `(day, key)` pairs maintaining the maximal
    /// and minimal values over the sliding window.
    ///
    /// The NAV considering inflation on day `i` is
    /// `nav[i] * (1 + inflation) ^ ((t - i) / DAYS_PER_YEAR)`, where
    /// `t` is the last day of the window. As the ratio between any
    /// two days is independent of `t`, comparisons within the window
    /// never change while the window slides. Therefore the deques are
    /// keyed by the time-invariant anchored value
    /// `nav[i] * (1 + inflation) ^ (-i / DAYS_PER_YEAR)`, and the real
    /// (inflated) value is reconstructed only for the extremum when
    /// querying.
    max_dq: VecDeque<(usize, f64)>,
    min_dq: VecDeque<(usize, f64)>,
    /// Monotonic deques for the extrema of the raw NAV (`y0`).
    nav_max_dq: VecDeque<(usize, f64)>,
    nav_min_dq: VecDeque<(usize, f64)>,
    /// Prefix sums of the winning flags, aligned with `samples`:
    /// `win_pref[s]` is the number of wins among the sample pairs
    /// `(samples[0], samples[1])`, ..., `(samples[s-1], samples[s])`.
    win_pref: Vec<usize>,
    /// Anchored key of the latest sample.
    last_sample_key: Option<f64>,
}

impl KellyFundState {
    fn new(n: usize, inflation: f64, risk_bound: f64) -> Self {
        KellyFundState {
            n,
            inflation,
            risk_bound,
            max_dq: VecDeque::new(),
            min_dq: VecDeque::new(),
            nav_max_dq: VecDeque::new(),
            nav_min_dq: VecDeque::new(),
            win_pref: Vec::new(),
            last_sample_key: None,
        }
    }

    /// Update the state with the NAV of a new day.
    ///
    /// `is_sample` tells whether the day matches the target weekday;
    /// the caller is responsible for pushing the day into `samples`.
    fn push_day(&mut self, idx: usize, nav: f64, is_sample: bool) {
        let key = nav * (1. + self.inflation).powf(-(idx as f64) / DAYS_PER_YEAR);
        // Strict comparisons keep the earliest day among equal
        // values, matching a linear scan over the window.
        while self.max_dq.back().is_some_and(|&(_, k)| k < key) {
            self.max_dq.pop_back();
        }
        self.max_dq.push_back((idx, key));
        while self.min_dq.back().is_some_and(|&(_, k)| k > key) {
            self.min_dq.pop_back();
        }
        self.min_dq.push_back((idx, key));
        while self.nav_max_dq.back().is_some_and(|&(_, v)| v < nav) {
            self.nav_max_dq.pop_back();
        }
        self.nav_max_dq.push_back((idx, nav));
        while self.nav_min_dq.back().is_some_and(|&(_, v)| v > nav) {
            self.nav_min_dq.pop_back();
        }
        self.nav_min_dq.push_back((idx, nav));
        if is_sample {
            let prev = self.win_pref.last().copied().unwrap_or(0);
            let flag = self.last_sample_key.map_or(0, |k| (key > k) as usize);
            self.win_pref.push(prev + flag);
            self.last_sample_key = Some(key);
        }
    }

    /// Compute the position of fund `fund` given by the Kelly
    /// equation and risk control, based on the sliding window ending
    /// at the current status of the iterator.
    fn position(
        &mut self,
        it: &TransactionIterator,
        fund: usize,
        samples: &[usize],
    ) -> Result<f64, KellyError> {
        let hi = it.index();
        let lo = hi - self.n;
        // Winning rate over the sample pairs within the window.
        let a = samples.partition_point(|&i| i < lo);
        let b = samples[a..].partition_point(|&i| i < hi) + a;
        if b - a < 2 {
            return Err(KellyError("cannot calculate winning rate"));
        }
        let total_count = b - a - 1;
        let win_count = self.win_pref[b - 1] - self.win_pref[a];
        let p = win_count as f64 / total_count as f64;
        // Drop the days that just slid out of the window.
        while self.max_dq.front().is_some_and(|&(i, _)| i < lo) {
            self.max_dq.pop_front();
        }
        while self.min_dq.front().is_some_and(|&(i, _)| i < lo) {
            self.min_dq.pop_front();
        }
        while self.nav_max_dq.front().is_some_and(|&(i, _)| i < lo) {
            self.nav_max_dq.pop_front();
        }
        while self.nav_min_dq.front().is_some_and(|&(i, _)| i < lo) {
            self.nav_min_dq.pop_front();
        }
        let t = hi - 1;
        let navs = it.navs();
        // Reconstruct the inflated values of the extrema. The
        // expression is identical to the elementwise product of the
        // NAV window with the inflation multipliers, so the result is
        // numerically the same as a full recomputation.
        let imax = self.max_dq.front().unwrap().0;
        let imin = self.min_dq.front().unwrap().0;
        let y_max =
            navs[[imax, fund]] * (1. + self.inflation).powf((t - imax) as f64 / DAYS_PER_YEAR);
        let y_min =
            navs[[imin, fund]] * (1. + self.inflation).powf((t - imin) as f64 / DAYS_PER_YEAR);
        let y0_max = self.nav_max_dq.front().unwrap().1;
        let y0_min = self.nav_min_dq.front().unwrap().1;
        // The multiplier of the last day is always 1, so both
        // `y.last()` and `y0.last()` equal the latest NAV.
        let y_last = navs[[t, fund]];
        // Kelly.
        let f = get_kelly_position(y_last, y_max, y_min, p);
        // Risk control.
        Ok(risk_control(f, y_last, y0_max, y0_min, self.risk_bound))
    }
}

/// Calculate the position given by kelly startegy.
///
/// The expected income when win is estimated by the current position
/// and the maximal net value in history. The expected loss is
/// estimated by the current position and the minimal net value in
/// history.
///
/// # Arguments
///
/// * `y` - The net assert values of the past.
/// * `p` - Estimated winning rate.
fn get_kelly_position(current_y: f64, max_y: f64, min_y: f64, p: f64) -> f64 {
    let b = max_y / current_y - 1.;
    let c = 1. - min_y / current_y;
    kelly_equation(p, b, c)
}

/// Adjust position to control risk.
///
/// A simple risk control strategy. If the current NAV is very low,
/// adjust position to zero. If the current NAV is very high, adjust
/// position to 1.0.
///
/// # Arguments
///
/// * `f` - The reference position.
/// * `y` - The net asset values of the past.
/// * `risk_bound` - The bound for controlling risk.
fn risk_control(f: f64, current_y: f64, max_y: f64, min_y: f64, risk_bound: f64) -> f64 {
    let bound_width = (max_y - min_y) * risk_bound;
    // let current_y = *y.last().unwrap();
    if current_y >= max_y - bound_width {
        1.
    } else if current_y <= min_y + bound_width {
        0.
    } else {
        f
    }
}

fn kelly_equation(p: f64, b: f64, c: f64) -> f64 {
    let q = 1. - p;
    let f = if b == 0. {
        f64::NEG_INFINITY
    } else if c == 0. {
        f64::INFINITY
    } else {
        (b * p - c * q) / (b * c)
    };
    f.clamp(0., 1.)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::read_tdx;
    use crate::*;

    #[test]
    fn test_kelly_1() {
        let hs300 = Fund::from(&read_tdx("../tdx/test-hs300.txt").unwrap());
        let gz2000 = Fund::from(&read_tdx("../tdx/test-gz2000.txt").unwrap());
        let start_date = NaiveDate::parse_from_str("20170101", "%Y%m%d").unwrap();
        let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").unwrap();
        let trans = Transaction::new(&[&hs300, &gz2000], None, Some(end_date));

        let ns = [1300, 1600];
        let inflations = [0.015, 0.015];
        let risk_bounds = [0.01, 0.01];
        let mut results = Vec::new();
        for save_log in [true, false] {
            for save_record in [true, false] {
                let mut res = Vec::new();
                for weekday in 0..5 {
                    let mut it = trans.iter(save_log, save_record);
                    it.goto(start_date);
                    it.inflow(1.).unwrap();
                    kelly_weekly(
                        &mut it,
                        Weekday::try_from(weekday).unwrap(),
                        &ns,
                        &inflations,
                        &risk_bounds,
                    )
                    .unwrap();
                    res.push(it.asset());
                }
                results.push(res);
            }
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[0], results[2]);
        assert_eq!(results[0], results[3]);
    }

    #[test]
    fn test_kelly_2() {
        let hs300 = Fund::from(&read_tdx("../tdx/test-hs300.txt").unwrap());
        let gz2000 = Fund::from(&read_tdx("../tdx/test-gz2000.txt").unwrap());
        let start_date = NaiveDate::parse_from_str("20170101", "%Y%m%d").unwrap();
        let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").unwrap();
        let trans = Transaction::new(&[&hs300, &gz2000], None, Some(end_date));

        let ns = [1300, 1600];
        let inflations = [0.015, 0.015];
        let risk_bounds = [0.01, 0.01];
        let mut result = Vec::new();
        for weekday in 0..5 {
            let mut it = trans.iter(false, false);
            it.goto(start_date);
            it.inflow(1.).unwrap();
            kelly_weekly(
                &mut it,
                Weekday::try_from(weekday).unwrap(),
                &ns,
                &inflations,
                &risk_bounds,
            )
            .unwrap();
            result.push(it.asset());
        }
        assert!((result[0] - 1.538617807495912).abs() < 1e-6);
        assert!((result[1] - 1.6655186489198273).abs() < 1e-6);
        assert!((result[2] - 1.5012221777553958).abs() < 1e-6);
        assert!((result[3] - 1.489728842992303).abs() < 1e-6);
        assert!((result[4] - 1.3982690518133718).abs() < 1e-6);
    }

    // Test whether kelly_weekly and kelly_hint provides the same
    // result.
    #[test]
    fn test_kelly_3() {
        let hs300 = Fund::from(&read_tdx("../tdx/test-hs300.txt").unwrap());
        let gz2000 = Fund::from(&read_tdx("../tdx/test-gz2000.txt").unwrap());
        let start_date = NaiveDate::parse_from_str("20170101", "%Y%m%d").unwrap();
        let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").unwrap();
        let trans = Transaction::new(&[&hs300, &gz2000], None, Some(end_date));

        let ns = [1300, 1600];
        let inflations = [0.015, 0.015];
        let risk_bounds = [0.01, 0.01];

        let weekday = Weekday::Tue;

        let mut it1 = trans.iter(true, true);
        it1.goto(start_date);
        it1.inflow(1.).unwrap();
        kelly_weekly(&mut it1, weekday, &ns, &inflations, &risk_bounds).unwrap();

        let mut it2 = trans.iter(true, true);
        it2.goto(start_date);
        it2.inflow(1.).unwrap();
        while it2.next_weekday(Some(weekday)).is_some() {
            for i in 0..trans.nfunds() {
                let indicator =
                    kelly_hint(&it2, i, weekday, ns[i], inflations[i], risk_bounds[i]).unwrap();
                // Adjust position the same way as kelly_weekly does.
                let total = it2.asset() / trans.nfunds() as f64 * indicator.position;
                let amount = total - it2.fund_asset(i);
                it2.buy_comment(
                    i,
                    amount,
                    0.0,
                    &format!("position = {:.2}%", 100. * indicator.position),
                )
                .unwrap();
            }
        }

        // Compare the comments of each fund's records.
        for i in 0..trans.nfunds() {
            let record1 = it1.fund_record(i).unwrap();
            let record2 = it2.fund_record(i).unwrap();
            assert_eq!(record1.len(), record2.len());
            for j in 0..record1.len() {
                assert_eq!(record1[j].comment(), record2[j].comment());
            }
        }

        // Compare the asset logs.
        let log1 = it1.asset_log().unwrap();
        let log2 = it2.asset_log().unwrap();
        assert!(
            log1.iter()
                .zip(log2.iter())
                .all(|(&a, &b)| (a - b).abs() < 1e-10)
        );
    }

    /// Compare the optimized `kelly_weekly` against the baseline
    /// implementation (full recomputation via `kelly_hint`) on all
    /// weekdays, including the trades (records) and final assets.
    #[test]
    fn test_kelly_baseline() {
        let hs300 = Fund::from(&read_tdx("../tdx/test-hs300.txt").unwrap());
        let gz2000 = Fund::from(&read_tdx("../tdx/test-gz2000.txt").unwrap());
        let start_date = NaiveDate::parse_from_str("20170101", "%Y%m%d").unwrap();
        let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").unwrap();
        let trans = Transaction::new(&[&hs300, &gz2000], None, Some(end_date));

        let ns = [1300, 1600];
        let inflations = [0.015, 0.015];
        let risk_bounds = [0.01, 0.01];

        for weekday in 0..5 {
            let weekday = Weekday::try_from(weekday).unwrap();

            let mut it1 = trans.iter(true, true);
            it1.goto(start_date);
            it1.inflow(1.).unwrap();
            kelly_weekly(&mut it1, weekday, &ns, &inflations, &risk_bounds).unwrap();

            // Baseline: replicate the original full-recompute
            // implementation via `kelly_hint`.
            let mut it2 = trans.iter(true, true);
            it2.goto(start_date);
            it2.inflow(1.).unwrap();
            while it2.next_weekday(Some(weekday)).is_some() {
                for i in 0..trans.nfunds() {
                    let indicator =
                        kelly_hint(&it2, i, weekday, ns[i], inflations[i], risk_bounds[i]).unwrap();
                    // Adjust position the same way as kelly_weekly
                    // does (`perfect_position = true` is equivalent
                    // to buying the exact target amount).
                    let total = it2.asset() / trans.nfunds() as f64 * indicator.position;
                    let amount = total - it2.fund_asset(i);
                    let comment = format!("position = {:.2}%", 100. * indicator.position);
                    it2.buy_comment(i, amount, 0.0, &comment).unwrap();
                }
            }

            // The two implementations must produce the same trades.
            assert!((it1.asset() - it2.asset()).abs() < 1e-10);
            for i in 0..trans.nfunds() {
                let record1 = it1.fund_record(i).unwrap();
                let record2 = it2.fund_record(i).unwrap();
                assert_eq!(record1.len(), record2.len());
                for j in 0..record1.len() {
                    assert_eq!(record1[j].comment(), record2[j].comment());
                    assert!(
                        (record1[j].present_value() - record2[j].present_value()).abs() < 1e-10
                    );
                }
            }
        }
    }
}
