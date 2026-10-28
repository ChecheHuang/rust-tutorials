# 故事版：露營區的對講機頻道 📻

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「露營區每頂帳篷都拿到一支對講機，大家在同一個頻道聊天」的故事，幫你「感覺」出 WebSocket 握手升級、`split` 成 sender / receiver、`broadcast` 聊天室、兩個 task 與 `tokio::select!`、訊息類型與心跳、lag、集中 writer，以及跟 SSE 的取捨。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 平常是寫紙條

星空露營區的**營主阿山**住在中央小木屋。露營的人有事，平常是**寫一張紙條**送到小木屋，阿山看完**回一張紙條**，事情就結束了。

想知道「現在有沒有人在烤棉花糖？」就得一直跑去送紙條問，好累。

> 這就是 **普通 HTTP**：一問一答，回完就結束。範例的 `index()` 用普通 HTTP 回一個小網頁（`Html(...)`），讓瀏覽器拿到聊天室的畫面。

---

## 「我們改用對講機吧！」

小豆受不了了，他寫了最後一張紙條：「**我們改用對講機好嗎？**」

阿山看到，回答：「好！」然後遞給他一支對講機。從這一刻起，小豆和小木屋之間**一直保持通話**，誰想說話都可以隨時說，不用再送紙條。

> 這就是 **WebSocket 握手升級**：瀏覽器 `new WebSocket("ws://.../ws")` 先送一個 HTTP 請求要求升級。`ws_handler` 收到 `WebSocketUpgrade`，呼叫 `ws.on_upgrade(move |socket| handle_socket(socket, state))`——升級成功後，拿到一條雙向、持續的 `WebSocket` 連線。

---

## 麥克風和耳機分開拿

對講機有兩個部分：**麥克風**和**耳機**。阿山把它們**拆開**，交給兩位不同的小幫手：

- **小幫手 A** 拿麥克風，負責**對小豆說話**。
- **小幫手 B** 戴耳機，負責**聽小豆說話**。

這樣一來，聽的時候不用停下來等說的，說的時候也不會漏聽。

> 這就是 **`socket.split()`**：`let (mut sender, mut receiver) = socket.split();`，`sender` 是 `SplitSink`（寫），`receiver` 是 `SplitStream`（讀）。兩邊分別丟進 `tokio::spawn` 的 task，**讀跟寫不會互卡**。

---

## 營地大喇叭頻道

露營區有一個**公共頻道**。每一頂帳篷拿到對講機時，阿山都會幫它多接一台**收音機**，調到這個頻道。

任何人在公共頻道說一句話，**每一台收音機都會收到同一句**。

> 這就是 **`tokio::sync::broadcast`**：`main` 裡 `broadcast::channel::<ChatMsg>(100)` 建立頻道，`tx` 放進 `AppState`。每個連線 `state.tx.subscribe()` 拿到自己的 `rx`（收音機）。`tx.send(m)` 一次送出，所有訂閱者都收到一份——多個發送者、多個接收者（MPMC）。

---

## 小幫手 A：把廣播轉給帳篷

小幫手 A 一直守著收音機。收音機一響，他就把那句話**用標準格式寫好**，對著麥克風念給小豆聽。

如果有一次怎麼念，小豆那邊都**沒反應**（對講機壞了或小豆走了），小幫手 A 就**收工**。

> 這就是 **`send_task`**：`while let Ok(msg) = rx.recv().await`，用 `serde_json::to_string(&msg)` 轉成 JSON，再 `sender.send(Message::Text(json)).await`。送失敗（`is_err()`）就 `break`。

---

## 小幫手 B：把帳篷的話放上頻道

小幫手 B 戴著耳機。小豆說：「我是小豆，我在烤棉花糖！」小幫手 B 確認這句話是**正確格式**（有名字、有內容），就把它**放上公共頻道**，大家的收音機都會響。

如果小豆亂講一串**聽不懂的雜音**，小幫手 B 只在本子上記「收到奇怪的話」，**不會因此把小豆的對講機沒收**。

如果小豆說「**通話結束**」，小幫手 B 就收工。

> 這就是 **`recv_task`**：`while let Some(Ok(msg)) = receiver.next().await`，`Message::Text(t)` 用 `serde_json::from_str::<ChatMsg>(&t)` 解析，成功就 `tx.send(m)`，失敗就 `warn!(?e, "bad message")` 跳過，不斷線。`Message::Close(_)` 就 `break`。

---

## 對講機的幾種訊號

對講機傳的不只有說話：

- **說話**：一般的文字。
- **傳照片**：一串資料。
- 「**喂，還在嗎？**」和「**在！**」：確認對方還在線上。
- 「**通話結束**」：要掛了。

其實「喂，還在嗎？」不用小幫手回答，**對講機自己會回「在！」**。

如果太久沒人講話，山上的**中繼站**會以為沒人用了，**偷偷把訊號切掉**。所以帳篷最好每 30 秒就問一次「還在嗎？」；阿山這邊如果 60 秒什麼都沒聽到，就主動掛斷。

> 這就是 **訊息類型與心跳**：`Message::Text`、`Binary`、`Ping`、`Pong`、`Close`。axum **自動回 pong**，範例收到 `Message::Ping(p)` 只是 `info!` 示意。沒有心跳的話，連線可能被 NAT / 代理默默切斷；可以用 `tokio::select!` + `tokio::time::interval` 實作 idle 檢查。

---

## 一個小幫手收工，另一個也收工

小豆收拾帳篷回家了。小幫手 B 聽到「通話結束」先收工。這時小幫手 A 如果還守著收音機，就是在**對一支沒人拿的對講機說話**，白白浪費力氣。

所以阿山規定：「**兩個人只要有一個收工，另一個也馬上收工**。」

> 這就是 **`tokio::select!` + `abort`**：`select! { _ = (&mut send_task) => recv_task.abort(), _ = (&mut recv_task) => send_task.abort() }`，任一個 task 結束，就把另一個停掉，最後印出 `"websocket disconnected"`。

---

## 收音機只記得 100 句

公共頻道的收音機有個小毛病：它**最多只能暫存 100 句**。如果某頂帳篷的小幫手 A 動作太慢，收音機堆了超過 100 句還沒念，**最舊的幾句就被擠掉了**，下次他會收到一張小紙條：「你漏掉了 N 句。」

> 這就是 **broadcast 的 lag**：容量是 `broadcast::channel(100)` 的 100。慢的訂閱者會收到 `RecvError::Lagged(n)`。範例的 `while let Ok(msg)` 碰到任何錯誤就結束，實務上要分辨 `Lagged` 並繼續。

---

## 一支麥克風不能兩個人搶

後來阿山想讓**好幾個小幫手**都能對小豆說話：有人報天氣、有人報晚餐。可是麥克風只有一支，兩個人一起搶，話就會黏在一起。

阿山的做法是：只讓**一位小幫手**拿麥克風，其他人有話要說，就**把紙條丟進他的籃子**，由他依序念。

> 這就是 **集中 writer**：`Sink::send` 需要 `&mut self`，`sender` 不能直接給多個 task 共用。推薦用 `mpsc::unbounded_channel()` 建一個籃子，只有一個 task 拿著 `sender`，其他地方 `out_tx.send(msg)` 丟進去。

---

## 只聽不說的收音機

隔壁營區只需要**每天早上廣播天氣**，大家聽就好，不用回話。他們沒買對講機，只發了**一台單向收音機**，簡單又便宜。

> 這就是 **SSE 與 WebSocket 的取捨**：只需要 server → client 的單向推播（log、通知），**優先選 SSE**，走一般 HTTP、`EventSource` 還會自動重連。需要雙向（聊天、遊戲、協作）才用 WebSocket。

### 對講機的安全守則

阿山也貼了守則：對講機要用**加密頻道**；發對講機前，先確認對方真的是**本營區的人**；一次講太長的話會被切掉。

> 對應 **安全設定**：production 用 `wss://`（TLS）；在 upgrade handler 檢查 `Origin` header 防 CSWSH；用 `max_message_size` / `max_frame_size` 限制訊息大小；auth 可以放 query string token 或第一則訊息。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 送紙條、回紙條 | 一般 **HTTP** request / response |
| 「改用對講機好嗎？」 | WebSocket **握手升級** |
| 阿山答應、遞對講機 | `ws.on_upgrade(...)` → `handle_socket` |
| 一直通話的對講機 | `WebSocket` 連線 |
| 拆開麥克風和耳機 | `socket.split()` → `sender` / `receiver` |
| 公共頻道 | `broadcast::channel::<ChatMsg>(100)` |
| 每頂帳篷的收音機 | `state.tx.subscribe()` |
| 對頻道說話 | `tx.send(m)` |
| 小幫手 A | `send_task`：`rx.recv()` → `sender.send(...)` |
| 小幫手 B | `recv_task`：`receiver.next()` → `tx.send(...)` |
| 標準格式的話 | `ChatMsg { user, text }`（JSON） |
| 雜音只記下、不沒收 | 解析失敗 `warn!` 後跳過 |
| 「通話結束」 | `Message::Close` |
| 「還在嗎？」「在！」 | `Ping` / `Pong`（axum 自動回 pong） |
| 中繼站偷偷切訊號 | NAT / 代理切斷 idle 連線 → 要**心跳** |
| 一個收工另一個也收工 | `tokio::select!` + `abort()` |
| 收音機只記 100 句 | broadcast 容量、`RecvError::Lagged` |
| 只讓一人拿麥克風 | **集中 writer**（`mpsc`） |
| 單向收音機 | **SSE** |
| 加密頻道 | `wss://` |

---

## 一句話記住

> **WebSocket 就是先用紙條約好、再改用對講機：麥克風和耳機拆給兩個小幫手（`split` + 兩個 task），大家的話放上公共頻道（`broadcast`），一個收工另一個也收工（`select!`）。**
