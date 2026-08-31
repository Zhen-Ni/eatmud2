use std::ops::Index;

use chrono::NaiveDate;

use crate::{SIDE, utility::search_sorted};

pub trait DataSlice {
    fn date(&self) -> NaiveDate;
    fn value(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct FundSlice {
    date: NaiveDate,
    value: f64,
}

impl FundSlice {
    pub fn date(&self) -> NaiveDate {
        self.date
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl DataSlice for FundSlice {
    fn date(&self) -> NaiveDate {
        self.date
    }
    fn value(&self) -> f64 {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct StockSlice {
    date: NaiveDate,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl StockSlice {
    pub fn date(&self) -> NaiveDate {
        self.date
    }
    pub fn open(&self) -> f64 {
        self.open
    }
    pub fn high(&self) -> f64 {
        self.high
    }
    pub fn low(&self) -> f64 {
        self.low
    }
    pub fn close(&self) -> f64 {
        self.close
    }
    pub fn volume(&self) -> f64 {
        self.volume
    }
}

impl DataSlice for StockSlice {
    fn date(&self) -> NaiveDate {
        self.date
    }

    fn value(&self) -> f64 {
        self.close
    }
}

#[derive(Debug)]
pub struct Data<Ds: DataSlice> {
    name: String,
    code: String,
    data: Vec<Ds>,
}

pub type Fund = Data<FundSlice>;
pub type Stock = Data<StockSlice>;

impl<Ds: DataSlice> Data<Ds> {
    /// Constructs an empty Fund object.
    ///
    /// # Examples
    /// ```
    /// # use eatmud::Fund;
    /// let mut fund = Fund::new("hs300", "123456");
    /// assert!(fund.name() == "hs300");
    /// assert!(fund.code() == "123456");
    /// assert!(fund.data().is_empty());
    /// ```
    pub fn new(name: &str, code: &str) -> Self {
        Data {
            name: String::from(name),
            code: String::from(code),
            data: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn data(&self) -> &[Ds] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn truncate(&mut self, start_date: Option<NaiveDate>, end_date: Option<NaiveDate>) {
        let start_idx = if let Some(date) = start_date {
            search_sorted(self.data(), &date, |x| x.date(), Some(SIDE::Left))
        } else {
            0
        };
        let end_idx = if let Some(date) = end_date {
            search_sorted(self.data(), &date, |x| x.date(), Some(SIDE::Left))
        } else {
            self.len()
        };
        self.data.drain(end_idx..);
        self.data.drain(..start_idx);
    }
}

impl Data<FundSlice> {
    /// Appends fund records to the end of data storage.
    ///
    /// # Examples
    /// ```
    /// # use chrono::NaiveDate;
    /// # use eatmud::Fund;
    /// let mut fund = Fund::new("hs300", "123456");
    /// let date = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d")
    ///     .unwrap();
    /// fund.append(date, 1.0);
    /// assert!(fund[0].date() == date);
    /// assert!(fund[0].value() == 1.0);
    /// ```
    pub fn append(&mut self, date: NaiveDate, value: f64) {
        self.data.push(FundSlice { date, value });
    }
}

impl Data<StockSlice> {
    /// Appends stock records to the end of data storage.
    ///
    /// # Examples
    /// ```
    /// # use chrono::NaiveDate;
    /// # use eatmud::prelude::*;
    /// # use eatmud::Stock;
    /// let mut stock = Stock::new("hs300", "123456");
    /// let date = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d")
    ///     .unwrap();
    /// stock.append(date, 1.0, 2.0, 0.5, 0.8, 100f64);
    /// assert!(stock[0].date() == date);
    /// assert!(stock[0].value() == 0.8);
    /// ```
    pub fn append(
        &mut self,
        date: NaiveDate,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) {
        self.data.push(StockSlice {
            date,
            open,
            high,
            low,
            close,
            volume,
        });
    }
}

impl<Ds: DataSlice> Index<usize> for Data<Ds> {
    type Output = Ds;
    fn index(&self, index: usize) -> &Ds {
        &self.data[index]
    }
}

impl<Ds: DataSlice> From<&Data<Ds>> for Fund {
    fn from(data: &Data<Ds>) -> Fund {
        let mut fund = Fund::new(data.name(), data.code());
        fund.data = data
            .data()
            .iter()
            .map(|ss| FundSlice {
                date: DataSlice::date(ss),
                value: DataSlice::value(ss),
            })
            .collect();
        fund
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::read_tdx;
    use chrono::Days;

    #[test]
    fn test_fund() {
        let filename = "../tdx/test-hs300.txt";
        let stock = read_tdx(filename).expect("failed to read file");
        let mut fund = Fund::from(&stock);
        let n = fund.len();
        let date = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        fund.append(date, 2.0);
        assert_eq!(fund[n].date(), date);
        assert_eq!(fund[n].value(), 2.0);
    }

    #[test]
    fn test_stock() {
        let mut stock = Stock::new("ndsd", "SZ300750");
        let date = NaiveDate::parse_from_str("2024-01-11", "%Y-%m-%d").unwrap();
        stock.append(date, 150.66, 151.37, 148.51, 154.82, 10000.);
        stock.append(date + Days::new(1), 157.34, 159.87, 148.51, 153.45, 9754.);
        assert_eq!(stock.len(), 2);
        assert_eq!(stock[0].date(), date);
        assert_eq!(stock[0].value(), 154.82);
    }
}
