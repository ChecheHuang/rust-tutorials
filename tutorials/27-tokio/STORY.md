# 故事版：夜市雞排攤的四位店員 🍗

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「夜市超大雞排攤，四位店員輪流處理一大堆訂單」的故事，幫你「感覺」出 Tokio runtime、multi-thread 與 work-stealing、`spawn` 和 `JoinHandle`、`spawn_blocking`、cooperative scheduling 與 `yield_now`、`current_thread` 和 `block_on`、`JoinSet`、取消與 `tokio::time`。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 夜市最大的雞排攤

夜市裡有一攤超有名的雞排，叫「**東京雞排**」。攤子不只賣雞排，還附送一整套設備：油鍋、計時器、出餐叫號機、排隊動線……

老闆**阿龍**請了**四位店員**站在四個油鍋前。客人的訂單會被貼在一面大白板上，店員有空就去拿一張來做。

> 這就是 **Tokio runtime**：提供 scheduler（誰來做哪張單）、IO event loop（油鍋和計時器）、`tokio::time`、`tokio::net` 等工具。`#[tokio::main(worker_threads = 4)]` 就是「請四位店員」——4 個 worker thread。不寫的話，預設是 CPU 核心數。

---

## 遞單子進去，拿號碼牌回來

小芸點了一份雞排。阿龍把單子**貼上白板**，同時撕一張**號碼牌**給小芸：「好了會叫你。」

小芸拿著號碼牌在旁邊等，叫到號碼就去領雞排，上面寫著「**42 號餐**」。

> 這就是 **`tokio::spawn` 與 `JoinHandle`**：`let h = tokio::spawn(async { ...; 42 });` 立刻把 task 交給 scheduler，回傳 `JoinHandle<T>`（號碼牌）。`h.await.unwrap()` 拿到 `42`。如果那張單做到一半出事（task panic），領餐時會拿到 `Err(JoinError)`。

---

## 一千張單子也不怕

晚上八點，一口氣湧進 **一千張** 訂單。每張單都要炸 10 秒。

如果一次只炸一塊，要炸一萬秒。可是油鍋很大，店員把雞排丟下去以後**不用盯著**，馬上去拿下一張單。結果一千張單**不到一下子就全部好了**。

而且四位店員很有默契：誰手邊的單先做完，就**去別人的那疊單子裡拿幾張過來幫忙**。

> 這就是 **大量輕量 task + work-stealing**：`main` 裡 `for i in 0..1000 { tokio::spawn(...) }`，1000 個 task 各 `sleep(10ms)`，總時間遠小於 10 秒。task 很輕，只是一點狀態，不是一條 OS thread。multi-thread runtime 會讓閒著的 worker **偷**別人的 task 來做。因為 task 可能換人做，所以 spawn 的 Future 要 `Send + 'static`。

---

## 揉一大團麵團，別佔著油鍋

有客人訂了一個超大號炸雞桶，要先**揉一千萬下麵團**。

如果讓某位油鍋店員去揉，他那一整區就**完全停擺**：他排的單沒人炸，計時器響了也沒人理。

阿龍說：「揉麵這種一直出力、不用等的工作，交給**後場的揉麵師傅**。他們有一大群人，專門做這種事。」

> 這就是 **`spawn_blocking`**：`task::spawn_blocking(|| (1..=10_000_000u64).sum::<u64>())` 把 CPU 密集的工作丟到獨立的 **blocking thread pool**（預設上限 512 條），不會卡住 worker thread。判斷準則：超過大約 10 µs 的同步工作就考慮用它。

---

## 沒有人會搶你的鍋鏟

東京雞排有一條規矩：**沒有人會從你手上搶走鍋鏟**。店員只有在「把雞排丟進油鍋、要等的時候」才會轉身去做別的事。

所以如果有個店員一直在切高麗菜、切了半小時都不停下來，他那邊的單就一直卡著。

阿龍教新人**小豪**：「切菜切一陣子，就**自己停一下**，看看有沒有別的單要先處理。」小豪每切一顆就喊一聲「換人！」然後讓別的單先跑一下。

> 這就是 **cooperative scheduling 與 `yield_now`**：Tokio 不會強迫 task 讓出，**只有 `.await` 點**才是讓位的時機。純 CPU 的長迴圈沒有 `.await` 就會霸佔 worker。`yield_demo()` 每圈呼叫 `task::yield_now().await`，主動把位置讓給別的 task。

---

## 小推車，只有一個人

阿龍的姪女在夜市另一頭擺了一台**小推車**，只賣飲料，**只有她一個人**。

她不需要四個人，也不需要大白板。但是做法一樣：飲料搖好要等冰塊融一點時，她先去收錢。

有位客人跑來推車前面說：「我就**站在這裡等**，等你做好我才走。」

> 這就是 **`current_thread` runtime 與 `block_on`**：`Builder::new_current_thread().enable_all().build()` 建立只用一條 thread 的 runtime，適合 CLI、GUI、embedded。`local_rt.block_on(async { ... })` 讓**當下這條 thread** 站著把這個 Future 跑完。`#[tokio::main]` 背後也是 `block_on`。⚠️ 不要在 async 裡面再叫 `block_on`：店員自己站著等自己，會 deadlock。

---

## 一疊號碼牌，誰先好先叫

阿龍幫一個旅行團訂了五份餐，拿到**一疊號碼牌**。這五份炸的時間不一樣：第 1 份最久，第 5 份最快。

他不是照號碼順序等，而是**哪份先好就先叫哪份**：5、4、3、2、1。

如果旅行團臨時說「不吃了」，阿龍把整疊號碼牌丟掉，還沒炸好的那幾份**就自動取消**。

> 這就是 **`JoinSet`**：`set.spawn(...)` 動態加入 task，`while let Some(res) = set.join_next().await` 依**完成順序**拿結果（程式裡印出 5、4、3、2、1）。`JoinSet` 被 drop 時，還沒完成的 task 會被取消。

---

## 取消訂單、計時、鬧鐘

夜市還有幾樣好用的小工具：

- **取消單子**：客人說不要了，阿龍可以把單子撕掉。但正在油鍋裡的那塊，要等到店員下一次轉身（有個停頓點）時才真的停。
- **限時**：「五分鐘內沒好就退錢。」
- **鬧鐘**：每一分鐘響一次，提醒店員換油。

> 這就是 **取消與 `tokio::time`**：`handle.abort()` 或 drop Future 都會取消 task，但取消只會在 `.await` 點發生——所以要注意 **cancel safety**。`sleep` 等一段時間、`timeout(dur, op())` 超時回 `Err(Elapsed)`、`interval(dur)` 配 `tick().await` 做週期性工作。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 東京雞排攤（附整套設備） | **Tokio runtime** |
| 四位店員 | `#[tokio::main(worker_threads = 4)]` |
| 貼上白板的單子 | **task** |
| 把單子貼上白板 | **`tokio::spawn`** |
| 號碼牌 | **`JoinHandle<T>`** |
| 做到一半出事 | task panic → `Err(JoinError)` |
| 一千張單一下子就好 | **大量輕量 task** |
| 去別人那疊拿單子幫忙 | **work-stealing** |
| 後場的揉麵師傅 | **`spawn_blocking`**（blocking thread pool） |
| 沒人搶你的鍋鏟 | **cooperative scheduling** |
| 「換人！」 | **`task::yield_now().await`** |
| 只有一個人的小推車 | **`current_thread` runtime** |
| 站在推車前等做好 | **`block_on`** |
| 一疊號碼牌，誰先好先叫 | **`JoinSet`** + `join_next()` |
| 撕掉單子 | `abort()` / drop = **取消** |
| 五分鐘內沒好就退錢 | `tokio::time::timeout` |
| 每分鐘響的鬧鐘 | `tokio::time::interval` |

---

## 一句話記住

> **Tokio 是一整間附設備的雞排攤：`spawn` 把單子貼上白板換號碼牌，店員只在等油炸時（`.await`）才轉身去做別的；揉麵這種一直出力的工作交給後場（`spawn_blocking`），切菜切太久要自己喊「換人」（`yield_now`）。**
