#!/usr/bin/env python3

import unittest
import eatmud


class TestTransaction(unittest.TestCase):
    def setUp(self):
        self.hs300 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-hs300.txt"))
        self.gz2000 = eatmud.Fund.from_stock(
            eatmud.io.read_tdx("tdx/test-gz2000.txt"))

    def test_transaction_construct(self):
        hs300 = self.hs300
        gz2000 = self.gz2000
        start_date = eatmud.Date(2020, 1, 1)
        t = eatmud.Transaction([hs300, gz2000], start_date, None)
        self.assertAlmostEqual(t.navs()[0, 0], 4152.24)
        self.assertAlmostEqual(t.navs()[1, 0], 4144.97)
        self.assertAlmostEqual(t.navs()[0, 1], 6262.91)
        self.assertAlmostEqual(t.navs()[1, 1], 6299.53)

    def test_transaction1(self):
        """Test iter `next_day.`"""
        hs300 = self.hs300
        gz2000 = self.gz2000
        start_date = eatmud.Date(2020, 1, 1)
        t = eatmud.Transaction([hs300, gz2000], start_date, None)
        it = t.iter()
        idx = 0
        while it.next_day():
            it.inflow(1.0)
            self.assertEqual(it.cash(), idx)
            self.assertEqual(it.asset(), idx)
            idx += 1
        self.assertTrue(it.cash() > 980)

    def test_transaction2(self):
        """Test iter `next_weekday`."""
        hs300 = self.hs300
        gz2000 = self.gz2000
        start_date = eatmud.Date(2024, 1, 1)
        end_date = eatmud.Date(2024, 1, 20)
        t = eatmud.Transaction([hs300, gz2000], start_date, end_date)
        it = t.iter()
        it.inflow(100)
        self.assertEqual(it.asset(), 0)
        while it.next_weekday(eatmud.Weekday.Wed):
            self.assertTrue(it.asset() > 90)
            self.assertTrue(it.asset() < 110)
            it.buy(0, 10, 0)
            it.buy(1, 10, 0)
        self.assertEqual(it.cash(), 40)
        self.assertAlmostEqual(it.share(0), 0.009108, delta=0.000001)

    def test_transaction3(self):
        """Test iter `next_month`."""
        hs300 = self.hs300
        gz2000 = self.gz2000
        start_date = eatmud.Date(2023, 11, 1)
        end_date = eatmud.Date(2024, 1, 22)
        t = eatmud.Transaction([hs300, gz2000], start_date, end_date)
        it = t.iter()
        it.inflow(100)
        nav = 7459.99
        self.assertEqual(it.asset(), 0)
        while it.next_month(28):
            it.buy(1, 100, 0.1)
        self.assertEqual(it.cash(), 0.)
        self.assertAlmostEqual(it.share(1), 99.9 / nav)
        self.assertAlmostEqual(it.asset(), it.share(1) * 6842.75)

    def test_transaction4(self):
        """Test iter `goto`."""
        hs300 = self.hs300
        gz2000 = self.gz2000
        start_date = eatmud.Date(2024, 1, 1)
        end_date = eatmud.Date(2024, 1, 22)
        t = eatmud.Transaction([hs300, gz2000], start_date, end_date)
        it = t.iter()
        it.inflow(100)
        self.assertEqual(it.today(), eatmud.Date(2024, 1, 2))
        it.buy(1, 100, 1)       # nav = 7562.02
        it.goto(eatmud.Date(2024, 1, 4))
        self.assertEqual(len(it.cash_log()), 2)
        self.assertEqual(it.navs().shape, (2, 2))

        it.goto(eatmud.Date(2024, 1, 13))
        self.assertEqual(it.today(), eatmud.Date(2024, 1, 15))
        # The following statement is equal to `it.sell(1, it.share(1), 1)`.
        it.position(1, 0., 1, perfect_position=False)  # nav = 7195.25
        it.next_day()
        self.assertEqual(it.cash(), it.asset())
        self.assertAlmostEqual(99. / 7562.02 * 7195.25 - 1., it.asset())
        self.assertFalse(it.goto(eatmud.Date(2024, 1, 20)))

    def test_trans_after_finish(self):
        hs300 = self.hs300
        gz2000 = self.gz2000
        start_date = eatmud.Date(2023, 11, 1)
        end_date = eatmud.Date(2024, 1, 22)
        t = eatmud.Transaction([hs300, gz2000], start_date, end_date)
        it = t.iter()
        it.inflow(100)
        while it.next_month(28):
            it.position(1, 1.0, 0.1)
        with self.assertRaises(Exception) as context:
            it.sell(1, it.share(1), 0.2)
        self.assertTrue(context.exception)


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
