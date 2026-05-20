# 01. Cargo 與開發環境

> 範圍：rustup、cargo、edition、rust-analyzer、cargo fmt/clippy、Hello World

## 為什麼從 Cargo 開始

Rust 與 C/C++ 最大的工程差別之一是「**工具鏈統一**」。其他語言常常 build system、套件管理、formatter、linter、文件產生各自為政；Rust 從 day 1 就是 `cargo` 一條鞭。學語言之前先把工具鏈摸熟，後面所有章節都靠它。

## rustup — Rust 的版本管理器

```bash
# 安裝（Linux/macOS）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Windows：到 https://rustup.rs 下載 rustup-init.exe

rustup update                      # 更新到最新 stable
rustup default stable              # 設定預設 channel
rustup component add rustfmt clippy rust-analyzer
rustup target add x86_64-pc-windows-gnu   # cross-compile target
```

每個專案可以用 `rust-toolchain.toml` 釘住版本，避免 contributor 之間版本漂移：

```toml
[toolchain]
channel = "1.75"
components = ["rustfmt", "clippy"]
```

## cargo — 萬用工具

| 指令 | 用途 |
|------|------|
| `cargo new my-app` | 建立 binary crate |
| `cargo new my-lib --lib` | 建立 library crate |
| `cargo build` / `cargo build --release` | 編譯（debug / release） |
| `cargo run` | 編譯後執行 |
| `cargo check` | 只檢查能否編譯（**最快**，IDE 用這個） |
| `cargo test` | 跑測試 |
| `cargo fmt` | 格式化（rustfmt） |
| `cargo clippy` | lint |
| `cargo doc --open` | 產生並開啟文件 |
| `cargo add serde` | 加 dependency（也可手改 Cargo.toml） |
| `cargo update` | 更新 Cargo.lock 內版本 |

## edition — Rust 的「語言版次」

Rust 每 3 年發一個 edition（2015 / 2018 / 2021 / 2024），引入不相容語法變更，但**舊 edition 永久支援**——同個 binary 可以混用不同 edition 的 crate。本系列統一 **edition 2021**。

```toml
[package]
name = "my-app"
edition = "2021"
```

## IDE 設定

- **VS Code**：裝 `rust-analyzer` 擴充（不是舊的 RLS）
- **JetBrains**：RustRover（免費）或 IntelliJ + Rust plugin
- **vim/neovim/emacs**：透過 LSP 接 `rust-analyzer`

`rust-analyzer` 是事實標準，啟動稍慢（吃 1-2 GB RAM），但 inlay hints、type info、refactor 都是頂級水準。

## 程式碼解說

`src/main.rs` 透過 `env!` 宏在**編譯期**讀 Cargo 設定：

```rust
let name = env!("CARGO_PKG_NAME");
```

`env!` 是 macro，找不到變數會直接編譯失敗——比 runtime 的 `std::env::var` 安全。

`println!("Hello from {name} v{version}!")` 用了 **edition 2021 的 captured identifier**——直接把 scope 內變數塞進 format string。2018 之前要寫 `println!("Hello from {}", name)`。

## 跑跑看

```bash
cd tutorials/01-cargo-setup
cargo run
```

預期輸出：

```
Hello from ch01-cargo-setup v0.1.0!
Edition: 2021
Run `rustc --version` to see your toolchain.
```

## 對照 TypeScript

TS 端的工具鏈是「**多套並存**」（npm/pnpm/yarn 管套件、tsc/swc/esbuild 編譯、eslint/prettier 各管一邊、jest/vitest 測試）。Rust 端是「**cargo 一條鞭**」——同一個指令做完所有事。

### 對照表

| 動作 | TypeScript/Node | Rust |
|---|---|---|
| 版本管理 | `nvm` / `volta` / `fnm` | `rustup` |
| 釘住版本 | `.nvmrc` / `volta` 欄位 | `rust-toolchain.toml` |
| 套件管理 | `npm` / `pnpm` / `yarn` | `cargo`（內建） |
| 套件設定 | `package.json` | `Cargo.toml` |
| lock 檔 | `package-lock.json` / `pnpm-lock.yaml` | `Cargo.lock` |
| 安裝依賴 | `npm install` / `pnpm add x` | `cargo add x` |
| 跑專案 | `npm start` / `node dist/index.js` | `cargo run` |
| 編譯 | `tsc` / `tsup` / `esbuild` | `cargo build` |
| 「只檢查不產出」 | `tsc --noEmit` | `cargo check`（最快、IDE 用） |
| 測試 | `npm test`（背後跑 jest/vitest） | `cargo test`（內建 harness） |
| 格式化 | `prettier --write` | `cargo fmt` |
| Lint | `eslint .` | `cargo clippy` |
| 文件 | `typedoc` | `cargo doc --open` |
| Monorepo | pnpm workspace / nx / turbo | `[workspace]` in Cargo.toml |
| IDE LSP | `typescript-language-server` | `rust-analyzer` |
| 預設快取 | `node_modules/`（每專案一份） | `target/`（每 workspace 一份） |

### 程式碼對照

```json
// package.json
{
  "name": "my-app",
  "version": "0.1.0",
  "scripts": { "start": "node dist/index.js", "test": "vitest" },
  "dependencies": { "axios": "^1.6.0" }
}
```

```toml
# Cargo.toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = "0.11"
```

### 心智模型差異

1. **不必組裝工具鏈**。TS 起新專案常要選 ts-node / tsx / swc / esbuild、ESLint config、Prettier config——Rust 全是 cargo 內建，少了「拼裝」這個階段。
2. **`cargo check` 是新習慣**。TS 開發者通常邊存邊讓 IDE 跑 tsc，Rust 用 `cargo check` 做型別檢查、`cargo build` 才真的編譯——前者**遠快**。
3. **lock 檔的 commit 策略不同**。Node 一般 binary / library 都 commit lock；Rust 慣例 **binary commit、library 不 commit**（讓使用者的版本主導）。
4. **edition 不等於 version**。TS 沒有對應概念，最接近的是 `target: "es2022"`。edition 是**語法層**版次，不影響 ABI。
5. **debug vs release 差距巨大**。TS 的 `tsc` 跑出來不分等級；Rust `cargo build` 預設 debug（沒最佳化、跑得慢），上線必須 `--release`。差個 10–100×。
6. **`target/` 比 `node_modules/` 大很多**。Rust 編譯產物含 incremental cache + 每個 dep 的 .rlib，幾百 MB 起跳；CI 強烈建議 cache `~/.cargo` 跟 `target/`。

## 常見陷阱

1. **Windows 上 cargo build 慢**：防毒軟體掃 `target/`。把 workspace 的 `target/` 加進排除清單可加速 5-10 倍。
2. **debug vs release 行為不同**：debug build 預設整數溢位 panic，release build wrap-around。第 02 章詳述。
3. **rust-analyzer 吃 RAM**：大型 workspace 可達 2-3 GB；可關掉 `proc-macro` server 換速度。
4. **Cargo.lock 要不要 commit**：binary crate **要**、library crate **不要**。

## 練習

1. 用 `cargo new` 在本機別處建一個專案，跑 `cargo add serde` 後看 Cargo.toml 與 Cargo.lock 變化。
2. 對本章執行 `cargo clippy -- -W clippy::pedantic`，看會被警告什麼。
3. 用 `cargo doc --open` 開 std 文件，找 `String::new` 簽章。

## 延伸閱讀

- [The Rust Book — Ch 1](https://doc.rust-lang.org/book/ch01-00-getting-started.html)
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [rust-analyzer manual](https://rust-analyzer.github.io/manual.html)
