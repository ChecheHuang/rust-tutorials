# 16. 宣告式宏 `macro_rules!`

> 範圍：fragment specifier、repetition、衛生規則、實作小型 DSL

## 為什麼要 macro

函式做不到的事：
- **拿不固定數量的參數**（`println!`、`vec!`）
- **接受 expression / type / pattern** 等不能當值傳的東西
- **在編譯期生成程式碼**（重複的 impl block、struct 定義）

macro 操作在 **syntax tree** 層級，函式操作在 value 層級。

Rust 兩種 macro：
- **declarative macro**（本章）：`macro_rules!`，pattern matching 寫法
- **procedural macro**（第 17 章）：寫 Rust code 接收 / 產出 token

## 最簡單的 macro

```rust
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}

say_hello!();   // 展開成 println!("Hello!");
```

## Pattern 與展開

`macro_rules!` 跟 `match` 很像：

```rust
macro_rules! name {
    (pattern1) => { expansion1 };
    (pattern2) => { expansion2 };
    // ...
}
```

每個 arm 由 **matcher**（左側）與 **transcriber**（右側）組成。matcher 內可以用 fragment specifier 捕獲 token。

## Fragment specifier

| Specifier | 接受 |
|-----------|------|
| `$x:expr` | 任何 expression |
| `$x:stmt` | statement |
| `$x:pat` | pattern |
| `$x:ty` | type |
| `$x:ident` | identifier |
| `$x:path` | path（如 `std::vec::Vec`） |
| `$x:literal` | literal（字串/數字） |
| `$x:block` | `{ ... }` block |
| `$x:item` | item（fn、struct、impl 等） |
| `$x:meta` | attribute 內容 |
| `$x:tt` | token tree（最寬鬆） |

```rust
macro_rules! square {
    ($x:expr) => { $x * $x };
}

square!(5);        // → 5 * 5
square!(2 + 3);    // → (2 + 3) * (2 + 3) = 25 ✓（自動加括號）
```

> 注意：直接寫 `$x * $x` 看起來會被 `2 + 3 * 2 + 3` 騙；其實 macro 展開時 `$x` 已被視為 expression atom（加上隱形括號），所以不會。

## Repetition — `$(...)`

```rust
macro_rules! my_min {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        std::cmp::min($x, my_min!($($rest),+))
    };
}
```

語法：

```
$( ... ) sep rep
```

- `sep`：分隔符（`,` 或省略）
- `rep`：`*`（0+）、`+`（1+）、`?`（0 或 1）

`$(,)?` 在尾巴常用，允許 trailing comma：

```rust
$($field:ident : $ty:ty),* $(,)?
```

## 衛生（hygiene）

macro 內宣告的變數**不會**洩漏到呼叫端，反之亦然：

```rust
macro_rules! bad {
    () => { let x = 5; };
}
bad!();
println!("{x}");   // ERROR: x 不存在
```

這跟 C macro 完全不同——C `#define` 是純文字替換，極容易踩坑。

但 hygiene **僅限於 identifier**。`$x` 引入的識別字仍是 caller scope 的。

## 例：用 macro 自動產生 impl

```rust
macro_rules! impl_display_via_debug {
    ($name:ty) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

#[derive(Debug)]
struct Foo(i32);
impl_display_via_debug!(Foo);
```

對多個型別重複的 boilerplate，這比手動寫快得多。

## DSL 範例：`map!`

```rust
macro_rules! map {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($key, $val);)*
        m
    }};
}

let m = map! {
    "a" => 1,
    "b" => 2,
};
```

[`maplit`](https://crates.io/crates/maplit) crate 就是這樣設計。

## `vec!` 是怎麼回事

```rust
let v = vec![1, 2, 3];     // 攤平
let z = vec![0; 5];         // 重複
```

`vec!` 由標準函式庫的 `macro_rules!` 定義，大致：

```rust
macro_rules! vec {
    () => { Vec::new() };
    ($elem:expr; $n:expr) => { ... };
    ($($x:expr),+ $(,)?) => { ... };
}
```

## 何時用 macro，何時用函式

優先用函式。改用 macro 的訊號：
- 參數數量浮動（`println!` 那種）
- 需要操作 type 名稱（`impl_display_via_debug!`）
- 需要生成 impl block / struct 定義
- 想做小 DSL（`map!`、`json!`）

不要：
- 為了「DRY」就 macro 化純運算邏輯——函式更好讀、更好 debug。
- 寫超過 100 行的 macro_rules——進入 procedural macro 領域。

## debug 工具

- `cargo expand`（裝 cargo-expand）：看 macro 展開後的 code
- `stringify!($x)`：把 token 轉成 `&'static str`，debug 訊息用
- 編譯器錯誤訊息通常會指到展開後的位置

## 對照 TypeScript

**TS 沒有直接對應**。最接近 declarative macro 的是 TS template literal types（編譯期字串操作）或 utility types（`Pick`、`Omit`），但範圍小很多。`macro_rules!` 操作的是**語法樹層級**——可以生成 impl block、struct、function、任意 expression。

### 對照表

| 需求 | TypeScript | Rust |
|---|---|---|
| 變長參數 | `...args: T[]` | macro：`$($x:expr),*` |
| 編譯期生成程式碼 | TS 編譯外掛 / decorators | `macro_rules!` / proc macro |
| 接 expression 當參數 | 沒有 | `$x:expr` |
| 接 type 當參數 | generic `<T>` | `$x:ty` |
| 重複 token | 沒有 | `$(...)*` / `$(...)+` |
| 自訂 DSL | template literal types（弱） | `macro_rules!`（強） |
| 衛生（不污染外部變數） | 函式 scope 自然有 | macro hygiene 規則 |
| 編譯期失敗 | TS type-level 條件 | `compile_error!("...")` |

### 程式碼對照

`vec!` macro vs TS `Array.of`：

```ts
// TS — 變長參數已是語言內建
const v = [1, 2, 3];
const z = Array(5).fill(0);
const m = new Map([["a", 1], ["b", 2]]);
```

```rust
// Rust — macro 提供類似語法糖
let v = vec![1, 2, 3];
let z = vec![0; 5];
let m: HashMap<&str, i32> = HashMap::from([("a", 1), ("b", 2)]);
```

「為什麼 Rust 需要 macro 做這些」：因為**函式不能拿不同型別 / 不固定數量的參數**，且不能在編譯期生成程式碼。TS 一個變長參數加 generic 就解決，Rust 用 macro 補上同等表達力。

自訂 DSL 例：

```ts
// TS — 想做 map literal 沒辦法做得很漂亮
const m = new Map<string, number>([["a", 1], ["b", 2]]);

// 或自己包 helper
function map<K, V>(entries: [K, V][]): Map<K, V> { return new Map(entries); }
const m2 = map([["a", 1]]);
```

```rust
// Rust — macro 提供 inline literal 語法
macro_rules! map {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($key, $val);)*
        m
    }};
}

let m = map! {
    "a" => 1,
    "b" => 2,
};
```

### 心智模型差異

1. **macro 操作語法層，函式操作值層**。TS 函式拿值、回值；Rust macro 拿 **token tree** 吐 token tree，編譯前的事。可以做「**接 expression 然後生成 if-let 拆解**」這類函式做不到的事。
2. **Rust 為什麼比 TS 需要 macro**：(1) 沒有變長參數，要 `println!`、`vec!`、`format!` 這種彈性語法只能 macro；(2) 想生成 trait impl 區塊（如 `#[derive]`）；(3) 編譯期驗證 input（如 `sqlx::query!` 連 DB 驗 SQL）。TS 大部分這些靠 generic + decorator + tsc 處理。
3. **`macro_rules!` 是「pattern matching 寫的 macro」**。看 fragment specifier (`expr` / `ty` / `ident` / `tt`) 像在限定參數型別。`match` 風格的多 arm 寫法降低門檻。
4. **hygiene 是 TS 自然有的**。TS 函式 scope 不會泄漏。Rust macro hygiene 規則明確：macro 內 `let x` 不會撞到呼叫端的 `x`。比 C `#define` 純文字替換安全得多。
5. **`stringify!` 沒對應**。Rust `stringify!(x + 1)` 在編譯期把 token 轉成 `"x + 1"` 字串。TS 沒有編譯期 token-to-string 機制（runtime 只能 `Function.prototype.toString`）。
6. **`cargo expand` 是 macro 學習神器**。TS 沒對應（decorator 展開要看 source map）。`cargo install cargo-expand` 後可看 `vec![1, 2, 3]` 展開成什麼 — 對學 macro 極有幫助。
7. **不要為 DRY 就上 macro**。TS 同樣建議「先 function，不夠才上 generic / decorator」；Rust 「先 function，再 generic，最後才 macro」——macro debug 痛、IDE 支援差。

## 常見陷阱

1. **fragment specifier 沒搞清楚** — `$x:expr` 與 `$x:tt` 在 follow 規則不同；亂選會 cryptic compile error。
2. **遞迴 macro 沒 base case** — 變成無限展開，編譯器愛護自己會停。
3. **expansion 中變數捕獲混亂** — 衛生規則保護你，但跨 macro 的 `$crate` 路徑要小心。
4. **macro 在 module 外不可見** — `pub use` 或 `#[macro_export]` 才能 export。
5. **`tt` 萬用但難 debug** — 用 `tt` 接 fallback，但越具體的 specifier 越好。

## 練習

1. 寫 `assert_eq_approx!(a, b, eps)`：浮點近似相等斷言。
2. 寫 `time!(expr)`：印 expr 執行時間、回傳 expr 結果。
3. 寫 `for_each!($i, $start..$end, $body)`：簡化 for loop（其實沒必要，純練習）。
4. 看 [`thiserror` source](https://github.com/dtolnay/thiserror)，知道現代 macro 多半已升級成 procedural（下章）。

## 延伸閱讀

- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [Rust Reference — Macros](https://doc.rust-lang.org/reference/macros-by-example.html)
