mod http;

use axum::{routing::{get, post}, Router};
use http::create;
#[tokio::main]
async fn main() {
    let app = Router::new()
    .route("/healthz", get(healthz))
    .route("/v1/decisions", post(create));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> &'static str {
    "ok"
}