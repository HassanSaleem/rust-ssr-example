mod domains;

use std::sync::Arc;

use axum::Router;
use oracle::pool::PoolBuilder;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use domains::stock_history::StockHistoryRow;

#[derive(OpenApi)]
#[openapi(
    paths(domains::stock_history::routes::list_stock_history),
    components(schemas(StockHistoryRow))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let username = std::env::var("ORACLE_USERNAME").expect("ORACLE_USERNAME must be set");
    let password = std::env::var("ORACLE_PASSWORD").expect("ORACLE_PASSWORD must be set");
    let connect_string =
        std::env::var("ORACLE_CONNECT_STRING").expect("ORACLE_CONNECT_STRING must be set");

    // PoolBuilder::build() is blocking, so it must not run on the async runtime thread.
    let pool = tokio::task::spawn_blocking(move || {
        PoolBuilder::new(username, password, connect_string).build()
    })
    .await
    .expect("pool creation task panicked")
    .expect("failed to create Oracle connection pool");
    let pool = Arc::new(pool);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(domains::stock_history::routes::router())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("server error");
}
