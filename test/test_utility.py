#!/usr/bin/env python3

import unittest
import numpy as np
from eatmud import max_drawdown
from eatmud import irr
from eatmud import moving_average, exponential_moving_average


class TestUtility(unittest.TestCase):
    def test_irr(self):
        days = [720, 360, 0]
        investment = [1, 2, 0]
        end_value = 8
        x0 = 0
        res = irr(days, investment, end_value, x0)
        self.assertAlmostEqual(res, 1., delta=1e-4)

    def test_max_drawdown(self):
        x = [1, 2, 3, 4, 5]
        res = max_drawdown(x)
        self.assertEqual(res, (0, 0, 0))
        res = max_drawdown([100, 90, 180, 70])
        self.assertEqual(res, (110. / 180., 2, 3))
        res = max_drawdown([180, 70, 180, 90])
        self.assertEqual(res, (110. / 180., 0, 1))
        res = max_drawdown([180, 70, 80, 90])
        self.assertEqual(res, (110 / 180., 0, 1))
        res = max_drawdown([180, 70, 80, 60])
        self.assertEqual(res, (120 / 180., 0, 3))
        res = max_drawdown([180, 70, 200, 180])
        self.assertEqual(res, (110 / 180., 0, 1))

    def test_moving_average(self):
        data = [60.0, 100.0, 80.0, 120.0]
        result = moving_average(data, 3)
        self.assertEqual(result, [60.0, 80.0, 80.0, 100.0])

    def test_exponential_moving_average(self):
        data = [60.0, 100.0, 80.0, 120.0]
        result = exponential_moving_average(data, 3)
        self.assertEqual(len(result), 4)
        self.assertEqual(result[0], 60)


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
