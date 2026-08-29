#!/usr/bin/env python3

from eatmud._core import io as _core_io
read_tdx = _core_io.read_tdx

from .xlsx import *


__all__ = ['read_tdx', ]
