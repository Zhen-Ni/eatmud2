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


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
