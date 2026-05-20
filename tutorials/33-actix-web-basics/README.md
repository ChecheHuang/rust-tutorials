# 33. actix-web 入門

> 範圍：App、HttpServer、web::Path / web::Json、Responder trait

## actix-web 是什麼

- Rust web 早期王者，目前仍主流之一
- 自己的 runtime（actix-rt，基於 tokio），不完全等同 tokio
- attribute macro 註冊 route：`#[get("/users")]`、`#[post(...)]`
- benchmark 上常進前 5（TechEmpower）

跟 axum 的關鍵差別：actix 用 **macro 註冊路由**，axum 用 **Router builder**。

## 對照第 32 章

兩章寫**完全相同的 user CRUD**：endpoint、行為、錯誤碼一樣。差別只在 framework API。

## 對照表

| | actix-web | axum 對應 |
|---|-----------|-----------|
| Server | `HttpServer::new(\|\| App::new()...)` | `axum::serve(listener, app)` |
| Router | `App::new().service(handler)` | `Router::new().route(...)` |
| Handler | `#[get("/")] async fn h() -> impl Responder` | `async fn h() -> impl IntoResponse` |
| Path | `Path<u64>` | `Path<u64>` |
| Query | `Query<T>` | `Query<T>` |
| Body JSON | `Json<T>` | `Json<T>` |
| State | `Data<T>`（透過 `app_data`） | `State<T>`（透過 `with_state`） |
| Response | `impl Responder` | `impl IntoResponse` |
| Middleware | `App::wrap(middleware)` | `Router::layer(layer)` |

API 形狀**極相似**，主要差在註冊機制與生態整合。

## 三種註冊路由方式

### 1. attribute macro（idiomatic）

```rust
#[get("/users")]
async fn list_users(...) -> impl Responder { ... }

App::new().service(list_users)
```

### 2. `web::resource`

```rust
App::new().service(
    web::resource("/users")
        .route(web::get().to(list_users))
        .route(web::post().to(create_user))
)
```

### 3. scope（巢狀）

```rust
App::new().service(
    web::scope("/api/v1")
        .service(list_users)
        .service(create_user)
)
```

## Extractors

```rust
async fn h(
    id: web::Path<u64>,
    q: web::Query<Params>,
    s: web::Data<AppState>,
    payload: web::Json<Body>,
) -> impl Responder { ... }
```

細節：
- 拿值要 `id.into_inner()` 或 deref
- `Data<T>` 內包 `Arc<T>`（不必自己 Arc）
- `Json` 預設 limit 256K，超過回 400；要更大設 `JsonConfig::new().limit(N)`

## 自訂錯誤 — `ResponseError`

```rust
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("not found")]
    NotFound,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode { ... }
    fn error_response(&self) -> HttpResponse { ... }
}
```

handler 回 `Result<T, AppError>`，自動轉成 HTTP response。

## State 共享

```rust
let state = Data::new(AppState { pool });
HttpServer::new(move || {
    App::new()
        .app_data(state.clone())
        .service(handler)
})
```

⚠️ **多 worker 時**：actix 預設啟動 CPU 核數個 worker，**每個 worker 各自跑 closure**。`Data::new` 出來的 `Arc<T>` 一定要在 closure **外**建好、closure 內 clone Arc——不然每個 worker 各自一份 state，無法共享。

## Middleware

```rust
App::new()
    .wrap(tracing_actix_web::TracingLogger::default())
    .wrap(actix_cors::Cors::permissive())
    .service(handler)
```

`wrap` 順序：**最後 wrap 最先進來**（request 流向）。

寫自己的 middleware 較複雜——要實作 `Transform` + `Service`。實務上多用現成的或 `actix-web-lab::from_fn`。詳見第 37 章對比。

## 跟 axum 怎麼選

| 想要 | 用 |
|------|---|
| 跟 tokio 生態緊密整合（tonic / hyper） | axum |
| tower middleware ecosystem | axum |
| attribute macro 風格 | actix-web |
| benchmark 最強 | actix-web（差距漸小） |
| 文件 / 範例量多 | actix-web（成熟） |
| 學習新框架 | axum（架構更現代） |
| 已用 actix 多年 | 沿用 |

實務建議：**新專案優先評估 axum**，除非團隊已有 actix 經驗或特定需求。

## 常見陷阱

1. **`HttpServer::new` 的 closure 每 worker 跑一次** — state 必須是 Arc clone，不能在 closure 內 `new`。
2. **`JsonConfig` limit** — 預設 256K 太小，大 payload 會 413。
3. **actix runtime ≠ tokio** — 直接 `tokio::spawn` 在 actix-rt 內 corner case；穩定做法 `actix_rt::spawn`。
4. **`#[actix_web::main]` 不是 `#[tokio::main]`** — 各自啟動不同 runtime。
5. **錯誤訊息直接洩漏內部** — `ResponseError` 預設用 `Display`，要小心別把 SQL 錯誤吐到 client。

## 練習

1. 把第 32 章的 curl 指令拿來測，行為應與 axum 版完全一致。
2. 加 PATCH endpoint（actix 沒 `#[patch]` macro，用 `web::resource(...).route(web::patch().to(...))`）。
3. 加 `actix-cors` 與 `actix-governor`（rate limit）。
4. 比較自訂 middleware 在兩個 framework 的複雜度差異。

## 延伸閱讀

- [actix-web docs](https://actix.rs/docs)
- [actix-web examples](https://github.com/actix/examples)
