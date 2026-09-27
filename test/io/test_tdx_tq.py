#!/usr/bin/env python3

import unittest

import eatmud

try:
    import eatmud.io.tdx_tq

    HAS_TDX_TQ = True
except Exception:
    HAS_TDX_TQ = False


class TestTdxTq(unittest.TestCase):
    def test_all(self):
        """Check that the lazy exports are listed in __all__."""
        for name in ["TdxTqError", "get_market_data", "read_tdx"]:
            self.assertTrue(name in eatmud.io.__all__)

    def test_dir(self):
        """Check that the lazy exports appear in dir()."""
        names = dir(eatmud.io)
        self.assertIn("get_market_data", names)
        self.assertIn("TdxTqError", names)
        self.assertIn("read_tdx", names)

    @unittest.skipUnless(HAS_TDX_TQ, "optional tdx module is not available")
    def test_lazy_import(self):
        """Check that the tdx_tq exports can be lazily imported."""
        from eatmud.io import TdxTqError, get_market_data

        self.assertTrue(issubclass(TdxTqError, Exception))
        self.assertTrue(callable(get_market_data))

    @unittest.skipUnless(HAS_TDX_TQ, "optional tdx module is not available")
    def test_tdx_tq(self):
        data_txt = eatmud.io.read_tdx("tdx/test-hs300.txt")
        data_tq = eatmud.io.get_market_data("000300.SH")
        self.assertEqual(data_tq.code, "000300")
        dates = ["20251122", "20260101"]
        for date in dates:
            slice_txt = data_txt[data_txt.search_index(date)]
            slice_tq = data_tq[data_tq.search_index(date)]
            self.assertEqual(slice_txt.date, slice_tq.date)
            self.assertEqual(slice_txt.open, slice_tq.open)
            self.assertEqual(slice_txt.high, slice_tq.high)
            self.assertEqual(slice_txt.low, slice_tq.low)
            self.assertEqual(slice_txt.close, slice_tq.close)
            self.assertEqual(slice_txt.volume, slice_tq.volume)
            self.assertEqual(slice_txt.value, slice_tq.value)


if __name__ == "__main__":
    unittest.main(argv=[""], exit=False)
