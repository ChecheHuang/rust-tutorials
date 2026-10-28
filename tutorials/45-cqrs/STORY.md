# 故事版：遊樂園的代幣卡和流水帳 🎡

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「遊樂園代幣卡櫃台」的故事，幫你「感覺」出 CQRS、Command 和 Event、aggregate 的 `apply` / `handle`、event store、projection 讀模型、最終一致性、樂觀並發、snapshot 和事件版本演進是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 一個櫃台，被問到快爆炸

快樂島遊樂園裡，每個小朋友都有一張**代幣卡**，坐遊樂設施要扣代幣。卡片本身不存錢，所有的帳都在**代幣櫃台**的小玲姐姐那裡。

一開始，小玲姐姐只有一張表，寫著「誰現在剩多少」。可是每天跑來櫃台的人：

- 真的要**開卡、儲值、扣款**的，大概只有一成；
- 剩下九成都只是來**問**：「我還剩多少？」

問問題的人排在儲值的人前面，隊伍長到繞摩天輪一圈。

> 這就是 **CQRS 想解決的問題**：讀寫負載差很多（讀 100×、寫 1×）。把「改變狀態」的**寫端**和「只想查」的**讀端**分開，用不同的模型各自最佳化。

---

## 申請單：我「想要」做什麼

小玲姐姐規定，想改帳的人要先填一張**申請單**：

- 「我要**開卡**，名字叫阿哲。」
- 「我要**儲值** 100 枚。」
- 「我要**扣** 30 枚去坐雲霄飛車。」

申請單只是「**我想要**」，**不一定會通過**。

> 這就是 **Command（意圖）**：`enum Command { Open { id, owner }, Deposit { id, cents }, Withdraw { id, cents } }`。Command 可能被**拒絕**。

---

## 流水帳：已經「發生了」什麼

申請通過後，小玲姐姐不是去擦掉舊數字，而是在一本**流水帳**上**往下寫一行**，還會蓋上時間章：

| 行 | 發生了什麼 |
|---|---|
| 1 | 10:00 阿哲開卡 |
| 2 | 10:05 阿哲儲值 100 枚 |
| 3 | 10:20 阿哲扣了 30 枚 |

寫上去的就是**已經發生的事**，永遠不擦、不改。

> 這就是 **Event（事實）與 Event Sourcing**：`enum Event { Opened { id, owner, at }, Deposited { id, cents, at }, Withdrawn { id, cents, at } }`，`at` 是 `DateTime<Utc>`，由伺服器（櫃台）填，不靠客人的手錶。事件 **append-only**，不存「目前狀態」，存「狀態的變化」。

---

## 小玲姐姐怎麼判斷能不能扣

阿哲又來了：「我要扣 999 枚，買下整台旋轉木馬！」

小玲姐姐沒有一張「目前餘額表」可以看。她翻開流水帳，**從第一行開始一行一行唸**：

「開卡了 → 0 枚；儲值 100 → 100 枚；扣 30 → 70 枚。」

算出目前 70 枚之後，她才看申請單：「999 比 70 多，**不行，餘額不足**。」這張申請單被退回，流水帳**什麼都沒多寫**。

如果是正常的申請，她就寫出一行或幾行新的流水帳，交給登記員抄進去。

> 這就是 **aggregate 的 `apply` 和 `handle`**：`Account { opened, balance_cents }`。`apply(&mut self, ev)` 是「唸一行就更新心裡的數字」，**永遠不會失敗**（事實已經發生）；`handle(&self, cmd)` 負責驗證規則，回傳 `Ok(Vec<Event>)` 或 `Err(DomainError::InsufficientFunds)`（還有 `NotFound`、`AlreadyOpened`）。

### 櫃台的標準流程

小玲姐姐每次都照同一套步驟：

1. 收到申請單，看是哪張卡。
2. 把這張卡的流水帳**全部拿出來**。
3. 從頭唸一遍，算出現在的狀態。
4. 判斷申請能不能過，寫出新的流水帳行。
5. 把新行**抄進流水帳**，同時**大聲廣播**。

> 這就是 **`dispatch()`**：`store.load(id)` → `for ev in &history { agg.apply(ev) }` → `agg.handle(&cmd)?` → `store.append(id, new_events)`。`?` 讓被拒絕的申請直接回傳錯誤，不會 append。

---

## 只能往後加的登記櫃

流水帳本身收在一個**登記櫃**裡，登記櫃只會做兩件事：

- 「把這幾行**接在後面**。」
- 「把某張卡的**所有行**拿出來。」

登記員每抄一行，還會用**廣播器**喊一次：「阿哲儲值 100 枚！」

> 這就是 **event store**：`trait EventStore { async fn append(...); async fn load(...) -> Vec<Event>; }`。範例的 `InMemoryStore` 用 `RwLock<HashMap<AccountId, Vec<Event>>>` 存 log，並透過 `broadcast::Sender<Event>` 把每個事件 `send` 出去。實務上常用 Postgres 一張 `events` 表，或 EventStoreDB、Kafka、NATS JetStream。

---

## 入口的電子看板

為了不讓大家再擠到櫃台問餘額，遊樂園在入口做了一面**電子看板**：「阿哲：70 枚」「小美：250 枚」。

看板旁邊坐著一位**看板員**，他戴著耳機一直聽廣播：

- 聽到「開卡」→ 看板加一列，從 0 開始。
- 聽到「儲值」→ 那列加上去。
- 聽到「扣款」→ 那列減下來。

小朋友要查餘額，**直接看看板**，不用排隊找小玲姐姐，也不用翻流水帳。

> 這就是 **projection 與 read model**：`run_projection(rx, view)` 從 `broadcast::Receiver<Event>` 一直 `rx.recv()`，收到事件就更新 `BalanceView { map: HashMap<AccountId, (String, i64)> }`。查詢直接讀 view，**不經過 aggregate**。`main` 用 `tokio::spawn(run_projection(bus.subscribe(), view.clone()))` 讓看板員在背景工作。

---

## 看板慢了一點點

阿哲剛儲值完，立刻衝到看板前面——咦，怎麼還是舊的數字？

原來看板員才剛聽到廣播，正在改。一眨眼，看板就更新了。

遊樂園跟大家說好：「看板**會**對，只是可能慢一下下。」如果真的要馬上看到，櫃台會直接在收據上印「目前 170 枚」給你。

> 這就是 **最終一致性（eventually consistent）**：寫完之後讀端可能還沒更新（範例在 `main` 裡 `sleep(20ms)` 才讀 view）。需要立刻看到時，可用 **read-your-own-writes**（寫完直接回給 client 顯示），或在同一個 transaction 同步更新 read model（犧牲一點 CQRS 純度）。如果看板員漏聽了，看板就永遠是舊的——所以 projection 失敗一定要能重試。

---

## 兩個櫃台搶著寫第 4 行

遊樂園生意太好，開了**第二個櫃台**。有一次，兩個櫃台**同時**幫阿哲處理扣款，兩邊都算出「目前 70 枚，可以扣 50」，都想寫**第 4 行**。

登記櫃有一條鐵律：**同一張卡的同一個行號，只能寫一次**。先到的寫成功；後到的被擋下來：「第 4 行已經有人寫了，請重新唸一遍再來。」後到的櫃台重新唸，發現只剩 20 枚，這次就拒絕了。

> 這就是 **樂觀並發**：`events` 表上的 `UNIQUE (aggregate_id, seq)`。append 時帶上預期的 seq，衝突就失敗 → 重新 load 再試。

---

## 書籤：不用每次從第一行唸

阿哲是年票會員，流水帳已經寫了好幾千行。每次都從第一行唸，小玲姐姐嗓子都啞了。

於是她每隔一陣子就夾一張**書籤**：「到第 3000 行為止，阿哲剩 420 枚。」下次只要從書籤開始，往後唸新的幾行就好。

> 這就是 **snapshot**：存一份「state + last_seq」，load 時先取 snapshot，再從 `last_seq + 1` 套用後續事件。只在效能真的成問題時才加。

---

## 流水帳格式改了怎麼辦

後來遊樂園想在「開卡」那行多記一個**家長電話**。可是以前寫的幾萬行都沒有這格。

小玲姐姐**不會回去改舊的行**。她做法是：唸到舊格式的開卡時，在心裡自動補一句「電話：未填」；新的開卡則寫成「**開卡（第 2 版）**」。

還有，如果有人要求「把我的資料**完全刪掉**」，她不撕帳本，而是當初就把電話用**只有一把鑰匙的密碼**寫；要刪的時候，把那把鑰匙丟掉，那行就再也沒人看得懂。

> 這就是 **事件 schema 演進**：用 **upcaster**（load 時把舊事件轉成新格式）或 **versioning**（`AccountOpenedV2`）。**永遠不要 UPDATE / DELETE 舊事件**。GDPR 刪除權用 **crypto-shredding**：敏感欄位加密，丟掉 key 就等於刪除。

---

## 不是每個攤位都需要這套

園區裡賣棉花糖的小攤，只要記「今天賣了幾支」。老闆聽完這整套說：「我寫一張紙就好，幹嘛搞流水帳、看板員、書籤？」

小玲姐姐笑著說：「對，你**不需要**。等你真的要查『上個月每天幾點賣最多』，再來換也不遲。」

> 這就是 **什麼時候不該用**：純 CRUD 後台、沒有審計或 replay 需求、團隊不熟事件思維時，CQRS + ES 只是多 3–5 倍的複雜度。建議先寫 CRUD，真的需要事件回溯再轉。Rust 生態有 `cqrs-es` 等框架，但手寫 + Postgres events 表 + projection worker 通常就夠。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 櫃台分「改帳」和「查帳」 | **CQRS**（寫端 / 讀端分離） |
| 申請單「我想要…」 | **Command**：`Open` / `Deposit` / `Withdraw` |
| 流水帳上的一行 | **Event**：`Opened` / `Deposited` / `Withdrawn` |
| 只往下寫、不擦不改 | **append-only / Event Sourcing** |
| 櫃台蓋的時間章 | `at: DateTime<Utc>`（server 填） |
| 唸一行、更新心裡的數字 | **`Account::apply`** |
| 判斷申請能不能過 | **`Account::handle`** → `Result<Vec<Event>, DomainError>` |
| 「餘額不足」退件 | `DomainError::InsufficientFunds` |
| 櫃台的標準流程 | **`dispatch()`**：load → apply → handle → append |
| 登記櫃 | **`EventStore`** / `InMemoryStore` |
| 登記員的廣播器 | `broadcast::Sender<Event>` |
| 入口電子看板 | **read model**：`BalanceView` |
| 戴耳機的看板員 | **projection**：`run_projection` |
| 看板慢一下下 | **最終一致性** |
| 同一行號只能寫一次 | **樂觀並發** `UNIQUE (aggregate_id, seq)` |
| 書籤 | **snapshot** |
| 舊格式在心裡自動補 | **upcaster** / 事件版本化 |
| 丟掉密碼鑰匙 | **crypto-shredding** |
| 棉花糖小攤 | 不需要 CQRS 的 **純 CRUD** |

---

## 一句話記住

> **申請單（Command）可能被退回，流水帳（Event）寫下就永遠不改；櫃台每次從流水帳唸出現在的狀態再判斷（`apply` / `handle`），看板員聽廣播更新入口看板（projection）——看板可能慢一下下，但一定會對。**
