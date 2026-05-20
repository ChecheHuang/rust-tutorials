# Rust Tutorials Roadmap

本文件說明 50 章的設計理由、學習目標與相依關係。對應索引見 [README.md](./README.md)。

## 設計原則

1. **按 Rust 學習曲線排序** — 所有權、生命週期、宏這些 Rust 特有的概念該擺哪裡、要拆多細，全部以 Rust 自身的節奏為準。
2. **前 19 章是語言核心** — Rust 的陡峭曲線集中在語言本身；工程化（測試、log、config）等到語言觀念紮實後再進。
3. **async 獨立成 Part** — Tokio 生態的複雜度需要連貫 5 章鋪陳，不適合一章帶過。
4. **Web 雙框架對照** — axum 與 actix-web 在路由、middleware、extractor 上設計哲學不同；每章用同一個例子（如 user CRUD）兩邊各寫一次，幫助讀者建立選型直覺。
5. **每章可獨立運作** — 完成後預計以 Cargo workspace 組織，每章是一個 member crate，可獨立 `cargo run`。本次先建文件骨架，workspace 結構在第 01 章撰寫時一併引入。

## 章節詳述

### Part 1 — 基礎與所有權（01–10）

> 目標：能寫出可編譯、能處理錯誤、能組裝資料結構的 Rust 程式。

- **01 cargo-setup** — rustup toolchain、cargo new、edition、rust-analyzer、`cargo fmt/clippy`。輸出第一個 Hello World。
- **02 variables-types** — `let`/`mut`、基本型別、shadowing、型別推導、`as` 轉型、整數溢位。
- **03 control-flow** — `if`/`else`、`loop`/`while`/`for`、`match` 完整模式、`if let`、`while let`、`@` binding。
- **04 functions-closures** — 函式簽章、`fn` vs 閉包、`Fn`/`FnMut`/`FnOnce`、高階函式、move closure。
- **05 ownership** — move 語意、Copy vs Clone、Drop、函式參數移動。是整套教材的轉折點。
- **06 borrowing** — `&T` vs `&mut T`、借用規則（一寫多讀互斥）、NLL、reborrow。
- **07 slices-strings** — `&str` vs `String`、`&[T]` vs `Vec<T>`、字串切片陷阱（UTF-8 邊界）、`String::from` vs `to_string`。
- **08 struct-enum** — struct、tuple struct、unit struct、enum、`Option<T>`、`#[derive(Debug, Clone)]`。
- **09 collections** — `Vec`、`HashMap`、`HashSet`、`BTreeMap`、`VecDeque`、容量與配置。
- **10 error-handling** — `Result<T, E>`、`Option<T>`、`?` 運算子、`panic!` vs 可恢復錯誤、`From` 與錯誤轉型。

### Part 2 — 型別系統與抽象（11–19）

> 目標：能設計可重用的抽象、撰寫自己的宏、跨越 `unsafe` 邊界。

- **11 generics** — 函式 / struct / enum 泛型、`where` clause、monomorphization 與 binary size 取捨。
- **12 traits** — trait 定義、impl、default method、associated types、trait object（`dyn Trait`）、orphan rule。
- **13 lifetimes** — lifetime 標註、`'static`、HRTB、lifetime elision 規則、struct 內的引用。
- **14 smart-pointers** — `Box<T>`、`Rc<T>`、`Arc<T>`、`RefCell<T>`、`Mutex<T>`、`Cow<T>`，何時用哪一個。
- **15 iterators** — `Iterator` trait、adapter chain、`collect`、`fold`、lazy evaluation、自訂 iterator。
- **16 declarative-macros** — `macro_rules!`、fragment specifier、衛生規則、實作小型 DSL。
- **17 procedural-macros** — derive macro、attribute macro、function-like macro、`syn` + `quote` + `proc-macro2`。
- **18 unsafe-ffi** — `unsafe` 邊界、raw pointer、`extern "C"`、與 C 互通、cbindgen 入門。
- **19 advanced-errors** — `thiserror`（library 端）vs `anyhow`（application 端）、自訂錯誤型別、錯誤鏈、`backtrace`。

### Part 3 — 工程化基礎（20–25）

> 目標：能組織多 crate 專案、寫得出測試與 benchmark、有觀測性。

- **20 modules-workspace** — `mod`、`pub`、`use`、`pub(crate)`、workspace 設定、跨 crate 依賴。
- **21 testing** — `#[test]`、整合測試 `tests/`、doc test、`assert_eq!` 家族、`mockall`、`cargo nextest`。
- **22 cargo-advanced** — features、profiles、`build.rs`、`patch`、`workspace.dependencies`、`cargo-make`。
- **23 tracing-log** — `tracing` crate、`tracing-subscriber`、span、event、結構化欄位、與 OTel 接合預備。
- **24 config** — `config` crate、`figment`、環境變數 + 檔案 + CLI 三源合併、secret 處理。
- **25 benchmark** — `criterion`、micro-benchmark、避免被編譯器優化掉、與 profiling 章節（50）的銜接。

### Part 4 — 非同步 Rust（26–30）

> 目標：理解 Future polling 模型，能寫出正確的 async 程式。

- **26 async-await** — `async fn`、`.await`、`Future` trait、`Poll`、為什麼需要 executor。
- **27 tokio** — `#[tokio::main]`、runtime、`spawn`、`block_on`、task、cooperative scheduling。
- **28 send-sync-pin** — `Send` / `Sync` 規則、`Pin` / `Unpin`、self-referential 為什麼難。
- **29 async-sync-channels** — `tokio::sync::{Mutex, RwLock}`、`mpsc`、`broadcast`、`watch`、`oneshot`，與 std 同步原語的差異。
- **30 streams** — `Stream` trait、`StreamExt`、async iteration、`tokio-stream`。

### Part 5 — Web 開發（31–39）

> 目標：能用 axum 或 actix-web 蓋出含資料庫、認證、middleware 的服務。每章「同一個例子，兩個框架」。

- **31 http-hyper** — `hyper` 1.x 的 client/server、`reqwest`、HTTP/1 vs HTTP/2，是 axum/actix 的底層基礎。
- **32 axum-basics** — `Router`、handler、extractor（`Path`/`Query`/`Json`/`State`）、`IntoResponse`。
- **33 actix-web-basics** — `App`、`HttpServer`、`web::Path`/`web::Json`、Responder trait。
- **34 serde-json** — `Serialize`/`Deserialize`、`#[serde(...)]` 屬性、自訂、`validator` crate。
- **35 sqlx-postgres** — `sqlx::query!` 編譯期 SQL 驗證、connection pool、`sqlx-cli` migration。
- **36 seaorm** — Entity、Active Model、relation、migration、與 sqlx 的取捨。
- **37 middleware-tower** — Tower `Service` / `Layer`（axum 端）vs actix middleware，同樣需求兩種實作。
- **38 jwt-auth** — `jsonwebtoken`、Bearer token 解析、把驗證做成 extractor（axum）與 middleware（actix）。
- **39 websocket** — broadcast pattern、心跳、斷線重連，兩框架對照。

### Part 6 — 分散式與架構（40–45）

> 目標：建立分散式系統需要的工程觀念與生態工具。

- **40 clean-architecture** — Rust 風格的層次劃分：trait 作為 port、`Arc<dyn UseCase>` 注入，避開 Java 風 boilerplate。
- **41 dependency-injection** — 手刻 DI vs `shaku`、編譯期 vs 執行期注入。
- **42 redis** — `redis` crate、`deadpool-redis`、cache aside、pub/sub。
- **43 grpc-tonic** — proto 定義、server / client、interceptor、streaming RPC。
- **44 message-queue** — RabbitMQ（`lapin`）與 Kafka（`rdkafka`），生產者 / 消費者範本。
- **45 cqrs** — CQRS + Event Sourcing 概念、`cqrs-es` crate、與 44 訊息佇列整合。

### Part 7 — 部署、可觀測性、韌性（46–50）

> 目標：把服務送上 Kubernetes、看得到、撐得住。

- **46 docker** — 多階段建置、distroless image、`cargo-chef` 加速 dependency 編譯。
- **47 cicd** — GitHub Actions、`cargo fmt --check`、`clippy -- -D warnings`、`cargo-deny`、`cargo-audit`、cross-compile release。
- **48 kubernetes** — deployment / service / configmap、Helm chart、`kube-rs` 寫 operator。
- **49 prometheus-otel** — `metrics-rs` 或 `prometheus` crate、`/metrics` endpoint、`opentelemetry-otlp` traces。
- **50 profiling-resilience** — `tokio-console`、`flamegraph`、`pprof-rs`、tower 的 retry / timeout / circuit breaker / load shedding。

### Part 8 — 桌面開發（51–54）

> 目標：能用 Rust 寫出跨平台、可發佈的桌面應用，並能直接與 USB 硬體溝通。

- **51 tauri-basics** — Tauri 架構（前端 + Rust core）、IPC、`#[tauri::command]`、`invoke` / event、開發環境設定。前端任選 Vue / React / Svelte，範例採 Vue + TypeScript。
- **52 tauri-system** — 檔案系統 API、系統托盤、原生通知、全域 shortcut、視窗管理、deep link。
- **53 hidapi-usb** — `hidapi` crate、USB HID 報告（input / output / feature）、enumerate 裝置、與 Tauri 整合的事件流（裝置插拔 → 前端通知）。
- **54 desktop-packaging** — `tauri build`、Windows 程式碼簽章、macOS notarization、Linux AppImage / Flatpak、Tauri updater 自動更新。

## Capstones

整合多章節技能的小型實戰專案，獨立於 50 章主線，放在 `capstones/` 目錄。

### Capstone 1 — CLI 工具

- **時機**：完成 Part 3（工程化基礎）後
- **預設題目（三選一）**：mini ripgrep、log 分析器、git 統計工具
- **練到的技能**：`clap`、`indicatif`、`rayon` / `tokio` 並發、`serde`、整合測試（`assert_cmd`）、跨平台 binary
- **詳細規格**：見 [`capstones/01-cli-tool/README.md`](./capstones/01-cli-tool/README.md)

### Capstone 2 — RGB / 巨集控制器

- **時機**：完成 Part 8 後（綜合性最高的 capstone）
- **題目**：OpenRGB 風格的跨品牌、跨平台 RGB 與巨集統一控制工具
- **練到的技能**：Tauri IPC、`hidapi` USB HID、trait abstraction（多品牌裝置抽象）、async 事件流、設定持久化、跨平台簽章發佈
- **詳細規格**：見 [`capstones/02-rgb-controller/README.md`](./capstones/02-rgb-controller/README.md)

## 待決議事項

- **Cargo workspace 結構**：第 01 章撰寫時引入根 `Cargo.toml`，每章為 member crate。Capstone 是否獨立於主 workspace 待定。
- **每章長度**：以「一坐能讀完」為基準（`README.md` + `main.rs`），複雜章節可拆 `examples/` 目錄。
- **CLI 框架（clap）何時帶入**：傾向**穿插在 Part 1–3 各章的 `main.rs`** 中自然使用，不另開教學章；正式整合留到 Capstone 1。
