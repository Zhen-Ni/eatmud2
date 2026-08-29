#!/usr/bin/env python3

import unittest
import datetime
import eatmud


def almost_eq(x1: float, x2: float) -> bool:
    return abs(x1 - x2) < 0.001


def is_same_display(r1: eatmud.Record,
                    r2: eatmud.Record) -> bool:
    s1 = str(r1).split('\n')
    s2 = str(r2).split('\n')
    if len(s1) != len(s2):
        return False
    for (l1, l2) in zip(s1, s2):
        si1 = l1.split()
        si2 = l2.split()
        if len(si1) != len(si2):
            return False
        for (w1, w2) in zip(si1, si2):
            if w1.startswith('-0.00'):
                w1 = w1[1:]
            if w2.startswith('-0.00'):
                w2 = w2[1:]
            if w1 != w2:
                return False
    return True


class TestXlsx(unittest.TestCase):
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

    def test_xlsx(self):
        start_date = self.start_date
        trans = self.trans
        ns = [1300, 1600]
        inflations = [0.015, 0.015]
        risk_bounds = [0.01, 0.01]
        it = trans.iter(False, True)
        it.goto(start_date)
        it.inflow(10000.)
        eatmud.strategy.kelly_weekly(it, eatmud.Weekday(1), ns, inflations,
                                     risk_bounds)
        rec = it.cash_record()
        eatmud.io.save_record(rec, 'test_save.xlsx', 'cash')
        rec = it.fund_record(0)
        eatmud.io.save_record(rec, 'test_save.xlsx', 'hs300')
        rec = it.fund_record(1)
        eatmud.io.save_record(rec, 'test_save.xlsx', 'gz2000')

        rec = eatmud.io.read_record('test_save.xlsx', 'cash')
        self.assertTrue(is_same_display(rec, it.cash_record()))
        for rs1, rs2 in zip(rec, it.cash_record()):
            self.assertAlmostEqual(rs1.date, rs2.date)
            self.assertAlmostEqual(rs1.investment, rs2.investment)
            self.assertAlmostEqual(rs1.present_value, rs2.present_value)
            self.assertAlmostEqual(rs1.comment, rs2.comment)
            self.assertAlmostEqual(rs1.total_investment, rs2.total_investment)
            self.assertAlmostEqual(rs1.profit, rs2.profit)

        rec = eatmud.io.read_record('test_save.xlsx', 'hs300')
        self.assertTrue(is_same_display(rec, it.fund_record(0)))
        for rs1, rs2 in zip(rec, it.fund_record(0)):
            self.assertAlmostEqual(rs1.date, rs2.date)
            self.assertAlmostEqual(rs1.investment, rs2.investment)
            self.assertAlmostEqual(rs1.present_value, rs2.present_value)
            self.assertAlmostEqual(rs1.comment, rs2.comment)
            self.assertAlmostEqual(rs1.total_investment, rs2.total_investment)
            self.assertAlmostEqual(rs1.profit, rs2.profit)
            self.assertAlmostEqual(rs1.fee, rs2.fee)
            self.assertAlmostEqual(rs1.nav, rs2.nav)
            self.assertAlmostEqual(rs1.share, rs2.share)

        rec = eatmud.io.read_record('test_save.xlsx', 'gz2000')
        self.assertTrue(is_same_display(rec, it.fund_record(1)))
        for rs1, rs2 in zip(rec, it.fund_record(1)):
            self.assertAlmostEqual(rs1.date, rs2.date)
            self.assertAlmostEqual(rs1.investment, rs2.investment)
            self.assertAlmostEqual(rs1.present_value, rs2.present_value)
            self.assertAlmostEqual(rs1.comment, rs2.comment)
            self.assertAlmostEqual(rs1.total_investment, rs2.total_investment)
            self.assertAlmostEqual(rs1.profit, rs2.profit)
            self.assertAlmostEqual(rs1.fee, rs2.fee)
            self.assertAlmostEqual(rs1.nav, rs2.nav)
            self.assertAlmostEqual(rs1.share, rs2.share)


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
