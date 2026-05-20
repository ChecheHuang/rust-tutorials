use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

// ── 自訂 axum middleware：函式風格 ───────────────────────────
async fn request_id(req: Request, next: Next) -> Response {
    let id = uuid_like();
    let mut req = req;
    req.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&id).unwrap(),
    );
    let mut resp = next.run(req).await;
    resp.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&id).unwrap(),
    );
    resp
}

async fn auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    if req.headers().get(header::AUTHORIZATION).is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

async fn timing(req: Request, next: Next) -> Response {
    let start = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let resp = next.run(req).await;
    let elapsed = start.elapsed();
    info!(?method, %uri, status = ?resp.status(), ?elapsed, "request");
    resp
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!(
        "req-{:x}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    )
}

async fn hello() -> &'static str {
    "Hello, middleware!"
}

async fn slow() -> impl IntoResponse {
    tokio::time::sleep(Duration::from_secs(2)).await;
    "slowly done"
}

async fn echo(body: Body) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(body, 1024 * 1024).await.unwrap_or_default();
    bytes
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 應用層 middleware：自訂的 timing / request_id
    // tower-http：常見橫切關注
    // 注意：tower-http 0.6 + axum 0.7 同時堆 RequestBodyLimit + Compression
    // 會在 body type 推導上踩到 Default 限制，這裡示範常見組合即可。
    // 限 body 大小用 axum 的 DefaultBodyLimit (extractor) 比較順。
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(1)))      // slow → 408
        .layer(CompressionLayer::new())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

    // /private 才需要 auth
    let private = Router::new()
        .route("/me", get(|| async { "secret" }))
        .layer(middleware::from_fn(auth));

    let app = Router::new()
        .route("/", get(hello))
        .route("/slow", get(slow))
        .route("/echo", get(echo))
        .nest("/private", private)
        .layer(middleware::from_fn(timing))
        .layer(middleware::from_fn(request_id))
        .layer(middleware);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("listening on 3000");
    println!("\n試試：");
    println!("  curl -i localhost:3000/             # 看 x-request-id");
    println!("  curl -i localhost:3000/slow         # 超過 timeout → 408");
    println!("  curl -i localhost:3000/private/me   # 401");
    println!("  curl -i -H 'authorization: x' localhost:3000/private/me   # OK");
    axum::serve(listener, app).await.unwrap();
}
