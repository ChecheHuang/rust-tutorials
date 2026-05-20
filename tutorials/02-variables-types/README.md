# 02. 變數、型別、不可變性

> 範圍：let / mut、基本型別、shadowing、型別推導、as 轉型、整數溢位

## 核心觀念：預設不可變

```rust
let x = 5;
x = 6;  // 編譯錯誤：cannot assign twice
```

Rust 變數預設不可變。要可變必須明示 `mut`：

```rust
let mut x = 5;
x = 6;  // OK
```

這個設計鼓勵：能 immutable 就 immutable。看到 `let x` 就知道之後不會變——比 C++ `const correctness` 還嚴格，因為是預設。

## shadowing — 同名再宣告

```rust
let z = "5";
let z: i32 = z.parse().unwrap();
let z = z * 2;
```

第二、三個 `z` 是**新變數**，不是 mutation。差別：

|  | `mut` | shadowing |
|---|---|---|
| 型別可變 | ❌ | ✅ |
| 是新 binding | ❌ | ✅ |
| Drop 時機 | 同一個變數 | 後者覆蓋前者 |

shadowing 常用於：型別轉換（`"5"` → `5`）、把暫時的可變值轉成不可變。

## 基本型別

### 整數

| 有號 | 無號 | 大小 |
|------|------|------|
| `i8` `i16` `i32` `i64` `i128` `isize` | `u8` `u16` `u32` `u64` `u128` `usize` | 8 / 16 / 32 / 64 / 128 / 指標寬 |

預設整數字面值是 `i32`。`usize` / `isize` 視平台是 32 或 64 bit，**陣列索引必須是 `usize`**。

### 浮點

`f32`、`f64`。預設字面值是 `f64`。

### 布林與字元

- `bool`：`true` / `false`（**不能**用 `0` / `1` 替代，`if 0` 編譯失敗）
- `char`：**4 bytes**，一個 Unicode scalar value

```rust
let e: char = '我';  // 合法
```

> 注意：`char` ≠ byte。要 byte 用 `u8`；要 UTF-8 序列用 `&str` / `String`（第 07 章）。

## 型別推導與 turbofish

```rust
let v = vec![1, 2, 3];   // Vec<i32>
```

推不出來時：

```rust
let v: Vec<u8> = vec![1, 2, 3];           // 標變數型別
let parsed = "42".parse::<i64>().unwrap(); // 標函式回傳型別（turbofish）
```

`::<...>` 唸 turbofish。`parse`、`collect`、`into` 這些回傳泛型的函式常用到。

## 整數溢位

```rust
let max = u8::MAX;
let _ = max + 1;
```

- **debug build**：直接 panic（讓你開發時抓到）
- **release build**：wrap around（255 + 1 = 0）

正式程式碼**不該**靠這個 default。要顯式選語意：

| 方法 | 行為 |
|------|------|
| `wrapping_add` | wrap |
| `checked_add` | 回 `Option<T>`，溢位給 `None` |
| `saturating_add` | 飽和（停在 MAX） |
| `overflowing_add` | 回 `(T, bool)` |

## `as` — 顯式型別轉換

```rust
let big: i32 = 300;
let small: u8 = big as u8;  // 截斷：300 % 256 = 44
let f: f32 = big as f32;
```

`as` 的特點：
- **一律不會 panic**
- 整數縮窄會**截斷**（不是 panic，不是飽和）
- 浮點轉整數會**飽和**（Rust 1.45 起）

對「**安全**」的轉型，prefer trait：

```rust
let x: u32 = 100;
let y: u8 = u8::try_from(x).unwrap();
```

- `From` / `Into`：絕對不失敗（如 `u8 → u32`）
- `TryFrom` / `TryInto`：可能失敗，回 `Result`

## 常數

```rust
const MAX_USERS: u32 = 1_000;
```

- 必須標型別
- 只能是**編譯期可計算**表達式
- 命名 `SCREAMING_SNAKE_CASE`
- 可宣告在 module top-level

`const` vs `let`：
- const 無 runtime 開銷，編譯期 inline
- const 可放 module 頂層、function body 都行
- const 不能 shadowing

`static` 與 `const` 不同：`static` 有單一記憶體位址、可 `&'static`。日常 prefer `const`，需要 `&'static` reference 時才用 `static`。

## unit type

`()` 是 unit type，只有一個值 `()`。用途：「**沒有有意義的回傳值**」。

```rust
fn log(msg: &str) -> () { println!("{msg}"); }
// 等價於：
fn log(msg: &str) { println!("{msg}"); }   // 預設回 ()
```

Rust 沒有 `void` 概念；不回傳 = 回傳 `()`。

## 對照 TypeScript

TS 的型別系統強在「**結構化型別 + 漸進式擴張**」，Rust 強在「**名稱型別 + 嚴格不放水**」。表面寫法乍看相像，背後對「型別」與「runtime」的關係差很多。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 不可變宣告 | `const x = 5` | `let x = 5;` |
| 可變宣告 | `let x = 5` | `let mut x = 5;` |
| 預設 | **可變**（`let`） | **不可變**（`let`） |
| shadowing | TS 同 scope 不能重宣告 | OK（產生**新** binding，可改型別） |
| 型別存在期 | 編譯後**消失**（type erasure） | 編譯期完整保留，影響 codegen |
| number | 統一 `number`（64-bit float） | `i8..i128`、`u8..u128`、`f32`、`f64` 分明 |
| boolean | `if (0)` 走 falsy | `if 0` **編譯失敗**（必須 `bool`） |
| 整數運算混型別 | 自動 coerce | 編譯失敗，要 `.into()` / `as` |
| 顯式轉型 | `as`（**型別斷言，不轉值**） | `as`（**真的轉值**，整數截斷） |
| 安全轉型 | `Number.isSafeInteger(x)` 手寫 | `u8::try_from(x)` 回 `Result` |
| 常數 | `const X = 5`（值或 binding 都用） | `const X: u32 = 5;`（**只**指編譯期常數） |
| 找不到值 | `undefined` / `null`（兩個） | `Option::None`（只一個，第 8 章） |
| 函式無回傳 | `void`（型別） | `()`（unit type，**有一個值**） |

### 程式碼對照

```ts
// TS
const x: number = 5;
const y = x + 0.5;          // OK，number 是 float
const s: string = "5";
const n = parseInt(s, 10);   // n: number

let arr: number[] = [];
arr[0] = 1;                  // 沒先 push 也 OK，sparse array
const c = "hello"[0];        // c: string（單字元也是 string）

if (n) { /* falsy 檢查 */ }
```

```rust
// Rust
let x: i32 = 5;
// let y = x + 0.5;          // 編譯失敗：i32 + f64 不允許
let y = x as f64 + 0.5;
let s: &str = "5";
let n: i32 = s.parse().unwrap();   // 必須標型別或 turbofish

let mut arr: Vec<i32> = Vec::new();
arr.push(1);                        // 沒 push 不能用 [0]
let c: char = "hello".chars().next().unwrap();   // 字串不能 [0] 索引

if n != 0 { /* 必須是 bool */ }
```

### 心智模型差異

1. **TS 的型別在 runtime 不存在**。`x as u8` 在 TS 只是「**騙編譯器**這是 u8」，runtime 還是 64-bit float；Rust `x as u8` 是真的把 i32 截斷成 8 bit。把 TS 的 `as` 直覺帶過來會很危險，Rust 用 `TryFrom` 才安全。
2. **TS 的 `number` 把 int / float 混為一談**。Rust 強迫你選位寬與有號性——一開始煩，但 overflow / 截斷 bug 在編譯期就被擋下。索引必須是 `usize` 也是同個哲學。
3. **TS 沒有「整數溢位」概念**（IEEE 754 float 隨便加）。Rust 在 debug build 直接 panic、release 預設 wrap，要明確語意請用 `checked_add` / `saturating_add` / `wrapping_add`——TS 沒有這套工具。
4. **TS 用 `null | undefined` 表達「沒值」**。Rust 沒這兩個，全部走 `Option<T>`，並且要 pattern match 拆開（第 8 章）。少了「忘記 null check」這類 bug。
5. **`const` 在兩邊不同意義**。TS `const` 是「binding 不能重新指向」（物件內容仍可改）；Rust `const` 是「**編譯期常數**」，必須 inline 進每個使用點。Rust 對應 TS `const` 的概念其實是預設的 `let`。
6. **`as`**：在 TS 是 type assertion（編譯期），在 Rust 是 value conversion（runtime）。同字不同義，常見誤用來源。

## 常見陷阱

1. **想用 `0` 當 false** — Rust `if x` 必須是 `bool`，不能寫 `if 0`、`if some_int`。
2. **整數運算混型別** — `i32 + i64` 編譯失敗，須 `.into()` 或 `as`。
3. **`as` 切記不會檢查** — 縮窄轉型 silently 截斷；對外輸入用 `TryFrom`。
4. **`let _ = expr` 與 `let _x = expr`** — 前者**立即** drop，後者持有到 scope 結束（影響鎖、guard）。

## 練習

1. 把 `parse::<i64>().unwrap()` 換成不 panic 的寫法（提示：用 `match` 處理 `Result`）。
2. 寫 `safe_add(a: u32, b: u32) -> Option<u32>`，溢位回 `None`。
3. 印 `'中' as u32` 與 `'A' as u32`，確認 codepoint。
4. 用 `TryFrom` 把 `i64` 轉 `u8`，輸入超出範圍時要印錯誤而不 panic。

## 延伸閱讀

- [Rust Reference — Types](https://doc.rust-lang.org/reference/types.html)
- [The Rust Book — Ch 3](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html)
