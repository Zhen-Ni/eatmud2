#!/usr/bin/env python3

import unittest
import datetime
import eatmud


class TestData(unittest.TestCase):
    def test_fund(self):
        filename = 'tdx/test-hs300.txt'
        stock = eatmud.io.read_tdx(filename)
        fund = eatmud.Fund.from_stock(stock)
        n = len(fund)
        self.assertAlmostEqual(fund[0].value, 982.79)
        self.assertEqual(fund.code, '000300')
        date = datetime.date(2025, 1, 1)
        fund.append(date, 2.0)
        self.assertEqual(fund[n].date, date)
        self.assertEqual(fund[n].value, 2.0)

        date = datetime.date(2024, 1, 1)
        fund.truncate(date)
        self.assertTrue(fund[0].date > date)

    def test_stock(self):
        stock = eatmud.Stock("ndsd", "SZ300750")
        date = datetime.date(2024, 1, 11)
        stock.append(date, 150.66, 156.37, 148.51, 154.82, 10000.)
        stock.append(date + datetime.timedelta(1),
                     157.34, 159.87, 148.51, 153.45, 9754.)
        self.assertEqual(len(stock), 2)
        self.assertEqual(stock[0].date, date)
        self.assertEqual(stock[0].value, 154.82)
        self.assertEqual(stock[0].open, 150.66)
        self.assertEqual(stock[0].high, 156.37)
        self.assertEqual(stock[0].low, 148.51)
        self.assertEqual(stock[0].close, 154.82)
        self.assertEqual(stock[1].value, 153.45)

class TestIndicators(unittest.TestCase):
    def setUp(self):
        self.fund = eatmud.Fund("test", "000000")
        base_date = datetime.date(2024, 1, 1)
        for i in range(1, 11):
            self.fund.append(base_date + datetime.timedelta(days=i - 1), float(i))

    def test_ma(self):
        fund = self.fund
        ma = fund.ma(3)
        self.assertEqual(len(ma), 10)
        self.assertAlmostEqual(ma[0], 1.0)  # [1.0]
        self.assertAlmostEqual(ma[1], 1.5)  # [1.0, 2.0]
        self.assertAlmostEqual(ma[2], 2.0)  # [1.0, 2.0, 3.0]
        self.assertAlmostEqual(ma[3], 3.0)  # [2.0, 3.0, 4.0]
        self.assertAlmostEqual(ma[9], 9.0)  # [8.0, 9.0, 10.0]

    def test_ema(self):
        fund = self.fund
        ema = fund.ema(3)
        self.assertEqual(len(ema), 10)
        self.assertAlmostEqual(ema[0], 1.0)
        # alpha = 2 / (3 + 1) = 0.5
        expected_1 = 0.5 * 2.0 + 0.5 * 1.0  # 1.5
        self.assertAlmostEqual(ema[1], expected_1)
        expected_2 = 0.5 * 3.0 + 0.5 * expected_1  # 2.25
        self.assertAlmostEqual(ema[2], expected_2)

    def test_macd(self):
        fund = self.fund
        macd = fund.macd(3, 5, 2)
        self.assertEqual(len(macd.dif), 10)
        # The first values of dif and hist are 0, because ema_short[0] == ema_long[0] == data[0]
        self.assertAlmostEqual(macd.dif[0], 0.0)
        self.assertAlmostEqual(macd.hist[0], 0.0)
        # dif[1] = ema_short[1] - ema_long[1]
        # ema_short[1] = 1.5
        # ema_long[1] = 2/6 * 2.0 + 4/6 * 1.0 = 4/3
        # dif[1] = 1.5 - 4.0 / 3.0 = 1.0 / 6.0
        self.assertAlmostEqual(macd.dif[1], 1.0 / 6.0)

    def test_boll(self):
        fund = self.fund
        boll = fund.boll(3, 2.0)
        self.assertEqual(len(boll.mid), 10)

        # Data: [1.0]
        # mid=1.0, std=0.0 -> upper=1.0, lower=1.0
        self.assertAlmostEqual(boll.mid[0], 1.0)
        self.assertAlmostEqual(boll.upper[0], 1.0)
        self.assertAlmostEqual(boll.lower[0], 1.0)

        # Data: [1.0, 2.0]
        # mid=1.5, std=0.5 -> upper=2.5, lower=0.5
        self.assertAlmostEqual(boll.mid[1], 1.5)
        self.assertAlmostEqual(boll.upper[1], 2.5)
        self.assertAlmostEqual(boll.lower[1], 0.5)


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
