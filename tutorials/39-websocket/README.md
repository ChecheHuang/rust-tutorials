# 39. WebSocket

> 範圍：握手升級、split sink/stream、broadcast 聊天室、心跳、與 SSE 比較

## WebSocket 在 axum 的形狀

```rust
ws.on_upgrade(|socket| handle(socket))
```

升級成功後拿到 `WebSocket`，split 出 `Sender` 跟 `Receiver`：

```rust
let (sender, receiver) = socket.split();
// sender: SplitSink<WebSocket, Message>  → 寫
// receiver: SplitStream<WebSocket>       → 讀
```

兩端可以**分別**被丟到 tokio task —— 寫跟讀不會互卡。

## 訊息類型

| Message | 用途 |
|---------|------|
| `Text(String)` | 文字 |
| `Binary(Vec<u8>)` | 位元組 |
| `Ping(Vec<u8>)` | 心跳問 |
| `Pong(Vec<u8>)` | 心跳答 |
| `Close(Option<CloseFrame>)` | 關閉 |

axum 預設**自動回 pong**，不必自己處理 ping。

## 廣播 pattern

`tokio::sync::broadcast` 是 MPMC：

```
client A ──┐                    ┌──→ task A → client A
client B ──┼─→ broadcast::Tx ──┼──→ task B → client B
client C ──┘                    └──→ task C → client C
```

每個 ws 連線：
1. 訂閱一份 `Receiver`（`tx.subscribe()`）
2. 起一個寫 task：`rx.recv() → sender.send()`
3. 起一個讀 task：`receiver.next() → tx.send()`

**注意 broadcast 的 lag**：訂閱者太慢、buffer 滿，會收到 `RecvError::Lagged(n)`——示範簡化跳過。

## 心跳設計

WebSocket 沒應用層心跳 = 過 NAT / 代理會被切。建議：
- **客戶端**每 30s 發 `ping` 或業務 ping 訊息
- **伺服器**收 ping、回 pong；若 idle 60s 沒收任何 frame 主動關
- `tokio::select!` + `tokio::time::interval` 實作

## 跟 SSE / long-poll 取捨

| | WebSocket | SSE | long-poll |
|---|-----------|-----|-----------|
| 雙向 | ✅ | ❌（server → client） | ✅ |
| HTTP/1.1 友好 | 升級 | 原生 | 原生 |
| 簡單度 | 中 | 簡單 | 簡單 |
| 重連自動 | 自己寫 | EventSource 原生 | 自己 |
| 適用 | chat / game / collab | log / notification | legacy |

**單向推播優先選 SSE**——比 ws 簡單，又能用 HTTP 通道。

## 安全

1. **`wss://`**：production 一定要 TLS。
2. **Origin 檢查**：避免 CSWSH（cross-site websocket hijacking）。在 upgrade handler 檢查 `Origin` header。
3. **Auth**：query string token 或第一個 message 傳 token；不要靠 cookie（CSRF 風險）。
4. **訊息大小限制**：`WebSocketUpgrade::max_message_size`、`max_frame_size`。

## 並發模式

每個 connection 跑兩個 task：read + write。**不要共用 sender**——`Sink::send` 需要 `&mut self`，多任務寫要包 `Mutex` 或集中到一個 writer task（推薦）。

```rust
// 集中 writer pattern
let (out_tx, mut out_rx) = mpsc::unbounded_channel();
tokio::spawn(async move {
    while let Some(msg) = out_rx.recv().await {
        sender.send(msg).await.ok();
    }
});
// 多處可以 out_tx.send(msg) 不衝突
```

## actix-web 對照

actix 用 `actix_ws::handle`：

```rust
async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<HttpResponse> {
    let (resp, mut session, mut stream) = actix_ws::handle(&req, body)?;
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            session.text(format!("echo: {msg:?}")).await.ok();
        }
    });
    Ok(resp)
}
```

概念一樣，API 不同。

## 常見陷阱

1. **`sender` 跨多 task** — 必須 mutex 或集中 writer，不能直接 clone。
2. **broadcast buffer 太小** — 慢 consumer 會被 lag 丟棄訊息；要對應錯誤。
3. **沒處理 `Close`** — client 已關，server 繼續 send，浪費資源。
4. **JSON 解析錯誤直接斷線** — 應該 log + 跳過，不要踢人。
5. **長 idle 被 NAT 切** — 沒心跳就 silent drop。
6. **背壓** — 訊息產生快過 send 速度，buffer 爆炸；要 drop 舊訊息或 throttle。

## 練習

1. 加 user join / leave 廣播（broadcast 自己的事件）。
2. 把訊息持久化（最近 50 條進 SQLite，client 連上時 replay）。
3. 寫一個 SSE 版本，做同樣的廣播功能，比較程式碼複雜度。
4. 加 message rate limit（每 client 每秒最多 5 條）。

## 延伸閱讀

- [axum websocket example](https://github.com/tokio-rs/axum/tree/main/examples/websockets)
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite) — 底層 ws 函式庫
- [Mozilla — Writing WebSocket servers](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers)
