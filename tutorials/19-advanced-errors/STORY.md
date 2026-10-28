# 故事版：宅配公司的問題單與大信封 📦

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「宅配公司的分站填問題單、總部客服把問題裝進大信封再貼便利貼」的故事，幫你「感覺」出 `thiserror` 與 `anyhow` 的分工、`#[error]`、`#[from]`、`?`、`.context()` / `.with_context()`、錯誤鏈 `chain()`、backtrace、何時 panic 何時回 `Result`。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 分站的勾選式問題單

小凱在宅配公司的**總部客服**上班。公司底下有好幾個**分站**，像「會員資料站」和「設定站」。

會員資料站送不到東西時，會填一張**勾選式問題單**，上面只有幾個固定的格子：

- ☐ 連線不通：（原因）
- ☐ 查詢失敗：（原因）
- ☐ 倉庫門打不開
- ☐ 找不到 {幾號} 會員

每一格旁邊都印好一句**範本句子**，填好就能直接唸出來。

> 這就是 **`thiserror`**：`#[derive(Debug, Error)] pub enum DbError { Connection(String), Query(String), Io(..), NotFound { id: u64 } }`。`#[error("user {id} not found")]` 就是範本句子，變成 `Display`；`{0}` 是第 0 個欄位，`{id}` 是具名欄位。手寫要 20 行的 `Display`、`Error`、`From`，一個 derive 就生好。

---

## 看格子就知道怎麼處理

小凱接到會員資料站的問題單，**一眼就看到勾的是哪一格**：

- 勾「找不到 0 號會員」→ 請客人重新確認會員編號。
- 勾「連線不通」→ 等一下再試。

因為格子是固定的，小凱可以照格子**分類處理**。

> 這就是 **caller 可以 pattern match**：`fetch_user(0)` 回 `Err(DbError::NotFound { id: 0 })`，呼叫端寫 `matches!(err, DbError::NotFound { id: 0 })` 或 `match` 每個 variant 決定不同處理。這是 library 用 thiserror 的最大理由。

---

## 自動轉單機

設定站在讀「港口號碼」的時候，常常遇到**別家公司格式的錯誤紙條**：「這不是數字」。

設定站在問題單上裝了一台**自動轉單機**：別家的「這不是數字」紙條一塞進去，就自動變成自己問題單上的「解析錯誤」那一格，而且**原本的紙條夾在後面**。

於是設定站寫流程時，只要在可能出錯的地方畫一個「**有問題就往上交**」的小記號就好，不用自己抄單。

> 這就是 **`#[from]` + `?`**：`ConfigError::Parse(#[from] std::num::ParseIntError)` 自動產生 `From<ParseIntError> for ConfigError`。在 `load_port()` 裡 `let port: u16 = s.parse()?;`，`?` 會自動把 `ParseIntError` 轉成 `ConfigError` 往上丟。

---

## 總部的萬用大信封

總部客服每天收到**各種分站、各種格式**的問題單。小凱不想為每一種單子各做一個抽屜。

總部有一種**萬用大信封**：不管哪一站的問題單，**通通裝得進去**，也不用先轉格式。

缺點是：信封封起來之後，小凱**看不到裡面勾的是哪一格**，只能把它整封交給主管看，或是特地拆開來確認（而且要先猜對是哪一種單）。

> 這就是 **`anyhow`**：`anyhow::Result<T>` = `Result<T, anyhow::Error>`，任何實作 `std::error::Error + Send + Sync + 'static` 的錯誤都能用 `?` 直接裝進去，**不用寫 `From`**。代價是 caller 不能直接 match，只能 log 或 `downcast`。

---

## 貼在信封上的便利貼

光是一封「找不到 0 號會員」，主管看了會問：「這是在**做什麼**的時候發生的？」

所以小凱每轉交一次，就在信封上**貼一張便利貼**：「在**第一次查會員**的時候」、「在**讀港口號碼**的時候」。

有些便利貼要寫上地址，寫起來比較花時間。小凱學聰明了：**只有真的出問題，才動手寫那張便利貼**；沒出問題就不浪費時間。

> 這就是 **`.context()` / `.with_context()`**：`application_main()` 裡 `fetch_user(0).context("during initial fetch")?`、`load_port().context("failed to load port")?`。`load_config` 用 `.with_context(|| format!("reading config from '{path}'"))`，閉包**只在出錯時才執行** `format!`。

---

## 一層一層撕開便利貼

主管拿到大信封，有兩種看法：

- **只看最上面那張便利貼**：「在第一次查會員的時候出問題」。
- **一層一層往下看**：最上面的便利貼 → 下面那張 → 最底下的原始問題單「找不到 0 號會員」。

讀設定檔失敗時也一樣：最上面寫「在讀 /nonexistent.toml 的時候」，最底下是「沒有這個檔案」。

> 這就是 **錯誤鏈**：`println!("Error: {e}")` 只印最上層；`{e:?}` 印整條 chain（`Caused by:`）；`for (i, cause) in e.chain().enumerate()` 逐層走。每一層靠 `source()` 指向下一層，thiserror 的 `#[from]` 和 anyhow 的 `context` 都會幫你接好。

---

## 分站不要用大信封

有一天，會員資料站偷懶，也開始用**萬用大信封**交問題。

結果別的部門收到之後，**完全不知道勾的是哪一格**，沒辦法分類處理，只能說「呃，有問題」。

總部規定：「**分站對外**一律用勾選式問題單；**只有總部最上層**才用大信封收齊、加便利貼、交給主管。」

> 這就是 **library 用 thiserror、application 用 anyhow**：library 對外 API 回具體的 `DbError` / `ConfigError`，讓使用者能分類；binary 的 `main` 或高層函式用 `anyhow::Result` 收齊一切、加 context、統一輸出。兩者可以共存。`main` 也可以直接回 `anyhow::Result<()>`，失敗時自動印出 chain。

---

## 行車紀錄器

如果主管想知道「貨車**一路上經過哪些路口**才出事」，便利貼就不夠了，要看**行車紀錄器**。

行車紀錄器平常不開，因為很佔空間；要查的時候，出發前把開關打開就有。另外也有別家做的**彩色版信封**，印出來比較好看，用法幾乎一樣。

> 這就是 **backtrace** 與 **eyre**：設定 `RUST_BACKTRACE=1` 後，`anyhow::Error` 的 `{:?}` 會附上 backtrace（std 的 `Error::backtrace` 目前還不穩定）。`eyre` / `color-eyre` 是 anyhow 的兄弟，API 幾乎一樣，可以自訂漂亮的 reporter。

---

## 機器壞了 vs 送不到

小凱最後學到一件事：

- **送不到**（地址錯、沒人在家）是**正常會發生的事**，要填單往上交，讓人處理。
- **貨車輪子掉了**（照理說絕對不會發生）是公司內部的大問題，**直接停車大喊**。

還有，千萬不要把原本的問題單**丟掉**再大喊「完蛋了」，這樣大家就不知道一開始發生了什麼。

> 這就是 **panic vs `Result`**：真實、值得 caller 處理的錯誤用 `Result` + thiserror；內部不變式被破壞（bug）才 `panic!` / `unreachable!`。prototype 可以 `unwrap()` / `expect()`。不要 `unwrap_or_else(|_| panic!(...))` 吞掉原始錯誤。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 勾選式問題單 | **thiserror** 定義的 `enum DbError` / `ConfigError` |
| 每格的範本句子 | `#[error("...")]` → `Display` |
| 看格子分類處理 | `match` / `matches!(err, DbError::NotFound { .. })` |
| 自動轉單機，原紙條夾後面 | **`#[from]`** → `From` impl + `source()` |
| 「有問題就往上交」記號 | **`?`** |
| 萬用大信封 | **`anyhow::Error`** / `anyhow::Result` |
| 封起來看不到勾哪格 | 不能直接 match，只能 `downcast` |
| 便利貼 | **`.context("...")`** |
| 出問題才寫的便利貼 | **`.with_context(\|\| format!(...))`** |
| 只看最上面 / 一層層撕開 | `{e}` / `{e:?}`、**`e.chain()`** |
| 分站用問題單、總部用大信封 | **library 用 thiserror、application 用 anyhow** |
| 行車紀錄器 | **backtrace**（`RUST_BACKTRACE=1`） |
| 彩色版信封 | `eyre` / `color-eyre` |
| 送不到 vs 輪子掉了 | `Result` vs `panic!` |

---

## 一句話記住

> **分站對外填勾選式問題單（thiserror），讓人能分類處理；總部用萬用大信封（anyhow）收齊一切，每轉一手就貼一張便利貼（`.context()`），主管一層層撕開（`chain()`）就知道「在做什麼時、因為什麼」出了問題。**
