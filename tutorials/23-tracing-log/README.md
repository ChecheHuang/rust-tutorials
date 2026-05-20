# 23. 結構化日誌（tracing）

> 範圍：tracing、tracing-subscriber、span、event、結構化欄位、與 OTel 接合預備

## 為什麼不是 `println!` / `log` crate

`println!` 沒等級、沒過濾、沒結構。`log` crate 有等級但**沒結構**——還是純字串。

`tracing` 兩個關鍵升級：

1. **結構化欄位**：`info!(user = "alice", status = 200, "request done")` 而非 `format!("user={alice} status={200}")`。輸出可給 Datadog / Loki 直接 query。
2. **Span**：一段事件範圍的概念，能跨函式、跨 async task 傳遞 context。

事實上：**現代 Rust 服務應用一律用 `tracing`**，不要新寫 `log`。

## 三個概念

| 概念 | 意思 |
|------|------|
| **Event** | 一個瞬間發生的事（log 那一行） |
| **Span** | 一段時間 / 範圍（請求、交易、batch） |
| **Subscriber** | 接收 event / span 並決定如何輸出（terminal / JSON / OTel） |

## 基本使用

```rust
use tracing::{info, warn, error, debug, trace};

info!("started");
info!(user = "alice", "logged in");
info!(method = "GET", path = "/users", status = 200, "request handled");
```

`tracing::info!` 的 syntax：

```
info!(field1 = val, field2 = val, "message");
```

- `field = val`：結構化欄位（`val` 可任意 `Debug`/`Display`）
- 訊息結尾字串

格式化提示：

| 寫法 | 行為 |
|------|------|
| `field = value` | `value` 必須 `Display` 或 `Debug` |
| `?value` | 強制用 `Debug` 印 |
| `%value` | 強制用 `Display` 印 |
| `field` | shorthand 等同 `field = field` |

```rust
info!(user, %url, ?body, "request");
// = info!(user = user, url = %url, body = ?body, "request");
```

## Subscriber — 接收與輸出

```rust
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_target(true)
    .with_line_number(true)
    .init();
```

`tracing_subscriber::fmt` 是預設的 terminal output。常見設定：

| Method | 用途 |
|--------|------|
| `.with_env_filter(filter)` | 依環境變數 `RUST_LOG` 過濾 |
| `.with_target(bool)` | 是否印 module path |
| `.with_line_number(bool)` | 是否印行號 |
| `.with_ansi(false)` | 關掉顏色 |
| `.json()` | JSON 格式輸出（給 Loki / Datadog） |
| `.compact()` / `.pretty()` | 排版風格 |

### RUST_LOG 過濾

```bash
RUST_LOG=info cargo run                    # 全部 info+
RUST_LOG=debug cargo run                   # 全部 debug+
RUST_LOG=ch23=trace cargo run              # 特定 crate
RUST_LOG=warn,ch23=debug cargo run         # 預設 warn，本 crate debug
RUST_LOG=hyper=off,info cargo run          # 關掉 hyper
```

Level 由低到高：`trace < debug < info < warn < error`。Production 預設 `info`。

## Span — 範圍 context

```rust
let span = tracing::info_span!("processing", request_id = "abc123");
let _enter = span.enter();
info!("inside span");          // 自動帶上 request_id
fetch_user(7);
drop(_enter);
```

span 可巢狀。一個 event 會繼承所有上層 span 的欄位——這對追 request 流向極有用。

### `#[instrument]` — 函式自動 span

```rust
#[instrument]
fn fetch_user(id: u64) -> Result<String, &'static str> {
    debug!("looking up user");
    ...
}
```

自動產生 span：
- span 名稱 = 函式名
- span 欄位 = 函式參數
- 進入 / 離開 自動記錄

選項：

```rust
#[instrument(skip(big_arg))]           // 跳過某參數（太大不想 log）
#[instrument(level = "debug")]         // 改 span level
#[instrument(fields(request_id = %id))] // 加自訂欄位
#[instrument(ret)]                       // 自動 log 回傳值
#[instrument(err)]                       // Err 時自動 error event
```

## 跟 async / Tokio 配合

async 函式跨 await 點切換 task，普通 thread-local context 會壞。`tracing` 為此設計：

```rust
#[instrument]
async fn handle(req: Request) -> Response {
    fetch_data().await       // span 仍綁定到這個 future
}
```

span 跟著 future 移動，不會跨 task 漏。`tokio` runtime 整合 `tracing` 已內建。

## JSON 輸出（給 log 收集器）

```rust
tracing_subscriber::fmt()
    .json()
    .flatten_event(true)
    .with_current_span(true)
    .with_span_list(true)
    .init();
```

輸出範例：

```json
{"timestamp":"2026-01-15T10:23:45Z","level":"INFO","fields":{"message":"request done","user":"alice","status":200},"target":"ch23","spans":[{"name":"processing","request_id":"abc123"}]}
```

直接 ship 到 Loki / Datadog / OTel collector。

## 何時用哪個 level

| Level | 用途 |
|-------|------|
| `error` | 出錯了，要人處理 |
| `warn` | 異常但能繼續（retry、fallback） |
| `info` | 重要的業務事件（請求、登入、交易） |
| `debug` | 開發 / 除錯資訊 |
| `trace` | 極細粒度（每個 SQL、每個 packet） |

慣例：**production default = info**，trace/debug 用 env var 動態打開。

## 與 OpenTelemetry 整合（第 49 章預告）

```toml
tracing-opentelemetry = "..."
opentelemetry-otlp = "..."
```

`tracing` 的 span 可以直接 export 成 OTel traces。本章只用 fmt subscriber，第 49 章詳述完整 OTel pipeline。

## 對照 TypeScript

Node 端標配是 `pino` 或 `winston`——結構化 logging 已成主流。Rust 的 `tracing` 比這兩個再多一層：**Span**（跨 async 邊界的 context propagation）。對應 Node OTel SDK 的 trace context，但 tracing 把這東西做進語言層級。

### 對照表

| 需求 | Node / TypeScript | Rust |
|---|---|---|
| 結構化 logger | pino / winston | `tracing` + `tracing-subscriber` |
| Level | trace/debug/info/warn/error | 同（`trace!` / `info!` / etc） |
| Pretty / JSON | pino-pretty / 自訂 transport | `.fmt().pretty()` / `.json()` |
| 結構化欄位 | `logger.info({ userId, status }, 'msg')` | `info!(user_id = ..., status = ..., "msg")` |
| 子 logger | `logger.child({ requestId })` | Span |
| Context propagation 跨 await | AsyncLocalStorage | Span 自動跟著 future |
| 過濾等級 | LOG_LEVEL env / config | `RUST_LOG=info` env / `EnvFilter` |
| 函式進入退出 | 手動 / 中介軟體 | `#[instrument]` 屬性 |
| 整合 OTel | `@opentelemetry/api` SDK | `tracing-opentelemetry` |
| Async-safe | 看實作 | `tracing` 設計就是 async-first |

### 程式碼對照

```ts
// TS — pino
import pino from 'pino';
const log = pino();

log.info({ user: 'alice', status: 200 }, 'request done');

// child logger 帶 context
const reqLog = log.child({ requestId: 'abc123' });
reqLog.info('inside request');
// → 自動帶 requestId

// 跨 async 要靠 AsyncLocalStorage
import { AsyncLocalStorage } from 'async_hooks';
const als = new AsyncLocalStorage<{ requestId: string }>();
als.run({ requestId: 'abc' }, async () => {
  await doWork();
  // doWork 內 als.getStore() 拿得到 requestId
});
```

```rust
// Rust — tracing
use tracing::{info, info_span, instrument};

// init subscriber 一次
tracing_subscriber::fmt::init();

info!(user = "alice", status = 200, "request done");

// Span — 對應 child logger
let span = info_span!("request", request_id = "abc123");
let _enter = span.enter();
info!("inside request");        // 自動帶 request_id

// async 內：span 自動跟著 future（不必 AsyncLocalStorage）
#[instrument(fields(request_id = "abc"))]
async fn do_work() {
    info!("working");           // 自動帶 request_id
    fetch().await;              // span 跟著 future 移動
}
```

JSON 輸出（給 Loki / Datadog）：

```ts
// TS — pino 預設就 JSON
log.info({ user: 'alice' }, 'done');
// {"level":30,"time":...,"user":"alice","msg":"done"}
```

```rust
// Rust — fmt 切到 JSON
tracing_subscriber::fmt()
    .json()
    .with_current_span(true)
    .init();

info!(user = "alice", "done");
// {"timestamp":...,"level":"INFO","fields":{"message":"done","user":"alice"},"spans":[...]}
```

### 心智模型差異

1. **Span 是「比 child logger 強」的概念**。pino child logger 帶 context；tracing span 不只帶 context，還記錄「進入時間 / 離開時間 / 巢狀關係」——直接是 distributed tracing 的 span 概念。`tracing-opentelemetry` 把 tracing span export 成 OTel trace。
2. **`#[instrument]` 替代手寫 enter/exit**。對應 TS 用 decorator wrap function 印進出。Rust attribute macro 直接幫你產：span 名 = 函式名、span 欄位 = 函式參數、進入/離開自動記。
3. **async 內 context propagation 自動**。Node 跨 async 要靠 AsyncLocalStorage（V8 提供，比較新）；Rust tracing span 跟著 future 走，因為 future 編譯後是個 state machine，span 進入點 = 那個 state——天然 async-safe。
4. **沒 init 就靜默丟**。tracing **不會** panic 也不會印錯——沒 subscriber 時所有 `info!` 都被丟掉。比 pino 預設行為更詭異，第一次寫的人 90% 會踩。
5. **結構化欄位是 first-class**。`info!(user_id = id, "msg")` 是內建語法，欄位送到 subscriber 後可以分流（terminal pretty print、JSON 給 log aggregator、丟 OTel trace）。pino 也是這方向，但 Rust 把語法做進 macro。
6. **`tracing` 不是 logging library**。社群刻意這樣命名——它是 telemetry 系統，logging 是 events 的一個 use case。一個 trace 包含 events + spans + nested spans，可以同時 ship 給 log（grep）跟 trace（waterfall view）。
7. **RUST_LOG 是 fact-standard env var**。`RUST_LOG=info,hyper=warn,my_crate=debug` 用 env filter 表示式精準控制——比 `LOG_LEVEL=info` 強。EnvFilter 也支援 span 名稱過濾。

## 常見陷阱

1. **沒 init subscriber** — 沒 init 時所有 `info!` 都被丟掉（沒 error，靜默丟）。
2. **同步函式內 `block_on(async fn)`** — span context 可能跨不過，建議統一用 async。
3. **欄位名衝突** — span 跟 event 同名欄位時，event 蓋過 span。
4. **`Debug` 印大物件** — `?big_vec` 印整個 Vec 可能阻塞 logger，用 `len()` 或 sample。
5. **prod 開 debug** — 量大時 log 變瓶頸；用 `tracing-appender` 非阻塞 writer。

## 練習

1. 用 `#[instrument]` 包一個遞迴函式（例如 fib），跑跑看每層 span 怎麼呈現。
2. 把 fmt 換成 `.json()` 輸出，配 `jq` 過濾欄位。
3. 用 `EnvFilter::new("debug,hyper=warn")` 顯式設定，不靠 env var。
4. 寫一個 middleware：在每個 request handler 進入時建一個含 `request_id` 的 span。

## 延伸閱讀

- [tracing 文件](https://docs.rs/tracing/)
- [tracing-subscriber](https://docs.rs/tracing-subscriber/)
- [Tokio tracing 教學](https://tokio.rs/tokio/topics/tracing)
