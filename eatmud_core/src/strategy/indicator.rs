use crate::HistoryView;
use crate::TransactionIterator;
use crate::Weekday;

/// Buy if indicator is larger than 0.
pub fn indicator_weekly(
    it: &mut TransactionIterator,
    weekday: Weekday,
    indicators: &[&HistoryView<f64>],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut status = vec![false; it.nfunds()];
    let position = 1.0 / it.nfunds() as f64;
    while it.next_weekday(Some(weekday)).is_some() {
        let index = it.index();
        if index < 1 {
            continue;
        }
        for i in 0..it.nfunds() {
            let s = &mut status[i];
            let indicator = indicators[i].get(it).unwrap();
            let v = indicator[index - 1];
            if v > 0.0 && !*s {
                it.position(i, position, 0.0, false)?;
                *s = true;
            }
            if v < 0.0 && *s {
                it.position(i, 0.0, 0.0, false)?;
                *s = false;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::read_tdx;
    use crate::*;

    fn diff(v: Vec<f64>) -> Vec<f64> {
        let mut r = Vec::new();
        if v.len() == 0 {
            return r;
        }
        r.push(0.);
        for i in 1..v.len() {
            r.push(v[i] - v[i-1]);
        }
        r
    }
    
    #[test]
    fn test_extremum_ma() {
        let mut hs300 = Fund::from(&read_tdx("../tdx/test-hs300.txt").unwrap());
        let mut gz2000 = Fund::from(&read_tdx("../tdx/test-gz2000.txt").unwrap());
        let start_date = NaiveDate::parse_from_str("20110101", "%Y%m%d").ok();
        let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").ok();
        let trans = Transaction::new(&[&hs300, &gz2000], start_date, end_date);

        hs300.truncate(start_date, end_date);
        gz2000.truncate(start_date, end_date);

        let mut results = Vec::new();
        for ma_period in [5, 10, 20, 30, 60, 120, 200, 500] {
            let ind1 = HistoryView::from_vec(&trans, diff(hs300.ma(ma_period))).unwrap();
            let ind2 = HistoryView::from_vec(&trans, diff(gz2000.ma(ma_period))).unwrap();
            let inds = [&ind1, &ind2];
            let mut resi = Vec::new();
            for weekday in 0..5 {
                let mut it = trans.iter(false, true);
                it.inflow(1.0).unwrap();
                indicator_weekly(&mut it, Weekday::try_from(weekday).unwrap(), &inds).unwrap();
                resi.push(it.asset());
            }
            results.push(resi);
        }

        for i in results {
            println!("{:?}", i);
        }
    }
}
