# 32. axum 入門

> 範圍：Router、handler、extractor（Path / Query / Json / State）、IntoResponse

## axum 是什麼

- 建立在 **hyper + tokio + tower** 上
- 哲學：**用 Rust 型別系統定義 HTTP API**——extractor / response 都是型別
- 沒有 macro 魔法（少數 derive 例外），純函式 + trait
- Tokio 官方系列的一員，跟 tower（middleware）整合最緊

最近 3 年逐漸成為 Rust web framework 的事實標準。

## 三個核心概念

| 概念 | 是什麼 |
|------|--------|
| **Router** | path → handler 的對應表 |
| **Handler** | `async fn(...) -> impl IntoResponse` |
| **Extractor** | 從 request 抽出資料的型別（Path、Query、Json、State 等） |

## 最小 server

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Routing

```rust
Router::new()
    .route("/", get(root))
    .route("/users", get(list_users).post(create_user))
    .route("/users/:id", get(get_user).delete(delete_user))
    .nest("/api", api_routes())                // 巢狀
    .route("/static/*path", get(static_handler)) // wildcard
```

## Extractors — 從 request 拿資料

handler 的參數型別 = extractor。axum 看簽章決定怎麼拆 request：

| Extractor | 來源 |
|-----------|------|
| `Path<T>` | URL path param（`/users/:id`） |
| `Query<T>` | URL query string |
| `Json<T>` | body 反序列化（`application/json`） |
| `Form<T>` | x-www-form-urlencoded |
| `Bytes` / `String` | raw body |
| `HeaderMap` | 所有 headers |
| `State<AppState>` | 共用 state（透過 `.with_state`） |
| `Extension<T>` | request extension（middleware 注入） |

```rust
async fn handler(
    Path(id): Path<u64>,
    Query(p): Query<MyParams>,
    State(s): State<AppState>,
    Json(body): Json<MyPayload>,
) -> impl IntoResponse { ... }
```

順序自由，axum 看型別決定。

⚠️ **吃 body 的 extractor 只能有一個，且必須最後**：

```rust
async fn h(State(s): State<S>, Json(b): Json<B>) { ... }    // OK
async fn h(Json(b): Json<B>, State(s): State<S>) { ... }    // ❌
```

## Responses — IntoResponse

| 回傳 | 變成 |
|------|------|
| `&'static str` / `String` | 200 + `text/plain` |
| `Json<T>` | 200 + `application/json` |
| `Html<T>` | 200 + `text/html` |
| `(StatusCode, T)` | 自訂 status + body |
| `Result<T, E>` | Ok 走 T、Err 走 E 的 IntoResponse |
| `StatusCode` | 該 status，空 body |

```rust
async fn create(...) -> impl IntoResponse {
    (StatusCode::CREATED, Json(user))
}

async fn maybe() -> Result<Json<User>, StatusCode> {
    db.find(id).map(Json).ok_or(StatusCode::NOT_FOUND)
}
```

## State — 共用資源

```rust
#[derive(Clone)]
struct AppState { pool: PgPool }

let app = Router::new()
    .route("/users", get(list_users))
    .with_state(AppState { pool });

async fn list_users(State(s): State<AppState>) -> ... {
    let users = sqlx::query_as!(...).fetch_all(&s.pool).await?;
    ...
}
```

`AppState` 必須 `Clone`；通常用 `Arc<Inner>` 包重物。

## 自訂錯誤 → IntoResponse

```rust
enum AppError { NotFound, BadInput(String), Internal }

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::NotFound    => (StatusCode::NOT_FOUND,            "not found"),
            AppError::BadInput(_) => (StatusCode::BAD_REQUEST,          "bad input"),
            AppError::Internal    => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
```

handler 用 `Result<T, AppError>` 就能讓錯誤自動變漂亮 HTTP response。

## Middleware — tower Layer

axum 用 `tower::Layer` 系統：

```rust
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

let app = Router::new()
    .route("/", get(root))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive());
```

執行順序：**最後 layer 最先包**——request 從外層流入、response 從內層流出。

詳見第 37 章。

## Static files

```rust
use tower_http::services::ServeDir;
Router::new().nest_service("/static", ServeDir::new("public"));
```

## 跟 actix-web 對比

| | axum | actix-web |
|---|---|---|
| 模型 | tower Service | actor + system |
| 錯誤 | `IntoResponse` trait | `ResponseError` trait |
| state | `with_state` + `State<T>` | `app_data` + `Data<T>` |
| middleware | tower `Layer` | actix middleware |
| async runtime | tokio | actix-rt（基於 tokio） |
| 生態整合 | tonic / tower / hyper | 自有 |

下一章 actix-web 寫**同樣的 user CRUD**，可同步對照。

## 常見陷阱

1. **handler 簽章對不上 trait** — error 訊息又臭又長；通常 extractor 順序錯或型別漏 `Json` wrapper。
2. **state 非 Clone** — `with_state` 要 `S: Clone`，所以一般 `Arc<Inner>`。
3. **多個 body extractor** — axum 編譯期擋。
4. **async handler 沒 await** — compile 過但漏 IO。
5. **panic in handler** — 預設沒包，會殺 connection；加 `CatchPanicLayer`。

## 練習

1. 加 `PATCH /users/:id`：接 `Json<Partial>` 更新欄位。
2. 加 middleware log `method path status duration`（用 `TraceLayer`）。
3. 替換 `Mutex<HashMap>` 為 `tokio::sync::RwLock<HashMap>` 比較鎖行為。
4. 加 health check `/healthz` 與 readiness `/ready`。

## 延伸閱讀

- [axum 0.7 examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [tower-http](https://docs.rs/tower-http/)
- [Why axum?](https://tokio.rs/blog/2021-07-announcing-axum)
