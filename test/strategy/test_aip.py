#!/usr/bin/env python3

import unittest
import datetime
import eatmud


class TestAIP(unittest.TestCase):
    def setUp(self):
        hs300 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-hs300.txt"))
        gz2000 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-gz2000.txt"))

        start_date = datetime.datetime.strptime('20110101', '%Y%m%d').date()
        end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()
        trans = eatmud.Transaction([hs300, gz2000], start_date, end_date)
        self.trans = trans

    def test_aip_1(self):
        trans = self.trans
        results = []
        for save_log in (True, False,):
            for save_record in (True, False):
                res = []
                for day in range(1, 29):
                    it = trans.iter(save_log, save_record)
                    eatmud.strategy.aip_monthly(it, day, [1000.]*2, [0.]*2)
                    res.append(it.asset())
                results.append(res)
        self.assertTrue(results[0] == results[1])
        self.assertTrue(results[0] == results[2])
        self.assertTrue(results[0] == results[3])

    def test_aip_2(self):
        trans = self.trans
        results = []
        for day in range(1, 29):
            it = trans.iter(True, True)
            eatmud.strategy.aip_monthly(it, day, [1000.]*2, [2.]*2)
            rec = it.record()
            irr = rec.irr()
            results.append(irr)
        self.assertAlmostEqual(results[0], 0.03206, delta=1e-5)
        self.assertAlmostEqual(results[1], 0.03140, delta=1e-5)
        self.assertAlmostEqual(results[2], 0.03128, delta=1e-5)
        self.assertAlmostEqual(results[-2], 0.02339, delta=1e-5)
        self.assertAlmostEqual(results[-1], 0.02659, delta=1e-5)


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
