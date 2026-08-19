use std::fmt;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use oracle::Connection;

use super::model::StockHistoryRow;

#[derive(Debug)]
pub enum RepositoryError {
    Database(oracle::Error),
    Conversion(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::Database(err) => write!(f, "database error: {err}"),
            RepositoryError::Conversion(msg) => write!(f, "conversion error: {msg}"),
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<oracle::Error> for RepositoryError {
    fn from(err: oracle::Error) -> Self {
        RepositoryError::Database(err)
    }
}

pub fn list_stock_history(conn: &Connection) -> Result<Vec<StockHistoryRow>, RepositoryError> {
    let sql = r#"SELECT "DATE", TICKER, "OPEN", HIGH, LOW, "CLOSE", VOLUME
                 FROM STOCK_HISTORY ORDER BY "DATE" DESC"#;

    let mut result = Vec::new();
    for row_result in conn.query(sql, &[])? {
        let row = row_result?;

        let date: NaiveDate = row.get("DATE")?;
        let ticker: String = row.get("TICKER")?;
        // Fetch NUMBER columns as strings to preserve exact precision for BigDecimal.
        let open: String = row.get("OPEN")?;
        let high: String = row.get("HIGH")?;
        let low: String = row.get("LOW")?;
        let close: String = row.get("CLOSE")?;
        let volume: String = row.get("VOLUME")?;

        result.push(StockHistoryRow {
            date,
            ticker,
            open: BigDecimal::from_str(&open)
                .map_err(|e| RepositoryError::Conversion(format!("invalid open value: {e}")))?,
            high: BigDecimal::from_str(&high)
                .map_err(|e| RepositoryError::Conversion(format!("invalid high value: {e}")))?,
            low: BigDecimal::from_str(&low)
                .map_err(|e| RepositoryError::Conversion(format!("invalid low value: {e}")))?,
            close: BigDecimal::from_str(&close)
                .map_err(|e| RepositoryError::Conversion(format!("invalid close value: {e}")))?,
            volume: volume
                .parse()
                .map_err(|e| RepositoryError::Conversion(format!("invalid volume value: {e}")))?,
        });
    }

    Ok(result)
}
