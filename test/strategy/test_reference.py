#!/usr/bin/env python3

import unittest
import datetime
import eatmud


class TestReference(unittest.TestCase):

    def test_reference_1(self):
        hs300 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-hs300.txt"))
        gz2000 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-gz2000.txt"))

        start_date = datetime.datetime.strptime('20170101', '%Y%m%d').date()
        end_date = datetime.datetime.strptime('20240101', '%Y%m%d').date()
        trans = eatmud.Transaction([hs300, gz2000], None, end_date)
        it = trans.iter(False, False)
        start_date = datetime.datetime.strptime('20170101', '%Y%m%d').date()
        it.goto(start_date)
        it.inflow(1.)
        eatmud.strategy.reference(it)
        # 2024-01-01 is a holiday, so the last trading day should be earlier.
        self.assertEqual(it.today(), trans.dates()[-1])
        # The reference strategy holds no position, so the asset stays
        # unchanged.
        self.assertAlmostEqual(it.asset(), 1.)


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
