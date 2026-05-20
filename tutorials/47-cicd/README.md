# 47. CI/CD

> 範圍：GitHub Actions、fmt/clippy/test、cache、安全掃描、跨平台、docker push

## 一個能跑的 pipeline 要做什麼

1. **lint**：fmt、clippy（fail on warning）
2. **test**：nextest、doc test、跨平台
3. **security**：cargo-audit、cargo-deny
4. **build**：release binary、docker image
5. **deploy**：push image、滾動更新（K8s / 雲端）

每個 job **失敗都該擋 merge**。

## 範例 workflow

本章 `ci.yml.example` 是完整 4-job pipeline，重點：

### 1. cache：必須

```yaml
- uses: Swatinem/rust-cache@v2
```

Rust 編譯慢，沒 cache 每 PR 5–10 分鐘。`Swatinem/rust-cache` 是社群標準，自動處理：
- `target/` 目錄
- `~/.cargo/registry`
- 用 `Cargo.lock` hash 做 cache key

第一次 cold 3–5 min，後續 30 秒內。

### 2. fail fast：把 lint 放最前

```yaml
RUSTFLAGS: -D warnings
```

Lint 失敗最快，不必等 test 跑 5 分鐘才知道 unused import。

### 3. matrix：跨平台

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
```

如果你會出 Windows / macOS / Linux 三平台 binary，CI 要全跑。**只跑 Linux 但發 Windows binary 是常見地雷**。

### 4. nextest：更快的 test runner

```yaml
- uses: taiki-e/install-action@nextest
- run: cargo nextest run --workspace
```

平行更激進，輸出可讀。Doc test 不支援 → 另外跑 `cargo test --doc`。

## 安全掃描

### cargo-audit

掃 `Cargo.lock` 對 RustSec advisory DB：

```bash
cargo audit
# vulnerability found in `time 0.1.x` → bump 版本
```

### cargo-deny

更廣：licenses、duplicate dep、來源限制：

```toml
# deny.toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
deny = ["GPL-3.0"]
[bans]
multiple-versions = "warn"
```

### supply chain

- **`cargo-vet`** / **`cargo-crev`**：對依賴做 review trail
- **lockfile bot**：自動化 `cargo update` PR（dependabot 或 renovate）

## Pre-commit

本地擋住 push 失敗：

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: fmt
      - id: cargo-check
      - id: clippy
```

或 husky / git hook 跑：

```bash
#!/bin/sh
cargo fmt --check && cargo clippy -- -D warnings
```

## release pipeline

tag-driven 比較乾淨：

```yaml
on:
  push:
    tags: ["v*"]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release
      - uses: softprops/action-gh-release@v2
        with:
          files: target/release/myapp
```

跨平台 binary：用 `cross` crate 或 matrix build 各 target。

## cross-compile

`cross` 用 docker 幫你跑 target toolchain：

```bash
cargo install cross
cross build --target aarch64-unknown-linux-gnu --release
```

支援 ARM、musl、Windows、各種 Linux。

## 部署模式

### 1. push image → K8s rolling

```
CI → ghcr.io/org/app:abc123 → kubectl set image
```

### 2. GitOps（Argo CD / Flux）

```
CI → update tag in repo → Argo 看到 → sync 到 cluster
```

### 3. PaaS

Fly.io、Railway、Render：push 自動 deploy，省事但定制度低。

## 訊息與通知

- **PR check** 失敗自動評論 fail summary（用 `sccache`、`coverage` action 上傳）
- **deploy 完成** 推 Slack（webhook action）
- **release 出 nighly / canary** 自動 ping QA channel

## 常見陷阱

1. **沒 cache** — 每次 5+ 分鐘 lint，吃光配額。
2. **incremental compile 上 cache** — 不要 cache `incremental/`，會越長越大。
3. **`RUSTFLAGS=-D warnings` 在本地不開** — CI 才開，本地寫 code 期間吵。
4. **`stable` toolchain pin** — `dtolnay/rust-toolchain@stable` 跟版本變動；要鎖到具體版本就用 `1.83.0`。
5. **`sccache` 跨 PR 共用** — 不同 dependency hash 共用會中毒；用 PR 為 key。
6. **forget submodule** — `actions/checkout` 要 `submodules: recursive`。
7. **secret 漏 log** — 用 `secrets.*` 不要 echo 出來；`add-mask` 保險。
8. **docker layer cache GHA quota** — 大量 PR 會吃 cache 配額；定期清理或用外部 registry。

## 練習

1. 改 `ci.yml.example` 加 `cargo-deny`、`cargo-llvm-cov` 算 coverage。
2. 加 release pipeline：tag v* 自動 build 三平台 binary + release。
3. 加 docker image scan（trivy / scout）擋 critical CVE。
4. 評估 sccache 對你 codebase 編譯時間的影響。

## 延伸閱讀

- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)
- [cargo-deny book](https://embarkstudios.github.io/cargo-deny/)
- [cross](https://github.com/cross-rs/cross)
