use axum::{extract::Request, middleware::Next, response::Response, routing::get, Router};
use metrics::{counter, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Instant;
use tracing::{info, instrument};

#[instrument]
async fn root() -> &'static str {
    info!("hit /");
    counter!("http_requests_total", "route" => "/").increment(1);
    "hello"
}

#[instrument]
async fn slow() -> &'static str {
    info!("hit /slow");
    counter!("http_requests_total", "route" => "/slow").increment(1);
    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    "slow"
}

async fn timing(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().path().to_string();
    let resp = next.run(req).await;
    let elapsed = start.elapsed().as_secs_f64();
    histogram!("http_request_duration_seconds",
        "method" => method.as_str().to_string(),
        "path"   => uri.clone()
    ).record(elapsed);
    resp
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Prometheus exporter — render() 給 /metrics 用
    let recorder = PrometheusBuilder::new().install_recorder()?;

    let app = Router::new()
        .route("/", get(root))
        .route("/slow", get(slow))
        .route("/metrics", get(move || {
            let r = recorder.clone();
            async move { r.render() }
        }))
        .layer(axum::middleware::from_fn(timing));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("listening on :8080");
    println!("\n試試：");
    println!("  curl localhost:8080/");
    println!("  curl localhost:8080/slow");
    println!("  curl localhost:8080/metrics  # Prometheus 格式");
    axum::serve(listener, app).await?;
    Ok(())
}
