# 30. Streams

> 範圍：Stream trait、StreamExt、async iteration、tokio-stream、buffer / chunks

## 什麼是 Stream

```rust
trait Stream {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}
```

Stream = **async 版的 Iterator**。每次 `next` 是 `async`，可能要等 IO。

```
Iterator::next() -> Option<T>            // sync
Stream::next() -> impl Future<Output = Option<T>>   // async
```

典型例子：
- TCP 連線進來的 frame
- DB cursor 的 rows
- 從 channel 收到的訊息
- WebSocket message stream

## `StreamExt` — 方便 method 都在這

```rust
use futures::stream::{self, StreamExt};

let mut s = stream::iter(vec![1, 2, 3]);
while let Some(x) = s.next().await { ... }
```

`StreamExt` 提供跟 Iterator 類似的 adapter：

| Adapter | 行為 |
|---------|------|
| `next()` | 拿下一個 |
| `map(f)` | 每個 item 變換 |
| `filter(f)` | f 回 `impl Future<Output = bool>` |
| `filter_map(f)` | filter + map |
| `take(n)` / `skip(n)` | 前 n / 跳 n |
| `take_while(f)` / `skip_while(f)` | 條件版 |
| `collect()` | 蒐集成 Vec / 等 |
| `fold(init, f)` | reduce |
| `for_each(f)` | 純 side effect 消費 |
| `chain(other)` | 串接 |
| `zip(other)` | 配對 |
| `enumerate()` | yield `(index, item)` |

## 並發處理：`buffer_unordered` / `buffered`

普通 `map(...).collect()` 是**順序**處理。要並發：

```rust
let results: Vec<_> = stream::iter(urls)
    .map(|u| async move { fetch(u).await })
    .buffer_unordered(10)    // 同時最多 10 個並發
    .collect()
    .await;
```

| Adapter | 行為 |
|---------|------|
| `buffered(n)` | 同時 n 個並發、保持原順序輸出 |
| `buffer_unordered(n)` | 同時 n 個、誰先好誰先輸出 |

是「web crawler / API batch」的核心。

## mpsc → Stream

`tokio::sync::mpsc` 的 `Receiver` 不是 Stream（避免綁定）。要轉：

```rust
use tokio_stream::wrappers::ReceiverStream;

let (tx, rx) = mpsc::channel(16);
let mut stream = ReceiverStream::new(rx);

while let Some(x) = stream.next().await { ... }
```

或是用 `BroadcastStream`、`WatchStream` 等其他 wrapper。

## 批次處理：`chunks` / `ready_chunks`

```rust
stream.chunks(100)         // 每滿 100 個 yield Vec
stream.ready_chunks(100)   // 有多少拿多少（最多 100）
stream.chunks_timeout(100, Duration::from_millis(50))   // 滿或超時
```

實作「批次寫 DB」、「批次 flush」的標準模式。

## 自己實作 Stream

罕見場景才寫。一般用 `async-stream` macro：

```rust
use async_stream::stream;

let s = stream! {
    for i in 0..10 {
        yield i;
        sleep(Duration::from_millis(10)).await;
    }
};
tokio::pin!(s);
while let Some(v) = s.next().await { ... }
```

`yield` 是 async-stream 提供的偽關鍵字，編譯期改寫成 state machine。

## Stream vs Iterator vs Channel

| | Iterator | Stream | Channel |
|---|---|---|---|
| sync/async | sync | async | async |
| 推/拉 | 拉 (next) | 拉 (next) | 推（producer 主動 send） |
| 多 receiver | 不行 | 一般不行 | broadcast/watch 可 |
| 反壓 | 沒 | 自然有（async） | bounded channel |

實務上：**producer → mpsc → consumer 用 Stream wrap**。Stream API 漂亮、可 chain adapter。

## 跟 Iterator 的差異

```rust
// Iterator
let v: Vec<i32> = iter.map(f).filter(g).collect();

// Stream — 注意 filter 的 closure 要回 Future
let v: Vec<i32> = stream
    .map(f)
    .filter(|x| futures::future::ready(g(x)))   // <-- 需 Future
    .collect()
    .await;
```

許多 stream adapter 接受 closure 要回 `Future<Output = ...>`——因為 stream 本身是 async，predicate 也可能需要 IO。如果 predicate 是 sync，包 `futures::future::ready(...)`。

## 用途總表

| 場景 | Stream 用法 |
|------|-------------|
| 並發 HTTP 請求 | `iter.map(fetch).buffer_unordered(10)` |
| TCP server 接 connection | `TcpListener` → stream of connections |
| WebSocket 接 message | `ws_stream` 直接是 Stream |
| 批次寫 DB | `stream.chunks(1000).for_each(write_batch)` |
| 限速處理 | `stream.throttle(Duration::from_millis(100))` |
| 收到事件做 fan-out | 一個 stream 用 `broadcast` 給多 consumer |

## 對照 TypeScript

Node 端對應的是 **async iterator**（`for await...of`）+ Node Streams + RxJS Observable。Rust `Stream` 跟 async iterator **概念一致**——`next()` 是 async 的 Iterator——但 Rust 的 adapter chain（`buffer_unordered` 等）對並發處理更直接。

### 對照表

| 需求 | TypeScript / Node | Rust |
|---|---|---|
| async iterator 概念 | `AsyncIterable<T>` / `for await ... of` | `Stream<Item = T>` / `.next().await` |
| 自己定義 | `async function*() {}` | `async_stream::stream!` / 自己 impl `Stream` |
| 變換 | `for await` + 自己 map / RxJS `map` | `.map(f)` |
| 過濾 | RxJS `filter` / 手寫 | `.filter(\|x\| futures::future::ready(...))` |
| 蒐集 | `for await + push` / `Array.fromAsync(...)` | `.collect::<Vec<_>>().await` |
| 並發 N 個處理（保留順序） | `Promise.all`（一次全 spawn） | `.buffered(N)` |
| 並發 N 個處理（順序不重要） | `p-map`（concurrency 選項） | `.buffer_unordered(N)` |
| 批次 | RxJS `bufferCount` / 自己存 | `.chunks(n)` / `.chunks_timeout(n, d)` |
| 限速 | RxJS `throttleTime` | `.throttle(d)` |
| 串接 | RxJS `concat` | `.chain(other)` |
| Channel → stream | Stream API readable | `tokio_stream::wrappers::ReceiverStream` |
| Node Streams | `stream.Readable` / pipeline | `tokio::io::AsyncRead` + `tokio_util::io` 包成 stream |
| Backpressure | Node Streams 內建 | bounded channel + stream |

### 程式碼對照

並發 fetch：

```ts
// TS — p-map（保留順序）
import pMap from 'p-map';

const results = await pMap(
  urls,
  async (url) => await fetch(url),
  { concurrency: 10 }
);
```

```rust
// Rust — buffer_unordered（不保留順序、最快）
use futures::stream::{self, StreamExt};

let results: Vec<_> = stream::iter(urls)
    .map(|u| async move { fetch(u).await })
    .buffer_unordered(10)
    .collect()
    .await;

// 想保留原順序：.buffered(10)
```

async generator vs async_stream：

```ts
// TS — async generator
async function* fibStream(): AsyncGenerator<number> {
  let a = 0, b = 1;
  while (true) {
    yield a;
    await new Promise(r => setTimeout(r, 10));
    [a, b] = [b, a + b];
  }
}

for await (const v of fibStream()) {
  console.log(v);
  if (v > 100) break;
}
```

```rust
// Rust — async_stream macro
use async_stream::stream;
use tokio_stream::StreamExt;

let s = stream! {
    let (mut a, mut b) = (0u64, 1u64);
    loop {
        yield a;
        tokio::time::sleep(Duration::from_millis(10)).await;
        (a, b) = (b, a + b);
    }
};
tokio::pin!(s);

while let Some(v) = s.next().await {
    println!("{v}");
    if v > 100 { break; }
}
```

批次處理：

```ts
// TS — RxJS bufferCount + bufferTime
import { fromEvent, bufferTime } from 'rxjs';

source$.pipe(
  bufferTime(500),   // 每 500ms 一批
).subscribe(batch => writeBatch(batch));
```

```rust
// Rust — chunks_timeout
let mut stream = source
    .chunks_timeout(100, Duration::from_millis(500));
while let Some(batch) = stream.next().await {
    write_batch(batch).await;
}
```

### 心智模型差異

1. **Stream ≈ async Iterator**。TS `AsyncIterable` 跟 Rust `Stream` 概念一致——「**async 版的 iterator**」，`next` 是 async。`for await` 對應 `while let Some(x) = stream.next().await`。
2. **`buffer_unordered` 是並發處理的核心 idiom**。對應 TS `p-map` + concurrency 選項。Rust 在 stream 層級提供，可跟其他 adapter 自然 chain。差別：unordered 版會打亂順序（誰先完成誰先 yield），ordered 版（`buffered`）保留原順序但慢。
3. **adapter closure 通常要回 Future**。`filter(|x| futures::future::ready(g(x)))` 看起來啰嗦——因為 stream 是 async，predicate 也可能要 IO。對應 TS RxJS 不需要這層 wrap，因為 RxJS observable 不必預期 IO。
4. **mpsc 不是 stream**。tokio mpsc Receiver 不直接 impl Stream（避免綁定）。要當 stream 用要 `ReceiverStream::new(rx)` wrap。對應 TS Stream API 是兩種設計選擇——Rust 故意分開。
5. **Stream 跟 Iterator 共用很多 adapter 名字**。`map` / `filter` / `take` / `enumerate` / `chain` / `zip` 全在——學了第 15 章的 iterator 直接遷移大半。差別只在 closure 多半要回 Future、消費要 `.await`。
6. **`async-stream` 提供 yield 關鍵字**。Rust 沒有 native generator（`gen` 關鍵字還在演進）；`async-stream` crate 用 macro 模擬 TS async generator 的寫法。`yield` 在 macro 內合法。
7. **無限 stream 要小心 `collect`**。`stream.collect().await` 永遠等不完。先 `take(n)` / `take_while` / `take_until` 限縮。對應 TS `for await` 配 `break`。
8. **stream 跟 Node Streams 不同**。Node Streams API（`stream.Readable` 等）對應 Rust `tokio::io::AsyncRead`（byte stream）+ `tokio_util::io::ReaderStream` wrap 成 byte stream。`futures::Stream<Item = T>` 是更高層的 typed stream，更接近 RxJS Observable。

## 常見陷阱

1. **忘 await stream consumer** — `stream.collect()` 本身回 Future，沒 await 就是廢的。
2. **closure 沒 wrap 成 Future** — sync predicate 要 `futures::future::ready(...)` 包。
3. **無限 stream + `collect`** — 永遠 collect 不完。先 `take(n)` 或 `take_while`。
4. **跨 await 借用** — stream adapter 的 closure 也是 async，注意 lifetime。
5. **每個 item 都 spawn task** — 用 `buffer_unordered` 並發，不要 `map(|x| spawn(...))`。

## 練習

1. 用 `stream::iter(1..=100).map(fake_slow_fn).buffer_unordered(5).collect::<Vec<_>>()`，量並發 5 vs 10 的時間。
2. 把第 29 章的 mpsc demo 改成 Stream 風格 + `StreamExt::for_each`。
3. 用 `chunks_timeout` 寫一個 batch writer：每 100 筆或 500ms 寫一次。
4. 用 `async-stream` 寫一個 fibonacci stream（無限），`take(10).collect()`。

## 延伸閱讀

- [`futures::stream` 文件](https://docs.rs/futures/latest/futures/stream/index.html)
- [`tokio-stream`](https://docs.rs/tokio-stream/)
- [Async Book — Streams](https://rust-lang.github.io/async-book/05_streams/01_chapter.html)
