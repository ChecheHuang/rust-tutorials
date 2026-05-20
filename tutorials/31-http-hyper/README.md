# 31. HTTP 基礎與 hyper

> 範圍：hyper 1.x client/server、reqwest、HTTP/1 vs HTTP/2

## 層次圖

```
Application code
       ↓
   reqwest (high-level client)
       ↓
   hyper (low-level HTTP)
       ↓
   h2 / h3 (protocol implementations)
       ↓
   tokio (runtime + tcp)
```

寫服務一般不直接碰 hyper——透過 axum / actix（後面章節）/ reqwest 已涵蓋 99% 場景。但 hyper 是底層基礎，**整個 Rust web 生態都建立在它上面**。

## reqwest — high-level HTTP client

最常用的 HTTP client。本章 main.rs 主要展示它。

### 基本 GET

```rust
let resp = reqwest::get("https://api.example.com/users").await?;
let body = resp.text().await?;
```

不必建 client 也行（fire-and-forget）。但 production prefer 建一個 client 重用：

```rust
let client = reqwest::Client::builder()
    .user_agent("my-app/1.0")
    .timeout(Duration::from_secs(10))
    .build()?;
```

複用 client 才有 connection pool、keep-alive、HTTP/2 multiplexing。

### JSON

```rust
#[derive(Deserialize)]
struct Post { id: u32, title: String }

let p: Post = client
    .get("https://api.example.com/posts/1")
    .send().await?
    .error_for_status()?     // status >= 400 → Err
    .json::<Post>().await?;
```

`.json()` 同時 `Deserialize`。POST 用 `.json(&body)`：

```rust
client.post(url).json(&new_post).send().await?;
```

### Headers / Query

```rust
client.get(url)
    .header("Authorization", "Bearer xxx")
    .query(&[("limit", "10"), ("offset", "0")])
    .send().await?;
```

### Form

```rust
client.post(url)
    .form(&[("user", "alice"), ("pass", "secret")])
    .send().await?;
```

### 連線池調校

```rust
reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .connect_timeout(Duration::from_secs(5))
    .build()?;
```

### TLS backend

- `default-tls` (預設) — 用系統的 TLS（OpenSSL on Linux）
- `rustls-tls` — 純 Rust（推薦，跨平台一致）
- `native-tls` — 各 OS 原生

production prefer `rustls-tls`：

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

## hyper — 寫底層 HTTP server

絕大多數情況用 axum（建立在 hyper 上）。直接寫 hyper 適合：
- 寫 HTTP framework
- 極端 performance 需求
- 學習用

```rust
use hyper::{server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn hello(_req: Request<...>) -> Result<Response<...>, ...> {
    Ok(Response::new("Hello".into()))
}

#[tokio::main]
async fn main() -> ... {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::spawn(async move {
            http1::Builder::new()
                .serve_connection(io, service_fn(hello))
                .await
        });
    }
}
```

寫多了你會懂為什麼 axum 存在。

## HTTP/1 vs HTTP/2 vs HTTP/3

| 版本 | 特點 | 何時用 |
|------|------|--------|
| HTTP/1.1 | 文字協定、keep-alive、pipelining 弱 | 簡單、廣泛 |
| HTTP/2 | 二進位、multiplexing、server push | 高並發 API、gRPC |
| HTTP/3 | over QUIC（UDP）、抗封包遺失 | 行動網路、CDN |

reqwest 自動協商。要強制：

```rust
.http2_prior_knowledge()
.http2_keep_alive_interval(Duration::from_secs(30))
```

HTTP/3 在 Rust 生態還在 stabilize，`reqwest` 透過 `quinn` 提供基礎支援。

## reqwest 的錯誤處理

```rust
match client.get(url).send().await {
    Ok(resp) => match resp.error_for_status() {
        Ok(resp) => /* 2xx */,
        Err(e) => /* 4xx/5xx */,
    },
    Err(e) if e.is_timeout() => /* timeout */,
    Err(e) if e.is_connect() => /* DNS / connect 失敗 */,
    Err(e) => /* 其他 */,
}
```

`reqwest::Error` 有 `is_timeout`、`is_connect`、`is_decode` 等判斷 method。

## Streaming response（不一次讀進來）

```rust
let mut resp = client.get(big_file_url).send().await?;
let mut file = tokio::fs::File::create("out.bin").await?;
while let Some(chunk) = resp.chunk().await? {
    file.write_all(&chunk).await?;
}
```

下載大檔不爆記憶體的方式。或者用 `bytes_stream()` 配合 `tokio::io::copy`。

## Middleware：reqwest-middleware

```toml
reqwest-middleware = "0.3"
reqwest-retry = "0.5"
```

```rust
let client = ClientBuilder::new(reqwest::Client::new())
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(3)
    ))
    .build();
```

統一 retry、tracing、cache。

## 常見陷阱

1. **沒重用 Client** — 每次 `reqwest::get()` 建新 client，無 pool、無 keep-alive，**慢 10x+**。
2. **沒 `error_for_status`** — `send()` 拿到 4xx/5xx 仍是 Ok。漏這個就把錯當好回應 deserialize。
3. **同步 reqwest vs async** — 預設 async，要 sync 用 `reqwest::blocking::Client`（**別**在 async runtime 內用）。
4. **`tokio-rustls` vs `rustls`** — 直接用 `rustls` features 名，reqwest 會處理。
5. **timeout 不只一個** — connect / read / total，各設不同。

## 練習

1. 寫 `fetch_user(id) -> Result<User>`，配 timeout 5 秒、重試 3 次。
2. 用 `join_all` 同時拿 50 個 user，量總時間 vs sequential。
3. 寫 download 函式：streaming 寫檔，每 1MB 印一次進度（提示：`chunk()`）。
4. 用 hyper 寫一個 echo server（直接回 request body）。

## 延伸閱讀

- [reqwest 文件](https://docs.rs/reqwest/)
- [hyper 文件](https://docs.rs/hyper/)
- [hyper 1.0 migration guide](https://hyper.rs/guides/1/upgrading/)
