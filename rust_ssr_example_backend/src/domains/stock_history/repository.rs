use std::fmt;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use oracle::sql_type::{OracleType, ToSql};
use oracle::{Connection, SqlValue};

use crate::ssrm::{self, FilterModel, GetRowsRequest, GetRowsResponse};

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

impl RepositoryError {
    /// Whether this error stems from bad client input (filters/sort) vs. a real DB failure.
    pub fn is_client_error(&self) -> bool {
        matches!(self, RepositoryError::Conversion(_))
    }
}

const SELECT_COLUMNS: &str = r#""DATE", TICKER, "OPEN", HIGH, LOW, "CLOSE", VOLUME"#;

fn row_to_stock_history(row: &oracle::Row) -> Result<StockHistoryRow, RepositoryError> {
    let date: NaiveDate = row.get("DATE")?;
    let ticker: String = row.get("TICKER")?;
    // Fetch NUMBER columns as strings to preserve exact precision for BigDecimal.
    let open: String = row.get("OPEN")?;
    let high: String = row.get("HIGH")?;
    let low: String = row.get("LOW")?;
    let close: String = row.get("CLOSE")?;
    let volume: String = row.get("VOLUME")?;

    Ok(StockHistoryRow {
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
    })
}

pub fn list_stock_history(conn: &Connection) -> Result<Vec<StockHistoryRow>, RepositoryError> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM STOCK_HISTORY ORDER BY \"DATE\" DESC");

    let mut result = Vec::new();
    for row_result in conn.query(&sql, &[])? {
        result.push(row_to_stock_history(&row_result?)?);
    }

    Ok(result)
}

#[derive(Clone, Copy)]
enum ColumnKind {
    Text,
    Number,
    Date,
}

struct ColumnDef {
    sql_name: &'static str,
    kind: ColumnKind,
}

/// AG Grid colIds map 1:1 to these; anything else is rejected rather than
/// interpolated into SQL, since colId/sort/filterType all come from the client.
fn column_def(col_id: &str) -> Option<ColumnDef> {
    Some(match col_id {
        "date" => ColumnDef {
            sql_name: "\"DATE\"",
            kind: ColumnKind::Date,
        },
        "ticker" => ColumnDef {
            sql_name: "TICKER",
            kind: ColumnKind::Text,
        },
        "open" => ColumnDef {
            sql_name: "\"OPEN\"",
            kind: ColumnKind::Number,
        },
        "high" => ColumnDef {
            sql_name: "HIGH",
            kind: ColumnKind::Number,
        },
        "low" => ColumnDef {
            sql_name: "LOW",
            kind: ColumnKind::Number,
        },
        "close" => ColumnDef {
            sql_name: "\"CLOSE\"",
            kind: ColumnKind::Number,
        },
        "volume" => ColumnDef {
            sql_name: "VOLUME",
            kind: ColumnKind::Number,
        },
        _ => return None,
    })
}

#[derive(Clone)]
enum BindValue {
    Text(String),
    Num(f64),
    Date(NaiveDate),
}

impl ToSql for BindValue {
    fn oratype(&self, conn: &Connection) -> oracle::Result<OracleType> {
        match self {
            BindValue::Text(v) => v.oratype(conn),
            BindValue::Num(v) => v.oratype(conn),
            BindValue::Date(v) => v.oratype(conn),
        }
    }

    fn to_sql(&self, val: &mut SqlValue<'_>) -> oracle::Result<()> {
        match self {
            BindValue::Text(v) => v.to_sql(val),
            BindValue::Num(v) => v.to_sql(val),
            BindValue::Date(v) => v.to_sql(val),
        }
    }
}

fn push_bind(binds: &mut Vec<BindValue>, value: BindValue) -> String {
    binds.push(value);
    format!(":{}", binds.len())
}

fn parse_ag_grid_date(value: &str) -> Result<NaiveDate, RepositoryError> {
    let date_part = value.get(..10).unwrap_or(value);
    NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
        .map_err(|e| RepositoryError::Conversion(format!("invalid date filter value: {e}")))
}

fn text_predicate(
    col_sql: &str,
    filter: &ssrm::TextFilterModel,
    binds: &mut Vec<BindValue>,
) -> Result<String, RepositoryError> {
    Ok(match filter.filter_op.as_str() {
        "blank" => format!("{col_sql} IS NULL"),
        "notBlank" => format!("{col_sql} IS NOT NULL"),
        op => {
            let value = filter.filter.clone().ok_or_else(|| {
                RepositoryError::Conversion("missing text filter value".to_string())
            })?;
            match op {
                "equals" => format!("{col_sql} = {}", push_bind(binds, BindValue::Text(value))),
                "notEqual" => format!("{col_sql} <> {}", push_bind(binds, BindValue::Text(value))),
                "contains" => format!(
                    "{col_sql} LIKE {}",
                    push_bind(binds, BindValue::Text(format!("%{value}%")))
                ),
                "notContains" => format!(
                    "{col_sql} NOT LIKE {}",
                    push_bind(binds, BindValue::Text(format!("%{value}%")))
                ),
                "startsWith" => format!(
                    "{col_sql} LIKE {}",
                    push_bind(binds, BindValue::Text(format!("{value}%")))
                ),
                "endsWith" => format!(
                    "{col_sql} LIKE {}",
                    push_bind(binds, BindValue::Text(format!("%{value}")))
                ),
                other => {
                    return Err(RepositoryError::Conversion(format!(
                        "unsupported text filter type: {other}"
                    )));
                }
            }
        }
    })
}

fn number_predicate(
    col_sql: &str,
    filter: &ssrm::NumberFilterModel,
    binds: &mut Vec<BindValue>,
) -> Result<String, RepositoryError> {
    Ok(match filter.filter_op.as_str() {
        "blank" => format!("{col_sql} IS NULL"),
        "notBlank" => format!("{col_sql} IS NOT NULL"),
        "inRange" => {
            let from = filter.filter.ok_or_else(|| {
                RepositoryError::Conversion("missing inRange filter value".to_string())
            })?;
            let to = filter.filter_to.ok_or_else(|| {
                RepositoryError::Conversion("missing inRange filterTo value".to_string())
            })?;
            let from_p = push_bind(binds, BindValue::Num(from));
            let to_p = push_bind(binds, BindValue::Num(to));
            format!("{col_sql} BETWEEN {from_p} AND {to_p}")
        }
        op => {
            let value = filter.filter.ok_or_else(|| {
                RepositoryError::Conversion("missing number filter value".to_string())
            })?;
            let placeholder = push_bind(binds, BindValue::Num(value));
            match op {
                "equals" => format!("{col_sql} = {placeholder}"),
                "notEqual" => format!("{col_sql} <> {placeholder}"),
                "lessThan" => format!("{col_sql} < {placeholder}"),
                "lessThanOrEqual" => format!("{col_sql} <= {placeholder}"),
                "greaterThan" => format!("{col_sql} > {placeholder}"),
                "greaterThanOrEqual" => format!("{col_sql} >= {placeholder}"),
                other => {
                    return Err(RepositoryError::Conversion(format!(
                        "unsupported number filter type: {other}"
                    )));
                }
            }
        }
    })
}

fn date_predicate(
    col_sql: &str,
    filter: &ssrm::DateFilterModel,
    binds: &mut Vec<BindValue>,
) -> Result<String, RepositoryError> {
    Ok(match filter.filter_op.as_str() {
        "blank" => format!("{col_sql} IS NULL"),
        "notBlank" => format!("{col_sql} IS NOT NULL"),
        "inRange" => {
            let from = parse_ag_grid_date(filter.date_from.as_deref().ok_or_else(|| {
                RepositoryError::Conversion("missing dateFrom filter value".to_string())
            })?)?;
            let to = parse_ag_grid_date(filter.date_to.as_deref().ok_or_else(|| {
                RepositoryError::Conversion("missing dateTo filter value".to_string())
            })?)?;
            let from_p = push_bind(binds, BindValue::Date(from));
            let to_p = push_bind(binds, BindValue::Date(to));
            format!("{col_sql} BETWEEN {from_p} AND {to_p}")
        }
        op => {
            let date = parse_ag_grid_date(filter.date_from.as_deref().ok_or_else(|| {
                RepositoryError::Conversion("missing dateFrom filter value".to_string())
            })?)?;
            let placeholder = push_bind(binds, BindValue::Date(date));
            match op {
                "equals" => format!("{col_sql} = {placeholder}"),
                "notEqual" => format!("{col_sql} <> {placeholder}"),
                "lessThan" => format!("{col_sql} < {placeholder}"),
                "greaterThan" => format!("{col_sql} > {placeholder}"),
                other => {
                    return Err(RepositoryError::Conversion(format!(
                        "unsupported date filter type: {other}"
                    )));
                }
            }
        }
    })
}

/// Builds and runs the AG Grid Server-Side Row Model query: parameterized
/// WHERE from filterModel, validated ORDER BY from sortModel, and OFFSET/FETCH
/// pagination from startRow/endRow.
pub fn get_rows(
    conn: &Connection,
    request: &GetRowsRequest,
) -> Result<GetRowsResponse<StockHistoryRow>, RepositoryError> {
    let mut binds: Vec<BindValue> = Vec::new();
    let mut where_clauses: Vec<String> = Vec::new();

    for (col_id, filter) in &request.filter_model {
        let col = column_def(col_id).ok_or_else(|| {
            RepositoryError::Conversion(format!("unknown filter column: {col_id}"))
        })?;
        let predicate = match (col.kind, filter) {
            (ColumnKind::Text, FilterModel::Text(f)) => text_predicate(col.sql_name, f, &mut binds)?,
            (ColumnKind::Number, FilterModel::Number(f)) => {
                number_predicate(col.sql_name, f, &mut binds)?
            }
            (ColumnKind::Date, FilterModel::Date(f)) => date_predicate(col.sql_name, f, &mut binds)?,
            _ => {
                return Err(RepositoryError::Conversion(format!(
                    "filter type does not match column: {col_id}"
                )));
            }
        };
        where_clauses.push(predicate);
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    let mut order_by_parts = Vec::new();
    for sort in &request.sort_model {
        let col = column_def(&sort.col_id).ok_or_else(|| {
            RepositoryError::Conversion(format!("unknown sort column: {}", sort.col_id))
        })?;
        let direction = match sort.sort.to_lowercase().as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            other => {
                return Err(RepositoryError::Conversion(format!(
                    "unsupported sort direction: {other}"
                )));
            }
        };
        order_by_parts.push(format!("{} {direction}", col.sql_name));
    }
    // Stable tiebreaker so OFFSET/FETCH pagination doesn't skip or repeat rows.
    order_by_parts.push("\"DATE\" DESC".to_string());
    order_by_parts.push("TICKER ASC".to_string());
    let order_sql = format!(" ORDER BY {}", order_by_parts.join(", "));

    let filter_binds: Vec<&dyn ToSql> = binds.iter().map(|b| b as &dyn ToSql).collect();

    let count_sql = format!("SELECT COUNT(*) FROM STOCK_HISTORY{where_sql}");
    let total_row: i64 = conn.query_row(&count_sql, &filter_binds)?.get(0)?;

    // start_row/end_row are typed i64 from JSON, so inlining them is not an
    // injection risk the way string filter values would be.
    let start_row = request.start_row.max(0);
    let fetch_count = (request.end_row - start_row).max(0);
    let page_sql = format!(
        "SELECT {SELECT_COLUMNS} FROM STOCK_HISTORY{where_sql}{order_sql} \
         OFFSET {start_row} ROWS FETCH NEXT {fetch_count} ROWS ONLY"
    );

    let mut row_data = Vec::new();
    for row_result in conn.query(&page_sql, &filter_binds)? {
        row_data.push(row_to_stock_history(&row_result?)?);
    }

    Ok(GetRowsResponse {
        row_data,
        row_count: total_row,
    })
}

