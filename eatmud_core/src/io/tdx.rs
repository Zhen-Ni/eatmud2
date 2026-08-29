use crate::{Data, NaiveDate, Stock, StockSlice};
use encoding_rs;
use std::error::Error;
use std::io::BufRead;
use std::{fs, io};

#[derive(Debug)]
pub struct ReadTdxDataError(&'static str);

impl std::fmt::Display for ReadTdxDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Read Data Error: {}", self.0)
    }
}

impl std::error::Error for ReadTdxDataError {}

/// Read stock data from GuoTaiAn's txt output file.
pub fn read_tdx(path: &str) -> Result<Stock, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);

    let mut buffer = Vec::<u8>::new();
    reader.read_until(b'\n', &mut buffer)?;
    let (results, _encoding, _error) = encoding_rs::GBK.decode(&buffer);
    let line = results.to_string();
    buffer.clear();
    let first_line = line.split_whitespace().collect::<Vec<&str>>();
    let [name, code] = first_line.as_slice() else {
        return Err(Box::new(ReadTdxDataError(
            "Wrong file format: cannot parse header",
        )));
    };
    let code = &code.to_string()[1..code.len() - 1];
    let mut stock = Data::<StockSlice>::new(name, code);

    while let Ok(size) = reader.read_until(b'\n', &mut buffer) {
        if size == 0 {
            break;
        }
        let (results, _encoding, _error) = encoding_rs::GBK.decode(&buffer);
        let line = results.to_string();
        buffer.clear();

        let mut iter_words = line.split_whitespace();
        let Some(date) = iter_words.next() else {
            continue;
        };
        let Some(open) = iter_words.next() else {
            continue;
        };
        let Some(high) = iter_words.next() else {
            continue;
        };
        let Some(low) = iter_words.next() else {
            continue;
        };
        let Some(close) = iter_words.next() else {
            continue;
        };
        let Some(volume) = iter_words.next() else {
            continue;
        };

        let Ok(date): Result<NaiveDate, _> = NaiveDate::parse_from_str(date, "%Y/%m/%d") else {
            continue;
        };
        let Ok(open): Result<f64, _> = open.parse() else {
            continue;
        };
        let Ok(high): Result<f64, _> = high.parse() else {
            continue;
        };
        let Ok(low): Result<f64, _> = low.parse() else {
            continue;
        };
        let Ok(close): Result<f64, _> = close.parse() else {
            continue;
        };
        let Ok(volume): Result<f64, _> = volume.parse() else {
            continue;
        };
        stock.append(date, open, high, low, close, volume);
    }
    Ok(stock)
}

#[cfg(test)]
mod test {

    use crate::*;

    #[test]
    fn test_read_tdx() {
        let filename = "../tdx/test-hs300.txt";
        let stock = io::read_tdx(filename).expect("failed to read file");
        assert_eq!(stock[0].value(), 982.79);
        assert_eq!(stock.code(), "000300");
    }
}
