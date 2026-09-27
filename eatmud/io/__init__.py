#!/usr/bin/env python3

from eatmud._core import io as _core_io

from .xlsx import read_record, save_record

read_tdx = _core_io.read_tdx


# Lazily imported exports from the optional tdx_tq module.
# tdx is an optional dependency, so these are imported on first access
# to avoid requiring pandas/tqcenter at package import time.
_TDX_TQ_EXPORTS = ("get_market_data", "TdxTqError")


def __getattr__(name):
    """Lazily import tdx_tq exports to keep tdx optional."""
    if name in _TDX_TQ_EXPORTS:
        from .tdx_tq import TdxTqError, get_market_data

        return {"get_market_data": get_market_data,
                "TdxTqError": TdxTqError}[name]
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__():
    """Include the lazily imported exports in dir()."""
    return [*globals(), *_TDX_TQ_EXPORTS]


__all__ = [
    "TdxTqError",
    "get_market_data",
    "read_record",
    "read_tdx",
    "save_record",
]
