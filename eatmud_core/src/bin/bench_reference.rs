use std::time::Instant;

use eatmud::{Fund, NaiveDate, Transaction, io::read_tdx};

fn bench_reference(trans: &Transaction, start_date: NaiveDate) -> Vec<f64> {
    let mut results = Vec::new();
    for save_log in [true, false] {
        for save_record in [true, false] {
            let name = format!("save_log={}, save_record={}", save_log, save_record);
            let now = Instant::now();
            let mut it = trans.iter(save_log, save_record);
            it.goto(start_date);
            it.inflow(1.).unwrap();
            (0..it.nfunds()).for_each(|idx| it.buy(idx, 1.0 / it.nfunds() as f64, 0.0).unwrap());
            eatmud::strategy::reference(&mut it).unwrap();

            let total_time = Instant::now() - now;
            println!(
                "running reference({}) took {} milli seconds",
                name,
                total_time.as_micros() as f64 / 1000.
            );
            results.push(it.asset());
        }
    }
    results
}

fn main() {
    let hs300 = Fund::from(&read_tdx("tdx/test-hs300.txt").unwrap());
    let gz2000 = Fund::from(&read_tdx("tdx/test-gz2000.txt").unwrap());
    let start_date = NaiveDate::parse_from_str("20170101", "%Y%m%d").unwrap();
    let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").unwrap();
    let trans = Transaction::new(&[&hs300, &gz2000], None, Some(end_date));
    let results = bench_reference(&trans, start_date);
    println!("{:?}", results[0]);
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0], results[2]);
    assert_eq!(results[0], results[3]);
}
