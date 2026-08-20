use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use oracle::pool::Pool;

use crate::ssrm::{GetRowsRequest, GetRowsResponse};

use super::{model::StockHistoryRow, repository};

pub type DbPool = Arc<Pool>;

pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/api/stock-history", get(list_stock_history))
        .route("/api/stock-history/getRows", post(get_rows))
}

#[utoipa::path(
    get,
    path = "/api/stock-history",
    responses(
        (status = 200, description = "List stock history rows", body = [StockHistoryRow]),
        (status = 500, description = "Database error")
    )
)]
pub async fn list_stock_history(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<StockHistoryRow>>, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || -> Result<Vec<StockHistoryRow>, repository::RepositoryError> {
        let conn = pool.get()?;
        repository::list_stock_history(&conn)
    })
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("task error: {err}")))?
    .map(Json)
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

/// AG Grid Server-Side Row Model datasource endpoint: infinite scroll, sort
/// and filter. See https://www.ag-grid.com/react-data-grid/server-side-model-datasource/
#[utoipa::path(
    post,
    path = "/api/stock-history/getRows",
    request_body = GetRowsRequest,
    responses(
        (status = 200, description = "Server-side rows for AG Grid", body = StockHistoryGetRowsResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Database error")
    )
)]
pub async fn get_rows(
    State(pool): State<DbPool>,
    Json(request): Json<GetRowsRequest>,
) -> Result<Json<GetRowsResponse<StockHistoryRow>>, (StatusCode, String)> {
    tokio::task::spawn_blocking(
        move || -> Result<GetRowsResponse<StockHistoryRow>, repository::RepositoryError> {
            let conn = pool.get()?;
            repository::get_rows(&conn, &request)
        },
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("task error: {err}")))?
    .map(Json)
    .map_err(|err| {
        let status = if err.is_client_error() {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, err.to_string())
    })
}
