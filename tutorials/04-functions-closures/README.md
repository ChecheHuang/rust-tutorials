# 04. 函式與閉包

> 範圍：函式簽章、閉包、Fn / FnMut / FnOnce、高階函式、move closure

## 函式：expression-based

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b   // 無分號 = expression = 回傳值
}
```

關鍵：**有分號的 statement、無分號的 expression**。函式 body 是一個 block，block 是 expression，**最後一個 expression 即回傳值**。

```rust
fn double_block(x: i32) -> i32 {
    let doubled = {
        let temp = x * 2;
        temp + 0       // block 的回傳值
    };
    doubled
}
```

也可以用 `return`：

```rust
fn early_return(x: i32) -> i32 {
    if x < 0 { return 0; }
    x * 2
}
```

`return` 一般只在早返時用；尾端 expression 才是 idiomatic。

## 閉包：匿名函式 + 環境捕獲

```rust
let add_one = |x| x + 1;
add_one(4); // 5
```

- 參數型別**通常可推導**，不必寫
- 單行可省 `{}`
- 多行用 `|x| { ... }`

跟函式最大差別：**能捕獲外圍變數**。

## 三種捕獲方式

| 方式 | 觸發條件 | 對應 trait |
|------|----------|------------|
| 借用 `&T` | 只讀 | **`Fn`** |
| 借用 `&mut T` | 修改但不轉移所有權 | **`FnMut`** |
| 移動 `T` | 寫 `move` 或 by-value 用到 | **`FnOnce`**（或 `Fn` / `FnMut`，看內部行為） |

```rust
let s = String::from("hi");
let by_ref = || println!("{s}");        // Fn (borrow)
let mut n = 0;
let by_mut = || n += 1;                 // FnMut
let by_move = move || println!("{s}");  // 強制 move
```

關鍵：**`move` 只決定捕獲方式（move vs borrow），不決定呼叫次數限制**。能叫幾次取決於 closure body 對捕獲值做了什麼：
- 只讀 → `Fn`
- 修改 → `FnMut`
- 消費（drop / move out） → `FnOnce`

## Fn / FnMut / FnOnce — 三個 trait 的繼承關係

```
FnOnce  ←  FnMut  ←  Fn
（最寬鬆）        （最嚴格）
```

- 所有 closure 都實作 `FnOnce`（至少能叫一次）
- 不消費捕獲值的 closure 額外實作 `FnMut`
- 不修改也不消費的 closure 再額外實作 `Fn`

**接受 trait 的函式**呢？反過來：

```rust
fn want_fn<F: Fn()>(f: F)       { f(); f(); }       // 能叫多次
fn want_fn_mut<F: FnMut()>(mut f: F) { f(); f(); }  // 能叫多次（要 mut）
fn want_fn_once<F: FnOnce()>(f: F) { f(); }         // 只保證叫一次
```

要寫 generic API 時，**儘量寬鬆**：能用 `FnOnce` 就別要求 `Fn`。

## 高階函式

```rust
let nums = vec![1, 2, 3, 4, 5];
let sum: i32 = nums.iter().map(|x| x * 2).sum();
```

`map`、`filter`、`fold` 等吃 closure，造就 functional 風格。詳見第 15 章 iterators。

## 回傳 closure

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

`impl Trait` 在回傳位置代表「**一個實作 trait 的型別，編譯期決定**」。寫不出來具體型別（因為 closure 沒有可寫出的型別）時必用。

如需在執行期切換不同 closure 型別（同一函式回傳兩種以上），用 trait object：

```rust
fn pick_adder(positive: bool) -> Box<dyn Fn(i32) -> i32> {
    if positive {
        Box::new(|x| x + 1)
    } else {
        Box::new(|x| x - 1)
    }
}
```

`Box<dyn Fn>` 是 heap 配置 + 動態 dispatch（vtable）。詳見第 12 章 traits。

## function pointer `fn`

```rust
let mul: fn(i32, i32) -> i32 = |a, b| a * b;
```

`fn` 型別不能捕獲環境，跟 C function pointer 等價。**所有 fn 型別也都實作 `Fn` trait**——可以把 `fn` 傳給要 `Fn` 的地方，但反過來不行。

## 對照 TypeScript

函式與 closure 在表面上幾乎是直譯。**最大差異是 Rust 把 closure 切成 3 種 trait**（`Fn` / `FnMut` / `FnOnce`），來表達它對捕獲值做什麼——TS / JS 都是 GC + reference 語意，根本不必區分。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 函式定義 | `function f() {}` / `const f = () => {}` | `fn f() {}` |
| Arrow function | `(x) => x + 1` | closure：`|x| x + 1` |
| 隱式 return | arrow 單行：`(x) => x` | block 尾端 expression 無分號 |
| 顯式 return | `return x;` | `return x;`（**通常只在早返用**） |
| 預設參數 | `f(x = 0)` | **沒有**（用 `Option<T>` 或 builder） |
| 變長參數 | `...args: T[]` | **沒有**（用 slice 或 macro） |
| 命名參數 | 物件解構：`f({name})` | **沒有**（用 struct 參數） |
| Closure 捕獲 | 自動、reference | **三種**：`&T`、`&mut T`、`T`（`move`） |
| Closure 型別 | 函式 type alias | `Fn` / `FnMut` / `FnOnce` trait |
| 可呼叫多次保證 | 沒這概念 | `Fn`：多次；`FnOnce`：**只一次** |
| 高階函式 | first-class，無代價 | first-class，**static dispatch + zero-cost** |
| 回傳 closure | `(): () => T` 直接寫 | `-> impl Fn() -> T` 或 `Box<dyn Fn>` |
| 跨 thread 的 closure | 自然可（JS 主執行緒，Worker 要序列化） | 需 `Send` bound（編譯期擋住共享可變狀態） |

### 程式碼對照

```ts
// TS
const add = (a: number, b: number) => a + b;

const items = [1, 2, 3];
const doubled = items.map((x) => x * 2);

function makeCounter(): () => number {
  let n = 0;
  return () => ++n;
}
const c = makeCounter();
c(); c();   // 1, 2
```

```rust
// Rust
let add = |a: i32, b: i32| a + b;

let items = vec![1, 2, 3];
let doubled: Vec<i32> = items.iter().map(|x| x * 2).collect();

fn make_counter() -> impl FnMut() -> u32 {
    let mut n = 0;
    move || { n += 1; n }
}
let mut c = make_counter();
c(); c();   // 1, 2
```

注意 Rust 版的兩個差異：
- 必須 `move` 把 `n` 搬進 closure（不然 `n` 是 local，函式返回後就 drop）
- 必須宣告 `mut c`（因為 closure 是 `FnMut`，呼叫會改內部狀態）

### 心智模型差異

1. **closure 對捕獲的態度完全不同**。TS：closure 永遠對外面變數**保有 reference**，包含可變的（這就是經典 `for (var i ...) setTimeout()` bug 的原因）。Rust：編譯器分析 closure body 對捕獲值的使用，**選擇最寬鬆的捕獲方式**：只讀 → `&T`、有寫 → `&mut T`、有 move 出去 → `T`。
2. **`Fn` / `FnMut` / `FnOnce` 是「契約」**。當你寫 `fn callback<F: Fn()>(f: F)` = 「我會叫 f 多次，所以 f 不准消費自己捕獲的資源」。TS 沒這概念是因為沒有 ownership——可以無限叫。
3. **`move` 是「捕獲方式」不是「呼叫次數」**。常見誤解：以為 `move || ...` 就只能叫一次。實際上 `move || println!("{x}")` 仍是 `Fn`（沒消費 x），可以叫多次。
4. **closure 型別不可命名**。TS 可以寫 `type Cb = (x: number) => number` 然後到處用。Rust 每個 closure 是匿名型別，沒辦法寫 `Vec<MyClosure>`，必須 `Vec<Box<dyn Fn(i32) -> i32>>`（dynamic dispatch）或 generic over `F: Fn(...)`（static dispatch）。
5. **沒有預設參數 / 命名參數 / 變長參數**。TS 寫慣 `f({ name, age = 0 })`，Rust 等價解法是接 struct（builder pattern）或多載 trait 實作——一開始覺得 verbose，但呼叫端可讀性反而高。
6. **`Send` 病毒擴散**。把 closure 丟進 `thread::spawn` 或 `tokio::spawn` 時，closure 內所有捕獲值都必須 `Send`。TS Worker 之間是序列化傳遞（postMessage），沒這個概念；Rust 編譯期就把競爭條件擋下。

## 常見陷阱

1. **closure 的型別不可命名** — 即使兩個 closure 長得一樣，它們是不同型別。要存進 `Vec` 用 `Vec<Box<dyn Fn(...)>>`。
2. **`move` 不等於 `FnOnce`** — `move || x.clone()` 是 `Fn`（clone 不消費 x）。
3. **borrow checker 與 closure** — 閉包持有借用期間，原變數不可再被同樣借用：
   ```rust
   let mut v = vec![1, 2];
   let c = || v.push(3);    // c 借用 v as &mut
   // v.len();              // ERROR: v 還被 c 借用
   c();
   ```
4. **`Fn` 接收方 vs `&dyn Fn` vs `Box<dyn Fn>`** — 靜態 dispatch (`impl Fn`) 最快但 monomorphize 後 binary 變大；動態 dispatch (`dyn Fn`) 靈活但有 vtable 開銷。
5. **`return` 在 closure 內** — 只 return 出 closure，**不會** return 函式。常見 bug 來源。

## 練習

1. 寫 `apply<F: Fn(i32) -> i32>(f: F, n: i32) -> i32`，呼叫 `apply(|x| x*x, 5)`。
2. 寫 `make_counter() -> impl FnMut() -> u32`，每次叫回傳遞增整數。
3. 把 `Vec<i32>` 的每個元素 `+10`，用 `iter_mut().for_each(|x| ...)`。
4. 思考：為什麼 `move || drop(s)` 是 `FnOnce` 而不是 `Fn`？

## 延伸閱讀

- [The Rust Book — Ch 13 Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [Closures: Magic Functions](https://rustyyato.github.io/rust/syntactic/sugar/2019/01/17/Closures-Magic-Functions.html)
