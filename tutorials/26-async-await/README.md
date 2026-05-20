# 26. async/await 與 Future

> 範圍：async fn、.await、Future trait、Poll、為什麼需要 executor

## 為什麼有 async

Rust 標準的 `std::thread::spawn` 是 **OS thread**：每個 thread 至少 ~2MB stack、context switch 要進 kernel。要服務 10K 並發連線時，10K thread 不可行（記憶體 / scheduling 都吃不消）。

**async** = 在**一個 OS thread 上**多工切換多個 task。每個 task 只是 stack frame + 一點 state；遇 IO 就 yield，OS poll 回來就 resume。**百萬級**並發都可能。

Rust 的 async：
- **編譯期**轉換 `async fn` 為 state machine
- **零成本**抽象（無 GC、無 virtual call）
- **runtime 可選**（不像 Go 內建 scheduler）——可選 Tokio、async-std、smol

## async fn

```rust
async fn fetch_data(id: u64) -> String {
    sleep(Duration::from_millis(200)).await;
    format!("data-{id}")
}
```

`async fn fetch_data(...)` 在編譯時被轉成：

```rust
fn fetch_data(...) -> impl Future<Output = String> { ... }
```

**呼叫 async fn 不會執行**，只是建立一個 `Future`。`Future` 像個「**懶**」的計算，必須交給 executor 跑：

```rust
let future = fetch_data(1);  // 還沒跑
let data = future.await;     // 現在才跑
```

## `.await`

`.await` 把 `Future` 跑到完成。在 async fn 內可用：

```rust
async fn handler() {
    let a = fetch_data(1).await;     // 等 a 完成
    let b = fetch_data(2).await;     // 再等 b 完成
}
```

注意這是 **sequential**——a 完成才開始 b。要並發要主動寫。

## 並發三種寫法

### 1. `tokio::join!` — 同時啟動，等全部

```rust
let (a, b) = tokio::join!(fetch_data(1), fetch_data(2));
```

兩個 future 在同個 task 內並發進行。**不會**跨 thread。

### 2. `tokio::spawn` — 真正派到 thread pool

```rust
let h1 = tokio::spawn(fetch_data(1));
let h2 = tokio::spawn(fetch_data(2));
let a = h1.await.unwrap();
let b = h2.await.unwrap();
```

`spawn` 把 task 交給 runtime，可能跑在另一個 OS thread。需要的 future 要是 `Send + 'static`。

### 3. `tokio::select!` — 哪個先完成走哪個

```rust
tokio::select! {
    n = compute() => println!("compute {n}"),
    _ = sleep(Duration::from_secs(5)) => println!("timeout"),
}
```

任一 future 完成，其他被取消。常見用法：**超時**、競爭資源、第一個 winner。

## `Future` trait

`async` 只是糖。底層：

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
```

`executor` 反覆叫 `poll`：
- 回 `Ready(value)` → 完成，拿值
- 回 `Pending` → 還沒好，executor 暫停這個 task，等被叫醒（`waker.wake()`）後再 poll

`async fn` 編譯成一個實作 `Future` 的隱藏 struct（state machine），每個 `.await` 點對應 state 轉換。

## 為什麼要 runtime

Rust 標準函式庫**不提供** executor。原因：embedded / WASM / server 對 runtime 的要求差很大，硬綁定就不通用。

選擇：

| Runtime | 風格 |
|---------|------|
| [`tokio`](https://tokio.rs) | 全功能、生態最大、production 標準 |
| `async-std` | API 像 std，較少維護 |
| `smol` | 輕量、可組合 |
| `embassy` | embedded（no_std） |

> 99% 場景選 **tokio**。本系列下章開始全部用 tokio。

## `#[tokio::main]`

```rust
#[tokio::main]
async fn main() {
    ...
}
```

這是 procedural macro，展開成：

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        ...
    });
}
```

`block_on` 是「同步等 async 完成」的入口。一個 binary 通常只在 main 用一次。

## async ≠ 平行

- **並發**（concurrency）：管理多個 task，可能交錯執行（單 thread 也可）
- **平行**（parallelism）：實際同時跑（多 CPU core）

async 預設**只是並發**——同個 thread 跑很多 task，遇 IO 就切換。`spawn` 才可能真的跨 thread 平行。

CPU bound 工作 prefer `tokio::task::spawn_blocking` 或 thread pool（rayon），不要丟給 async runtime。

## async 的痛點

1. **lifetime 變複雜**：跨 await 的借用要小心（第 28 章詳述）
2. **Send 病毒擴散**：spawn 的 future 要 Send，內部一個 `Rc` 就鏈式失敗
3. **`Pin` 概念**：self-referential struct 為什麼需要 pin（第 28 章）
4. **錯誤訊息長**：state machine 展開後的型別名稱可讀性極差
5. **生態雙態**：sync code 跟 async code 不能簡單互通；library 要選邊站

## 對照 TypeScript

`async` / `await` 表面語法**幾乎一樣**——這也是 Rust 故意設計的。但底層機制差很多：JS Promise 是「**eager**」（一建立就開跑），Rust Future 是「**lazy**」（不 await / 不 poll 就不跑）。

### 對照表

| 概念 | TypeScript / JS | Rust |
|---|---|---|
| async fn | `async function f()` | `async fn f()` |
| 等結果 | `await promise` | `.await` |
| 回傳型別 | `Promise<T>` | `impl Future<Output = T>` |
| 建立 ≠ 執行 | **eager**（建 Promise 就跑） | **lazy**（建 Future 不跑） |
| Runtime | V8 內建 event loop | **沒內建**，要選 tokio / async-std |
| 並發等全部 | `Promise.all([a, b])` | `tokio::join!(a, b)` |
| 並發等任一 | `Promise.race([a, b])` | `tokio::select! { ... }` |
| 派到其他 thread | Worker（要序列化） | `tokio::spawn(future)`（同 process） |
| 平行運算 | Worker 數量手動 | `spawn_blocking` / rayon |
| sleep | `setTimeout` / `await sleep(...)` | `tokio::time::sleep(...).await` |
| Cancel | AbortController + signal | task drop = cancel |
| 取消後狀態 | 看怎麼接 abort signal | cancel safety 是議題 |
| 錯誤 | `try/catch` 接 reject | `Result<T, E>` 配 `?` |
| Promise unhandled | unhandledRejection warning | 沒處理 Future 就被 drop |

### 程式碼對照

```ts
// TS — Promise eager
const p = fetchData(1);          // 已經開始 fetch
const v = await p;               // 拿結果

// 並發
const [a, b] = await Promise.all([fetchData(1), fetchData(2)]);

// 超時
const v = await Promise.race([
  fetchData(1),
  new Promise<never>((_, reject) =>
    setTimeout(() => reject(new Error('timeout')), 5000)
  ),
]);
```

```rust
// Rust — Future lazy
let f = fetch_data(1);           // 還沒 fetch（只是建了 future）
let v = f.await;                 // 現在才 fetch + 拿結果

// 並發（同 task 內）
let (a, b) = tokio::join!(fetch_data(1), fetch_data(2));

// 超時
let v = tokio::select! {
    v = fetch_data(1) => v,
    _ = tokio::time::sleep(Duration::from_secs(5)) => panic!("timeout"),
};
// 或更簡潔：
let v = tokio::time::timeout(Duration::from_secs(5), fetch_data(1)).await?;
```

派到 thread pool：

```ts
// TS — Worker 要序列化
const w = new Worker('./worker.ts');
w.postMessage({ task: 'work', data });
w.onmessage = (e) => console.log(e.data);
```

```rust
// Rust — 同 process spawn
let handle = tokio::spawn(async {
    do_async_work().await
});
let result = handle.await.unwrap();

// CPU 密集要用 spawn_blocking
let handle = tokio::task::spawn_blocking(|| {
    expensive_calc()
});
let result = handle.await.unwrap();
```

### 心智模型差異

1. **lazy vs eager 是最大差別**。TS `const p = fetchData()` 就已經發 HTTP request 了；Rust `let f = fetch_data()` 還沒做任何事。要 `.await` 或 `tokio::spawn` 才會跑。後果：忘 `.await` 不會炸，只是什麼都沒發生（編譯器有 `must_use` warning）。
2. **runtime 不是內建的**。Node 一裝下去就有 event loop；Rust 要選 `tokio`（99% 場景的選擇）。寫 `#[tokio::main] async fn main()` 才有 runtime。沒 runtime 的 async fn 是一個建得出來的 Future，但沒人 poll 它。
3. **`async` 不是並發**。Rust `let a = fetch(1).await; let b = fetch(2).await;` 是**順序**——a 完了才開始 b。TS 同寫法也是順序，但 JS 開發者多半知道；Rust 因為 Future lazy，必須**明確** `tokio::join!` / `tokio::spawn` 才並發。
4. **`spawn` 跨 thread 不必序列化**。TS Worker 之間要 postMessage（序列化）；Rust task 在同 process 同 memory 內，spawn 把 future 丟給 thread pool 直接跑——但編譯器要求 future 是 `Send + 'static`（第 28 章）。
5. **CPU bound 要 `spawn_blocking`**。Async runtime 用少數 worker thread 跑很多 task，遇到 CPU 密集會卡所有 task。`tokio::task::spawn_blocking` 丟給獨立 thread pool。Node 同樣建議 CPU 密集走 Worker——但實務 Node 服務常忽略，因為 Worker 用起來重。
6. **取消是 first-class**。TS 用 AbortController 手動傳 signal；Rust task drop 就是 cancel——`tokio::select!` 一個分支贏，其他自動 drop（取消）。**Cancel safety** 是 Rust async 的議題：「**這個 future 在任何 await 點被取消後，state 還對嗎**」。Node 較少遇到。
7. **錯誤型別整合**。TS async fn throw 變 Promise reject，try/catch 接；Rust async fn 回 `Result<T, E>`，`?` 仍然能用——`async fn f() -> Result<T, E>` 然後 `f().await?` 是日常。
8. **`.await` 是 yield 點**。Rust 編譯後的 async fn 是 state machine，`.await` 點是「**讓出 thread 給別人**」的位置。長 CPU loop **沒有 await 點**就不會 yield，要 `tokio::task::yield_now().await` 主動讓位。Node 因為單緒+event loop 也有類似議題。

## 常見陷阱

1. **忘 `.await`** — `fetch_data(1)` 沒 await 等於建了 future 就丟掉，編譯器會警告 `unused must_use`。
2. **`std::thread::sleep` in async fn** — block 整個 thread，其他 task 也停。要用 `tokio::time::sleep`。
3. **`std::sync::Mutex` 跨 await** — guard 跨 await 點會炸（不是 Send，且 block 整 thread）。用 `tokio::sync::Mutex`。
4. **spawn 的 future 不是 Send** — 內部用了 `Rc<T>` 就 GG。換 `Arc<T>`。
5. **async fn 內 `return ...?`** — `?` 只是早返 `Err`，async fn 仍正常完成 future；沒問題，但要記得 future 仍是 `Future<Output = Result<T, E>>`。

## 練習

1. 用 `tokio::join!` 同時 fetch 3 個 user，比較 sequential vs concurrent 的時間。
2. 寫 `with_timeout<F: Future>(f: F, dur: Duration) -> Option<F::Output>`，用 `select!`。
3. 把以下 sync code 轉成 async（用 `tokio::fs`）：
   ```rust
   let s = std::fs::read_to_string("a.txt")?;
   process(&s);
   ```
4. 用 `tokio::spawn` 開 10 個 task 各自 sleep 隨機時間，全部完成後印總時間。

## 延伸閱讀

- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [async/await internals — withoutboats](https://without.boats/blog/why-async-rust/)
