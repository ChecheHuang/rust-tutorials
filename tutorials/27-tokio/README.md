# 27. Tokio 執行期

> 範圍：#[tokio::main]、runtime、spawn、block_on、cooperative scheduling

## Tokio 是什麼

Tokio 提供：
1. **執行期（runtime）**：跑 async task 的 scheduler + IO event loop
2. **async IO 原語**：`tokio::fs`、`tokio::net`、`tokio::time`
3. **同步原語**：async-aware 的 `Mutex`、`RwLock`、`channel`
4. **生態**：90% 的 Rust async lib 都建立在 Tokio 之上

是 production Rust async 的事實標準。

## Runtime 兩種風格

### multi-thread（預設）

```rust
#[tokio::main(worker_threads = 4)]
async fn main() { ... }
```

- N 個 worker thread（預設 = CPU 核數）
- task 可在 thread 間被 work-steal
- 高吞吐、適合 server

### current-thread

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() { ... }

// 或手動：
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;
rt.block_on(async { ... });
```

- 只用一個 thread
- 適合 GUI、CLI、embedded、整合既有 thread
- task 必須 collaborative

## spawn — 派 task 進 runtime

```rust
let handle = tokio::spawn(async {
    // task body
    42
});
let value = handle.await.unwrap();   // JoinHandle<T>
```

- `spawn` 立刻把 future 提交 scheduler
- 回傳 `JoinHandle<T>`，可 `.await` 拿結果
- multi-thread runtime 下 future 須 `Send + 'static`
- 若 task panic，`handle.await` 回 `Err(JoinError)`

## block_on — 從同步呼叫 async

```rust
let rt = tokio::runtime::Runtime::new()?;
rt.block_on(async {
    let user = fetch_user(1).await;
});
```

`block_on` 占用當前 thread 跑這個 future 直到完成。常見場景：
- `main` entry（`#[tokio::main]` 背後做的事）
- 測試中需要跑 async code
- 與舊有 sync code 整合

⚠️ **不要**在 async 內叫 `block_on`——會 deadlock（runtime 想 spawn 工作但 thread 被自己 block 住）。

## CPU bound：用 `spawn_blocking`

async runtime 預設給 IO 工作用。**長時間 CPU 計算會卡 scheduler**（block 住整個 worker thread，其他 task 沒法輪到）。

正確做法：

```rust
let handle = tokio::task::spawn_blocking(|| {
    // CPU heavy work
    expensive_calculation()
});
let result = handle.await.unwrap();
```

`spawn_blocking` 跑在獨立的 blocking thread pool（預設 512 個），不影響 main runtime。

判斷準則：**超過 10 µs 的 sync work** → 考慮 `spawn_blocking`。

## Cooperative scheduling

Tokio 是 **cooperative**——task 必須主動 yield 才能讓出。`.await` 點是天然 yield 點：

```rust
// 不會卡其他 task
async fn good() {
    for _ in 0..100 {
        do_io().await;   // yield 點
    }
}

// 會卡（純 CPU loop 沒 yield）
async fn bad() {
    for _ in 0..1_000_000_000 {
        compute();        // 沒 .await 不會 yield
    }
}
```

長 CPU loop 中要主動：

```rust
for i in 0..N {
    compute(i);
    if i % 1000 == 0 {
        tokio::task::yield_now().await;   // 讓 scheduler 跑別人
    }
}
```

## JoinSet — 管理一群 task

```rust
let mut set = task::JoinSet::new();
for i in 0..10 {
    set.spawn(async move { i * 2 });
}
while let Some(res) = set.join_next().await {
    println!("{:?}", res?);
}
```

比 `Vec<JoinHandle>` 好用：
- 動態加 task
- `join_next` 拿**最先完成**的（不是 spawn 順序）
- drop 時自動 cancel 未完成的

## 取消（cancellation）

Tokio task **可隨時取消**：

```rust
let handle = tokio::spawn(async {
    loop { sleep(Duration::from_millis(100)).await; }
});
handle.abort();           // 取消
handle.await.unwrap_err();
```

但「取消」只在 await 點發生。`drop` future 也會取消，這是 `select!` 的基礎。

⚠️ **取消安全（cancel safety）**：寫 async code 時要考慮「**任何 await 點都可能被取消**」。中途取消後狀態還對嗎？這是 async Rust 的進階課題。

## tokio::time

```rust
use tokio::time::{sleep, timeout, interval, Duration};

sleep(Duration::from_secs(1)).await;          // 等

let r = timeout(Duration::from_secs(5), op()).await; // 超時
// r: Result<OpOutput, Elapsed>

let mut tick = interval(Duration::from_secs(1));
loop {
    tick.tick().await;
    do_work();
}
```

## Runtime 設定

```rust
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(8)
    .max_blocking_threads(256)
    .thread_name("my-worker")
    .thread_stack_size(2 * 1024 * 1024)
    .enable_io()
    .enable_time()
    .build()?;
```

`#[tokio::main]` 提供 `flavor`、`worker_threads`、`current_thread` 等 macro 參數。

## 對照 TypeScript

Tokio 對應 Node 的 **libuv event loop + Worker**，但**手動可選**而非預設。Node 一定有 event loop，Rust 要 `#[tokio::main]` 才有；Node 用 Worker thread 處理 CPU 密集，Rust 用 `spawn_blocking` 丟給專屬 blocking pool。

### 對照表

| 需求 | Node | Tokio |
|---|---|---|
| Event loop | libuv（內建、單一） | tokio runtime（可建多個） |
| 跑 async fn | 自動 | `#[tokio::main]` / `Runtime::new().block_on(...)` |
| 工作 thread | Worker（重，要 postMessage） | `tokio::spawn`（同 memory，輕） |
| CPU 密集 | Worker | `tokio::task::spawn_blocking` |
| 一個 thread vs 多 thread | 預設單緒 + Worker | `current_thread` flavor vs `multi_thread`（預設） |
| Sleep | `setTimeout` / `await sleep` | `tokio::time::sleep` |
| Timer / Interval | `setInterval` | `tokio::time::interval` |
| Timeout | `Promise.race` + setTimeout | `tokio::time::timeout` |
| 等多個 Promise | `Promise.all` / `Promise.allSettled` | `tokio::join!` / `JoinSet` |
| Cancel | AbortController | `JoinHandle::abort()` / drop |
| Graceful shutdown | 自己接 SIGTERM | `tokio::signal` + `CancellationToken` |
| Cooperative scheduling | event loop 微任務 | tokio cooperative scheduler |

### 程式碼對照

```ts
// TS — Node async
async function main() {
  await new Promise(r => setTimeout(r, 1000));   // sleep
  
  // 並發
  const [a, b] = await Promise.all([fetch1(), fetch2()]);
  
  // 超時
  const v = await Promise.race([
    fetch1(),
    new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error('timeout')), 5000)
    ),
  ]);
}
main();
```

```rust
// Rust — tokio
#[tokio::main]
async fn main() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 並發
    let (a, b) = tokio::join!(fetch1(), fetch2());
    
    // 超時
    let v = tokio::time::timeout(Duration::from_secs(5), fetch1()).await?;
}
```

CPU 密集：

```ts
// TS — Worker
const { Worker } = require('worker_threads');
const w = new Worker('./cpu-work.js');
w.postMessage(data);
w.on('message', (result) => console.log(result));
```

```rust
// Rust — spawn_blocking（同 process）
let handle = tokio::task::spawn_blocking(move || {
    expensive_cpu_work(data)
});
let result = handle.await.unwrap();
```

### 心智模型差異

1. **runtime 可組合**。Node 一個 process 一個 event loop；Rust 一個 process 可有多個 tokio runtime（測試常用、library 想隔離也用）。每個 runtime 是獨立 thread pool + scheduler。
2. **`current_thread` vs `multi_thread`**。Node 永遠單緒 event loop + Worker；Rust 兩種 flavor：`current_thread`（單緒、跟 Node 像）、`multi_thread`（多 worker、可 work-steal、預設）。GUI / embedded 用 current_thread，server 用 multi_thread。
3. **`spawn_blocking` 是 Worker 的輕量版**。Node Worker 重（獨立 V8 instance、postMessage 序列化）；Rust `spawn_blocking` 把 sync 函式丟給專用 blocking pool（512 個 thread），共享 memory、無序列化。CPU 密集或 sync IO（檔案、舊版 DB client）都丟這。
4. **不要在 async 內 `std::thread::sleep`**。會卡整個 worker thread（不是只卡你這個 task），所有 task 都停。Node 對應錯誤是寫 sync 程式碼（`fs.readFileSync`）卡 event loop。
5. **`block_on` 巢狀會 deadlock**。寫 `rt.block_on(async { rt.block_on(...) })` 直接 panic——runtime 想 spawn 工作但 thread 被自己 block 住。Node 沒這問題（沒人會手動建 event loop）。
6. **Cooperative scheduling 細節**。Tokio task 必須主動 yield（`.await` 點是天然 yield 點）。長 CPU 計算沒 await 點就不會讓位——`tokio::task::yield_now().await` 主動讓。Node event loop 也有「微任務 starvation」類似問題。
7. **Cancel = drop**。Tokio task 在任何 `.await` 點都可能被取消（drop future）——對應 TS AbortController 但更隱含。寫 async 程式碼要思考「中途斷在 await 點，state 還對嗎」（cancel safety）。
8. **`#[tokio::main]` 是 macro**。展開成 `fn main() { Runtime::new().unwrap().block_on(async { ... }) }`。對應 TS 不需要 marker——任何 async function 在 Node 都能跑。

## 常見陷阱

1. **`std::thread::sleep` 在 async 內** — 整個 worker thread 卡住。永遠用 `tokio::time::sleep`。
2. **大量 sync DB 呼叫** — 沒包 `spawn_blocking`，整個 runtime 堵死。
3. **panic 沒處理** — task panic 後 `handle.await` 回 `JoinError`；server loop 沒檢查的話悄無聲息死掉。
4. **`block_on` 巢狀** — 第二個 `block_on` 直接 panic。
5. **spawn 過於頻繁** — spawn 不是免費，每次都要 heap alloc。loop 內密集 spawn 考慮用 channel + 持續 task。

## 練習

1. 寫一個 server-style loop：每秒 tick 一次印當前時間；用 `tokio::time::interval`。
2. spawn 100 個 task，每個各自 `spawn_blocking` 跑 CPU 工作，看 blocking pool 怎麼工作。
3. 用 `timeout` 包一個慢函式，分別測試 5 秒 / 500ms 兩種設定。
4. 寫 `JoinSet` 版本的 web crawler：先 spawn 10 個 URL fetch，第一個完成後再 spawn 下一個，維持 10 個並發。

## 延伸閱讀

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Tokio Docs](https://docs.rs/tokio/)
- [Inside Tokio's Scheduler](https://tokio.rs/blog/2019-10-scheduler)
