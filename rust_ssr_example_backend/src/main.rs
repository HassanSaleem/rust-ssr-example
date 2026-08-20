mod domains;
mod ssrm;

use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use axum::Router;
use oracle::pool::PoolBuilder;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use domains::stock_history::StockHistoryRow;
use ssrm::{DateFilterModel, FilterModel, GetRowsRequest, NumberFilterModel, SortModelItem, StockHistoryGetRowsResponse, TextFilterModel};

#[derive(OpenApi)]
#[openapi(
    paths(
        domains::stock_history::routes::list_stock_history,
        domains::stock_history::routes::get_rows,
    ),
    components(schemas(
        StockHistoryRow,
        GetRowsRequest,
        SortModelItem,
        FilterModel,
        TextFilterModel,
        NumberFilterModel,
        DateFilterModel,
        StockHistoryGetRowsResponse,
    ))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // `cargo run -- --export-openapi [path]` writes the spec without needing a DB connection.
    let mut args = std::env::args().skip(1);
    if let Some(arg) = args.next() {
        if arg == "--export-openapi" {
            let yaml = ApiDoc::openapi()
                .to_yaml()
                .expect("failed to serialize OpenAPI spec to YAML");
            match args.next() {
                Some(path) => std::fs::write(&path, yaml).expect("failed to write OpenAPI file"),
                None => print!("{yaml}"),
            }
            return;
        }
    }

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

    let frontend_origin =
        std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let cors = CorsLayer::new()
        .allow_origin(
            frontend_origin
                .parse::<HeaderValue>()
                .expect("FRONTEND_ORIGIN must be a valid origin"),
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(domains::stock_history::routes::router())
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("failed to bind to port 8000");
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("server error");
}
