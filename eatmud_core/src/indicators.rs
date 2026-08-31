use crate::{
    data::{Data, DataSlice},
    exponential_moving_average, moving_average,
};

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
        let data = self.data().iter().map(|s| s.value()).collect::<Vec<_>>();
        moving_average(&data, period)
    }

    fn ema(&self, period: usize) -> Vec<f64> {
        let data = self.data().iter().map(|s| s.value()).collect::<Vec<_>>();
        exponential_moving_average(&data, period)
    }

    fn macd(&self, short: usize, long: usize, signal: usize) -> MacdResult {
        let data = self.data().iter().map(|s| s.value()).collect::<Vec<_>>();
        let ema_short = exponential_moving_average(&data, short);
        let ema_long = exponential_moving_average(&data, long);
        let dif = ema_short
            .into_iter()
            .zip(ema_long)
            .map(|(s, l)| s - l)
            .collect::<Vec<_>>();
        let dea = exponential_moving_average(&dif, signal);
        let hist = dif.iter().zip(&dea).map(|(&x, &y)| 2. * (x - y)).collect();
        MacdResult { dif, dea, hist }
    }

    fn boll(&self, period: usize, multiplier: f64) -> BollResult {
        let data = self.data().iter().map(|s| s.value()).collect::<Vec<_>>();
        let mid = moving_average(&data, period);
        let mut std = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            let window_len = usize::min(i + 1, period);
            let start_idx = i + 1 - window_len;
            let mean = mid[i];
            let sum_sq = data[start_idx..=i]
                .iter()
                .map(|&x| {
                    let diff = x - mean;
                    diff * diff
                })
                .sum::<f64>();
            std.push((sum_sq / window_len as f64).sqrt());
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
