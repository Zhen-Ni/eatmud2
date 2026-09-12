#!/usr/bin/env python3

import unittest
import datetime
import eatmud


def diff(v: list[float]) -> list[float]:
    if not v:
        return []
    r = [0.0]
    for i in range(1, len(v)):
        r.append(v[i] - v[i - 1])
    return r


class TestIndicator(unittest.TestCase):

    def test_extremum_ma_weekly(self):
        hs300 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-hs300.txt"))

        gz2000 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-gz2000.txt"))

        start_date = datetime.datetime.strptime('20110101', '%Y%m%d').date()
        end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()

        trans = eatmud.Transaction([hs300, gz2000], start_date, end_date)

        hs300.truncate(start_date, end_date)
        gz2000.truncate(start_date, end_date)

        results = []
        for ma_period in [5, 10, 20, 30, 60, 120, 200, 500]:
            ind1 = eatmud.HistoryView.from_vec(
                trans, diff(hs300.ma(ma_period)))
            ind2 = eatmud.HistoryView.from_vec(
                trans, diff(gz2000.ma(ma_period)))
            inds = [ind1, ind2]
            resi = []
            for weekday in range(5):
                it = trans.iter(False, True)
                it.inflow(1.0)
                eatmud.strategy.indicator_weekly(
                    it, eatmud.Weekday(weekday), inds)
                resi.append(it.asset())
            results.append(resi)

    def test_extremum_ma_daily(self):
        hs300 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-hs300.txt"))

        gz2000 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-gz2000.txt"))

        start_date = datetime.datetime.strptime('20110101', '%Y%m%d').date()
        end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()

        trans = eatmud.Transaction([hs300, gz2000], start_date, end_date)

        hs300.truncate(start_date, end_date)
        gz2000.truncate(start_date, end_date)

        results = []
        for ma_period in [5, 10, 20, 30, 60, 120, 200, 500]:
            ind1 = eatmud.HistoryView.from_vec(
                trans, diff(hs300.ma(ma_period)))
            ind2 = eatmud.HistoryView.from_vec(
                trans, diff(gz2000.ma(ma_period)))
            inds = [ind1, ind2]
            it = trans.iter(False, True)
            it.inflow(1.0)
            eatmud.strategy.indicator_daily(
                it, inds)
            results.append(it.asset())


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
