# 故事版：迴轉壽司的輸送帶 🍣

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「坐在迴轉壽司店，等輸送帶一盤一盤送過來」的故事，幫你「感覺」出 Stream 和 Iterator 的差別、`next().await`、`map` / `filter` / `fold`、為什麼 filter 要包 `future::ready`、`buffer_unordered` / `buffered`、`ReceiverStream`、`take` / `take_while`，以及 `ready_chunks` / `chunks_timeout` 批次處理。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 擺在桌上的便當 vs 慢慢送來的壽司

小晴和爸爸去吃飯。

**第一家店**賣便當。便當一打開，五樣菜**全部已經擺在裡面**，想吃哪口就夾哪口，不用等。

**第二家店**是迴轉壽司。小晴面前有一條**輸送帶**，壽司是師傅**一盤一盤做好才放上來**的。她想吃下一盤，就得**等**它轉到面前。

有時候馬上就來，有時候要等師傅捏一陣子。等的時候，小晴可以先喝口茶、跟爸爸聊天。

> 這就是 **Stream vs Iterator**：Iterator 的 `next()` 立刻回 `Option<T>`（便當全在裡面）；Stream 的 `next()` 回一個 Future，要 `.await` 才拿到 `Option<T>`（等下一盤轉過來）。Stream 就是 **async 版的 Iterator**，底層是 `poll_next(self: Pin<&mut Self>, cx) -> Poll<Option<Self::Item>>`。

---

## 一盤一盤拿

店員先示範：把五盤壽司（1、2、3、4、5 號）排上輸送帶。小晴每次說「**下一盤**」，就等一下、拿起一盤，直到輸送帶上空了。

> 這就是 **`stream::iter` + `while let Some(x) = s.next().await`**：`main` 第一段用 `stream::iter(vec![1, 2, 3, 4, 5])` 把普通集合變成 Stream，再一個一個 `next().await`。拿到 `None` 就代表結束。要用這些方法，得 `use futures::stream::StreamExt`。

---

## 一路加工，最後算總帳

爸爸點了一個特別的玩法：

1. 輸送帶送來 1 到 10 號盤。
2. 經過師傅面前時，每盤的份量**變兩倍**。
3. 經過檢查員面前時，**份量太小（不超過 5）的拿掉**。
4. 最後小晴把剩下的全部**加起來**，算出今天吃了多少。

檢查員有個怪規矩：他每次判斷完，不是直接說「留」或「丟」，而是遞出一張**小卡片**，上面寫著答案。因為有些時候他得**打電話問廚房**才能決定，所以規定一律用卡片回答。就算他當場就知道，也要寫一張「**已經知道答案**」的卡片。

> 這就是 **stream adapter**：`.map(|x| x * 2).filter(...).fold(0, |acc, x| async move { acc + x }).await`，結果 `sum = 104`。重點是 `filter` 的 closure 要回 **Future**，因為判斷本身也可能要等 IO。判斷是同步的話，就包成 `futures::future::ready(*x > 5)`——那張「已經知道答案」的卡片。

---

## 三位師傅同時捏

有一張團體訂單：a、b、c、d、e 五種壽司，每一種都要捏一陣子。

如果只有一位師傅，一次捏一種，大家要等很久。店長說：「**最多三位師傅同時捏**，誰先捏好誰先放上輸送帶。」

所以送上來的順序不一定是 a、b、c、d、e，而是誰先好誰先到。如果客人堅持要照順序吃，店長也可以改成「三位同時捏，但**照點餐順序**放上去」。

> 這就是 **`buffer_unordered(3)` / `buffered(3)`**：`stream::iter(urls).map(|u| async move { fake_fetch(u).await }).buffer_unordered(3).collect().await`，同時最多跑 3 個 Future，誰先完成誰先輸出。`buffered(n)` 一樣同時跑 n 個，但**保持原順序**輸出。這是 web crawler、API batch 的核心技巧。

---

## 廚房的傳菜口接上輸送帶

廚房後面有一個**傳菜口**，師傅做好就從小窗口推出去。可是傳菜口本身**不是輸送帶**，客人不能用「下一盤」的方式去拿。

店長裝了一個**轉接器**，讓傳菜口推出來的盤子直接接上輸送帶。這樣客人就可以照平常的方式，一盤一盤等著拿，也能用上所有加工的玩法。

> 這就是 **`ReceiverStream`**：`tokio::sync::mpsc` 的 `Receiver` 本身不是 Stream。用 `tokio_stream::wrappers::ReceiverStream::new(rx)` 包起來，就能 `stream.next().await` 和串接 adapter。另外還有 `BroadcastStream`、`WatchStream` 等 wrapper。

---

## 只吃前五盤、吃到不想吃為止

小晴說：「我只吃**前五盤**。」於是她拿了 1 到 5 號，就不再拿了。

弟弟的規則不一樣：「我一直拿，**拿到 7 號以上就停**。」所以他拿了 1 到 6 號，看到 7 號就收手——後面就算又有小盤的，他也不拿了。

> 這就是 **`take` / `take_while`**：`stream::iter(1..=20).take(5)` 拿到 `[1, 2, 3, 4, 5]`；`.take_while(|x| futures::future::ready(*x < 7))` 拿到 `[1, 2, 3, 4, 5, 6]`，條件一不成立就整個結束。`take_while` 的 predicate 一樣要回 Future。

---

## 一次端一托盤

服務生**阿健**負責把盤子收到洗碗區。一盤一盤跑太累了，他改用**托盤**：

- 走到輸送帶旁邊，**現在有幾盤就收幾盤**，最多一托盤 4 盤。
- 如果只有 2 盤，他也不等，直接端 2 盤走。

> 這就是 **`ready_chunks(4)`**：`ReceiverStream::new(rx).ready_chunks(4)` 每次把「**現在已經準備好的**」item 收成一個 `Vec`，最多 4 個，不會為了湊滿而等。`chunks(n)` 則是一定湊滿 n 個（或 Stream 結束）才給；tokio-stream 的 `chunks_timeout` 是「滿了或時間到」。這是批次寫 DB、批次 flush 的標準模式。

---

## 滿一托盤，或是沙漏倒完

晚一點客人變少，盤子半天才來一盤。阿健要是堅持「湊滿 4 盤才走」，前面的盤子會在輸送帶旁邊放到乾掉。

店長給他一個**小沙漏**：

- 托盤**一滿 4 盤**，馬上端走。
- 沒滿也沒關係——**沙漏一倒完**，手上有幾盤就端幾盤。

剛開店時盤子連著來，阿健每次都是「滿了就走」；客人少了以後，就變成「沙漏到了才走」，一托盤只有一兩盤。

> 這就是 **`chunks_timeout(4, 50ms)`**：滿 4 個**或**等滿 50ms 就 yield 一批，量大時靠數量、量小時靠時間，不會無限等。它來自 **`tokio_stream::StreamExt`**（要開 `"time"` feature），跟 `futures::StreamExt` 同名，所以範例用完整路徑 `tokio_stream::StreamExt::chunks_timeout(...)` 呼叫；它裡面藏了計時器，不是 `Unpin`，所以要先 `tokio::pin!(s)` 才能 `.next().await`。

---

## 輸送帶、傳菜口、便當有什麼不一樣？

店長最後幫小晴整理：

- **便當**：全部擺好，自己拿，不用等。
- **輸送帶**：客人說「下一盤」才去拿，**客人拉**。可以一路加工、限制同時幾位師傅。
- **傳菜口**：師傅做好就**主動推**出來；滿了師傅得停手。

所以店裡最常見的做法是：師傅從傳菜口推出來，**接上輸送帶**，客人再用各種玩法去拿。

> 這就是 **Stream vs Iterator vs Channel**：Iterator 是同步、拉；Stream 是 async、拉；Channel 是 async、**推**（producer 主動 `send`），bounded channel 有背壓。實務上常見 **producer → mpsc → `ReceiverStream` → consumer**，用 Stream 的 adapter 寫得乾淨漂亮。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 打開就全部擺好的便當 | **Iterator**（同步 `next()`） |
| 一盤一盤送來的輸送帶 | **Stream**（async iterator） |
| 「下一盤」並等它轉過來 | **`s.next().await`** |
| 輸送帶空了 | `next()` 回 **`None`** |
| 把五盤排上輸送帶 | **`stream::iter(...)`** |
| 份量變兩倍 | **`map`** |
| 檢查員拿掉太小的 | **`filter`** |
| 檢查員一律用卡片回答 | predicate 要回 **Future** |
| 「已經知道答案」的卡片 | **`futures::future::ready(...)`** |
| 把全部加起來 | **`fold`** |
| 三位師傅同時捏，誰好誰先上 | **`buffer_unordered(3)`** |
| 三位同時捏但照順序上 | **`buffered(3)`** |
| 傳菜口 | **`mpsc::Receiver`** |
| 接上輸送帶的轉接器 | **`ReceiverStream`** |
| 只吃前五盤 | **`take(5)`** |
| 看到 7 號就收手 | **`take_while`** |
| 現在有幾盤就收幾盤的托盤 | **`ready_chunks(4)`** |
| 滿一托盤或沙漏倒完就端走 | **`chunks_timeout(4, 50ms)`** |
| 輸送帶客人拉、傳菜口師傅推 | Stream 是 **pull**、Channel 是 **push** |

---

## 一句話記住

> **Stream 就是迴轉壽司的輸送帶：每一盤都要 `.await` 等它轉過來；可以一路 `map`、`filter`、`fold`，讓幾位師傅同時捏（`buffer_unordered`），用托盤一次收一批（`ready_chunks`，加沙漏就是 `chunks_timeout`），而廚房的傳菜口（`mpsc`）裝個轉接器（`ReceiverStream`）就能接上來。**
