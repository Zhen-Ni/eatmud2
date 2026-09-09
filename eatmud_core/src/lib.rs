mod common;
pub mod data;
pub mod indicators;
pub mod io;
pub mod prelude;
pub mod record;
pub mod strategy;
pub mod transaction;
pub mod utility;

pub use chrono::{Datelike, Duration, NaiveDate};
pub use data::{Data, DataSlice, Fund, FundSlice, Stock, StockSlice};
pub use record::{ConciseRecord, DetailedRecord, get_irrs};
pub use transaction::{HistoryView, Transaction, TransactionIterator, Weekday};

pub use prelude::*;
pub use utility::{DAYS_PER_YEAR, SIDE, irr, max_drawdown};
