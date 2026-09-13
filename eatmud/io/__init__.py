#!/usr/bin/env python3

from .xlsx import *
from eatmud._core import io as _core_io
read_tdx = _core_io.read_tdx


__all__ = ['read_tdx', ]
