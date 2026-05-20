# 29. async 同步原語與通道

> 範圍：tokio Mutex / RwLock、mpsc / broadcast / watch / oneshot

## 四種 channel — 各有所用

| Channel | 行為 | 用途 |
|---------|------|------|
| **`mpsc`** | 多 sender / 單 receiver | task → worker pipeline、event queue |
| **`oneshot`** | 一次性、單 sender / 單 receiver | 「等一個結果」（RPC、completion） |
| **`broadcast`** | 多 sender / 多 receiver、訊息**複製**給每個 receiver | pub/sub、通知 |
| **`watch`** | 單 sender / 多 receiver、只看**最新值** | config reload、state propagation |

```rust
use tokio::sync::{mpsc, oneshot, broadcast, watch};
```

## `mpsc` — Multi-Producer Single-Consumer

```rust
let (tx, mut rx) = mpsc::channel::<i32>(8);   // 容量 8

// producer
tx.send(42).await?;   // .await，滿了會等

// consumer
while let Some(msg) = rx.recv().await {
    process(msg);
}
```

- **bounded**：滿了 sender block；給 back-pressure
- **unbounded** (`mpsc::unbounded_channel()`)：永不 block，但無背壓 — **慎用**

關鍵：**所有 sender drop 後 `rx.recv()` 回 `None`**。所以 producer side 要 drop sender 讓 consumer 知道結束。

## `oneshot` — 一次性

```rust
let (tx, rx) = oneshot::channel::<String>();

tokio::spawn(async move {
    let result = compute().await;
    tx.send(result).unwrap();   // 同步 send（不必 await）
});

let value = rx.await?;
```

只能 send 一次。常見場景：spawn 一個 task 算結果、main 用 `rx.await` 等。

實作 RPC pattern 的關鍵：

```rust
struct Request { args: Args, reply: oneshot::Sender<Response> }

// caller
let (tx, rx) = oneshot::channel();
worker_tx.send(Request { args, reply: tx }).await?;
let response = rx.await?;
```

## `broadcast` — 廣播

```rust
let (tx, _) = broadcast::channel::<&str>(16);

let mut rx1 = tx.subscribe();
let mut rx2 = tx.subscribe();

tx.send("hello")?;
// rx1.recv().await -> Ok("hello")
// rx2.recv().await -> Ok("hello")  // 同個訊息，複製給每個 sub
```

- 每個 receiver 都收到**所有**訊息
- 容量滿了會丟最舊的；慢 receiver 會收到 `Lagged` error
- 訊息型別必須 `Clone`

典型場景：聊天室、WebSocket 廣播、events fanout。

## `watch` — 只在乎最新值

```rust
let (tx, mut rx) = watch::channel(0u64);

tokio::spawn(async move {
    loop {
        let val = *rx.borrow();
        process(val);
        rx.changed().await.unwrap();   // 等到變了
    }
});

tx.send(1)?;
tx.send(2)?;   // receiver 可能跳過 1，直接看到 2
```

- receiver 永遠看得到**最新值**，中間的會被覆蓋
- 適合 config reload、status broadcast、heartbeat

## async Mutex

```rust
use tokio::sync::Mutex;

let m = Arc::new(Mutex::new(0));
let mut g = m.lock().await;   // .await！
*g += 1;
```

跟 `std::sync::Mutex` 差別：
- `lock()` 是 async（不會 block thread，會 yield）
- guard 是 `Send`，**可跨 await 持有**
- 但 lock 開銷比 std 大（建議：cross-await 才用）

## async RwLock

```rust
use tokio::sync::RwLock;

let r = Arc::new(RwLock::new(data));

let g = r.read().await;     // 多個並發
let mut g = r.write().await; // 獨佔
```

讀多寫少場景比 Mutex 好。同上：跨 await 安全。

## Semaphore — 限流

```rust
use tokio::sync::Semaphore;

let sem = Arc::new(Semaphore::new(10));   // 最多 10 並發

for url in urls {
    let permit = sem.clone().acquire_owned().await?;
    tokio::spawn(async move {
        fetch(url).await;
        drop(permit);
    });
}
```

控制並發數的標準工具。

## Notify — async condition variable

```rust
use tokio::sync::Notify;

let notify = Arc::new(Notify::new());

let n = notify.clone();
tokio::spawn(async move {
    n.notified().await;
    println!("woke up!");
});

sleep(Duration::from_secs(1)).await;
notify.notify_one();
```

像 condvar，但不需要綁定 Mutex。

## CancellationToken — 取消傳播

```rust
use tokio_util::sync::CancellationToken;   // 需 tokio-util crate

let token = CancellationToken::new();
let child = token.child_token();

tokio::spawn(async move {
    tokio::select! {
        _ = child.cancelled() => println!("cancelled"),
        _ = work() => println!("done"),
    }
});

token.cancel();   // 取消所有 child
```

實作 graceful shutdown 的標準工具。

## 選型總表

| 需求 | 用 |
|------|---|
| pipeline / task 互傳訊息 | `mpsc` |
| 等一個 spawn 的結果 | `oneshot` |
| pub/sub 多訂閱者 | `broadcast` |
| 共享 latest state | `watch` |
| 跨 await 互斥 | `tokio::sync::Mutex` |
| sync code 互斥 | `std::sync::Mutex` |
| 讀多寫少 | `tokio::sync::RwLock` |
| 限制並發數 | `Semaphore` |
| 簡單的「等通知」 | `Notify` |
| 取消傳播 | `CancellationToken` |

## 對照 TypeScript

Node 端對應的是 **EventEmitter / RxJS / postMessage 之間訊息傳遞**——比較鬆散，沒有「channel 型別」的標準工具。Rust 的 channel 是型別化、bounded、async-aware 的標準工具，且每種 channel 解一種特定問題。

### 對照表

| 需求 | TypeScript / Node | Rust（tokio::sync） |
|---|---|---|
| 多 producer → 單 consumer | EventEmitter / Stream / mpsc 自己包 | `mpsc::channel` |
| 一次性結果（spawn 完拿） | Promise resolve | `oneshot::channel` |
| pub/sub 多訂閱 | EventEmitter / RxJS Subject | `broadcast::channel` |
| 共享最新值 | RxJS BehaviorSubject | `watch::channel` |
| 跨 async 互斥 | mutex lib（少用） | `tokio::sync::Mutex` |
| 讀多寫少 | 自己包 | `tokio::sync::RwLock` |
| 限流並發 | p-limit / 自己寫 | `Semaphore` |
| 條件變數 | 自己用 Promise 兜 | `Notify` |
| 取消傳播 | AbortController + signal | `CancellationToken` |
| Bounded（背壓） | 大多 unbounded | bounded 是預設、慎用 unbounded |

### 程式碼對照

mpsc：

```ts
// TS — 自己用 EventEmitter 或包個 queue
import { EventEmitter } from 'events';
const ee = new EventEmitter();

ee.on('msg', (m) => process(m));
ee.emit('msg', 'hello');

// 嚴格背壓要自己刻
```

```rust
// Rust — mpsc with back-pressure
let (tx, mut rx) = mpsc::channel::<String>(8);

tokio::spawn(async move {
    while let Some(m) = rx.recv().await {
        process(m).await;
    }
});

tx.send("hello".into()).await?;   // 滿了會等
```

oneshot：

```ts
// TS — Promise 直接是 oneshot
const result = await new Promise<string>(resolve => {
  worker.on('message', resolve);
});
```

```rust
// Rust — oneshot
let (tx, rx) = oneshot::channel::<String>();

tokio::spawn(async move {
    let v = compute().await;
    tx.send(v).unwrap();
});

let v = rx.await?;
```

broadcast：

```ts
// TS — RxJS Subject
import { Subject } from 'rxjs';
const subject = new Subject<string>();

subject.subscribe(v => console.log('sub1', v));
subject.subscribe(v => console.log('sub2', v));

subject.next('hello');   // 兩個 sub 都收到
```

```rust
// Rust — broadcast
let (tx, _) = broadcast::channel::<String>(16);
let mut rx1 = tx.subscribe();
let mut rx2 = tx.subscribe();

tx.send("hello".into())?;
// rx1.recv().await -> Ok("hello")
// rx2.recv().await -> Ok("hello")
```

限流：

```ts
// TS — p-limit
import pLimit from 'p-limit';
const limit = pLimit(5);
const results = await Promise.all(
  urls.map(url => limit(() => fetch(url)))
);
```

```rust
// Rust — Semaphore
let sem = Arc::new(Semaphore::new(5));
let mut handles = vec![];
for url in urls {
    let s = sem.clone();
    handles.push(tokio::spawn(async move {
        let _permit = s.acquire().await?;
        fetch(url).await
    }));
}
```

### 心智模型差異

1. **channel 是型別化的**。Node 大多用 EventEmitter（任意 payload）或 callback 風格；Rust `mpsc::channel::<T>()` 明確型別，wrong payload 編譯失敗。對 refactor 跟設計極友善。
2. **背壓（back-pressure）是預設**。Rust bounded channel 滿了 sender 會 `.await`——下游慢，上游自然減速。Node 一般 unbounded（容易 memory leak）。`mpsc::unbounded_channel()` 存在但慎用。
3. **`oneshot` 取代了 Promise pattern**。對應 TS 寫法本來就是 Promise；Rust 因為 Future 是 lazy，要顯式 channel 把 spawn task 的結果送出來。RPC pattern（Request 帶 reply channel）是 Rust 慣用法。
4. **`broadcast` 跟 `watch` 區分得很細**。RxJS Subject / BehaviorSubject 對應 broadcast / watch，但 RxJS 是「**hot observable + operators**」哲學；Rust channel 是純粹的訊息容器，不附帶 operator。要 operator 用 `tokio_stream::StreamExt`（第 30 章）。
5. **`Semaphore` 是 first-class 限流工具**。Node `p-limit` 是 user-space lib；Rust `tokio::sync::Semaphore` 是 runtime 內建——`acquire().await` 拿不到 permit 就 yield，效率高。
6. **`CancellationToken` 是 AbortController 的對應**。Tokio 配 `tokio-util::sync::CancellationToken`，可父子傳播（cancel 父 → 子也 cancel）。對應 TS AbortSignal 的 `chain` 機制（較新）。
7. **跨 await 持鎖的選擇**。`std::sync::Mutex` 持鎖跨 await 編譯失敗（guard 不 Send）；`tokio::sync::Mutex` 設計給跨 await 用（lock 也是 async）。經驗法則：鎖只在 sync 範圍內持有用 std（快）、跨 await 用 tokio。
8. **「sender 全 drop → receiver 收 None」的協議**。對應 TS Stream `end` 事件、Iterator `done: true`——Rust mpsc receiver 在所有 sender 都 drop 時收 `None`。生產端要刻意 `drop(tx)` 結束消費，常踩坑。

## 常見陷阱

1. **sender 沒 drop，receiver 永等** — producer side 全部 drop sender 後 receiver 才會收到 None。常忘 `drop(tx)`。
2. **`broadcast` 慢 receiver 被 Lagged** — 處理 `Err(Lagged(n))`，要嘛 skip 要嘛 disconnect。
3. **`watch` 沒覆蓋已讀** — `borrow()` 拿值不會清掉「changed」標記，搭配 `mark_changed`。
4. **`mpsc::unbounded` 用得太爽** — 記憶體爆炸。production prefer bounded。
5. **跨 await 持 `std::Mutex` guard** — guard 不 Send。改 tokio Mutex 或縮 scope。

## 練習

1. 寫一個 worker pool：mpsc 收 task、N 個 worker 並行處理；用 oneshot 回 result。
2. 用 `broadcast` 實作極簡 chat：3 個 task 同時送 / 收訊息。
3. 用 `watch` 實作 config hot reload：一個 task 寫，多個 reader 讀，每次值變了 print 一次。
4. 用 `Semaphore` 限制 fetch URL 並發為 5，跑 100 個 URL，量總時間。

## 延伸閱讀

- [Tokio Sync 文件](https://docs.rs/tokio/latest/tokio/sync/)
- [Tokio Tutorial — Channels](https://tokio.rs/tokio/tutorial/channels)
