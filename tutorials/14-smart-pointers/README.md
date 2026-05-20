# 14. 智慧指標

> 範圍：Box、Rc、Arc、RefCell、Mutex、Cow，何時用哪一個

## 決策樹

```
                        要跨 thread？
                       /            \
                      是             否
                      │              │
              要共享所有權？     要共享所有權？
              /        \         /         \
             是         否       是          否
             │         │         │          │
   ┌─────────┴───┐   單一  ┌────┴────┐   單一
   要可變？        擁有    要可變？    擁有
   /     \                  /    \
  是      否                是     否
  │      │                  │      │
Arc<Mutex<T>> Arc<T>   Rc<RefCell<T>> Rc<T>

      ↑                          ↑
   Box<T>: 任何時候要把資料移到 heap、做遞迴型別、做 trait object
   Cow<T>: 大多數時候借用，偶爾才複製
```

## `Box<T>` — 唯一擁有者，heap 配置

```rust
let b = Box::new(42);
println!("{}", *b);
```

跟 `T` 本身語意一樣（單一擁有者、move 語意），差別只是放 heap。

### 三大用途

1. **遞迴型別**（編譯期需固定大小）：
   ```rust
   enum List {
       Cons(i32, Box<List>),
       Nil,
   }
   ```
2. **trait object**：`Box<dyn Trait>`
3. **大型資料 move 時避免 stack copy**

## `Rc<T>` — Reference Counted（單執行緒）

```rust
let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a);
let c = Rc::clone(&a);
// 三個都指向同一份資料；最後一個 drop 時才釋放
```

- **共享 ownership**（多個 owner）
- 內部維護 strong count，歸零才 drop
- `Rc::clone(&a)` 只增 count，**不 deep copy**
- **不能跨 thread**（`Rc` 不是 `Send`）

### 何時要 Rc

樹/圖 結構共用 node、observer pattern 內共用 subject、UI tree 共用 context。

### Weak — 避免循環

```rust
use std::rc::Weak;
let strong = Rc::new(...);
let weak: Weak<...> = Rc::downgrade(&strong);
weak.upgrade();   // → Option<Rc<T>>
```

parent → child 用 `Rc`，child → parent 用 `Weak`，避免循環互引導致永遠不釋放。

## `RefCell<T>` — Runtime borrow check

```rust
let cell = RefCell::new(5);
*cell.borrow_mut() += 10;       // 暫時取得 &mut
println!("{}", cell.borrow());  // &T
```

跟 `&` / `&mut` 一樣的規則（多讀互斥），但**檢查時機從編譯期改成 runtime**——違反就 **panic**。

### 何時用

當你「**明知道**借用安全，但編譯器不肯放行」時。常見場景：
- struct 內某個 field 要「邏輯不可變但實際可變」（lazy cache、interior counter）
- 與外部 callback 互動（callback 借用 parent 又要修改）

### 跟 Rc 搭：`Rc<RefCell<T>>`

```rust
let v = Rc::new(RefCell::new(vec![1, 2, 3]));
let v2 = Rc::clone(&v);
v.borrow_mut().push(4);
v2.borrow_mut().push(5);
```

「共享可變」的單執行緒版。**90% 寫到 `Rc<RefCell<T>>` 都該想想能不能避開**——多數 case 用 owned + 傳 mutable reference 更乾淨。

## `Arc<T>` — Atomic Reference Counted（跨執行緒）

```rust
let data = Arc::new(vec![1, 2, 3]);
for _ in 0..3 {
    let d = Arc::clone(&data);
    thread::spawn(move || println!("{d:?}"));
}
```

跟 `Rc` 同 API，refcount 用 atomic 操作，可跨 thread。

代價：atomic 比一般加減慢 ~10x。但比 `Mutex` lock 快得多。

## `Mutex<T>` — Mutual Exclusion

```rust
let m = Mutex::new(5);
*m.lock().unwrap() += 1;
```

`lock()` 拿到 `MutexGuard<T>`，可 deref 成 `&mut T`。guard drop 時自動 unlock（RAII）。

### `Arc<Mutex<T>>` — 共享可變（跨執行緒）

```rust
let counter = Arc::new(Mutex::new(0));
for _ in 0..10 {
    let c = Arc::clone(&counter);
    thread::spawn(move || { *c.lock().unwrap() += 1; });
}
```

跨 thread 共享可變狀態的標準模式。

**陷阱**：lock poisoning。某 thread panic 時 mutex 被「污染」，後續 `lock()` 回 `Err`。常見處理：`unwrap()` 或 `into_inner()` 強拿。

### RwLock — 多讀單寫

```rust
use std::sync::RwLock;
let lock = RwLock::new(5);
let r = lock.read().unwrap();
// 多個 read() 可同時
let mut w = lock.write().unwrap();
// write() 獨佔
```

讀多寫少場景用 RwLock 比 Mutex 好。

## `Cow<T>` — Clone-on-Write

```rust
use std::borrow::Cow;
fn process(s: &str) -> Cow<str> {
    if s.contains(' ') {
        Cow::Owned(s.replace(' ', "_"))  // 需要改 → 配置
    } else {
        Cow::Borrowed(s)                  // 不需改 → 零 alloc 借用
    }
}
```

「大多數時候借用即可，偶爾才需要 own」的 API。`std::env::var_os` 回 `Cow<OsStr>` 就是這個模式。

`Cow` 是 enum，`Borrowed(&T)` 或 `Owned(T::Owned)`。對 caller 看起來像 `&T`（透過 deref）。

## 對比：Box vs Rc vs Arc

| | Box | Rc | Arc |
|---|---|---|---|
| owner | 唯一 | 多個 | 多個 |
| refcount overhead | 無 | 非 atomic | atomic |
| 跨 thread | 是 | **否** | 是 |
| 何時用 | 預設 | 單執行緒共享 | 多執行緒共享 |

效能：`Arc` ≈ 10× `Rc` ≈ 100× `Box`（refcount 變動時）。**先用 Box，需要共享才升級**。

## 對照 TypeScript

TS 沒有「智慧指標」這個概念——GC 替你管 lifetime、reference counting、aliasing 全部隱形。在 Rust 這些變成**顯式的型別選擇**：要共享所有權選 `Rc/Arc`、要 interior mutability 選 `RefCell/Mutex`、要 borrow-or-own 選 `Cow`。

### 對照表

| TS / JS 情境 | TS 用法 | Rust 對應 |
|---|---|---|
| 物件本來就 reference 共享 | `const b = a;`（兩個指同個） | `Rc::clone(&a)`（refcount + 1） |
| 跨 Worker 共享 | 不能直接（要 SharedArrayBuffer + Atomics） | `Arc<T>`（跨 thread refcount） |
| 子物件能反向改父狀態 | 隨意 | `Rc<RefCell<T>>`（interior mutability） |
| 跨 thread 共享可變 | postMessage 序列化 | `Arc<Mutex<T>>` / `Arc<RwLock<T>>` |
| 避免循環引用 | WeakMap / WeakRef | `Weak<T>` |
| 想避免 deep copy | 自然不會（GC reference） | `Cow<T>`（borrow until mutate） |
| 把資料放 heap | `new Object()`（自動） | `Box::new(value)` |
| trait object | interface 變數天然如此 | `Box<dyn Trait>` |
| 遞迴 data structure | `class Node { next?: Node }` | `enum List { Cons(i32, Box<List>), Nil }` |
| GC 何時收 | 不確定 | 立刻（owner drop 時） |

### 程式碼對照

```ts
// TS — 隨意共享物件 reference
const tree = { value: 1, children: [] as any[] };
const ref1 = tree;
const ref2 = tree;
ref1.children.push("a");
console.log(ref2.children);   // ["a"]，看得到
```

```rust
// Rust — 用 Rc<RefCell> 模擬「共享可變」
use std::rc::Rc;
use std::cell::RefCell;

struct Tree { value: i32, children: RefCell<Vec<String>> }

let tree = Rc::new(Tree { value: 1, children: RefCell::new(vec![]) });
let ref1 = Rc::clone(&tree);
let ref2 = Rc::clone(&tree);
ref1.children.borrow_mut().push("a".into());
println!("{:?}", ref2.children.borrow());   // ["a"]
```

跨 thread 共享：

```ts
// TS — 主執行緒之間共享物件不必想（單緒）
// 跨 Worker 要序列化或 SharedArrayBuffer
```

```rust
// Rust — Arc<Mutex<T>> 是跨 thread 標配
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
println!("{}", counter.lock().unwrap());   // 10
```

### 心智模型差異

1. **「共享 reference」要選工具**。TS 寫 `const b = a` 一秒解決；Rust 要選 `&` 借用、`Rc` 單緒 refcount、`Arc` 多緒 refcount、`Box` move 到 heap。乍看煩，實質效益：**你在程式碼中宣告了「我預期這份資料如何被共享」**，讀者一看就懂。
2. **`Rc<RefCell<T>>` 是 JS 物件的等價**。TS 物件天然「多 owner + 隨意 mutate」；Rust 要這個語意要 `Rc<RefCell<T>>`。但**多數 Rust 程式用了 `Rc<RefCell>` 都該退一步想**——是不是該用 owned + 借用、或重新組織資料流向。OK 用，但不要當預設。
3. **單緒 vs 多緒選對 refcount**。`Rc` 比 `Arc` 快 ~10×（非 atomic 加減）；跨 thread 必須 `Arc`，否則編譯失敗（`Rc` 不 `Send`）。看到 `Arc` 心理要警覺「這資料會被多個 thread 看到」。
4. **`Box<T>` 用得比想像多**。對應 TS 完全沒概念——`Box::new(x)` 把 x 從 stack 搬到 heap。用途：遞迴型別、`Box<dyn Trait>`、大 struct 移動避免複製、單一 ownership 仍想要 heap allocation。
5. **`Cow<T>` 是 TS 沒有的 idiom**。「**大多數時候借用就好，偶爾才需要 own**」的設計——`fn normalize(s: &str) -> Cow<str>` 不需要的場景零 alloc。TS 因為都是 reference 自然如此但不能拒絕 alloc。
6. **interior mutability 是「邏輯 immutable 但實際 mutable」**。lazy cache、counter 之類的——`&self` method 內部要改 field，但對外保持 immutable 介面。`RefCell` 把 borrow check 推遲到 runtime（違反就 panic）；`Cell<T>` 對 `Copy` 型別更簡單（直接 get/set 換值）。
7. **lock poisoning 是 Rust 特有**。Mutex 持有 thread panic 時 mutex 被「污染」，後續 `lock()` 回 `Err`——強迫你想「panic 後資料還一致嗎」。實務 90% 直接 `.unwrap()` 強拿；嚴格服務要 `match` 處理。TS 沒這概念。
8. **沒有 weak reference 就會循環 leak**。Rust `Rc` 循環互指不會自動釋放（refcount 永遠不歸零）。樹狀結構慣例：parent → child 用 `Rc`，child → parent 用 `Weak`。對應 TS `WeakRef`，但 Rust 必須**設計時就決定**否則永久 leak。

## 常見陷阱

1. **`Rc<RefCell<T>>` 上癮** — 很多時候根本不必，回到 owned + borrow。
2. **`Mutex` 死鎖** — 同一 thread 重複 `lock()` 不會被擋（std Mutex 不可重入），會死鎖。要小心 lock 範圍。
3. **借用跨 await** — 第 28 章詳述：跨 await 持有 `MutexGuard` 會炸（Send 規則）。Tokio 提供 `tokio::sync::Mutex`。
4. **`Rc::clone` vs `(*rc).clone()`** — 前者增 refcount（cheap），後者 deep copy 內容（expensive）。
5. **circular Rc 漏洞** — 樹/圖 結構要 weak reference，否則永久 leak。

## 練習

1. 用 `Box<Tree>` 實作二元樹，寫遞迴 `sum` method。
2. 用 `Rc<RefCell<Node>>` + `Weak<Node>` parent 指標，建一棵能向上追溯的樹。
3. 寫 `fn spawn_workers(data: Vec<i32>) -> i32`：把 data 用 `Arc` 分享給 4 個 thread 各算一段 sum，最後加總。
4. 用 `Cow<str>` 寫 `fn normalize(s: &str) -> Cow<str>`：只在含大寫時複製 lowercase。

## 延伸閱讀

- [The Rust Book — Ch 15 Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [`std::sync` 文件](https://doc.rust-lang.org/std/sync/)
