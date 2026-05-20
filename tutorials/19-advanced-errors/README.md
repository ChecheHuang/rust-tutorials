# 19. 進階錯誤處理

> 範圍：thiserror（library 端）vs anyhow（application 端）、錯誤鏈、backtrace

## 核心對立：library vs application

第 10 章學了手寫 `enum AppError` + `From` impl。實務上有兩個社群事實標準工具自動化：

| | `thiserror` | `anyhow` |
|---|---|---|
| 用途 | 定義**具體錯誤型別** | 收齊**多種錯誤**到一個型別 |
| 適用 | **library**（暴露給使用者） | **application**（main / binary 內部） |
| 提供型別 | 你自訂的 `enum MyError` | `anyhow::Error`（不透明 wrapper） |
| caller 可 pattern match | ✅ | 不容易（只能 downcast） |
| 加 context | 自己包 variant | `.context("...")` |
| Runtime cost | 零 | 一次 heap alloc（小） |

**最佳實踐**：library 用 thiserror、application 用 anyhow。兩者**可以共存**——你的 lib 用 thiserror 定義具體錯誤，binary `main` 用 anyhow 接所有錯誤統一輸出。

## `thiserror` — 一行 derive

```rust
#[derive(Debug, Error)]
pub enum DbError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("user {id} not found")]
    NotFound { id: u64 },
}
```

`#[error("...")]` 定義 `Display` 訊息。`{0}` 是 variant 第 0 個欄位、`{id}` 是 named field。

`#[from]` 自動生成 `impl From<std::io::Error> for DbError`——`?` 可直接轉。

`thiserror` 還會自動實作 `std::error::Error`、`source()` 串接、`backtrace`（需 nightly）。

### 何時用

- 寫 library
- 寫 application 的核心模組，想對外暴露**可分類**的錯誤
- 想讓 caller 寫 `match err { DbError::NotFound { .. } => ..., _ => ... }`

### 對比手寫

```rust
// 手寫
#[derive(Debug)]
enum DbError { Connection(String), Io(io::Error), ... }
impl Display for DbError { fn fmt(...) { match self { ... } } }
impl std::error::Error for DbError { fn source(&self) -> ... { ... } }
impl From<io::Error> for DbError { ... }

// thiserror：上面這些全自動產生
```

20 行縮成 5 行。

## `anyhow` — 萬用錯誤型別

```rust
use anyhow::{Context, Result};

fn application_main() -> Result<()> {
    let port = load_port().context("failed to load port")?;
    let user = fetch_user(0).context("during initial fetch")?;
    Ok(())
}
```

`anyhow::Result<T>` = `Result<T, anyhow::Error>`。`anyhow::Error` 可以**包任何實作 `std::error::Error + Send + Sync + 'static`** 的型別。

`?` 自動轉換：任何錯誤型別 → `anyhow::Error`。**不需要**寫 `From` impl。

### `.context()` — 加上下文

```rust
fs::read(path)
    .with_context(|| format!("reading config from {path}"))?;
```

每層往上串 context，最終錯誤訊息會包含完整的「為什麼失敗」路徑：

```
Error: reading config from /etc/app.toml

Caused by:
    0: No such file or directory (os error 2)
```

`.context(static_str)`：靜態字串
`.with_context(|| format!(...))`：lazy 動態字串（沒錯誤時不執行 format）

### 何時用

- binary 的 main / 高層
- prototype / 一次性 script
- 不在乎 caller 能否 pattern match

### 何時 **不要** 用

- library 對外 API（讓 caller 用 thiserror 看到具體型別）
- 需要根據錯誤類型決定不同處理（用 thiserror 才能 match）

## 錯誤鏈（chain）

任何實作 `std::error::Error` 的型別都可有 `source()`——指向「**造成這個錯誤的下層錯誤**」。

```rust
trait Error {
    fn source(&self) -> Option<&(dyn Error + 'static)>;
}
```

`thiserror` 的 `#[from]` 自動把來源錯誤設為 source。`anyhow` 透過 `context` 串接。

走 chain：

```rust
for cause in err.chain() {
    eprintln!("  - {cause}");
}
```

## main 也能回 Result

```rust
use anyhow::Result;

fn main() -> Result<()> {
    let config = load_config("app.toml")?;
    run(&config)?;
    Ok(())
}
```

`anyhow::Result` 的 main 預設印 chain，比手刻 `Box<dyn Error>` 好看：

```
Error: failed to start server

Caused by:
    0: cannot bind port 80
    1: Permission denied (os error 13)
```

## Backtrace

```rust
// Cargo.toml
[dependencies]
anyhow = { version = "1", features = ["backtrace"] }
```

跑時設 `RUST_BACKTRACE=1` 環境變數，`anyhow::Error` 的 `{:?}` 會印 backtrace。

stable Rust 對 `std::error::Error::backtrace` 還在演進（目前 unstable）；目前事實標準靠 `anyhow` 或 `eyre`。

## `eyre` — anyhow 的兄弟

[`eyre`](https://crates.io/crates/eyre) 是 anyhow fork，差別主要在**可自訂 reporter**（彩色終端 / JSON / etc）。`color-eyre` 提供漂亮的彩色錯誤輸出，在 CLI 工具尤其受歡迎。

API 跟 anyhow 幾乎一樣。可以直接互換。

## 慣例：何時 panic vs Result

| 情境 | 用 |
|------|---|
| 內部不變式（bug） | `panic!` / `unreachable!` |
| 公開 API 認為「絕對不該發生」 | `panic!` + 文件註明 |
| 寫不出處理邏輯的 prototype | `unwrap()` / `expect()` |
| 真實錯誤、值得 caller 處理 | `Result` + thiserror |
| 應用層中央集中錯誤 | `anyhow::Result` |

## 對照 TypeScript

第 10 章對照了基礎 `Result` vs `throw`。本章兩個工具的 TS 類比：`thiserror` ≈ **自訂 Error class 階層**（給 library）、`anyhow` ≈ **`Error` + cause chain**（給 application）。

### 對照表

| 需求 | TypeScript | Rust |
|---|---|---|
| 自訂錯誤型別 | `class MyError extends Error` | `#[derive(Error)] enum MyError { ... }` |
| Display 訊息 | `constructor` 設 `super(message)` | `#[error("...")]` |
| 錯誤鏈 / cause | `new Error(msg, { cause: prev })` | `#[from]` 自動產 `From` impl |
| 訪問下層錯誤 | `err.cause` | `err.source()` |
| 列舉錯誤類型 | discriminated union `\| ...` | enum variant |
| 上層收所有錯誤 | `Error` base class（`catch (e)`） | `anyhow::Error` |
| 加 context | 自己 wrap：`new Error("doing X", { cause: e })` | `.context("doing X")` |
| 印 chain | 自己走 `.cause` 印 | `anyhow` 預設印 |
| Stack trace | `Error.stack` 自動有 | `RUST_BACKTRACE=1` + `anyhow` |
| library vs app 分工 | 沒有標準 | thiserror（lib）+ anyhow（app）社群共識 |

### 程式碼對照

自訂錯誤：

```ts
// TS — 自訂 Error class
class DbError extends Error {
  constructor(msg: string, public override cause?: Error) {
    super(msg);
    this.name = 'DbError';
  }
}
class ConnectionError extends DbError {}
class QueryError extends DbError {
  constructor(public sql: string, cause?: Error) {
    super(`query failed: ${sql}`, cause);
  }
}

try { /* ... */ }
catch (e) {
  if (e instanceof ConnectionError) handleConn(e);
  else if (e instanceof QueryError) handleQuery(e);
  else throw e;
}
```

```rust
// Rust — thiserror
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("query failed: {sql}")]
    Query { sql: String, #[source] cause: Box<dyn std::error::Error> },
    #[error("io error")]
    Io(#[from] std::io::Error),
}

match db_call() {
    Ok(v) => v,
    Err(DbError::Connection(s)) => handle_conn(s),
    Err(DbError::Query { sql, .. }) => handle_query(sql),
    Err(e) => return Err(e),
}
```

加 context（application 層）：

```ts
// TS — 手動 wrap
async function loadConfig(path: string): Promise<Config> {
  try {
    const s = await fs.readFile(path, 'utf-8');
    return JSON.parse(s);
  } catch (e) {
    throw new Error(`loading config from ${path}`, { cause: e });
  }
}
```

```rust
// Rust — anyhow context
use anyhow::{Context, Result};

fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path)
        .with_context(|| format!("loading config from {path}"))?;
    let cfg = serde_json::from_str(&s)
        .context("parsing config json")?;
    Ok(cfg)
}
```

輸出 chain：

```
Error: loading config from /etc/app.toml

Caused by:
    0: No such file or directory (os error 2)
```

### 心智模型差異

1. **library vs app 的分工是社群共識**。TS 沒這個——library 想 throw 什麼就 throw 什麼，使用者 catch unknown 自己解。Rust 明確：library 給具體 enum（thiserror），application 上層用 `anyhow::Error` 統一包。
2. **`#[from]` 是 `?` 的好朋友**。Rust `?` 要自動轉換錯誤型別，依賴 `From<E> for MyError` impl。`#[from]` 一行幫你產出來。TS 沒這層——`throw new MyError(... , { cause: e })` 要顯式包。
3. **`.context()` 是 Rust 的「為什麼失敗」**。`fs::read_to_string(path)?` 失敗只說「No such file」；`.context(|| format!("loading config from {path}"))` 加上「正在做什麼」——對 debug 至關重要。TS 對應的 `Error("...", { cause })` 寫起來啰嗦，且印 chain 要自己寫。
4. **anyhow 不 expose 具體錯誤**。caller 拿到 `anyhow::Error` 只能 log，要 pattern match 必須 `downcast`。所以 library API 不該回 `anyhow`，要給具體型別（thiserror）。對應 TS：library throw 自訂 Error class，application 上層 catch base `Error`。
5. **backtrace 還在演進**。TS / JS `Error.stack` 是預設行為；Rust stable 還沒 std 化（unstable 中），實務靠 `anyhow` / `eyre` + `RUST_BACKTRACE=1` 印出來。
6. **lib 別用 anyhow**。常見初學者誤區（從 TS 過來特別容易犯）：覺得「我也想要簡單，就 `anyhow::Result` 到處」——但你的 lib 使用者就拿到不透明錯誤，無法分類處理。lib 用 thiserror，application 上層才換 anyhow。
7. **沒有「全域 catch」傾向**。Node 寫 `process.on('uncaughtException')`、TS 寫 ErrorBoundary——Rust 鼓勵每個邊界（main、tokio task、handler）明確接 `Result`。
8. **`Box<dyn Error>` 仍能用**。是 std-only 的萬用接，但沒 context、沒 backtrace、印 chain 醜——99% case 換 `anyhow::Error` 立刻變好。

## 常見陷阱

1. **library 用 anyhow** — 等於把 ownership 推給 caller，但 caller 拿到不透明 error 就只能 log，無法分支處理。
2. **thiserror variant 爆炸** — 太細的 variant 導致 enum 巨大、`?` 寫起來累。考慮分群（IO 系列、parse 系列）。
3. **`.context` 寫太籠統** — 「failed」貼到處沒幫助。寫**做什麼+為什麼**：`"reading user config from /etc/foo"`。
4. **吞錯誤再 panic** — `result.unwrap_or_else(|_| panic!("oh no"))`，丟失原始 source；應該 `expect("oh no")` 或保留錯誤鏈。
5. **`Box<dyn Error>` 仍有人用** — std-only、沒 context、堆疊呈現難看。99% case 換 anyhow 立刻變好。

## 練習

1. 把第 10 章手寫的 `AppError` 改用 `thiserror` derive。
2. 把第 10 章的 `load_count` 改成 anyhow + `.context`，故意讓檔案不存在，跑跑看錯誤訊息。
3. 寫 library API `fn parse_config(s: &str) -> Result<Config, ConfigError>` 用 thiserror；application `main` 用 anyhow 把它 `.context("loading app config")?`。
4. 加 `RUST_BACKTRACE=1 cargo run` 看 backtrace。

## 延伸閱讀

- [thiserror docs](https://docs.rs/thiserror/)
- [anyhow docs](https://docs.rs/anyhow/)
- [dtolnay's blog — error design](https://github.com/dtolnay/anyhow/discussions)
