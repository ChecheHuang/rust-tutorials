# 故事版：校刊社的自動審稿關卡 📰

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「校刊社每篇稿子都要過好幾道關卡才能印出來」的故事，幫你「感覺」出 CI/CD、GitHub Actions 的 job、fmt 和 clippy、`-D warnings`、快取、matrix 跨平台測試、nextest、cargo-audit / cargo-deny、`needs` 與 docker push、pre-commit、tag release 和部署模式是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 以前：社長一個人看稿

陽光國小的**校刊社**每個月出一期校刊。以前，社員交稿後，全部堆在社長小欣的桌上，她一篇一篇看：

字有沒有對齊？有沒有錯字？引用的書是不是可靠的？……

小欣常常看漏，也常常看到半夜。有一次，一篇有錯字的稿子就這樣被印了五百份。

後來指導老師說：「我們做一條**自動審稿線**。每次有人交稿或改稿，稿子就**自動**過一道一道關卡。哪一關沒過，就**不准**放進正式那一期。」

> 這就是 **CI/CD**：每次 push 或開 pull request，就自動跑 lint、test、安全掃描、build、deploy。本章的 `ci.yml.example` 用 GitHub Actions 寫成，`on: push: branches: [main]` 和 `pull_request:` 就是「交稿或改稿時觸發」。**任何一關失敗都該擋住 merge。**

---

## 第一關：排版和文法，最快的先跑

審稿線的**第一關**最簡單也最快：

- **排版小組**：段落有沒有空兩格、標點是不是全形。
- **文法老師**：「這句話主詞不見了」「這個詞重複了三次」。

文法老師很嚴格：只要有**一個小提醒**，就算整篇不通過。社員一開始抱怨，後來發現這樣反而不會累積一堆小毛病。

先跑這關，是因為它**一分鐘就跑完**。沒必要讓一篇連標點都錯的稿子，先排隊等很久的試讀，最後才被退回。

> 這就是 **`lint` job 與 fail fast**：`cargo fmt --all --check`（排版）、`cargo clippy --workspace --all-targets -- -D warnings`（文法老師）。全域 `env` 的 `RUSTFLAGS: -D warnings` 讓**警告變成錯誤**。注意：這個在 CI 開就好，本地寫程式時一直被擋會很吵。

---

## 不用每次重新磨墨

以前每次審稿，排版小組都要**從頭磨墨、裁紙、找字典**，準備就要十分鐘。

後來他們把磨好的墨、裁好的紙、翻好的字典**收進櫃子**，櫃子上貼一張標籤，寫著「**這是照第幾版參考書目準備的**」。下次只要書目沒變，打開櫃子就能直接用，三十秒就準備好。

不過有一樣東西**不收**：桌上那堆**改到一半的草稿碎紙**。那堆東西只會越堆越多，留著只會塞爆櫃子。

> 這就是 **`Swatinem/rust-cache@v2`**：快取 `target/` 和 `~/.cargo/registry`，用 `Cargo.lock` 的 hash 當 cache key。第一次 cold 3–5 分鐘，之後 30 秒內。陷阱：不要快取 `incremental/`，它會越長越大。

---

## 第二關：拿到三台不同的印表機試印

校刊會印在三種地方：學校的**大印表機**、圖書館的**彩色印表機**、還有**行政室的舊印表機**。

有一次，稿子在大印表機上印得好好的，到了舊印表機卻**整排字跑掉**。從此以後，每篇稿子都要在**三台印表機上各試印一次**，三台都沒問題才算過。

試讀員也換成一組**超快的閱讀小隊**，大家一起分頭讀，誰讀到哪裡出錯，一目了然。不過這個小隊**不讀**稿子旁邊的「範例小框框」，那部分要另外請一個人讀。

> 這就是 **`test` job 的 matrix**：`strategy.matrix.os: [ubuntu-latest, macos-latest, windows-latest]`，`runs-on: ${{ matrix.os }}`。**只在 Linux 測、卻發 Windows binary 是常見地雷。** 測試用 `taiki-e/install-action@nextest` 安裝後 `cargo nextest run --workspace`（平行更積極、輸出好讀），但 nextest 不跑 doc test，所以另外 `cargo test --doc --workspace`。本章 `src/main.rs` 裡的 `add_works` 就是給 CI 跑的小測試。

---

## 第三關：引用的書可不可靠

社員寫報告時常常引用別的書。可是有些書後來被發現**內容有錯**，被列進了全國的「**問題書單**」。

第三關的老師會把每篇稿子的**參考書目**拿去對照問題書單：「你引用的這本第 0.1 版，已經被發現有錯了，請改引新版。」

另一位老師管得更廣：「這本書的**授權**我們能不能用？」「你同一本書怎麼引用了**兩個不同版本**？」

> 這就是 **安全掃描**：`audit` job 用 `taiki-e/install-action@cargo-audit` 安裝後跑 `cargo audit --deny warnings`，把 `Cargo.lock` 對照 **RustSec advisory DB**。`cargo-deny` 管得更廣：`deny.toml` 的 `[licenses] allow / deny`、`[bans] multiple-versions`。還有 `cargo-vet` / `cargo-crev` 做依賴審查紀錄，dependabot / renovate 自動送 `cargo update` 的 PR。

---

## 最後一關：三關都過了，才送去印刷廠

前三關都過了，稿子才能進到**最後一關**：正式**印刷**，並送到學校的**發行處**。

而且只有「**正式那一期**」的稿子才會送去印；還在討論中的草稿，就算過了前三關，也只是證明它沒問題，不會真的印出來。

印好的每一疊校刊上都會貼兩張標籤：「**最新一期**」，以及「**第 abc123 號稿**」——這樣以後要找回某一份，靠號碼就找得到。

印刷廠的**鑰匙**，由老師保管在保險箱裡，審稿線要用時才借出來，**絕不寫在稿子上**。

> 這就是 **`docker` job**：`if: github.ref == 'refs/heads/main'` 只有 main 才跑；`needs: [lint, test, audit]` 要前三關都過；`docker/login-action@v3` 用 `secrets.GITHUB_TOKEN` 登入 `ghcr.io`（保險箱裡的鑰匙，`permissions: packages: write`）；`docker/build-push-action@v6` 推出 `:latest` 和 `:${{ github.sha }}` 兩個 tag，並用 `cache-from: type=gha` 重用 layer。Secret 千萬不要 echo 到 log。

---

## 交稿前自己先檢查

社員小哲老是被第一關退稿，覺得很丟臉。

他學會一招：**交稿之前**，自己先在家裡用排版尺量一量、把文法提醒清單看一遍。這樣交出去的稿子，第一關幾乎都會過。

> 這就是 **pre-commit**：在本地用 `.pre-commit-config.yaml`（例如 `doublify/pre-commit-rust` 的 `fmt`、`cargo-check`、`clippy` hooks）或 git hook 跑 `cargo fmt --check && cargo clippy -- -D warnings`，在 push 前就擋下問題。

---

## 正式期數和外語版

每到月底，社長會在當期最後一篇稿子上**蓋一個期數章**：「**第 7 期**」。一蓋章，審稿線就自動把這一期整理好，放到學校網站讓大家下載。

學校還有外籍同學，所以每期也要出**英文版、日文版**。校刊社沒有會這些語言的人，就請一位**翻譯小幫手**，他帶著自己的整套字典工具過來，幫忙產出各種版本。

> 這就是 **tag-driven release 與 cross-compile**：`on: push: tags: ["v*"]` 觸發，`cargo build --release` 後用 `softprops/action-gh-release@v2` 上傳 binary。跨平台用 matrix 各自 build，或 `cross build --target aarch64-unknown-linux-gnu --release`（`cross` 用 docker 帶好各 target 的 toolchain）。

---

## 怎麼送到大家手上

印好的校刊有三種發法：

1. **直接換**：發行處收到新一期，就把各班信箱裡的舊校刊**一格一格換成新的**。
2. **看公告板**：審稿線只在走廊的公告板上寫「**本期：第 abc123 號**」。發行處的值日生一直盯著公告板，看到變了，就自己去把信箱裡的校刊換成那一號。
3. **交給外包**：直接交給附近的影印店，影印店幫你印、幫你發。很省事，但你能要求的細節比較少。

> 這就是 **部署模式**：(1) push image 後 `kubectl set image` 做 rolling update（下一章）；(2) **GitOps**：CI 只更新 repo 裡的 image tag，**Argo CD / Flux** 監看 repo 再 sync 到 cluster；(3) **PaaS**：Fly.io、Railway、Render，push 就自動部署，省事但定制度低。

---

## 指導老師的叮嚀

1. **字典要指定版本**。「用最新的字典」聽起來很好，但字典一改版，昨天過關的稿子今天突然不過。
2. **附錄別忘了一起交**。有些稿子附了另一份小冊子，沒一起拿進來，審稿線根本看不到。
3. **櫃子別塞爆**。學校給的櫃子空間有限，舊的準備材料要定期清掉。

> 這就是 **常見陷阱**：`dtolnay/rust-toolchain@stable` 會跟著版本變動，要鎖就用具體版本如 `1.83.0`；`actions/checkout` 要 `submodules: recursive`；GitHub Actions 的 cache 有配額，大量 PR 的 docker layer cache 要定期清理。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust / CI 概念 |
|------|------|
| 自動審稿線 | **CI/CD pipeline**（`ci.yml.example`） |
| 交稿、改稿就觸發 | `on: push` / `pull_request` |
| 每一道關卡 | **job**：`lint`、`test`、`audit`、`docker` |
| 排版小組 | **`cargo fmt --all --check`** |
| 嚴格的文法老師 | **`cargo clippy ... -D warnings`** |
| 一個小提醒就不過 | **`RUSTFLAGS: -D warnings`** |
| 最快的關卡先跑 | **fail fast** |
| 櫃子裡磨好的墨 | **`Swatinem/rust-cache`** |
| 草稿碎紙不收 | 不快取 `incremental/` |
| 三台印表機試印 | **matrix**：ubuntu / macos / windows |
| 超快閱讀小隊 | **`cargo nextest run`** |
| 範例小框框另外讀 | **`cargo test --doc`** |
| 問題書單 | **RustSec** / `cargo audit` |
| 授權和重複版本檢查 | **`cargo-deny`** |
| 前三關都過才印 | **`needs: [lint, test, audit]`** |
| 只有正式那一期才印 | `if: github.ref == 'refs/heads/main'` |
| 「最新一期」「第 abc123 號」 | image tag `:latest` / `:${{ github.sha }}` |
| 保險箱裡的印刷廠鑰匙 | **`secrets.GITHUB_TOKEN`** |
| 交稿前自己先檢查 | **pre-commit** hook |
| 蓋期數章 | **tag-driven release**：`tags: ["v*"]` |
| 翻譯小幫手 | **`cross`** 交叉編譯 |
| 盯著公告板的值日生 | **GitOps**（Argo CD / Flux） |
| 交給影印店 | **PaaS** |

---

## 一句話記住

> **CI/CD 是校刊社的自動審稿線：每次交稿就自動過關卡——最快的排版文法先跑（fmt / clippy），再到三台印表機試印（matrix test）、查參考書目（audit），三關全過、而且是正式那一期，才送去印刷廠（docker push）。**
