# Rust Tutorials

從零開始學 Rust，由語言核心一路走到分散式系統。共 **50 章**、分 **7 個 Part**，Web 層採 **axum + actix-web 雙框架對照**。

> 完整章節規劃與設計理由見 [ROADMAP.md](./ROADMAP.md)。
>
> 從 TypeScript 來學 Rust：每章 Part 1–4（01–30）都附「對照 TypeScript」區段；頂層速查表見 [TS-RUST-CHEATSHEET.md](./TS-RUST-CHEATSHEET.md)。

## 目錄

### Part 1 — 基礎與所有權（01–10）
| # | 章節 | 主題 |
|---|------|------|
| 01 | [cargo-setup](./tutorials/01-cargo-setup) | Cargo 與開發環境 |
| 02 | [variables-types](./tutorials/02-variables-types) | 變數、型別、不可變性 |
| 03 | [control-flow](./tutorials/03-control-flow) | 控制流與 Pattern Matching |
| 04 | [functions-closures](./tutorials/04-functions-closures) | 函式與閉包 |
| 05 | [ownership](./tutorials/05-ownership) | Ownership |
| 06 | [borrowing](./tutorials/06-borrowing) | References 與 Borrowing |
| 07 | [slices-strings](./tutorials/07-slices-strings) | Slices 與字串 |
| 08 | [struct-enum](./tutorials/08-struct-enum) | Struct 與 Enum |
| 09 | [collections](./tutorials/09-collections) | 集合型別 |
| 10 | [error-handling](./tutorials/10-error-handling) | 錯誤處理基礎 |

### Part 2 — 型別系統與抽象（11–19）
| # | 章節 | 主題 |
|---|------|------|
| 11 | [generics](./tutorials/11-generics) | 泛型 |
| 12 | [traits](./tutorials/12-traits) | Traits |
| 13 | [lifetimes](./tutorials/13-lifetimes) | 生命週期 |
| 14 | [smart-pointers](./tutorials/14-smart-pointers) | 智慧指標 |
| 15 | [iterators](./tutorials/15-iterators) | Iterator 與函式式風格 |
| 16 | [declarative-macros](./tutorials/16-declarative-macros) | 宣告式宏 `macro_rules!` |
| 17 | [procedural-macros](./tutorials/17-procedural-macros) | 過程宏 |
| 18 | [unsafe-ffi](./tutorials/18-unsafe-ffi) | unsafe 與 FFI |
| 19 | [advanced-errors](./tutorials/19-advanced-errors) | 進階錯誤處理（thiserror / anyhow） |

### Part 3 — 工程化基礎（20–25）
| # | 章節 | 主題 |
|---|------|------|
| 20 | [modules-workspace](./tutorials/20-modules-workspace) | Module、Crate、Workspace |
| 21 | [testing](./tutorials/21-testing) | 測試（單元 / 整合 / doc test） |
| 22 | [cargo-advanced](./tutorials/22-cargo-advanced) | Cargo 進階（features、profiles、build.rs） |
| 23 | [tracing-log](./tutorials/23-tracing-log) | 結構化日誌（tracing） |
| 24 | [config](./tutorials/24-config) | 設定管理 |
| 25 | [benchmark](./tutorials/25-benchmark) | 效能基準測試（criterion） |

### Part 4 — 非同步 Rust（26–30）
| # | 章節 | 主題 |
|---|------|------|
| 26 | [async-await](./tutorials/26-async-await) | async/await 與 Future |
| 27 | [tokio](./tutorials/27-tokio) | Tokio 執行期 |
| 28 | [send-sync-pin](./tutorials/28-send-sync-pin) | Send、Sync、Pin |
| 29 | [async-sync-channels](./tutorials/29-async-sync-channels) | async 同步原語與通道 |
| 30 | [streams](./tutorials/30-streams) | Streams |

### Part 5 — Web 開發（axum + actix-web 對照，31–39）
| # | 章節 | 主題 |
|---|------|------|
| 31 | [http-hyper](./tutorials/31-http-hyper) | HTTP 基礎與 hyper |
| 32 | [axum-basics](./tutorials/32-axum-basics) | axum 入門 |
| 33 | [actix-web-basics](./tutorials/33-actix-web-basics) | actix-web 入門 |
| 34 | [serde-json](./tutorials/34-serde-json) | JSON 與 serde |
| 35 | [sqlx-postgres](./tutorials/35-sqlx-postgres) | sqlx + PostgreSQL |
| 36 | [seaorm](./tutorials/36-seaorm) | SeaORM |
| 37 | [middleware-tower](./tutorials/37-middleware-tower) | Middleware：tower vs actix |
| 38 | [jwt-auth](./tutorials/38-jwt-auth) | JWT 認證 |
| 39 | [websocket](./tutorials/39-websocket) | WebSocket |

### Part 6 — 分散式與架構（40–45）
| # | 章節 | 主題 |
|---|------|------|
| 40 | [clean-architecture](./tutorials/40-clean-architecture) | Clean Architecture（Rust 風格） |
| 41 | [dependency-injection](./tutorials/41-dependency-injection) | 依賴注入 |
| 42 | [redis](./tutorials/42-redis) | Redis |
| 43 | [grpc-tonic](./tutorials/43-grpc-tonic) | gRPC：tonic |
| 44 | [message-queue](./tutorials/44-message-queue) | 訊息佇列（RabbitMQ / Kafka） |
| 45 | [cqrs](./tutorials/45-cqrs) | CQRS 與 Event Sourcing |

### Part 7 — 部署、可觀測性、韌性（46–50）
| # | 章節 | 主題 |
|---|------|------|
| 46 | [docker](./tutorials/46-docker) | Docker（含 cargo-chef） |
| 47 | [cicd](./tutorials/47-cicd) | CI/CD（GitHub Actions、cargo-deny） |
| 48 | [kubernetes](./tutorials/48-kubernetes) | Kubernetes |
| 49 | [prometheus-otel](./tutorials/49-prometheus-otel) | Prometheus 與 OpenTelemetry |
| 50 | [profiling-resilience](./tutorials/50-profiling-resilience) | Profiling、Circuit Breaker、高可用 |

### Part 8 — 桌面開發（51–54）
| # | 章節 | 主題 |
|---|------|------|
| 51 | [tauri-basics](./tutorials/51-tauri-basics) | Tauri 入門：架構、IPC、commands |
| 52 | [tauri-system](./tutorials/52-tauri-system) | 系統整合：檔案、托盤、通知、shortcut |
| 53 | [hidapi-usb](./tutorials/53-hidapi-usb) | USB HID 與硬體裝置控制 |
| 54 | [desktop-packaging](./tutorials/54-desktop-packaging) | 跨平台打包與簽章 |

## Capstones

整合多章節技能的小型實戰專案，獨立於 50 章主線。

| # | 專案 | 預計章節依賴 |
|---|------|--------------|
| 01 | [CLI 工具](./capstones/01-cli-tool) | Part 1–3 |
| 02 | [RGB / 巨集控制器](./capstones/02-rgb-controller) | Part 1–4、Part 8 |
