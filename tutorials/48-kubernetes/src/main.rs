// Demo app：含 readiness、liveness、graceful shutdown，
// 給 Kubernetes 部署當範例。

use axum::{routing::get, Router};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tracing::info;

#[derive(Clone)]
struct State {
    ready: Arc<AtomicBool>,
}

async fn root() -> &'static str { "ch48 demo" }

async fn healthz() -> &'static str { "alive" }                  // liveness

async fn readyz(s: axum::extract::State<State>) -> (axum::http::StatusCode, &'static str) {
    if s.ready.load(Ordering::Relaxed) {
        (axum::http::StatusCode::OK, "ready")
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}

async fn shutdown(state: State) {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("install ctrl+c");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("ctrl+c received"),
        _ = terminate => info!("SIGTERM received"),
    }

    // 先停接新流量，再等舊請求結束
    state.ready.store(false, Ordering::Relaxed);
    info!("readyz off — waiting for in-flight requests to finish");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    info!("shutting down");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = State { ready: Arc::new(AtomicBool::new(false)) };

    // 模擬 cold start：DB connect、cache warmup...
    let warmup_state = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        warmup_state.ready.store(true, Ordering::Relaxed);
        info!("warm-up done → readyz=ok");
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("listening on :8080");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown(state))
        .await
        .unwrap();
}
