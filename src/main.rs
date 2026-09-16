mod cypher; mod error; mod hydra; mod http;
use axum::{routing::{get, post}, Router};
use http::S;

#[tokio::main]
async fn main() {
    let hydra = hydra::Hydra::from_env();
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let app = Router::new()
        .route("/healthz", get(http::healthz))
        .route("/v1/decisions", post(http::create))
        .route("/v1/agents/{agent}/decisions", get(http::list))
        .route("/v1/decisions/{id}/lineage", get(http::lineage))
        .with_state(S { hydra });
    let l = tokio::net::TcpListener::bind(&bind).await.unwrap();
    eprintln!("http://{bind}");
    axum::serve(l, app).await.unwrap();
}