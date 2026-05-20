# 37. Middleware：tower vs actix

> 範圍：tower Service/Layer、`middleware::from_fn`、tower-http、與 actix middleware 對比

## tower：Service trait

tower 的核心抽象：

```rust
trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

**任何 request → response 的東西都是 Service**。Handler 是 Service、Middleware 也是 Service（包另一個 Service）。

## Layer：建構 Service 的工廠

```rust
trait Layer<S> {
    type Service;
    fn layer(&self, inner: S) -> Self::Service;
}
```

`ServiceBuilder::new().layer(A).layer(B).layer(C).service(handler)` 等同 `A(B(C(handler)))`，最外的 A 最先看到 request。

## axum 三種寫 middleware 的方法

### 1. `middleware::from_fn` — 函式風格（推薦）

```rust
async fn timing(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let resp = next.run(req).await;
    info!(elapsed = ?start.elapsed());
    resp
}
Router::new().layer(middleware::from_fn(timing));
```

可以提早回應（短路）：

```rust
async fn auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    if req.headers().get("authorization").is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}
```

### 2. tower-http 現成 layer

| Layer | 用途 |
|-------|------|
| `TraceLayer` | tracing span + log |
| `TimeoutLayer` | request timeout |
| `CompressionLayer` | gzip/br 壓縮 |
| `CorsLayer` | CORS |
| `RequestBodyLimitLayer` | body 大小上限 |
| `SetResponseHeaderLayer` | 加 header |
| `NormalizePathLayer` | trailing slash |

### 3. 自己 impl `Service` + `Layer`

需要 stateful、有 init logic 才用。寫法繁瑣，先嘗試 `from_fn`。

## ServiceBuilder：避免反向

```rust
let stack = ServiceBuilder::new()
    .layer(TraceLayer::new_for_http())     // 1. 最外層
    .layer(TimeoutLayer::new(...))         // 2.
    .layer(CompressionLayer::new());       // 3. 最內

Router::new().layer(stack);
```

ServiceBuilder 「順序就是 wrap 順序」——比直接 chain `.layer()` 更直覺。

## actix-web 對比

| | tower / axum | actix-web |
|---|--------------|-----------|
| 簡單 middleware | `middleware::from_fn(f)` | `actix_web_lab::from_fn(f)` |
| 完整 middleware | `impl Service + impl Layer` | `impl Transform + impl Service` |
| 現成集合 | `tower-http`（大且通用） | `actix-web-*` 系列散落 |
| 與 hyper / tonic 共用 | ✅ | ❌ |

關鍵差異：**tower 是跨框架抽象**，同一個 Layer 可以給 axum、tonic、hyper client、reqwest 用。actix 的 middleware 只能 actix 用。

## 局部 middleware（特定路由）

```rust
let private = Router::new()
    .route("/me", get(me))
    .layer(middleware::from_fn(auth));   // 只套到 /me

let app = Router::new()
    .route("/", get(public))             // 不要 auth
    .nest("/private", private);
```

⚠️ **`.layer()` 的順序敏感**：先 `.route()` 再 `.layer()`，layer 才會套到那些 route。`.layer()` 後再 `.route()` 不會被 wrap。

## 常見 middleware pattern

### Request ID

req 進來生 UUID，塞 header + log context，回 response 一起帶。

### Auth

驗 JWT → 把 user info 用 `req.extensions_mut().insert(user)` 塞進 request，handler 用 `Extension<User>` 取。

### Rate limit

`tower::ServiceBuilder` + `tower::limit::RateLimitLayer`（每 IP 用 governor crate）。

### Retry

對外呼叫的 client 端用 `tower::retry::RetryLayer`。注意：對「server 端 retry」不該套（client 才知道 idempotent）。

## 常見陷阱

1. **`.layer()` 在 route 之後** — 沒套到那個 route。
2. **`from_fn` 多參數版本** — 要拿 state 用 `from_fn_with_state`。
3. **middleware 改 body** — body 是 stream，要先 `to_bytes` 再重組；非小心會破壞 streaming。
4. **timeout = 完整 response time** — 不是 idle timeout，慢 client 也會被砍。
5. **CORS 的 preflight** — OPTIONS 沒 body，要 CorsLayer 處理而不是手寫 if。
6. **tower 0.4 vs 0.5** — API 有調整，看版本對齊。

## 練習

1. 寫一個 `RequestId` middleware：req 沒帶就生新的、有帶就保留。
2. 用 `RateLimitLayer` + `governor` 做 per-IP rate limit。
3. 寫一個 metrics middleware：每 endpoint 統計 P50/P95（用 `metrics` crate）。
4. 比較同一個 logging middleware 在 axum 跟 actix 的程式碼複雜度。

## 延伸閱讀

- [tower](https://docs.rs/tower)
- [tower-http](https://docs.rs/tower-http)
- [axum middleware docs](https://docs.rs/axum/latest/axum/middleware/)
