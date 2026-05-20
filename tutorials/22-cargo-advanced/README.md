# 22. Cargo 進階

> 範圍：features、profiles、build.rs、workspace.dependencies、cargo-make

## Features — 編譯期開關

```toml
[features]
default = ["fancy"]    # 預設啟用哪些
fancy = []
extra = []
gpu = ["dep:cuda"]      # feature 自動啟用 optional dep
full = ["fancy", "extra", "gpu"]

[dependencies]
serde = { version = "1", optional = true }
cuda = { version = "0.1", optional = true }
```

啟用：

```bash
cargo build                              # default features
cargo build --no-default-features        # 不啟用任何
cargo build --features gpu               # 加 gpu
cargo build --features "fancy gpu"       # 多個
cargo build --all-features               # 全開
```

程式內判斷：

```rust
#[cfg(feature = "fancy")]
fn fancy_render() { ... }
```

### 設計原則

- **additive**：feature 只能**加**功能，**不能**改變現有 API
- 預設 features 應該是「一般使用者要的最小集合」
- library 用 features 控制 dep 大小（如 `serde` 的 `derive` feature）

## Profiles — 編譯設定

```toml
[profile.dev]
opt-level = 0
debug = true
incremental = true

[profile.release]
opt-level = 3
debug = false
lto = "thin"          # link-time optimization
codegen-units = 1     # 跨單元優化
strip = "debuginfo"   # 縮 binary

[profile.release-with-debug]
inherits = "release"
debug = true          # release 但保 debug info
```

| 場景 | profile |
|------|---------|
| `cargo run` / `cargo test` | `dev` |
| `cargo run --release` / `cargo build --release` | `release` |
| `cargo bench` | `bench`（繼承 release） |

### release profile 重點選項

| 選項 | 用途 |
|------|------|
| `opt-level` | `0`（無）`1` `2` `3`（最高）`"s"`（小）`"z"`（最小） |
| `lto` | `false` `"thin"` `true` — 跨 crate 優化 |
| `codegen-units` | 越少優化越好但編譯越慢 |
| `strip` | `"none"` `"debuginfo"` `"symbols"` |
| `panic` | `"unwind"`（預設） / `"abort"`（binary 變小、無 stack unwind） |

CLI 工具常用：

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

換來 ~20% binary 縮小、~5-10% 執行速度提升，編譯時間翻倍。

## `build.rs` — 編譯期腳本

放在 crate root 的 `build.rs` 會在編譯**前**執行：

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=schema.sql");
    println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now());
    // codegen
    // compile C lib
    // 從環境變數產 const
}
```

### 用途

1. **codegen**：從 proto / schema 生成 Rust（`tonic-build`、`prost-build`）
2. **C library**：編譯 / link C 程式碼（`cc` crate）
3. **環境變數**：把 git commit / build time 餵進 binary
4. **feature 偵測**：根據編譯環境決定 cfg flag

### 從 build.rs 設定環境變數

```rust
// build.rs
println!("cargo:rustc-env=GIT_HASH={}", git_hash());

// src/main.rs
println!("built from {}", env!("GIT_HASH"));
```

## Workspace dependencies

```toml
# workspace 根 Cargo.toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }

# member crate
[dependencies]
tokio.workspace = true
serde.workspace = true
```

讓所有 member crate 用同一個版本，**避免** dependency tree 內存在多份 tokio（編譯慢 + binary 變大）。

## Cargo.lock — 鎖定版本

- `Cargo.toml` 寫 **range**：`"1"` 表示 `>=1.0, <2.0`
- `Cargo.lock` 鎖**具體** version 與 dep hash

慣例：
- binary：commit Cargo.lock（保證可重現 build）
- library：不 commit（讓使用者的 lock 主導）

## 常用 cargo 子命令

| 命令 | 用途 |
|------|------|
| `cargo install <crate>` | 全域安裝 binary crate |
| `cargo install --path .` | 安裝當前 crate 為全域 binary |
| `cargo tree` | 顯示 dependency 樹 |
| `cargo tree --duplicates` | 找重複版本的 dep |
| `cargo outdated`（需 cargo-outdated） | 找可升級的 dep |
| `cargo audit` | 檢查已知 CVE |
| `cargo deny check` | 政策檢查（許可、CVE、來源） |
| `cargo update` | 在 SemVer 範圍內更新 |
| `cargo update -p tokio --precise 1.35` | 釘住特定版本 |
| `cargo bloat` | 看 binary 哪個 crate 最肥 |

## `cargo-make` — 跨平台 task runner

```toml
# Makefile.toml
[tasks.ci]
dependencies = ["fmt-check", "clippy", "test"]

[tasks.fmt-check]
command = "cargo"
args = ["fmt", "--all", "--check"]

[tasks.clippy]
command = "cargo"
args = ["clippy", "--", "-D", "warnings"]
```

```bash
cargo make ci
```

跨平台、可組合，比 Makefile 友善。或用 [`just`](https://github.com/casey/just)（語法更接近 Makefile）。

## Workspace 級的 lints 集中

```toml
# workspace root
[workspace.lints.rust]
unused = "warn"
missing_docs = "warn"

[workspace.lints.clippy]
pedantic = "warn"
nursery = "warn"
```

```toml
# member crate
[lints]
workspace = true
```

統一全 workspace 的 lint level。

## 對照 TypeScript

`Cargo.toml` 對應 `package.json` 但內建更多東西：**features**（編譯期條件編譯，TS 沒對應）、**profiles**（debug/release 等多套編譯設定，TS 對應 `tsconfig.json` 不同模式）、**build.rs**（編譯前腳本，TS 對應 `prebuild` script 或 `tsc` plugin）。

### 對照表

| 需求 | TypeScript / Node | Rust |
|---|---|---|
| 套件設定 | `package.json` | `Cargo.toml` |
| 套件版本範圍 | `"^1.2.3"` | `"1"` / `"=1.2.3"` |
| Optional dep | `optionalDependencies` | `optional = true` + feature gate |
| 條件編譯 | `process.env.NODE_ENV` runtime 判斷 | `#[cfg(feature = "x")]` 編譯期 |
| 多種設定 | `tsconfig.json` 多套 + 切換 | `[profile.dev]` / `[profile.release]` / 自訂 |
| 編譯前腳本 | `"prebuild": "..."` script | `build.rs` |
| 環境變數注入 binary | `process.env.X`（runtime） | `env!("X")` 編譯期 / `build.rs` 設定 |
| Monorepo 版本統一 | pnpm catalog | `[workspace.dependencies]` |
| Lock 檔 | `package-lock.json` / `pnpm-lock.yaml` | `Cargo.lock` |
| 全域安裝 binary | `npm install -g xxx` | `cargo install xxx` |
| 看 dep tree | `npm ls` / `pnpm why` | `cargo tree` |
| 查重複版本 | `pnpm dedupe` | `cargo tree --duplicates` |
| Lint dep（CVE） | `npm audit` | `cargo audit` |
| 政策檢查 | 沒標準（用 license-checker） | `cargo deny check` |
| 任務 runner | npm scripts | `cargo-make` / `just` |
| 看 binary size | webpack-bundle-analyzer | `cargo bloat` |

### 程式碼對照

Features（**TS 沒有對應概念**）：

```toml
# Cargo.toml
[features]
default = ["postgres"]
postgres = ["dep:sqlx"]
mysql = ["dep:mysql"]
embedded = []        # 改變編譯行為，不依新增 dep

[dependencies]
sqlx = { version = "0.7", optional = true }
mysql = { version = "23", optional = true }
```

```rust
// 程式碼內條件編譯
#[cfg(feature = "postgres")]
pub fn connect() { /* PG impl */ }

#[cfg(feature = "mysql")]
pub fn connect() { /* MySQL impl */ }
```

```bash
cargo build --no-default-features --features mysql
```

TS 對應通常是 runtime if：

```ts
// TS — 只能 runtime 切換（程式碼都 bundle）
if (process.env.DB === 'mysql') { /* ... */ }
else                            { /* ... */ }
```

Profile：

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
```

TS 對應 tsconfig multi-config（沒 Rust 那麼集中）。

build.rs：

```rust
// build.rs — 編譯前跑
fn main() {
    println!("cargo:rustc-env=GIT_HASH={}", git_hash());
    println!("cargo:rerun-if-changed=schema.sql");
}

// src/main.rs
println!("built from {}", env!("GIT_HASH"));   // 編譯期注入
```

```json
// TS / package.json — 用 prebuild script
{
  "scripts": {
    "prebuild": "node scripts/inject-git.js",
    "build": "tsc"
  }
}
```

### 心智模型差異

1. **features 是「編譯期菜單」**。TS 沒對應——所有 code 都 bundle，靠 runtime 判斷或 tree-shaking 簡化。Rust 用 features 控制哪些 dep / code 進 binary，沒選的根本不編。設計時用於：「**支援多種 DB 後端但只裝一種**」「**embedded vs server 兩套行為**」。
2. **features 必須 additive**。不能設計 `feature a` 跟 `feature b` 互斥——Cargo 依賴解析會多版本合併 features，違反 additive 會炸。TS runtime if 沒這限制。
3. **profile 不只是 debug/release**。可以自訂 profile（如 `release-with-debug`、`profile.bench`），分別優化 binary size、執行速度、編譯時間。TS 對應只有 `tsc --watch` vs 正式編譯，沒這麼多維度。
4. **`build.rs` 比 prebuild script 強**。可以呼叫 C compiler 編 native code、跑 codegen（tonic-build 從 proto 生 Rust）、抓 git hash 注入。`cargo:rerun-if-changed` 告訴 cargo 何時要重跑。TS 對應的 prebuild 一律每次跑。
5. **`env!` 是編譯期、`std::env::var` 是 runtime**。對應 TS：`process.env.X` 永遠 runtime。Rust 把「**這個值編譯時就鎖死**」（如 build version）跟「**runtime 從環境讀**」（如 DATABASE_URL）分得很清。
6. **Cargo.lock 釘住 commit 與否的慣例**。前面 ch20 提過。`cargo update -p tokio --precise 1.35` 釘特定版本是日常操作。
7. **`cargo tree --duplicates` 是 Rust 特有痛**。多個 dep 各自要不同版本的某共用 lib，Rust 不像 npm 能容忍兩份共存（編譯期型別不通）——必須在 workspace 統一版本。`cargo tree --duplicates` 找出衝突。
8. **`cargo deny` 是現代專案標配**。檢查 license（避免引入 GPL 污染商用）、CVE、來源（只允許 crates.io + 內部 git）——TS 對應沒這麼集中（要拼 license-checker + audit + 自訂）。

## 常見陷阱

1. **feature 互斥** — Cargo features 設計上是 additive，**不能**有「a 或 b」這種 either-or。違反會造成大量 dep resolution 痛苦。
2. **build.rs 慢** — 每次 dep 變動都跑，寫太多 IO 會嚴重拖編譯。慎用。
3. **lock 衝突** — 多人改 Cargo.toml 常造成 Cargo.lock 衝突；用 `cargo update --workspace` 重新生成。
4. **`opt-level = "z"` 不總是最小** — 可能反而變大；測過再用。
5. **`strip = true` 影響 panic 訊息** — 沒 symbols 的 backtrace 不好看。

## 練習

1. 為本章加 `extra` feature，feature 啟用時印額外資訊；用 `cargo run --features extra` 驗證。
2. 寫一個 `build.rs` 把當前 git commit 短 hash 注入 binary（提示：`std::process::Command`）。
3. 比較 `cargo build --release` 與加 `lto = true` 後的 binary 大小。
4. 用 `cargo tree --duplicates` 看本系列 workspace 是否有重複版本。

## 延伸閱讀

- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [build.rs](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
