#!/usr/bin/env python3
import datetime
import abc
from typing import Optional
from . import _core
from ._core import FundSlice, StockSlice, Fund, Stock
from ._core import ConciseRecordSlice, DetailedRecordSlice, \
    ConciseRecord, DetailedRecord, merge_records, get_irrs
from ._core import Transaction, Weekday
from ._core import irr, max_drawdown, moving_average, \
    exponential_moving_average
from . import io
from . import strategy

__all__ = ['_core',
           'Date',
           'Fund', 'Stock',
           'ConciseRecord', 'DetailedRecord', 'merge_records', 'get_irrs',
           'irr', 'max_drawdown', 'moving_average',
           'exponential_moving_average',
           'Transaction', 'Weekday',
           'io', 'strategy']

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
    def irr(self,
            start_date: Optional[datetime.date],
            end_date: Optional[datetime.date],
            start_value: Optional[float],
            end_value: Optional[float],
            x0: Optional[float]
            ) -> float: ...

    @abc.abstractmethod
    def irr_naive(self) -> float: ...

    def irr_direct(self,
                   start_date: datetime.date,
                   end_date: datetime.date,
                   start_value: float,
                   end_value: float,
                   start_idx: int,
                   end_idx: int,
                   x0: float
                   ) -> float: ...


Record.register(ConciseRecord)
Record.register(DetailedRecord)
