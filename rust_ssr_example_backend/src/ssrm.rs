//! AG Grid Server-Side Row Model request/response types, shared across domains.
//! See https://www.ag-grid.com/react-data-grid/server-side-model-datasource/
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRowsRequest {
    pub start_row: i64,
    pub end_row: i64,
    #[serde(default)]
    pub sort_model: Vec<SortModelItem>,
    #[serde(default)]
    pub filter_model: HashMap<String, FilterModel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortModelItem {
    pub col_id: String,
    pub sort: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "filterType", rename_all = "lowercase")]
pub enum FilterModel {
    Text(TextFilterModel),
    Number(NumberFilterModel),
    Date(DateFilterModel),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFilterModel {
    #[serde(rename = "type")]
    pub filter_op: String,
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberFilterModel {
    #[serde(rename = "type")]
    pub filter_op: String,
    pub filter: Option<f64>,
    pub filter_to: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateFilterModel {
    #[serde(rename = "type")]
    pub filter_op: String,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRowsResponse<T> {
    pub row_data: Vec<T>,
    pub row_count: i64,
}
