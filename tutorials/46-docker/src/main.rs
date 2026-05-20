// 最小的 axum server，給 Docker image build / 部署示範用。

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;

async fn root() -> &'static str {
    "Hello from inside Docker!"
}

async fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
