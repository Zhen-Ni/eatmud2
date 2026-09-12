use std::{
    collections::{HashMap, hash_set},
    time::Instant,
};

use eatmud::{Fund, HistoryView, Indicators, NaiveDate, Transaction, Weekday, strategy};

fn bench_indicator(trans: &Transaction, indicators: &[&HistoryView<f64>], name: &str) {
    let now = Instant::now();
    let mut results = Vec::new();
    for weekday in 0..5 {
        let mut it = trans.iter(false, false);
        it.inflow(1.0).unwrap();
        strategy::indicator_weekly(&mut it, Weekday::try_from(weekday).unwrap(), indicators)
            .unwrap();
        results.push(it.asset());
    }
    let total_time = Instant::now() - now;
    println!(
        "{}, result = [{}], took {} ms",
        name,
        results
            .iter()
            .map(|x| format!("{:.3}", x))
            .collect::<Vec<String>>()
            .join(", "),
        total_time.as_micros() as f64 / 1000.
    );
}

fn diff(v: &[f64]) -> Vec<f64> {
    let mut r = Vec::new();
    if v.is_empty() {
        return r;
    }
    r.push(0.);
    for i in 1..v.len() {
        r.push(v[i] - v[i - 1]);
    }
    r
}

fn ma_extremum(trans: &Transaction, funds: &[&Fund]) {
    let start_date = trans.start_date();
    let end_date = trans.end_date();
    for ma_period in [5, 10, 20, 50, 100, 200, 500, 1000, 2000] {
        let views: Vec<_> = funds
            .iter()
            .map(|&f| diff(&f.ma(ma_period)[f.search_index(start_date)..f.search_index(end_date)]))
            .map(|ma| HistoryView::from_vec(trans, ma).unwrap())
            .collect();
        bench_indicator(
            trans,
            &views.iter().collect::<Vec<_>>(),
            &format!("MA{}", ma_period),
        );
    }
}

fn ma_cross(trans: &Transaction, funds: &[&Fund]) {
    let start_date = trans.start_date();
    let end_date = trans.end_date();
    let ma_short_periods: [usize; _] = [5, 10, 20, 50, 100];
    let ma_long_periods = [20, 50, 100, 200, 500];
    let mut keys = hash_set::HashSet::new();
    ma_short_periods.iter().for_each(|&x| {
        keys.insert(x);
    });
    ma_long_periods.iter().for_each(|&x| {
        keys.insert(x);
    });
    let ma_map: HashMap<_, _> = keys
        .iter()
        .map(|&period| {
            (
                period,
                funds
                    .iter()
                    .map(|f| {
                        f.ma(period)[f.search_index(start_date)..f.search_index(end_date)].to_vec()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    for short_period in ma_short_periods {
        for long_period in ma_long_periods {
            if !(short_period < long_period) {
                continue;
            }
            let ma_shorts = &ma_map[&short_period];
            let ma_longs = &ma_map[&long_period];
            let difs: Vec<_> = ma_shorts
                .iter()
                .zip(ma_longs)
                .map(|(s_vec, l_vec)| {
                    HistoryView::from_vec(
                        trans,
                        s_vec.iter().zip(l_vec).map(|(s, l)| s - l).collect(),
                    )
                    .unwrap()
                })
                .collect();
            bench_indicator(
                trans,
                &difs.iter().collect::<Vec<_>>(),
                &format!("MA({},{})", short_period, long_period),
            );
        }
    }
}

fn macd_cross(trans: &Transaction, funds: &[&Fund]) {
    let start_date = trans.start_date();
    let end_date = trans.end_date();
    for short in [5, 10, 12, 20, 50, 100] {
        for long in [20, 26, 50, 100, 200, 500] {
            for signal in [5, 9, 10, 20, 50] {
                if !(short < long) {
                    continue;
                }
                let macds: Vec<_> = funds
                    .iter()
                    .map(|f| {
                        HistoryView::from_vec(
                            trans,
                            f.macd(short, long, signal).hist
                                [f.search_index(start_date)..f.search_index(end_date)]
                                .to_vec(),
                        )
                        .unwrap()
                    })
                    .collect();
                bench_indicator(
                    trans,
                    &macds.iter().collect::<Vec<_>>(),
                    &format!("MACD({},{},{})", short, long, signal),
                );
            }
        }
    }
}

fn main() {
    let hs300 = Fund::from(&eatmud::io::read_tdx("tdx/test-hs300.txt").unwrap());
    let gz2000 = Fund::from(&eatmud::io::read_tdx("tdx/test-gz2000.txt").unwrap());
    let start_date = NaiveDate::parse_from_str("20170101", "%Y%m%d").unwrap();
    let end_date = NaiveDate::parse_from_str("20240101", "%Y%m%d").unwrap();
    let trans = Transaction::new(&[&hs300, &gz2000], Some(start_date), Some(end_date));
    let funds = [&hs300, &gz2000];
    ma_extremum(&trans, &funds);
    println!("--------");
    ma_cross(&trans, &funds);
    println!("--------");
    macd_cross(&trans, &funds);
}
