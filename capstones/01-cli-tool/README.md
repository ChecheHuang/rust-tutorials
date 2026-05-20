# Capstone 1 — CLI 工具

整合 Part 1–3 所學的小型實戰專案。完成後你應該對 Rust 的 ownership / error handling / iterator / 並發 / 測試流程都有實際手感。

> 上層規劃見 [../../ROADMAP.md](../../ROADMAP.md)。

## 題目（三選一，或自訂同等規模）

### A. `mini-rg` — 簡化版 ripgrep
- 遞迴搜尋目錄中符合正規式的行
- 支援 glob 過濾（`--include "*.rs"`）、`.gitignore`
- 平行掃描多個檔案，輸出彩色高亮
- 大檔案以 mmap 處理

### B. `log-lens` — 結構化 log 分析器
- 自動偵測格式：JSON、logfmt、nginx access log
- 支援 filter（`--level error --since 1h`）
- 支援 aggregate（每分鐘錯誤數、top error message）
- `--follow` 模式像 `tail -f` 即時跟讀

### C. `git-stats` — Git 統計報告
- 從 `git log` 統計 commit 頻率、作者貢獻、檔案熱度
- 輸出 markdown 報告或 ASCII 圖表
- 支援時間區間（`--since 2026-01-01`）
- 處理 monorepo（按子目錄分組）

## 學習目標

- 用 `clap` derive macro 設計 subcommand / flag / env
- 用 `indicatif` 顯示進度與 spinner
- 用 `serde` + `serde_json` / `csv` 序列化輸出
- 用 `rayon`（CPU bound）或 `tokio`（IO bound）並發
- 用 `thiserror` 設計錯誤型別、`anyhow` 在 `main` 收尾
- 用 `tracing` + `tracing-subscriber` 輸出結構化 log
- 用 `assert_cmd` + `predicates` 寫整合測試

## 前置章節

| 必須 | 章節 |
|------|------|
| 必須 | Part 1（01–10）全部 |
| 必須 | 11 generics、12 traits、15 iterators、19 advanced-errors |
| 必須 | 20 modules-workspace、21 testing、22 cargo-advanced、23 tracing-log |
| 建議 | 26–27（如果選擇 IO bound 並發） |

## 評估標準

- [ ] `cargo install --path .` 可全域安裝
- [ ] `--help` 訊息結構清楚，每個 flag 有描述
- [ ] 處理 broken pipe（如 `mini-rg foo | head` 不會 panic）
- [ ] 不吞錯誤：所有錯誤透過 `Result` 傳到 `main`，附帶上下文
- [ ] 至少 5 個整合測試（含成功路徑、錯誤路徑、edge case）
- [ ] 跨平台：Windows、macOS、Linux 都能 build + run
- [ ] README 含 3 個以上的使用範例
- [ ] CI（GitHub Actions）跑 `fmt --check`、`clippy -- -D warnings`、`test`、`build --release`

## 建議實作節奏

1. **第 1 週** — 骨架：cargo new、clap 設計、定主資料結構與 `Error` enum
2. **第 2 週** — 核心邏輯（單執行緒）、基本測試
3. **第 3 週** — 加並發、progress UI、彩色輸出
4. **第 4 週** — CI、跨平台 release、README + demo gif

## 本目錄附帶的範例實作：`mini-rg`

本目錄已經提供題目 A 的**參考實作骨架**，方便你做 hands-on 對比。可以直接跑、改、擴。

### 結構

```
src/
  main.rs        ← 啟動、workers、印 thread 編排
  args.rs        ← clap derive
  walker.rs      ← ignore crate 走檔案樹（含 .gitignore）
  search.rs      ← 對每個檔案掃 regex
  printer.rs     ← termcolor 高亮、檔名/行號著色
tests/
  integration.rs ← assert_cmd + tempfile 整合測試
```

### 跑起來

```bash
cd capstones/01-cli-tool
cargo run --release -- "fn main" .
cargo run --release -- -i "TODO|FIXME" src/
cargo run --release -- -g "*.rs" "unsafe" .
cargo run --release -- -l "fn" .          # 只列檔名
```

### 設計重點

- **bounded channel** + **多 worker** + **印 thread**：解耦 walk、search、output，避免 stdout 鎖造成阻塞
- **regex on `&[u8]`**（不是 `&str`）：規避 UTF-8 strict 失敗
- **binary 偵測**：前 8K 含 NUL → skip
- **atty + termcolor**：管線時自動關顏色（`mini-rg foo | head` 不會吐 ANSI escape）
- **`ignore` crate**：尊重 `.gitignore` / `.ignore` / hidden（除非 `--hidden`）

### 進階練習（自己加）

1. 加 mmap 支援大檔（看 `memmap2` crate）— 注意 mmap 內檔案被改會 SIGBUS
2. 加 `--multiline` 跨行 regex 支援
3. 加 ANSI rainbow 模式（每個 match 不同色）
4. profile + flamegraph 比較 sync threading vs `rayon` vs `tokio` 對 CPU/IO bound 的差異
5. 加 `--context N`（顯示前後 N 行）
