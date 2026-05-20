# 07. Slices 與字串

> 範圍：&str vs String、&[T] vs Vec<T>、UTF-8 邊界、字串切片陷阱

## 兩種字串：`&str` 與 `String`

| | `&str` | `String` |
|---|---|---|
| 配置 | 唯讀切片 | heap，可變 |
| 大小 | 16 bytes（fat pointer：ptr + len） | 24 bytes（ptr + len + cap） |
| 修改 | ❌ | ✅ |
| 來源 | 字串字面值、`String` 借用、檔案 mmap 等 | `String::from`、`format!`、`to_string` |

```rust
let literal: &str = "hello";          // 'static，在 binary 的 .rodata
let owned: String = String::from("hi");
let borrowed: &str = &owned;          // String 自動 deref 成 &str
```

**規則**：函式吃字串 prefer `&str`（最通用），需要持有 / 修改才用 `String`。

```rust
fn print(s: &str) { println!("{s}"); }
print("literal");          // OK
print(&String::from("x")); // OK，自動 deref
```

## `&[T]` 與 `Vec<T>`

跟 `&str` / `String` 同樣的關係：

| | `&[T]` | `Vec<T>` |
|---|---|---|
| 配置 | 切片借用 | heap，可變 |
| 大小 | ptr + len | ptr + len + cap |
| 操作 | 讀（與 `&mut [T]` 寫單一元素） | push / pop / insert / remove |

```rust
let arr = [10, 20, 30, 40, 50];
let slice: &[i32] = &arr[1..4];     // [20, 30, 40]
let v: Vec<i32> = vec![1, 2, 3];
let all: &[i32] = &v[..];            // 整段
```

函式吃陣列 prefer `&[T]`：

```rust
fn sum(xs: &[i32]) -> i32 { xs.iter().sum() }
sum(&[1, 2, 3]);
sum(&vec![1, 2, 3]);
```

## Slice 是 fat pointer

`&str` / `&[T]` 都是 (ptr, len) 兩個 word，**指向別處的記憶體**，自己不持有資料：

```
&str ──┬─ ptr ──→ [bytes in heap or .rodata]
       └─ len
```

這就是為什麼 slice 是 cheap to pass、無 allocation。

## 字串方法

```rust
let mut s = String::from("hello");
s.push(' ');             // push char
s.push_str("world");     // push &str
s += "!";                // 等同 push_str
```

讀取：

| 方法 | 用途 |
|------|------|
| `len()` | **bytes 數**（不是 chars） |
| `chars()` | iterator over `char` |
| `bytes()` | iterator over `u8` |
| `split(c)` | 分割 |
| `trim()` | 去前後空白 |
| `replace(a, b)` | 替換 |
| `parse::<T>()` | 解析成型別 `T` |
| `to_uppercase()` / `to_lowercase()` | 大小寫 |
| `starts_with` / `ends_with` / `contains` | 判斷 |

## UTF-8 邊界（必懂）

```rust
let zh = String::from("你好");
println!("{}", zh.len());           // 6 (bytes!)
println!("{}", zh.chars().count()); // 2 (chars)
```

`String` 內部是 UTF-8 bytes。所以 `len()` 是 byte 長度，不是字元數。

**切片必須在字元邊界**：

```rust
let prefix = &zh[..3];   // OK，"你" 是 3 bytes
let bad = &zh[..1];      // RUNTIME PANIC: not a char boundary
```

要安全切：

```rust
// 取前 n 個字元
let first_n: String = zh.chars().take(5).collect();
// 找下一個 char boundary
if zh.is_char_boundary(2) { ... }
```

> 這是 Rust 字串設計的核心取捨：byte index O(1)、char index 需要 O(n) iterate。

## `&str` 不可索引

```rust
let s = "hello";
let c = s[0];     // ERROR
```

因為 byte 索引可能落在字元中間。要拿單一字元用 `chars()`：

```rust
let c = s.chars().next().unwrap();
```

要 byte 用：

```rust
let b = s.as_bytes()[0];   // u8
```

## 三種轉成 `String`

```rust
let a: String = "hi".to_string();
let b: String = String::from("hi");
let c: String = "hi".to_owned();
```

語意完全等價，**選一個風格用到底**就好。慣例上 `String::from` 與 `to_string()` 最常見；`to_owned()` 在泛型 context（`T: ToOwned`）更精確。

## 對照 TypeScript

TS 字串就一個 `string`（UTF-16 編碼）。Rust 切成 `&str` 與 `String` 兩種，外加陣列也分 `&[T]` / `Vec<T>` 的對應切割。理由都是 ownership：**借用版**（fat pointer 借過來）與 **owned 版**（持有 heap 配置）。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 字串字面值 | `"hi"` → `string` | `"hi"` → `&'static str`（存在 binary） |
| Owned 字串 | `string`（GC 管） | `String`（heap、可變） |
| 借用字串 | （沒這概念，永遠 reference） | `&str` |
| 字元編碼 | UTF-16 | UTF-8 |
| `.length` | UTF-16 code unit 數 | `.len()` → **bytes** 數 |
| 字元數 | `[..."你好"].length` → 2 | `s.chars().count()` → 2 |
| 索引單字元 | `s[0]` → 1 個 code unit 的字串 | **不能** `s[0]`（byte 索引可能切爆字元） |
| 拿單一字元 | `s.charAt(0)` / `s.codePointAt(0)` | `s.chars().next().unwrap()` |
| 拼接 | `a + b` / `\`${a}${b}\`` | `format!("{a}{b}")` / `s.push_str(t)` |
| 子字串 | `s.slice(0, 3)` | `&s[..3]`（**byte index**，可能 panic） |
| 安全切片 | （UTF-16 切到代理對也會壞） | `if s.is_char_boundary(i)` |
| 轉換 owned | 已是 string | `to_string()` / `String::from("")` |
| 陣列 owned | `T[]` | `Vec<T>` |
| 陣列借用 | （沒這概念） | `&[T]`（切片） |
| 函式參數最通用 | `string` / `T[]` | `&str` / `&[T]` |

### 程式碼對照

```ts
// TS — 沒有 owned/borrowed 區分
const s: string = "hello";
function printLen(t: string): number { return t.length; }
printLen(s);                 // OK
printLen("literal");         // OK
const sub = s.slice(0, 3);   // "hel"
const chars = [..."你好"];   // ["你", "好"]
const len = "你好".length;   // 2（UTF-16 code units，剛好對）
const emoji = "👨‍👩‍👧".length;  // 8（surrogate pairs）
```

```rust
// Rust — &str 通用、String 持有
let s: String = String::from("hello");
fn print_len(t: &str) -> usize { t.len() }
print_len(&s);             // &String 自動 deref 成 &str
print_len("literal");      // OK，"literal" 是 &str
let sub: &str = &s[..3];   // "hel"（byte index，剛好在邊界）
let chars: Vec<char> = "你好".chars().collect();  // ['你', '好']
let len = "你好".len();    // 6（UTF-8 bytes）
let count = "你好".chars().count();   // 2
```

### 心智模型差異

1. **`&str` vs `String` ≈ 「借的」vs「擁有的」**。TS 一個 `string` 一刀切；Rust 分兩種是 ownership 的延伸：函式參數收 `&str`（最通用、不取走）、要存進 struct 才 `String`（取得 ownership）。
2. **編碼換成 UTF-8 後 `len()` 不是字元數**。TS 因為內部是 UTF-16，`length` 巧合對 BMP 字元有效；Rust 直接拒絕 `s.len()` 等於「字元數」的幻想——byte 與 char 完全不同概念。處理人類視角的「幾個字」要 `chars().count()`（O(n)）。
3. **索引語法是陷阱**。TS `s[0]` 永遠回字串（一個 code unit）；Rust `s[0]` **編譯失敗**——因為 byte 0 可能落在字元中間。要拿字元用 `chars().next()`，要 byte 用 `as_bytes()[0]`。
4. **`&Vec<T>` vs `&[T]` 是同個道理**。寫 `fn sum(xs: &Vec<i32>)` 在 Rust 是反 pattern：太具體。寫 `fn sum(xs: &[i32])` 就能接陣列、Vec、`&[i32]`——對應 TS 寫 `function sum(xs: readonly number[])` 但更嚴格（不能 mutate）。
5. **字面值的 lifetime 是 `'static`**。TS 字面值跟 runtime 建的 string 一樣是 `string`；Rust `"hi"` 是 `&'static str`（存在 binary `.rodata`），跟 `String::from("hi").as_str()`（lifetime 跟 String 綁）型別等同但 lifetime 不同。第 13 章詳述。
6. **拼接策略要選**。TS `+` 拼接背後 V8 會自動處理（甚至 cons string）；Rust 要選 `format!`（每次新 alloc）、`push_str`（mutate）、`String::with_capacity` 預配置——loop 內密集拼接的性能差距很顯著。

## 常見陷阱

1. **`&String` vs `&str`** — 函式參數一律 `&str`，除非真的要持有 `String`。
2. **`String::new()` 不 alloc，但 push 第一個 byte 會 alloc** — pre-size 用 `String::with_capacity(n)`。
3. **拼接效率** — `format!("{a}{b}")` 會 alloc 一次新 `String`，loop 內 prefer `s.push_str(&t)`。
4. **`==` 比 String** — 比的是 byte 內容，OK；但比 `&str == String` 需要 deref / coercion 處理（多半透明）。
5. **`String` 是 valid UTF-8 invariant** — 用 unsafe API 才能破壞，不必擔心 random byte sequence。

## 練習

1. 寫 `truncate(s: &str, max_chars: usize) -> &str`，回前 `max_chars` 個字元（注意 char boundary）。
2. 統計 `"Rust 是 systems language"` 的：bytes 數、chars 數、單字數。
3. 寫 `is_palindrome(s: &str) -> bool`（忽略大小寫，只看 ASCII）。
4. 把 `Vec<i32>` 用 `,` 連起來成 `String`，例如 `[1,2,3]` → `"1,2,3"`。
5. 思考：為什麼 `&String` 自動 deref 成 `&str`，但 `&Vec<T>` 也自動 deref 成 `&[T]`？（提示：`Deref` trait）

## 延伸閱讀

- [The Rust Book — Ch 8.2 Strings](https://doc.rust-lang.org/book/ch08-02-strings.html)
- [Why Rust Strings Seem Hard](https://www.brandons.me/blog/why-rust-strings-seem-hard)
