#!/usr/bin/env python3

import time
import datetime
import eatmud


def bench_kelly(trans: eatmud.Transaction, start_date: eatmud.Date
                ) -> list[list[float]]:
    ns = [1300, 1600]
    inflations = [0.015, 0.015]
    risk_bounds = [0.01, 0.01]
    results = []
    for save_log in (True, False,):
        for save_record in (True, False):
            name = f'save_log={save_log}, save_record={save_record}'
            now = time.time()
            res = []
            for weekday in range(0, 5):
                it = trans.iter(save_log, save_record)
                it.goto(start_date)
                it.inflow(1.)
                eatmud.strategy.kelly_weekly(it, eatmud.Weekday(
                    weekday), ns, inflations, risk_bounds)
                res.append(it.asset())
            total_time = time.time() - now
            print(f'running kelly({name}) took '
                  f'{total_time*1000} milli seconds')
            results.append(res)
    return results


def test_kelly():
    hs300 = eatmud.Fund.from_stock(eatmud.io.read_tdx("tdx/test-hs300.txt"))
    gz2000 = eatmud.Fund.from_stock(eatmud.io.read_tdx("tdx/test-gz2000.txt"))
    start_date = datetime.datetime.strptime('20170101', '%Y%m%d').date()
    end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()
    trans = eatmud.Transaction([hs300, gz2000], None, end_date)
    results = bench_kelly(trans, start_date)
    print(results)


if __name__ == '__main__':
    test_kelly()
