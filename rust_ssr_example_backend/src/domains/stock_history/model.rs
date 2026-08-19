use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct StockHistoryRow {
    pub date: NaiveDate,
    pub ticker: String,
    #[schema(value_type = String)]
    pub open: BigDecimal,
    #[schema(value_type = String)]
    pub high: BigDecimal,
    #[schema(value_type = String)]
    pub low: BigDecimal,
    #[schema(value_type = String)]
    pub close: BigDecimal,
    pub volume: u64,
}
