# Rust Tutorials 章節列表與核心考點

## Part 1 — 基礎與所有權

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 01 | cargo-setup | 工具鏈與 Cargo | rustup、cargo new、edition、profile、`Cargo.toml` 區塊 |
| 02 | variables-types | 變數與型別 | let / let mut、shadowing、整數溢位、`i32` vs `usize`、`as` 轉型陷阱 |
| 03 | control-flow | 流程控制 | `if` 是表達式、`match` exhaustive、loop / while / for、break with value |
| 04 | functions-closures | 函式與閉包 | 函式指標、`Fn` / `FnMut` / `FnOnce`、move 捕捉、回傳閉包用 `impl Fn` 或 `Box<dyn Fn>` |
| 05 | ownership | 所有權 | move、Copy trait、drop 順序、所有權移交 vs 借用 |
| 06 | borrowing | 借用 | & vs &mut、N readers XOR 1 writer、reborrow、NLL、跨 await 持有借用陷阱 |
| 07 | slices-strings | Slice 與字串 | `&str` vs `String`、UTF-8 bytes vs chars、`&[T]` 切片、`.iter()` |
| 08 | struct-enum | 結構與列舉 | tuple struct、unit struct、enum variant、`Option` / `Result`、`#[derive(...)]` |
| 09 | collections | 集合 | `Vec`, `HashMap`, `HashSet`、`String` 與 `Vec<u8>`、capacity 與 reserve |
| 10 | error-handling | 錯誤處理 | `Result<T, E>`、`?` 運算子、`From` 自動轉換、panic vs Result 取捨 |

## Part 2 — 型別系統與抽象

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 11 | generics | 泛型 | T: Trait、where 子句、monomorphization、const generics 入門 |
| 12 | traits | Trait | trait 定義、blanket impl、orphan rule、`impl Trait` vs `dyn Trait`、object safety |
| 13 | lifetimes | 生命週期 | `'a` 標註、elision 規則、`'static`、struct 持有引用、HRTB `for<'a>` |
| 14 | smart-pointers | 智慧指標 | `Box<T>`、`Rc<T>`、`Arc<T>`、`RefCell<T>`、`Mutex<T>`、`Cow<T>`、何時用哪個 |
| 15 | iterators | 迭代器 | `Iterator` trait、adapter vs consumer、lazy 求值、collect 型別推斷 |
| 16 | declarative-macros | 宣告式巨集 | `macro_rules!`、metavar `expr`/`ident`/`ty`/`tt`、`$($x:expr),*`、hygiene |
| 17 | procedural-macros | 程序式巨集 | derive vs attribute vs function macro、`syn` + `quote`、proc-macro crate 限制 |
| 18 | unsafe-ffi | unsafe 與 FFI | unsafe 5 種能力、raw pointer、`extern "C"`、cbindgen / bindgen 流程 |
| 19 | advanced-errors | 進階錯誤處理 | `thiserror` 設計 lib 端、`anyhow` 用於 app、error context、backtrace |

## Part 3 — 工程化基礎

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 20 | modules-workspace | 模組與 workspace | `mod` / `use` / `pub`、workspace 設定、共用 dependency |
| 21 | testing | 測試 | unit / integration / doc test、`#[cfg(test)]`、mockall、`assert_*` |
| 22 | cargo-advanced | Cargo 進階 | features、`[profile.*]`、build.rs、conditional compile、`cargo install` |
| 23 | tracing-log | 結構化日誌 | `tracing` vs `log`、`#[instrument]`、subscriber、`EnvFilter` |
| 24 | config | 設定管理 | figment、env / file / argv 合併、`secrecy` crate、12-factor app |
| 25 | benchmark | 效能測試 | criterion、`black_box`、迴歸偵測、micro vs macro benchmark |

## Part 4 — 非同步 Rust

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 26 | async-await | async/await | Future trait、執行器是必要的、`.await` 釋放控制權、`async fn` 回傳 anonymous type |
| 27 | tokio | Tokio runtime | multi-thread vs current-thread、`spawn`、`spawn_blocking`、`JoinSet` |
| 28 | send-sync-pin | Send/Sync/Pin | Send / Sync trait 意義、`Rc` 為何 !Send、std Mutex vs tokio Mutex、Pin 為何需要 |
| 29 | async-sync-channels | Channel & async lock | mpsc / oneshot / broadcast / watch 取捨、async Mutex vs std Mutex、不要 hold lock 過 await |
| 30 | streams | Streams | Stream trait、`StreamExt`、`buffer_unordered`、`chunks_timeout`、async iteration |

## Part 5 — Web 開發（axum + actix 對照）

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 31 | http-hyper | HTTP 基礎 | hyper / reqwest、連接池、TLS（rustls vs native-tls） |
| 32 | axum-basics | axum 入門 | Router、extractor (Path/Query/Json/State)、IntoResponse、tower service |
| 33 | actix-web-basics | actix-web 入門 | App / HttpServer、attribute macro 路由、`Data<T>`、`#[actix_web::main]` 與 tokio 差異 |
| 34 | serde-json | JSON 與 serde | derive、`#[serde(rename_all)]`、`#[serde(tag)]`、`flatten`、serde_with、validator |
| 35 | sqlx-postgres | sqlx + Postgres | pool、`query_as!` 編譯期檢查、transaction、migration、`SQLX_OFFLINE` |
| 36 | seaorm | SeaORM | Entity、ActiveModel `Set` vs `Unchanged`、relation、與 sqlx 取捨 |
| 37 | middleware-tower | Middleware | tower Service / Layer、`middleware::from_fn`、tower-http、actix middleware 比較 |
| 38 | jwt-auth | JWT 認證 | argon2 雜湊（不用 bcrypt/SHA）、HS256 vs RS256、refresh token、blacklist 策略 |
| 39 | websocket | WebSocket | upgrade、split sink/stream、broadcast 聊天室、心跳、SSE 對照 |

## Part 6 — 分散式與架構

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 40 | clean-architecture | Clean Architecture | 三層（domain / application / infrastructure）、依賴方向、composition root、`Arc<dyn Trait>` 為 port |
| 41 | dependency-injection | 依賴注入 | manual wiring 為主、構造函式注入、`Arc<dyn>` vs 泛型、shaku 何時才需要 |
| 42 | redis | Redis | connection manager、cache-aside、rate limit (token bucket)、pub/sub vs streams、Lua atomic |
| 43 | grpc-tonic | gRPC | proto + build.rs、4 種 RPC 模式、interceptor、tower 整合、HTTP/2 plaintext (h2c) |
| 44 | message-queue | 訊息佇列 | NATS / Redis Streams / Kafka / RabbitMQ 取捨、at-least-once + 冪等、outbox pattern |
| 45 | cqrs | CQRS / Event Sourcing | command vs event、aggregate fold、projection 最終一致、event store schema、snapshot |

## Part 7 — 部署、可觀測性、韌性

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 46 | docker | Docker | 多階段 build、cargo-chef cache、distroless / scratch、musl static link、image 簽署 |
| 47 | cicd | CI/CD | GitHub Actions、`Swatinem/rust-cache`、nextest、cargo-audit / cargo-deny、release matrix |
| 48 | kubernetes | Kubernetes | liveness vs readiness、graceful shutdown、preStop sleep、PDB、kube-rs operator |
| 49 | prometheus-otel | 觀測 | metrics 四型、Prometheus exporter、label cardinality、tracing-opentelemetry、sampling |
| 50 | profiling-resilience | Profiling + Resilience | tokio-console、flamegraph、timeout / retry+jitter / circuit breaker / rate limit |

## Part 8 — 桌面開發（Tauri）

> Tauri 章節（51–54）需獨立 build，不在 workspace 內。

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| 51 | tauri-basics | Tauri 入門 | 架構（前端 + Rust core）、`#[tauri::command]`、State、event、與 Electron 取捨 |
| 52 | tauri-system | 系統整合 | plugin、檔案 / dialog / 通知 / 剪貼簿、tray、global shortcut、auto-start、deep link |
| 53 | hidapi-usb | USB HID 與硬體 | HID report (input/output/feature)、`hidapi` crate、Linux udev rule、actor pattern 包裝 IO |
| 54 | desktop-packaging | 跨平台打包 | tauri build、Windows code sign、macOS notarize、AppImage、updater（ed25519 簽章） |

## Capstones

| 編號 | 目錄名稱 | 主題 | 核心考點 |
|------|---------|------|---------|
| cap1 | 01-cli-tool (mini-rg) | CLI 整合專案 | clap、ignore crate、regex bytes、crossbeam channel、binary detect、assert_cmd 測試 |
| cap2 | 02-rgb-controller (rgb-core) | 桌面 + 硬體整合 | Device trait（port）、HidRgbDevice actor、watch channel 控效果、profile 序列化、Tauri 整合 |

## 章節相依推薦（答錯時引導複習）

| 答錯章節 | 建議回頭看 |
|----------|------------|
| 13 lifetimes | 05 ownership、06 borrowing |
| 14 smart-pointers | 05、06、12 traits |
| 17 procedural-macros | 16 declarative-macros、12 traits |
| 18 unsafe-ffi | 14 smart-pointers、05、06 |
| 28 send-sync-pin | 14 smart-pointers、26 async-await |
| 29 channels | 27 tokio、28 send-sync-pin |
| 30 streams | 15 iterators、26 async-await |
| 32 axum / 33 actix | 12 traits、26 async-await |
| 35 sqlx / 36 seaorm | 10 error-handling、34 serde-json |
| 37 middleware | 12 traits、32 axum-basics |
| 40 clean-architecture | 12 traits、20 modules-workspace |
| 43 grpc-tonic | 26 async-await、12 traits |
| 45 cqrs | 08 struct-enum、29 channels、40 clean-architecture |
| 48 kubernetes | 27 tokio（graceful shutdown）、46 docker |
| 49 prometheus-otel | 23 tracing-log |
| 50 profiling-resilience | 27 tokio、29 channels |
| Tauri 系列 | 24 config、29 channels、第 40 章 |
| cap1 | Part 1 全部 + 11、12、15、20、21 |
| cap2 | 12 traits、26–29、40 clean-architecture、第 53 章 |
