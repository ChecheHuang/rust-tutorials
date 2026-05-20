# 28. Send、Sync、Pin

> 範圍：Send / Sync 規則、Pin / Unpin、self-referential 結構

## Send 與 Sync — 跨執行緒的兩個 marker trait

```rust
unsafe trait Send {}    // T: Send 表示 T 可以「move」到另一個 thread
unsafe trait Sync {}    // T: Sync 表示 &T 可以跨 thread 共享
```

兩者都是 **marker trait**（沒有任何 method），只是給編譯器看的標記。

關係：**`T: Sync` ⟺ `&T: Send`**。

## 規則

Rust 編譯器**自動**判斷一個型別是否 Send / Sync——你不必寫 impl，但要懂規則：

- 所有 field 都 Send → 自身 Send
- 所有 field 都 Sync → 自身 Sync

例外（unsafe impl 標記成相反）：

| 型別 | Send | Sync |
|------|------|------|
| `i32`、`String`、`Vec<T>`、`HashMap` | ✅（內部全 Send） | ✅ |
| `Rc<T>` | **❌** | **❌** |
| `RefCell<T>` | ✅ | **❌** |
| `Cell<T>` | ✅ | ❌ |
| `Arc<T>` | ✅（若 T: Sync） | ✅（若 T: Sync） |
| `Mutex<T>` | ✅ | ✅（若 T: Send） |
| 原始指標 `*const T` / `*mut T` | ❌ | ❌ |

### 為什麼 `Rc` 不 Send

`Rc::clone` 改 refcount 是**非 atomic** 加法。兩個 thread 同時 clone 會 race。`Arc` 用 atomic 增減所以可以 send。

### 為什麼 `RefCell` 不 Sync

`borrow_mut()` 改的 internal counter 不是 atomic。多 thread 共享 `&RefCell` 時兩個都拿 `borrow_mut` 不會被擋。**不 Sync 就避免了這個錯誤**。

## 編譯期錯誤訊息

```
error[E0277]: `Rc<i32>` cannot be sent between threads safely
   --> src/main.rs:15:5
    |
15  |     thread::spawn(move || {
    |     ^^^^^^^^^^^^^ `Rc<i32>` cannot be sent
```

看到「cannot be sent between threads」= **缺 Send**。看到「cannot be shared between threads」= **缺 Sync**。

## async 與 Send 病毒擴散

`async fn` 編譯成 state machine struct。**狀態機的 Send 性**取決於跨 await 點時持有哪些東西。

```rust
async fn bad() {
    let r = Rc::new(5);
    yield_now().await;       // r 跨 await
    println!("{r}");
}

tokio::spawn(bad());           // ERROR: future is not Send
```

`r` 跨 await 點，所以 state machine 含 `Rc`，整個 future 非 Send。`tokio::spawn` 要 Send → 編譯失敗。

修法：
- 換用 `Arc<T>`
- 把非 Send 的東西用完就 drop（提前 scope）
- 用 `LocalSet` + `spawn_local`（current_thread runtime）

```rust
async fn ok() {
    {
        let r = Rc::new(5);
        // 不跨 await，scope 結束 drop
    }
    yield_now().await;
}
```

## `tokio::sync::Mutex` vs `std::sync::Mutex`

```rust
// std Mutex：lock() 是 blocking → 跨 await 危險（不 Send）
use std::sync::Mutex;
let m = Mutex::new(0);
let mut g = m.lock().unwrap();
something_else().await;       // ERROR: MutexGuard 不 Send + blocking
*g += 1;

// tokio Mutex：lock() 也是 async → 跨 await 安全
use tokio::sync::Mutex;
let m = Mutex::new(0);
let mut g = m.lock().await;
something_else().await;        // OK
*g += 1;
```

**經驗法則**：
- 鎖只在 sync code 內持有 → `std::sync::Mutex`（更快）
- 跨 await 持有 → `tokio::sync::Mutex`

## Pin — 阻止移動

`Pin<P>` 表示「`P` 指向的值**不會再被搬動**」。

### 為什麼需要

async fn 編譯後可能含 **self-referential** struct：

```rust
async fn example() {
    let buf = [0u8; 1024];
    let slice = &buf[..];        // 指標指回 buf
    use_slice(slice).await;
}
```

state machine 內同時有 `buf` 跟指向 `buf` 的指標。如果 future 被 move 到別的記憶體位址，內部指標**沒被更新**——指向錯誤位置。

`Pin` 的承諾：「**這個 future 一旦被 poll 就不會搬動**」。Future trait 強制 `poll` 拿 `Pin<&mut Self>` 就是這個原因。

### `Pin` vs `Unpin`

- **多數型別**實作 `Unpin`（意思：「我搬動沒差」），可以隨意 unpin。
- **未實作 `Unpin`**（如 async block 產生的 future）= 真的不能搬。

## 怎麼用 Pin

99% 場景**不必碰**——`.await` 與 `tokio::spawn` 內部都幫你 pin 好。

只有寫 **manual Future impl** 時要打交道：

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

impl Future for MyFuture {
    type Output = i32;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // self 是 Pin<&mut Self>，不能直接拿 &mut self
        // 用 `get_mut` 或 pin-projection
    }
}
```

或自己存 future：

```rust
use std::pin::Pin;
use futures::future::BoxFuture;

struct Holder {
    fut: Pin<Box<dyn Future<Output = i32> + Send>>,
}
```

`Box::pin(future)` 把 future 釘在 heap 上。

### `pin!` macro — stack pinning

```rust
use std::pin::pin;
let mut fut = pin!(async { 42 });
fut.as_mut().await;
```

把 future 釘在 stack 上，比 `Box::pin` 更快（無 heap alloc）。

## `'static` bound — 為什麼 spawn 要

`tokio::spawn<F: Future + Send + 'static>(f: F)`

`'static` 表示「F 內不含任何借用（除了 `&'static`）」。為什麼？

因為 spawn 後的 task 在另一個 thread 上**獨立執行**，可能比父 scope 活得更久。如果 task 內借了父 scope 的變數，父 scope 結束時借用就 dangle。

解法：把資料 move 進 closure（taking ownership），不要借用。

## 對照 TypeScript

**TS 幾乎完全沒對應**。Node 單緒設計讓你不必想 thread safety；跨 Worker 的物件是 postMessage 序列化、不共享記憶體（除了 `SharedArrayBuffer`）。Rust 把這層搬到語言核心——`Send` / `Sync` 是 marker trait，編譯期擋下 data race。

### 對照表

| 概念 | TypeScript / Node | Rust |
|---|---|---|
| 多 thread 共享物件 | Worker 不行（postMessage 序列化） | `Arc<T>` + T: Sync |
| 跨 thread 變數 move | postMessage | move closure |
| Data race | 預設不可能（單緒+Worker 隔離） | 編譯期擋下（Send/Sync 規則） |
| 共享記憶體 | SharedArrayBuffer + Atomics | `Arc<Mutex<T>>` |
| 不可跨 thread 的東西 | 什麼都可以（序列化過） | `Rc`、`RefCell`、raw pointer |
| Self-referential | GC 救（reference 永遠 valid） | `Pin<P>`、`PhantomPinned` |
| async 跨 await 借用 | 不必想 | 跨 await 持鎖要選對 Mutex |

### 程式碼對照

```ts
// TS — Worker 序列化，不會 data race
const w = new Worker('./worker.ts');
w.postMessage({ count: 0 });   // 物件序列化送過去

// SharedArrayBuffer 才有真共享，要自己用 Atomics 同步
const sab = new SharedArrayBuffer(4);
const view = new Int32Array(sab);
Atomics.add(view, 0, 1);
```

```rust
// Rust — 編譯期 thread safety
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));

let mut handles = vec![];
for _ in 0..10 {
    let c = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        *c.lock().unwrap() += 1;
    }));
}
for h in handles { h.join().unwrap(); }
// counter 一定是 10，編譯期保證 race-free
```

故意寫錯：

```rust
// Rc 跨 thread —— 編譯失敗
use std::rc::Rc;
let data = Rc::new(5);
thread::spawn(move || {
    println!("{}", data);   // ERROR: `Rc<i32>` cannot be sent between threads safely
});
```

```rust
// std::Mutex guard 跨 await —— 編譯失敗
let m = std::sync::Mutex::new(0);
async fn work() {
    let g = m.lock().unwrap();
    other_async().await;   // ERROR: future is not Send
    *g += 1;
}

// 改用 tokio::sync::Mutex
let m = tokio::sync::Mutex::new(0);
async fn work() {
    let mut g = m.lock().await;
    other_async().await;   // OK
    *g += 1;
}
```

### 心智模型差異

1. **`Send` / `Sync` 是看不見的 type-level 保證**。`Send` = 「能 move 到別的 thread」、`Sync` = 「`&T` 能跨 thread 共享」。一般型別**自動推導**——你不必寫 impl，但要懂規則：含 `Rc` 就不 Send、含 `RefCell` 就不 Sync。
2. **「Send 病毒擴散」是 async 痛點**。`async fn` 編譯成 state machine struct，內含**跨 await 持有的所有東西**。內部一個 `Rc<T>` 跨 await → 整個 future 不 Send → `tokio::spawn` 編譯失敗。修法：換 `Arc`、用完就 drop（縮 scope）、或 `spawn_local`（current_thread runtime）。
3. **`std::Mutex` vs `tokio::sync::Mutex` 不只是同步差異**。Std Mutex 的 guard 不 Send，跨 await 持有編譯失敗。Tokio Mutex 的 guard 是 Send 且 lock 本身是 async（不會卡 thread）。對應 TS 完全沒對應——Node 沒鎖（單緒）、Worker 也不共享變數。
4. **`Pin` 是 Rust 特有概念**。async fn 編譯後可能含 self-referential struct（同時持有 buffer + 指向 buffer 的指標）；如果 future 被 move，內部指標就 dangle。`Pin<P>` 承諾「**不再搬動**」。99% 不必碰，只有手寫 Future impl 時要懂。對應 TS：完全沒對應，GC 隨意 move 物件也沒事（reference 是抽象的）。
5. **`'static` 在 `tokio::spawn` 是「不能借父 scope」**。`tokio::spawn<F: Future + Send + 'static>` 表示 spawn 後 task 獨立執行，可能比父 scope 活得久——所以不能借父 scope 的變數。要嘛 move ownership 進去、要嘛只引用 `'static` 資料。TS Worker 序列化沒這問題。
6. **編譯失敗訊息要學會讀**。`cannot be sent between threads safely` = 缺 Send；`cannot be shared between threads safely` = 缺 Sync；`future is not Send` = state machine 內有 non-Send 東西跨 await。看訊息追原因，多半是某個 Rc / RefCell / `std::Mutex` guard。
7. **沒有 race condition runtime bug**。最大紅利。Rust 編譯過 = thread-safe（unsafe 除外）。對應 TS：Node 沒這紅利（單緒），但跨 Worker / SharedArrayBuffer + Atomics 場景沒 Rust 這層保護。
8. **Cancel safety 在跨 thread 也重要**。Tokio task 在 await 點可能被取消，持有 lock 中途取消後 lock 怎麼辦？Rust 的 RAII（drop 自動 unlock）救你——但邏輯一致性要自己想。

## 常見陷阱

1. **誤以為 Rc 跨 thread** — 編譯不過，換 Arc。
2. **MutexGuard 跨 await 編譯失敗** — std Mutex 不 Send；切 tokio Mutex 或縮 scope。
3. **async fn 內生命週期錯誤** — `&str` 參數綁定到呼叫者 scope，state machine 跨 await 易出問題；考慮 own 化（`String`）。
4. **`Box<dyn Future>` 不能 await** — 缺 Unpin。寫 `Pin<Box<dyn Future + Send>>`。
5. **`'static` 太黏** — 設計 API 時別動不動要 `T: 'static`；除非真要 spawn。

## 練習

1. 試把 `Rc<i32>` 放進 `tokio::spawn` 的 closure，看編譯訊息。
2. 寫 async fn 在跨 await 點各持 `std::sync::Mutex` 與 `tokio::sync::Mutex` 的 guard，比較編譯結果。
3. 寫一個 type `unsafe impl Send for ... {}` 的 wrapper 包裹原始指標（**不要**真的這樣用，純練習）。
4. 看 [`pin-project-lite`](https://docs.rs/pin-project-lite/) 怎麼用 macro 提供 pin-projection。

## 延伸閱讀

- [Async Book — Pinning](https://rust-lang.github.io/async-book/04_pinning/01_chapter.html)
- [`std::pin` 文件](https://doc.rust-lang.org/std/pin/)
- [Send / Sync deep dive](https://doc.rust-lang.org/nomicon/send-and-sync.html)
