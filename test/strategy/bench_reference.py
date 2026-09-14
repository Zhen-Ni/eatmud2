#!/usr/bin/env python3

import time
import datetime
import eatmud


def bench_reference(trans: eatmud.Transaction, start_date: eatmud.Date
                    ) -> list[float]:
    results = []
    for save_log in (True, False,):
        for save_record in (True, False):
            name = f'save_log={save_log}, save_record={save_record}'
            now = time.time()
            it = trans.iter(save_log, save_record)
            it.goto(start_date)
            it.inflow(1.)
            nfunds = it.nfunds
            for idx in range(nfunds):
                it.buy(idx, 1. / nfunds, 0.)
            eatmud.strategy.reference(it)
            total_time = time.time() - now
            print(f'running reference({name}) took '
                  f'{total_time*1000} milli seconds')
            results.append(it.asset())
    return results


def test_reference():
    hs300 = eatmud.Fund.from_stock(eatmud.io.read_tdx("tdx/test-hs300.txt"))
    gz2000 = eatmud.Fund.from_stock(eatmud.io.read_tdx("tdx/test-gz2000.txt"))
    start_date = datetime.datetime.strptime('20170101', '%Y%m%d').date()
    end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()
    trans = eatmud.Transaction([hs300, gz2000], None, end_date)
    results = bench_reference(trans, start_date)
    print(results[0])


if __name__ == '__main__':
    test_reference()
