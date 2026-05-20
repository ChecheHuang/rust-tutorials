# 20. Module、Crate、Workspace

> 範圍：mod / pub / use、pub(crate)、workspace 設定、跨 crate 依賴

## 三層組織結構

```
workspace
  ├── crate A (lib 或 bin)
  │     └── module（mod）
  │           └── 子 module
  │                 └── items（fn、struct、const、type）
  ├── crate B
  └── ...
```

- **workspace**：一個 `Cargo.toml`（root），管理多個 crate
- **crate**：編譯單位，獨立 binary 或 library
- **module**：crate 內部的命名空間，純編譯期概念

## Module

### 內嵌定義

```rust
mod auth {
    pub fn login(user: &str) -> String { ... }
    pub mod password {
        pub fn hash(p: &str) -> String { ... }
    }
}
```

### 拆檔案

`mod auth;` + `src/auth.rs`：

```
src/
├── main.rs        // mod auth;
└── auth.rs        // pub fn login() { ... }
```

或者多層：

```
src/
├── main.rs        // mod auth;
└── auth/
    ├── mod.rs     // pub mod password;
    └── password.rs
```

或 2018+ 新風格：

```
src/
├── main.rs        // mod auth;
├── auth.rs        // pub mod password;
└── auth/
    └── password.rs
```

新風格的 `auth.rs` 取代了舊風格的 `auth/mod.rs`，**避免**一堆同名 `mod.rs` 在 editor 標籤頁難辨識。

## Visibility

| Modifier | 可見範圍 |
|----------|----------|
| (預設) | 當前 module |
| `pub` | 完全公開 |
| `pub(crate)` | 整個 crate 內 |
| `pub(super)` | parent module |
| `pub(in path)` | 指定 path 之下 |

```rust
pub fn public_api() { ... }
pub(crate) struct InternalConfig { ... }
fn helper() { ... }   // private
```

慣例：先 `pub(crate)`，需要時才升為 `pub`。`pub` 等於對下游使用者承諾穩定 API，慎用。

## `use` 與 re-export

```rust
use auth::password::hash;          // 縮短路徑
use auth::{Session, login};        // 多個一次
use auth::password::hash as h;     // alias
use std::collections::*;            // glob（少用，難讀）

pub use auth::login;                // re-export，使 caller 也能用 your_crate::login
```

**re-export** 是 library 設計的重點工具。內部組織用多層 module，對外只 export 扁平 API：

```rust
// crate root
pub use crate::network::client::Client;
pub use crate::network::server::Server;
pub use crate::error::Error;
```

caller 寫 `your_crate::Client` 而非 `your_crate::network::client::Client`。

## Crate 種類

| Type | 用途 | `Cargo.toml` |
|------|------|--------------|
| `bin` | binary（有 main） | `src/main.rs` |
| `lib` | library | `src/lib.rs` |
| `cdylib` | 給 C / 其他語言用的 dynamic library | `crate-type = ["cdylib"]` |
| `staticlib` | 靜態 link 給 C | `crate-type = ["staticlib"]` |
| `proc-macro` | procedural macro | `[lib] proc-macro = true` |

一個 crate 可同時有 bin + lib（`src/main.rs` + `src/lib.rs`），binary 用 `your_crate::...` 呼叫 lib API。

多個 binary：

```
src/
├── lib.rs
└── bin/
    ├── server.rs
    └── client.rs
```

跑 `cargo run --bin server`。

## Workspace

`Cargo.toml`（root）：

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2021"
version = "0.1.0"

[workspace.dependencies]
serde = "1"
tokio = { version = "1", features = ["full"] }
```

每個 member crate 引用 workspace 設定：

```toml
# crates/foo/Cargo.toml
[package]
name = "foo"
edition.workspace = true
version.workspace = true

[dependencies]
serde.workspace = true
```

**好處**：
- 統一版本（避免不同 crate 用不同 serde version 編兩份）
- 共享 `target/`，編譯時間大幅縮減
- 一次 `cargo test` 跑全部
- 內部 crate 互相依賴用 `path`，不需 publish

## 跨 crate 依賴

```toml
[dependencies]
# 從 crates.io
serde = "1"

# 從 git
my_lib = { git = "https://github.com/user/my_lib", branch = "main" }

# 從本地 path
internal = { path = "../internal" }

# workspace 共用版本
tokio.workspace = true
```

## Workspace 範例（本系列）

```
rust-tutorials/
├── Cargo.toml              # [workspace], [workspace.dependencies]
├── tutorials/
│   ├── 01-cargo-setup/
│   │   ├── Cargo.toml      # edition.workspace = true
│   │   └── src/main.rs
│   ├── 02-variables-types/
│   └── ...
└── capstones/
    ├── 01-cli-tool/         # 獨立 workspace（exclude）
    └── 02-rgb-controller/
```

`exclude = ["capstones/..."]` 讓 capstone 有自己的 `Cargo.toml` 與 `target/`，避免互相影響。

## 慣例與風格

- 一個 module = 一個檔案（除非很小）
- module 名用 `snake_case`
- private 預設，需要時才 `pub`
- 對外 API 用 `pub use` 攤平
- workspace 用 `[workspace.dependencies]` 集中版本
- `src/lib.rs` 是 crate 的 public 介面入口

## 對照 TypeScript

TS / Node 的模組是「**檔案 = 模組**」，import 從檔案路徑開始。Rust 是「**module 樹是顯式宣告的**」——檔案存在不等於 module 存在，必須在 parent 寫 `mod xxx;` 才會被納入。Workspace 對應 pnpm / nx 的 monorepo。

### 對照表

| 概念 | TypeScript / Node | Rust |
|---|---|---|
| 模組單位 | 一個檔案 | 一個 module（可內嵌或拆檔） |
| 引入別處檔案 | `import { foo } from './foo'` | 先 `mod foo;` 再 `use foo::foo` |
| 預設 export | `export default ...` | 沒有；都明確命名 |
| 命名 export | `export const x = ...` | `pub fn x() { ... }` |
| 全部 export | `export * from './foo'` | `pub use foo::*;` |
| 重新命名 | `import { x as y }` | `use foo::x as y;` |
| 私有 | `const x = ...`（不 export） | 預設 private |
| 半公開 | 沒有（要嘛 export 要嘛不 export） | `pub(crate)` / `pub(super)` / `pub(in path)` |
| 編譯單位 | 通常一個 package | 一個 crate |
| 套件設定 | `package.json` | `Cargo.toml` |
| Monorepo | pnpm workspace / nx / turbo | `[workspace]` |
| 共用版本 | pnpm `catalog` | `[workspace.dependencies]` + `dep.workspace = true` |
| 內部相依 | `"foo": "workspace:*"` | `foo = { path = "../foo" }` |
| 多 binary | `package.json` `bin` 欄位 | `src/bin/*.rs` |

### 程式碼對照

```ts
// TS — 檔案就是 module
// foo/bar.ts
export function baz() { return 42; }

// main.ts
import { baz } from './foo/bar';
console.log(baz());
```

```rust
// Rust — 必須顯式宣告 module 樹
// src/foo/bar.rs
pub fn baz() -> i32 { 42 }

// src/foo/mod.rs（或 src/foo.rs）
pub mod bar;

// src/main.rs
mod foo;             // 把 foo module 納入這個 crate
use foo::bar::baz;
fn main() {
    println!("{}", baz());
}
```

Workspace：

```json
// pnpm-workspace.yaml + package.json
{
  "name": "monorepo",
  "private": true,
  "workspaces": ["packages/*"]
}
```

```toml
# Cargo.toml（root）
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = "1"

# crates/foo/Cargo.toml
[package]
name = "foo"
edition.workspace = true

[dependencies]
tokio.workspace = true
```

### 心智模型差異

1. **「檔案存在 ≠ module 存在」**。TS 新增 `src/utils.ts` 立刻能 `import`；Rust 新增 `src/utils.rs` 還要在 parent（`main.rs` 或 `lib.rs`）寫 `mod utils;` 才會被編譯。最常見的初學者卡關。
2. **`mod` 跟 `use` 不同**。`mod foo;` 是**宣告**「這裡有個叫 foo 的子 module」（要嘛內嵌、要嘛從 `foo.rs` / `foo/mod.rs` 讀）。`use foo::Bar;` 是**縮短路徑**讓你不必寫 full path。TS `import` 兩件事合一。
3. **沒有 `export default`**。Rust 一律明確命名。Pythonic / TS 寫 `import default from 'module'` 的習慣要丟掉。
4. **可見度有多層**。TS 只有「export 或不 export」；Rust 有 `pub`、`pub(crate)`、`pub(super)`、`pub(in path)`、private。慣例：先 `pub(crate)`，要對外才升 `pub`——對外 API 要謹慎，承諾穩定。
5. **re-export 是 library 設計重點**。TS index.ts pattern 也有，但 Rust 用 `pub use crate::network::Client;` 把深層型別「攤平」到 crate root——使用者寫 `your_crate::Client` 而非 `your_crate::network::client::Client`。
6. **crate 比 npm package 更剛性**。一個 crate 一份 `Cargo.toml`、一個 binary 入口（`main.rs`）或 lib 入口（`lib.rs`）。`src/bin/foo.rs` + `src/bin/bar.rs` 自動是兩個 binary。npm package 可以一個目錄一堆 entry，自由很多。
7. **Cargo.lock commit 慣例反過來**。Node：library 跟 application 都 commit lock；Rust：binary commit、library **不 commit**（讓使用者的 lock 主導 dep tree）。
8. **workspace 編譯時間是大議題**。Cargo workspace 共用 `target/` 避免重複編譯 dep，但每改一個 crate 仍可能重編下游。`cargo check -p foo` 只檢查 foo 是 incremental 開發常用招。

## 常見陷阱

1. **`mod foo` vs `use foo`** — `mod foo` **宣告**檔案結構，`use foo` 只是縮短路徑。新手常忘 `mod`。
2. **module path 與 file path 不一定一樣** — `mod auth;` 找的是 `auth.rs` **或** `auth/mod.rs`，跟你的目錄組織有關。
3. **`use` 與 `pub use` 不同** — 前者只在當前 scope，後者讓使用者也看得到。
4. **`pub` enum variant 的可見度**：variant 是否可見跟 enum 本身的可見度綁定。
5. **workspace virtual manifest** — root 的 `Cargo.toml` 若**沒有** `[package]`、只有 `[workspace]`，就是 virtual workspace。本系列就是。

## 練習

1. 把本章的 `auth` module 拆成 `src/auth.rs` + `src/auth/password.rs`，編譯確認等價。
2. 把 `math::private_helper` 改成 `pub(super)`，從 `math` 同層 module 試呼叫。
3. 建一個 workspace：root + 兩個 member crate（`lib_a` 與 `bin_b`），讓 `bin_b` 依賴 `lib_a`（用 `path`）。
4. 在 `lib_a` 寫個 `pub fn add` 與 internal `fn helper`，從 `bin_b` 呼叫並確認 `helper` 不可見。

## 延伸閱讀

- [The Rust Book — Ch 7 Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [The Cargo Book — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
