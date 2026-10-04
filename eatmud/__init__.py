#!/usr/bin/env python3
from __future__ import annotations

import abc
import datetime

from ._core import (
    DAYS_PER_YEAR,
    ConciseRecord,
    ConciseRecordSlice,
    DetailedRecord,
    DetailedRecordSlice,
    Fund,
    FundSlice,
    HistoryView,
    Stock,
    StockSlice,
    Transaction,
    Weekday,
    get_irrs,
    irr,
    max_drawdown,
    merge_records,
)
from . import _core, io, strategy

__all__ = [
    "DAYS_PER_YEAR",
    "ConciseRecord",
    "Date",
    "DetailedRecord",
    "Fund",
    "HistoryView",
    "Stock",
    "Transaction",
    "Weekday",
    "_core",
    "get_irrs",
    "io",
    "irr",
    "max_drawdown",
    "merge_records",
    "strategy",
]

Date = datetime.date


class DataSlice(abc.ABC):
    @abc.abstractproperty
    def date(self) -> datetime.date: ...

    @abc.abstractproperty
    def value(self) -> float: ...


DataSlice.register(FundSlice)
DataSlice.register(StockSlice)


class Data(abc.ABC):
    @abc.abstractproperty
    def name(self) -> str: ...

    @abc.abstractproperty
    def code(self) -> str: ...

    @abc.abstractmethod
    def __len__(self) -> int: ...

    @abc.abstractmethod
    def is_empty(self) -> bool: ...

    @abc.abstractmethod
    def __getitem__(self, index: int) -> DataSlice: ...


Data.register(Fund)
Data.register(Stock)


class RecordSlice(abc.ABC):
    @abc.abstractproperty
    def date(self) -> datetime.date: ...

    @abc.abstractproperty
    def investment(self) -> float: ...

    @abc.abstractproperty
    def present_value(self) -> float: ...

    @abc.abstractproperty
    def comment(self) -> str: ...

    @abc.abstractproperty
    def total_investment(self) -> float: ...

    @abc.abstractproperty
    def profit(self) -> float: ...


RecordSlice.register(ConciseRecordSlice)
RecordSlice.register(DetailedRecordSlice)


class Record(abc.ABC):
    @abc.abstractproperty
    def name(self) -> str: ...

    @abc.abstractproperty
    def code(self) -> str: ...

    @abc.abstractproperty
    def comment(self) -> str: ...

    @abc.abstractmethod
    def __len__(self) -> int: ...

    @abc.abstractmethod
    def is_empty(self) -> bool: ...

    @abc.abstractmethod
    def clear(self) -> None: ...

    @abc.abstractmethod
    def __str__(self) -> str: ...

    @abc.abstractmethod
    def __getitem__(self, index: int) -> DataSlice: ...

    @abc.abstractmethod
    def irr(
        self,
        start_date: datetime.date | None,
        end_date: datetime.date | None,
        start_value: float | None,
        end_value: float | None,
        x0: float | None,
    ) -> float: ...

    @abc.abstractmethod
    def irr_naive(self) -> float: ...

    def irr_direct(
        self,
        start_date: datetime.date,
        end_date: datetime.date,
        start_value: float,
        end_value: float,
        start_idx: int,
        end_idx: int,
        x0: float,
    ) -> float: ...


Record.register(ConciseRecord)
Record.register(DetailedRecord)
