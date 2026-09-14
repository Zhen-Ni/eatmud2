use crate::TransactionIterator;

/// Do nothing and just step to the last day.
pub fn reference(it: &mut TransactionIterator) -> Result<(), Box<dyn std::error::Error>> {
    it.goto(it.end_date());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::read_tdx;
    use crate::*;

    #[test]
    fn test_reference_1() {
        let hs300 = Fund::from(&read_tdx("../tdx/test-hs300.txt").unwrap());
        let gz2000 = Fund::from(&read_tdx("../tdx/test-gz2000.txt").unwrap());
        let start_date = NaiveDate::parse_from_str("20170101", "%Y%m%d").unwrap();
        let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").unwrap();
        let trans = Transaction::new(&[&hs300, &gz2000], None, Some(end_date));

        let mut it = trans.iter(false, false);
        it.goto(start_date);
        it.inflow(1.).unwrap();
        reference(&mut it).unwrap();
        // 2024-01-01 is a holiday, so the last trading day should be earlier.
        assert_eq!(it.today(), *trans.dates().last().unwrap());
        // The reference strategy holds no position, so the asset stays unchanged.
        assert!((it.asset() - 1.).abs() < 1e-12);
    }
}
