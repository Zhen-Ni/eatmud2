#!/usr/bin/env python3

from eatmud._core import strategy as _core_strategy

aip_monthly = _core_strategy.aip_monthly
kelly_hint = _core_strategy.kelly_hint
kelly_weekly = _core_strategy.kelly_weekly

__all__ = ['aip_monthly', 'kelly_hint', 'kelly_weekly']
