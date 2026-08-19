use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use oracle::pool::Pool;

use super::{model::StockHistoryRow, repository};

pub type DbPool = Arc<Pool>;

pub fn router() -> Router<DbPool> {
    Router::new().route("/api/stock-history", get(list_stock_history))
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
