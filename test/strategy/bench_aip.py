#!/usr/bin/env python3

import time
import datetime
import eatmud


def bench_aip(trans: eatmud.Transaction) -> list[list[float]]:
    results = []
    for save_log in (True, False,):
        for save_record in (True, False):
            name = f'save_log={save_log}, save_record={save_record}'
            now = time.time()
            res = []
            for day in range(1, 29):
                it = trans.iter(save_log, save_record)
                eatmud.strategy.aip_monthly(it, day, [1000.]*2, [0.]*2)
                res.append(it.asset())
            total_time = time.time() - now
            print(f'running aip({name}) took '
                  f'{total_time*1000} milli seconds')
            results.append(res)
    return results


def test_aip():
    hs300 = eatmud.Fund.from_stock(eatmud.io.read_tdx("tdx/test-hs300.txt"))
    gz2000 = eatmud.Fund.from_stock(eatmud.io.read_tdx("tdx/test-gz2000.txt"))
    start_date = datetime.datetime.strptime('20110101', '%Y%m%d').date()
    end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()
    trans = eatmud.Transaction([hs300, gz2000], start_date, end_date)
    results = bench_aip(trans)
    print(results)


if __name__ == '__main__':
    test_aip()
