use crate::data::{Data, DataSlice};

/// The result struct for MACD (Moving Average Convergence Divergence) calculations.
///
/// Contains the DIF (Difference), DEA (Signal Line), and HIST (Histogram) values.
#[derive(Debug, Clone)]
pub struct MacdResult {
    pub dif: Vec<f64>,
    pub dea: Vec<f64>,
    pub hist: Vec<f64>,
}

/// The result struct for Bollinger Bands calculations.
///
/// Contains the Middle Band (MID), Upper Band (UPPER), and Lower Band (LOWER) values.
#[derive(Debug, Clone)]
pub struct BollResult {
    pub mid: Vec<f64>,
    pub upper: Vec<f64>,
    pub lower: Vec<f64>,
}

pub trait Indicators {
    /// Calculate the Simple Moving Average (MA).
    ///
    /// The MA is calculated by averaging the values of the data slices over a specified period.
    /// Formula: MA = (Sum of values in period) / period
    ///
    /// # Arguments
    /// * `period` - The number of data slices to use for calculating the moving average.
    ///
    /// # Returns
    /// A `Vec<f64>` containing the MA values.
    fn ma(&self, period: usize) -> Vec<f64>;

    /// Calculate the Exponential Moving Average (EMA).
    ///
    /// EMA gives more weight to recent values. The initial EMA is a simple moving average.
    /// Formula: EMA_today = alpha * value_today + (1 - alpha) * EMA_yesterday
    /// Where alpha = 2 / (period + 1)
    ///
    /// # Arguments
    /// * `period` - The lookback period for EMA calculation.
    ///
    /// # Returns
    /// A `Vec<f64>` containing the EMA values.
    fn ema(&self, period: usize) -> Vec<f64>;

    /// Calculate the Moving Average Convergence Divergence (MACD).
    ///
    /// MACD is a trend-following momentum indicator that shows the relationship between two EMAs of prices.
    /// - DIF (Difference): EMA(short) - EMA(long)
    /// - DEA (Signal): EMA of DIF over `signal` period.
    /// - HIST (Histogram): 2 * (DIF - DEA)
    ///
    /// # Arguments
    /// * `short` - The period for the short-term EMA.
    /// * `long` - The period for the long-term EMA.
    /// * `signal` - The period for the signal line (DEA).
    ///
    /// # Returns
    /// A `MacdResult` struct containing `dif`, `dea`, and `hist` vectors.
    fn macd(&self, short: usize, long: usize, signal: usize) -> MacdResult;

    /// Calculate the Bollinger Bands (BOLL).
    ///
    /// Bollinger Bands consist of a middle band (SMA) and two outer bands (standard deviations away from the middle band).
    /// - MID: Simple Moving Average over `period`
    /// - UPPER: MID + multiplier * standard_deviation
    /// - LOWER: MID - multiplier * standard_deviation
    ///
    /// # Arguments
    /// * `period` - The period for calculating the middle band (SMA).
    /// * `multiplier` - The multiplier for the standard deviation.
    ///
    /// # Returns
    /// A `BollResult` struct containing `mid`, `upper`, and `lower` vectors.
    fn boll(&self, period: usize, multiplier: f64) -> BollResult;
}

impl<Ds: DataSlice> Indicators for Data<Ds> {
    fn ma(&self, period: usize) -> Vec<f64> {
        moving_average(self.data(), period, |s| s.value())
    }

    fn ema(&self, period: usize) -> Vec<f64> {
        exponential_moving_average(self.data(), period, |s| s.value())
    }

    fn macd(&self, short: usize, long: usize, signal: usize) -> MacdResult {
        let ema_short = exponential_moving_average(self.data(), short, |s| s.value());
        let ema_long = exponential_moving_average(self.data(), long, |s| s.value());
        let dif = ema_short
            .into_iter()
            .zip(ema_long)
            .map(|(s, l)| s - l)
            .collect::<Vec<_>>();
        let dea = exponential_moving_average(&dif, signal, |&x| x);
        let hist = dif.iter().zip(&dea).map(|(&x, &y)| 2. * (x - y)).collect();
        MacdResult { dif, dea, hist }
    }

    fn boll(&self, period: usize, multiplier: f64) -> BollResult {
        let data = self.data();
        let mut mid = Vec::with_capacity(data.len());
        let mut std = Vec::with_capacity(data.len());

        let mut mean = 0.0;
        let mut m2 = 0.0;

        for (i, slice) in data.iter().enumerate() {
            let x_new = slice.value();
            let window_len = if i < period { i + 1 } else { period };

            if i < period {
                // Standard Welford for expanding window
                let delta = x_new - mean;
                mean += delta / window_len as f64;
                let delta2 = x_new - mean;
                m2 += delta * delta2;
            } else {
                // Sliding window incremental update
                let x_old = data[i - period].value();
                let n = window_len as f64;

                let mean_old = mean;
                let mean_new = mean_old + (x_new - x_old) / n;

                // M2_new = M2_old + n * (mean_old - mean_new)^2 - (x_old - mean_new)^2 + (x_new - mean_new)^2
                m2 = m2 + n * (mean_old - mean_new).powi(2) - (x_old - mean_new).powi(2)
                    + (x_new - mean_new).powi(2);

                if m2 < 0.0 {
                    m2 = 0.0; // Guard against floating point precision errors
                }

                mean = mean_new;
            }

            let variance = m2 / window_len as f64;
            mid.push(mean);
            std.push(variance.sqrt());
        }

        let upper = mid
            .iter()
            .zip(&std)
            .map(|(&m, &s)| m + multiplier * s)
            .collect();
        let lower = mid
            .iter()
            .zip(&std)
            .map(|(&m, &s)| m - multiplier * s)
            .collect();

        BollResult { mid, upper, lower }
    }
}

/// Computes the moving average (MA) of a data sequence.
fn moving_average<T, F: Fn(&T) -> f64>(data: &[T], period: usize, get_val: F) -> Vec<f64> {
    let mut sum = 0.0;
    let mut result = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        sum += get_val(&data[i]);
        if i >= period {
            sum -= get_val(&data[i - period]);
        }
        result.push(sum / period.min(i + 1) as f64);
    }
    result
}

/// Computes the exponential moving average (EMA) of a data sequence.
fn exponential_moving_average<T, F: Fn(&T) -> f64>(
    data: &[T],
    period: usize,
    get_val: F,
) -> Vec<f64> {
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());
    let mut prev = get_val(&data[0]);
    for item in data.iter() {
        let val = get_val(item);
        let current = alpha * val + (1.0 - alpha) * prev;
        result.push(current);
        prev = current;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Duration, Fund, NaiveDate};

    fn create_test_fund() -> Fund {
        let mut fund = Fund::new("test", "000000");
        let base_date = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        for (i, &v) in values.iter().enumerate() {
            fund.append(base_date + Duration::days(i as i64), v);
        }
        fund
    }

    #[test]
    fn test_ma() {
        let fund = create_test_fund();
        let ma = fund.ma(3);
        assert_eq!(ma.len(), 10);
        assert!((ma[0] - 1.0).abs() < 1e-6); // [1.0]
        assert!((ma[1] - 1.5).abs() < 1e-6); // [1.0, 2.0]
        assert!((ma[2] - 2.0).abs() < 1e-6); // [1.0, 2.0, 3.0]
        assert!((ma[3] - 3.0).abs() < 1e-6); // [2.0, 3.0, 4.0]
        assert!((ma[9] - 9.0).abs() < 1e-6); // [8.0, 9.0, 10.0]
    }

    #[test]
    fn test_ema() {
        let fund = create_test_fund();
        let ema = fund.ema(3);
        assert_eq!(ema.len(), 10);
        assert!((ema[0] - 1.0).abs() < 1e-6);
        // alpha = 2 / (3 + 1) = 0.5
        let expected_1 = 0.5 * 2.0 + 0.5 * 1.0; // 1.5
        assert!((ema[1] - expected_1).abs() < 1e-6);
        let expected_2 = 0.5 * 3.0 + 0.5 * expected_1; // 2.25
        assert!((ema[2] - expected_2).abs() < 1e-6);
    }

    #[test]
    fn test_macd() {
        let fund = create_test_fund();
        let macd = fund.macd(3, 5, 2);
        assert_eq!(macd.dif.len(), 10);
        // The first values of dif and hist are 0, because ema_short[0] == ema_long[0] == data[0]
        assert!((macd.dif[0] - 0.0).abs() < 1e-6);
        assert!((macd.hist[0] - 0.0).abs() < 1e-6);
        // dif[1] = ema_short[1] - ema_long[1]
        // ema_short[1] = 1.5
        // ema_long[1] = 2/6 * 2.0 + 4/6 * 1.0 = 4/3
        // dif[1] = 1.5 - 4.0 / 3.0 = 1.0 / 6.0
        assert!((macd.dif[1] - 1.0 / 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_boll() {
        let fund = create_test_fund();
        let boll = fund.boll(3, 2.0);
        assert_eq!(boll.mid.len(), 10);

        // Data: [1.0]
        // mid=1.0, std=0.0 -> upper=1.0, lower=1.0
        assert!((boll.mid[0] - 1.0).abs() < 1e-6);
        assert!((boll.upper[0] - 1.0).abs() < 1e-6);
        assert!((boll.lower[0] - 1.0).abs() < 1e-6);

        // Data: [1.0, 2.0]
        // mid=1.5, std=0.5 -> upper=2.5, lower=0.5
        assert!((boll.mid[1] - 1.5).abs() < 1e-6);
        assert!((boll.upper[1] - 2.5).abs() < 1e-6);
        assert!((boll.lower[1] - 0.5).abs() < 1e-6);
    }
}
