#!/usr/bin/env python3

import time

import eatmud


def diff(v: list[float]) -> list[float]:
    if not v:
        return []
    r = [0.0]
    for i in range(1, len(v)):
        r.append(v[i] - v[i - 1])
    return r


def bench_indicator(
    trans: eatmud.Transaction,
    indicators: list[eatmud.HistoryView],
    name: str,
) -> None:
    now = time.perf_counter()
    results = []
    for weekday in range(5):
        it = trans.iter(False, False)
        it.inflow(1.0)
        eatmud.strategy.indicator_weekly(
            it, eatmud.Weekday(weekday), indicators)
        results.append(it.asset())
    total_time = time.perf_counter() - now
    print(
        f"{name}, result = [{', '.join(f'{x:.3f}' for x in results)}], "
        f"took {total_time * 1000:.3f} ms"
    )


def ma_extremum(trans: eatmud.Transaction, funds: list[eatmud.Fund]) -> None:
    start_date = trans.start_date
    end_date = trans.end_date
    for ma_period in [5, 10, 20, 50, 100, 200, 500, 1000, 2000]:
        views = []
        for f in funds:
            ma = f.ma(ma_period)
            s = f.search_index(start_date)
            e = f.search_index(end_date)
            views.append(eatmud.HistoryView.from_vec(trans, diff(ma[s:e])))
        bench_indicator(trans, views, f"MA{ma_period}")


def ma_cross(trans: eatmud.Transaction, funds: list[eatmud.Fund]) -> None:
    start_date = trans.start_date
    end_date = trans.end_date
    ma_short_periods = [5, 10, 20, 50, 100]
    ma_long_periods = [20, 50, 100, 200, 500]
    keys = set(ma_short_periods) | set(ma_long_periods)
    ma_map = {}
    for period in keys:
        ma_map[period] = [
            f.ma(period)[f.search_index(start_date):f.search_index(end_date)]
            for f in funds
        ]
    for short_period in ma_short_periods:
        for long_period in ma_long_periods:
            if not (short_period < long_period):
                continue
            ma_shorts = ma_map[short_period]
            ma_longs = ma_map[long_period]
            difs = [
                eatmud.HistoryView.from_vec(
                    trans, [s - l for s, l in zip(sv, lv)]
                )
                for sv, lv in zip(ma_shorts, ma_longs)
            ]
            bench_indicator(trans, difs, f"MA({short_period},{long_period})")


def macd_cross(trans: eatmud.Transaction, funds: list[eatmud.Fund]) -> None:
    start_date = trans.start_date
    end_date = trans.end_date
    for short in [5, 10, 12, 20, 50, 100]:
        for long in [20, 26, 50, 100, 200, 500]:
            for signal in [5, 9, 10, 20, 50]:
                if not (short < long):
                    continue
                macds = [
                    eatmud.HistoryView.from_vec(
                        trans,
                        f.macd(short, long, signal).hist[
                            f.search_index(start_date):f.search_index(end_date)
                        ],
                    )
                    for f in funds
                ]
                bench_indicator(trans, macds, f"MACD({short},{long},{signal})")


if __name__ == "__main__":
    hs300 = eatmud.Fund.from_stock(
        eatmud.io.read_tdx("tdx/test-hs300.txt")
    )
    gz2000 = eatmud.Fund.from_stock(
        eatmud.io.read_tdx("tdx/test-gz2000.txt")
    )
    trans = eatmud.Transaction([hs300, gz2000], "20170101", "20240101")
    funds = [hs300, gz2000]
    ma_extremum(trans, funds)
    print("--------")
    ma_cross(trans, funds)
    print("--------")
    macd_cross(trans, funds)
