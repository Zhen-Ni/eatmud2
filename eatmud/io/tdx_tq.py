#!/usr/bin/env python3

from __future__ import annotations

import os
import re
import sys
from datetime import date

from eatmud import Stock

_TDX_PATH = os.getenv("TDX_DIR") or r"C:/new_tdx64/"
_PYPLUGIN_DIR = os.path.join(_TDX_PATH, "PYPlugins/user")

_tq = None  # Lazily loaded tq object


def _load_tq():
    """Import and initialize tqcenter on first use."""
    global _tq
    if _tq is not None:
        return _tq

    try:
        sys.path.append(_PYPLUGIN_DIR)
        from tqcenter import tq as tq_module
    except ImportError as e:
        missing = getattr(e, "name", None)
        if missing == "tqcenter":
            raise TdxTqError(
                f"Failed to load tqcenter (TDX_DIR={_TDX_PATH}). "
                "Please check that the TDX PYPlugins are installed."
            ) from e
        raise TdxTqError(
            "Failed to load tqcenter due to a missing dependency: "
            f"{missing!r}. "
            "The tdx extra is optional; try installing it with: "
            "pip install eatmud[tdx]"
        ) from e

    if "__file__" in globals():
        tq_module.initialize(__file__)
    else:
        tq_module.initialize(os.path.join(_PYPLUGIN_DIR, "tdxdata_test.py"))
    _tq = tq_module
    return _tq


class TdxTqError(Exception):
    def __init__(self, s):
        super().__init__(s)


def get_market_data(
    tdx_name: str,
    start: date | str | None = None,
    end: date | str | None = None,
    name: str | None = None,
    code: str | None = None,
) -> Stock:
    """Get the market data from tdx tq interface.

    Parameters
    ----------
    tdx_name: str
        The stock name in tdx.
    start: datetime.date or str, optional
        The start date for the data. If None, the earliest date
        available is used.
    end: datetime.date or str, optional
        The end date for the data. If None, the latest date
        available is used.
    name: str, optional
        The name of the returned Stock object. If None, defaults
        to tdx_name.
    code: str, optional
        The code of the returned Stock object. If None, inferred
        from the first 6 consecutive digits in tdx_name.

    Returns
    -------
    Stock
        A Stock object filled with daily market data of the stock.

    Raises
    ------
    TdxTqError
        If the stock name is not found in tdx.
    """
    tq = _load_tq()

    # Format start and end to str.
    start = start.strftime("%Y%m%d") if isinstance(start, date) \
        else (start or "")
    end = end.strftime("%Y%m%d") if isinstance(end, date) else \
        (end or "")

    name = name or tdx_name
    if code is None:
        if res := re.match(r"\d{6}", tdx_name):
            code = res.group()
        else:
            code = ""

    data = tq.get_market_data(
        stock_list=[tdx_name],
        start_time=start,
        end_time=end,
        period="1d",
        # "front" for front adjusted, "none" for non-adjusted
        dividend_type="front",
    )
    if not data:
        raise TdxTqError(f"stock name {tdx_name} not found in tdx")

    opens = data["Open"].to_numpy().reshape(-1)
    highs = data["High"].to_numpy().reshape(-1)
    lows = data["Low"].to_numpy().reshape(-1)
    closes = data["Close"].to_numpy().reshape(-1)
    volumes = data["Volume"].to_numpy().reshape(-1)
    dates = data["Close"].index

    stock = Stock(name, code)
    for i, d in enumerate(dates):
        stock.append(d.date(), opens[i], highs[i],
                     lows[i], closes[i], volumes[i])

    return stock
