#!/usr/bin/env python3

import unittest
import datetime
import numpy as np
import eatmud


class TestKelly(unittest.TestCase):
    def setUp(self):
        hs300 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-hs300.txt"))
        gz2000 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-gz2000.txt"))
        start_date = datetime.datetime.strptime('20170101', '%Y%m%d').date()
        end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()
        trans = eatmud.Transaction([hs300, gz2000], None, end_date)

        self.start_date = start_date
        self.trans = trans

    def test_kelly_1(self):
        start_date = self.start_date
        trans = self.trans

        ns = [1300, 1600]
        inflations = [0.015, 0.015]
        risk_bounds = [0.01, 0.01]
        results = []
        for save_log in (True, False,):
            for save_record in (True, False):
                res = []
                for weekday in range(0, 5):
                    it = trans.iter(save_log, save_record)
                    it.goto(start_date)
                    it.inflow(1.)
                    eatmud.strategy.kelly_weekly(it, eatmud.Weekday(weekday), ns, inflations,
                                                 risk_bounds)
                    res.append(it.asset())
                results.append(res)
        self.assertTrue(results[0] == results[1])
        self.assertTrue(results[0] == results[2])
        self.assertTrue(results[0] == results[3])

    def test_kelly_2(self):
        start_date = self.start_date
        trans = self.trans

        ns = [1300, 1600]
        inflations = [0.015, 0.015]
        risk_bounds = [0.01, 0.01]
        result = []
        for weekday in range(0, 5):
            it = trans.iter(False, False)
            it.goto(start_date)
            it.inflow(1.)
            eatmud.strategy.kelly_weekly(
                it, eatmud.Weekday(weekday), ns, inflations,
                risk_bounds)
            result.append(it.asset())
        self.assertAlmostEqual(result[0], 1.53861, delta=0.00001)
        self.assertAlmostEqual(result[1], 1.66552, delta=0.00001)
        self.assertAlmostEqual(result[2], 1.50122, delta=0.00001)
        self.assertAlmostEqual(result[3], 1.48973, delta=0.00001)
        self.assertAlmostEqual(result[4], 1.39827, delta=0.00001)

    def test_kelly_3(self):
        """Test whether kelly_weekly and kelly_hint provides the same
        result."""
        start_date = self.start_date
        trans = self.trans

        ns = [1300, 1600]
        inflations = [0.015, 0.015]
        risk_bounds = [0.01, 0.01]

        weekday = eatmud.Weekday.Tue

        it1 = trans.iter(True, True)
        it1.goto(start_date)
        it1.inflow(1.)
        eatmud.strategy.kelly_weekly(it1, weekday, ns,
                                     inflations, risk_bounds)
        records1 = [it1.fund_record(i) for i in range(trans.nfunds)]

        it2 = trans.iter(True, True)
        it2.goto(start_date)
        it2.inflow(1.)
        while it2.next_weekday(weekday):
            for i in range(trans.nfunds):
                indicator = eatmud.strategy.kelly_hint(it2, i, weekday,
                                                       ns[i],
                                                       inflations[i],
                                                       risk_bounds[i])
                position = indicator.position / trans.nfunds
                it2.position(i, position,
                             comment=f'position = {100*position:.2f}%',
                             perfect_position=True)
        records2 = [it1.fund_record(i) for i in range(trans.nfunds)]

        for i in range(trans.nfunds):
            c1 = [records1[i][j].comment for j in range(len(records1[i]))]
            c2 = [records2[i][j].comment for j in range(len(records2[i]))]
            self.assertEqual(c1, c2)
        self.assertTrue(np.allclose(it1.asset_log(), it2.asset_log()))


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
