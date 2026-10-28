# 故事版：市立圖書館的地下書庫 📚

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「去圖書館櫃台請館員到地下書庫拿資料」的故事，幫你「感覺」出 connection pool、`query_as` 與 `bind`、`FromRow`、`fetch_one` / `fetch_optional` / `fetch_all` / `execute`、transaction、compile-time 檢查的 `query!`、migration 是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 資料都在地下書庫

市立圖書館的**讀者資料卡**都收在**地下書庫**裡。書庫很大、很安全，可是一般人**不能自己下去**。

想查資料，要到一樓**櫃台**，請館員幫你下去拿。

> 這就是 **資料庫（Postgres）**：資料存在另一個程式（書庫）裡，你的 Rust 程式要透過「連線」（館員）跟它溝通。`DATABASE_URL=postgres://...` 就是書庫的地址。

---

## 櫃台只有五位館員

櫃台固定有 **5 位館員**。讀者來了，有空的館員就接手，跑下去拿資料，回來交給讀者後，**再回到櫃台等下一位**。

如果 5 位都在忙，新來的讀者就排隊。館長規定：「**排超過 5 秒還沒輪到，就請他先回去**，別讓隊伍無限長。」

館長也說：「**整間圖書館只有這一組櫃台**。不要每來一個讀者就臨時再蓋一個新櫃台。」

> 這就是 **connection pool**：`PgPoolOptions::new().max_connections(5).acquire_timeout(Duration::from_secs(5)).connect(&db_url)` 建出一個 `PgPool`。連線用完會**還回池子**重複使用。整個 app 共用一個 pool，函式只拿 `&PgPool`（像 `insert_user(pool: &PgPool, ...)`），千萬不要每個 request 都建新 pool。

---

## 先把書架蓋好

圖書館剛開幕時，館員第一件事是確認：「讀者資料卡的**書架**有沒有蓋好？沒有的話就蓋一個，有的話就不用再蓋。」書架上每格都規定好放什麼：編號、名字、信箱（**不能重複**）、辦卡時間（沒寫就填現在）。

> 這就是 **`create_schema`**：`CREATE TABLE IF NOT EXISTS users (...)`，用 `sqlx::query(...).execute(pool)` 送出去。`email TEXT NOT NULL UNIQUE` 是「信箱不能重複」，`created_at ... DEFAULT NOW()` 是「沒寫就填現在」。

---

## 填空式的申請單

櫃台有一疊**填空式申請單**，上面印好：

> 「請找出編號是 **＿＿①＿＿** 的讀者資料卡。」

讀者只能在**空格裡**填東西。有個調皮的小孩想在空格裡寫「……順便把所有卡片都丟掉」，館員看了笑笑：「空格裡寫什麼，我都**只當成一個編號**來找，不會當成命令。」

> 這就是 **`query_as` + `bind`**：SQL 裡寫 `WHERE id = $1`，再用 `.bind(id)` 填進第 1 個空格，`$2`、`$3` 依序對應下一個 `.bind(...)`。這樣使用者輸入永遠只是「值」，不會變成 SQL 指令。**絕對不要用 `format!` 把使用者輸入直接拼進 SQL**，那就是 SQL injection。

---

## 照格子放好的卡片

館員從書庫拿回來的是一張張**資料卡**。為了方便讀者看，館員會把卡上的東西**照順序放進讀者手上的資料夾格子**：編號一格、名字一格、信箱一格、辦卡時間一格。格子名字要對得上，不然放不進去。

> 這就是 **`#[derive(FromRow)]`**：`struct User { id: Uuid, name: String, email: String, created_at: DateTime<Utc> }` 加上 `FromRow`，`query_as::<_, User>(...)` 就會把每一列的欄位放進對應的 struct 欄位。型別也要對得上：`UUID` → `Uuid`、`TEXT` → `String`、`TIMESTAMPTZ` → `DateTime<Utc>`，可以是 NULL 的欄位用 `Option<T>`。

---

## 四種跑腿方式

讀者提出的要求不一樣，館員跑腿的方式也不一樣：

1. 「幫我辦一張新卡，**辦好把卡給我看**。」——一定會有**剛好一張**。如果結果一張都沒有，館員會說：「這不對勁！」
2. 「幫我找編號 X 的卡。」——**可能有，也可能沒有**。沒有的話館員就說「查無此卡」，很正常。
3. 「給我最新的 10 張卡。」——抱回**一整疊**。
4. 「幫我把這張卡的名字改一下。」——館員去改，**回來只說『改好了』**，不用帶卡片回來。

> 這就是 **fetch 系列**：`insert_user` 用 `INSERT ... RETURNING ...` 加 `fetch_one`（一定剛好 1 列，0 列會回 `Error::RowNotFound`）；`get_user` 用 `fetch_optional`，回 `Option<User>`；`list_users` 用 `fetch_all`，回 `Vec<User>`；`UPDATE` 用 `execute`，不取資料。很大的查詢可以用 `fetch` 拿 `Stream` 一筆一筆處理。

---

## 兩件事要一起成功

有一天，小咪和阿凱要**交換**一些紀錄：小咪的卡要註記「給出去」，阿凱的卡要註記「收進來」。

館員說：「這兩件事**一定要一起成功**。如果只改了小咪、阿凱那邊卻出錯，就會亂掉。」

所以館員先在桌上放一個「**處理中**」的牌子，改小咪、改阿凱，**兩個都改好才按下『確定』**。如果中途有人把牌子撞倒、館員還沒按確定就離開，書庫會**當作什麼都沒發生**，全部還原。

> 這就是 **transaction**：`transfer_demo` 裡 `let mut tx = pool.begin().await?;`，兩個 `UPDATE` 都用 `.execute(&mut *tx)`，最後 `tx.commit().await?`。如果中途 `?` 提早回傳、`tx` 被 drop 卻沒 commit，就會 **ROLLBACK**。`&mut *tx` 是因為 `Transaction` 要以可變借用的方式當成 connection 使用。

---

## 先拿單子去核對

館員小林很細心。新的申請單**印出來之前**，他會先拿樣本跑下去書庫，確認「這些格子名稱書庫真的有」「填的東西型別對不對」。有錯的話，**單子根本不會被印出來**。

可是有些分館**沒辦法直接下去書庫**核對。小林就事先把核對結果**影印一份存檔**，分館照影本檢查就好。

> 這就是 **compile-time 檢查的 `query!` / `query_as!`**：編譯時就連上 DB 檢查 SQL 和型別，錯了直接編譯失敗。缺點是 build 需要 `DATABASE_URL`；CI 沒有 DB 時，先用 `cargo sqlx prepare` 產生 `.sqlx/` 離線快取，再設 `SQLX_OFFLINE=true`。範例用的是 runtime 版 `query_as::<_, User>`，不用 DB 也能編譯，但 SQL 寫錯要執行時才知道。

---

## 書架改裝日誌

圖書館每次改裝書架（加一格、換格子大小），都會寫一頁**改裝日誌**，按日期編號。新的分館開幕時，只要**照日誌從第一頁做到最後一頁**，書架就會跟總館一模一樣。

> 這就是 **migrations**：`sqlx migrate add create_users` 產生 `migrations/<時間>_create_users.sql`，`sqlx migrate run` 依序執行；程式裡也可以 `sqlx::migrate!("./migrations").run(&pool).await?`。單體 app 可以啟動時跑，微服務常改成獨立的 migration job。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 地下書庫 | **Postgres 資料庫** |
| 書庫地址 | `DATABASE_URL` |
| 5 位館員的櫃台 | `PgPool`、`max_connections(5)` |
| 排超過 5 秒就請回 | `acquire_timeout(Duration::from_secs(5))` |
| 只蓋一組櫃台 | 整個 app **共用一個 pool**，傳 `&PgPool` |
| 確認書架有沒有蓋 | `CREATE TABLE IF NOT EXISTS` |
| 填空式申請單 | `WHERE id = $1` + `.bind(id)` |
| 空格內容只當編號 | 防 **SQL injection**（不要 `format!`） |
| 照格子放進資料夾 | `#[derive(FromRow)]` + `query_as::<_, User>` |
| 一定剛好一張 | `fetch_one`（0 列 → `RowNotFound`） |
| 可能有可能沒有 | `fetch_optional` → `Option<T>` |
| 抱回一整疊 | `fetch_all` → `Vec<T>` |
| 只說「改好了」 | `execute` |
| 「處理中」牌子 | `pool.begin()` |
| 按下確定 | `tx.commit()` |
| 沒按確定就離開 → 還原 | drop 沒 commit → **ROLLBACK** |
| 印單前先下去核對 | `query!` / `query_as!` 編譯期檢查 |
| 核對結果的影本 | `.sqlx/` + `SQLX_OFFLINE=true` |
| 書架改裝日誌 | `migrations/` + `sqlx migrate run` |

---

## 一句話記住

> **sqlx 就是圖書館櫃台：一組館員（pool）大家共用，申請單只能填空格（`bind`），結果照格子放好（`FromRow`）；要一起成功的事放進「處理中」（transaction），沒按確定就全部還原。**
