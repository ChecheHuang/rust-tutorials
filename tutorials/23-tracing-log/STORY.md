# 故事版：機場塔台的飛行紀錄簿 ✈️

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「機場裡每個人回報狀況、塔台記錄員把它們寫進紀錄簿」的故事，幫你「感覺」出 `tracing` 的 event、結構化欄位、level、subscriber、`RUST_LOG` / `EnvFilter`、span、`#[instrument]`、`%` 與 `?` 的寫法、async 下的 span、JSON 輸出。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 隨口喊一句 vs 填好格子

小芸在機場塔台當**記錄員**。以前大家回報狀況都是**隨口喊一句**：「欸，alice 那班 GET 那個 /api/users 好像 200 喔！」

小芸要找「所有狀態是 200 的回報」時，只能一句一句讀，非常辛苦。

後來塔台發了一張**回報表**，上面有格子：「方法：GET」「路徑：/api/users」「狀態：200」，最後再寫一句話「處理完成」。這樣小芸只要看「狀態」那一格，馬上就能找出來。

> 這就是 **event 與結構化欄位**：`info!(method = "GET", path = "/api/users", status = 200, "request handled");`。前面是 `key = value` 欄位，最後是訊息。`println!` 沒有等級也沒有結構；`log` crate 有等級但還是純字串。`info!(user, "logged in")` 是 `user = user` 的簡寫。

---

## 回報有輕重

塔台的回報分五種**緊急程度**，從輕到重：

- **細節**：每一陣風、每一次無線電。
- **除錯**：「我正在查這位乘客。」
- **一般**：「班機起飛了。」
- **注意**：「這位乘客比較特別，要留意。」
- **緊急**：「出事了，要有人處理！」

平常塔台只記「一般」以上，不然紀錄簿一下就寫滿了。

> 這就是 **level**：`trace < debug < info < warn < error`。`fetch_user` 裡 `debug!("looking up user")`、`info!(%name, "found")`、`warn!(id, "this user is special")`、`error!("invalid id")`。production 預設 `info`，需要時再把 debug / trace 打開。

---

## 沒有記錄員，就沒人在聽

有一天小芸請假，塔台**沒有記錄員**。大家照樣用對講機回報，可是**沒人寫下來**，也沒人說「沒人在聽喔」——那些回報就這樣**默默消失了**。

所以每天開工第一件事，一定是**先派一位記錄員就位**，告訴他：「寫的時候順便記下是**哪個部門**回報的、在**第幾頁第幾行**。」

> 這就是 **subscriber**：`tracing_subscriber::fmt().with_target(true).with_line_number(true).init();`，在 `main` 開頭只做一次。沒有 init 的話，所有 `info!` 都會被**靜默丟掉**。`with_target` 印 module path，`with_line_number` 印行號。

---

## 今天要記多細

塔台門口有一塊**白板**，寫著今天要記到多細。白板上寫「一般」，就只記一般以上；寫「我們部門記到除錯，其他部門只記注意」，就照著分開記。

**白板沒寫的話**，記錄員就照老規矩：只記「一般」以上。

> 這就是 **`RUST_LOG` 與 `EnvFilter`**：`.with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))`，有設環境變數就照它，沒有就用 `info`。例如 `RUST_LOG=debug`、`RUST_LOG=ch23=debug`、`RUST_LOG=warn,ch23=debug`、`RUST_LOG=hyper=off,info`。

---

## 掛上航班牌

有一班飛機要處理，塔台把一塊**航班牌**掛起來：「**處理中，班號 abc123**」。

牌子掛著的時候，**所有回報都會自動帶上「班號 abc123」**，不用每個人自己再寫一次。處理完把牌子**拿下來**，之後的回報就不會再帶這個班號了。

航班牌還可以**一塊掛在另一塊裡面**：大航班牌底下掛小航班牌，回報會同時帶上兩塊牌子的資訊。

> 這就是 **span**：`let span = tracing::info_span!("processing", request_id = "abc123"); let _enter = span.enter();` 之後的 `info!("inside span")` 會自動帶 `request_id`；`drop(_enter)` 離開 span，`info!("outside span")` 就不帶了。span 可巢狀，event 會繼承所有上層 span 的欄位。

---

## 自動掛牌的櫃台

小芸發現，每次「**查乘客**」都要手動掛牌、拿牌，很麻煩。她在「查乘客」櫃台裝了一個**自動掛牌機**：只要有人來查，機器就自動掛上「**查乘客，編號 7**」的牌子，查完自動拿下。

但「**整批處理**」櫃台有點不一樣：一整批乘客名單太長了，掛在牌子上會擋住整面牆。所以小芸設定：「這個櫃台的牌子，**名單那一欄不要寫**。」

> 這就是 **`#[instrument]`**：`#[instrument] fn fetch_user(id: u64)` 自動建一個名為 `fetch_user`、帶 `id` 欄位的 span。`#[instrument(skip(items))] fn process_batch(items: &[u64])` 跳過太大的參數。另外還有 `level = "debug"`、`fields(...)`、`ret`（記回傳值）、`err`（`Err` 時自動記 error）。

---

## 格子的兩種寫法

回報表的格子有兩種填法：

- **給人看的寫法**：「名字：user-7」，乾乾淨淨。
- **給技師看的寫法**：把東西**原原本本、連括號引號**都寫上去，方便檢查細節。

小芸還提醒大家：別把**整本乘客名冊**用技師寫法塞進一格，寫太久整個塔台會卡住；寫「人數：5」就好。

> 這就是 **`%` 與 `?`**：`%value` 強制用 `Display`、`?value` 強制用 `Debug`，例如 `info!(%name, "found")`、`info!(user, %url, ?body, "request")`。大物件別直接 `?big_vec`，改記 `count = items.len()`，就像 `process_batch` 裡的 `info!(count = items.len(), "starting batch")`。

---

## 轉機也不會掉牌子

有些旅客要**轉機**：走到一半先去休息室等，等一下再回來繼續。以前牌子是掛在**櫃台**上的，旅客一離開櫃台，牌子就跟錯人了。

新的做法是把航班牌**掛在旅客的行李上**。不管旅客中途去哪裡等，回來時牌子都還在他身上。

> 這就是 **span 與 async**：async 函式在 `.await` 之間會被切換，綁在 thread 上的 context 會亂掉。`#[instrument] async fn handle(...)` 讓 span **跟著 future 走**，不會跨 task 漏掉；`tokio` 已經和 `tracing` 整合好。

---

## 交給總部的表格

總部要做統計，不想讀小芸手寫的紀錄，想要**整齊的表格**，每一格都標好名稱，電腦可以直接讀。

小芸換了一本「**表格版紀錄簿**」，每一筆都寫成同樣格式，還附上當時掛著哪些航班牌，整疊交給總部。

> 這就是 **JSON 輸出**：`tracing_subscriber::fmt().json().with_current_span(true).with_span_list(true).init()`，輸出可以直接送到 Loki / Datadog / OTel collector。之後接 `tracing-opentelemetry`，span 還能匯出成 OTel traces（第 49 章）。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 隨口喊一句 | `println!` / `log` 純字串 |
| 有格子的回報表 | **結構化欄位** `key = value` |
| 一筆回報 | **event**（`info!`、`warn!`…） |
| 五種緊急程度 | **level** `trace < debug < info < warn < error` |
| 塔台記錄員 | **subscriber**（`tracing_subscriber::fmt()`） |
| 沒記錄員，回報默默消失 | 沒 `.init()` → event 靜默丟掉 |
| 記下哪個部門、第幾行 | `.with_target(true)`、`.with_line_number(true)` |
| 門口白板 | **`RUST_LOG`** / `EnvFilter` |
| 白板沒寫就照老規矩 | `unwrap_or_else(\|_\| "info".into())` |
| 航班牌 | **span**（`info_span!`） |
| 掛牌 / 拿牌 | `span.enter()` / `drop(_enter)` |
| 回報自動帶班號 | event 繼承 span 欄位 |
| 自動掛牌機 | **`#[instrument]`** |
| 名單那一欄不要寫 | `#[instrument(skip(items))]` |
| 給人看 / 給技師看 | `%`（Display）/ `?`（Debug） |
| 掛在行李上的牌子 | span 跟著 future 走（async） |
| 表格版紀錄簿 | **`.json()`** 輸出 |

---

## 一句話記住

> **`tracing` 就是塔台紀錄簿：回報要填格子（結構化欄位）並標輕重（level），一定要先派記錄員（subscriber `init`）並用白板（`RUST_LOG`）決定記多細；處理一件事時掛上航班牌（span / `#[instrument]`），牌子掛著的期間每筆回報都自動帶上它。**
