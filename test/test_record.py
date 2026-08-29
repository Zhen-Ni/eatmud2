#!/usr/bin/env python3

import unittest

import eatmud


class TestRecord(unittest.TestCase):
    def test_record_from(self):
        record = eatmud.DetailedRecord("hs300", "123456")
        record.append(eatmud.Date(2024, 1, 1), 10.,
                      1., 10., "initial investment")
        record.append(eatmud.Date(2024, 2, 1), 10.,
                      1.2, 8.3, "investment 2")
        record2 = eatmud.ConciseRecord.from_detailed(record)
        self.assertEqual(record2[1].total_investment, 20)
        self.assertAlmostEqual(record2[len(record2)-1].profit, 1.96)

    def test_merge(self):
        record1 = eatmud.ConciseRecord("hs300", "123456")
        record1.append(eatmud.Date(2024, 1, 1), 10, 10, "investment 0")
        record1.append(eatmud.Date(2024, 2, 1), 15, 25, "investment 3")
        record2 = eatmud.DetailedRecord("hs300", "123456")
        record2.append(eatmud.Date(2024, 1, 1), 10, 1, 10, "investment 1")
        record2.append(eatmud.Date(2024, 1, 5), 20, 1, 20, "investment 2")
        record2.append(eatmud.Date(2024, 2, 5), 5, 1, 5, "investment 4",)
        merged = eatmud.merge_records(record1, record2)
        self.assertEqual(len(merged), 4)
        self.assertEqual(merged[3].total_investment, 60.)
        self.assertEqual(merged[0].comment,
                         "investment 0; investment 1")
        for i in range(1, 4):
            self.assertEqual(merged[i].comment,
                             f"investment {i + 1}; estimated")


if __name__ == '__main__':
    unittest.main(argv=[''], exit=False)
