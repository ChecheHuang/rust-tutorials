# 10. 錯誤處理基礎

> 範圍：Result / Option、? 運算子、panic vs 可恢復錯誤、From 轉型

## 兩條軸線

```
              可恢復                    不可恢復
              ─────────                 ─────────
有/無值        Option<T>                 (compile error)
成功/失敗      Result<T, E>              panic!
```

Rust 不像 Java / Python 有 exception。所有錯誤分兩類：

- **可恢復**（檔案不存在、parse 失敗、network timeout）→ `Result<T, E>`，**強制處理**
- **不可恢復**（陣列越界、unwrap None、bug）→ `panic!`，程式中止

沒有「**忘記 catch 結果上線爆炸**」這回事——編譯器強制你處理。

## `Result<T, E>`

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let r: Result<i32, _> = "42".parse();
match r {
    Ok(n) => println!("parsed {n}"),
    Err(e) => println!("failed: {e}"),
}
```

## `?` — 早返運算子

```rust
fn parse_double(s: &str) -> Result<i32, ParseIntError> {
    let n: i32 = s.parse()?;     // 失敗 → 立即 return Err(...)
    Ok(n * 2)
}
```

`expr?` 等同：

```rust
match expr {
    Ok(v) => v,
    Err(e) => return Err(e.into()),
}
```

注意 `.into()` —— Rust 用 `From` trait **自動把** error 型別轉成函式宣告的 error 型別。

## 跨多種 error 型別

```rust
#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self { AppError::Io(e) }
}
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self { AppError::Parse(e) }
}

fn load_count(path: &str) -> Result<u32, AppError> {
    let s = fs::read_to_string(path)?;   // io::Error → AppError
    let n: u32 = s.trim().parse()?;       // ParseInt → AppError
    Ok(n)
}
```

關鍵：實作 `From<SourceError> for AppError`，`?` 就自動轉。這是 Rust 錯誤處理的核心 idiom。

> 第 19 章用 `thiserror` 自動產生這些 boilerplate。本章先手寫一遍理解原理。

## `?` 也能用在 `Option`

```rust
fn first_digit(s: &str) -> Option<u32> {
    let c = s.chars().find(|c| c.is_ascii_digit())?;  // None → return None
    c.to_digit(10)
}
```

不能跨型別（Option 內 `?` 必須在回 Option 的函式裡）。

## Option / Result 常用 method

### Option

```rust
opt.unwrap()                    // panic if None
opt.expect("msg")               // unwrap + 自訂訊息
opt.unwrap_or(default)
opt.unwrap_or_else(|| heavy())
opt.unwrap_or_default()         // T 須 Default
opt.is_some() / opt.is_none()
opt.map(|x| ...)                // Some(x) → Some(f(x))
opt.and_then(|x| ...)           // flatten map
opt.or(Some(other))             // None → Some(other)
opt.ok_or(err)                  // → Result
opt.filter(|x| pred(x))
```

### Result

```rust
res.unwrap()
res.expect("msg")
res.unwrap_or(default)
res.is_ok() / res.is_err()
res.map(|t| ...)                // map Ok side
res.map_err(|e| ...)            // map Err side
res.and_then(|t| another())     // flatten map Ok
res.or_else(|e| fallback(e))
res.ok()                        // → Option<T>，捨棄 err
res.err()                       // → Option<E>，捨棄 ok
```

## `panic!`、`unwrap`、`expect`

```rust
panic!("crash: {value}");
```

何時 panic：
- 違反程式內部不變式（bug）
- prototype / example code 不想處理 error
- 確定不可能 None / Err（用 `expect` 比 `unwrap` 好，留訊息）

何時 **不要**：
- 處理使用者輸入
- IO / network / parsing
- library 內部（讓 caller 決定）

## `main` 也能回 Result

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = fs::read_to_string("config.toml")?;
    Ok(())
}
```

Rust 自動把 main 回傳的 Err 印出 `Debug` 並 exit 1。但訊息醜，正式程式建議：

```rust
fn run() -> Result<(), AppError> { ... }

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

## panic = abort vs unwind

預設 `panic = "unwind"`：panic 會跑 `Drop`，可被 `catch_unwind` 攔截。

`Cargo.toml` 設 `panic = "abort"` 改成立刻終止：
- 優點：binary 較小（無 unwind metadata）、快
- 缺點：不能 catch_unwind，destructor 不跑

embedded、CLI 工具常用 `abort`；library 別碰，由 binary 端決定。

## 對照 TypeScript

TS 沿用 JS 的 `throw` exception 模型——錯誤型別 `unknown`、可在任意層被攔，函式簽章**不顯示**會 throw 什麼。Rust 走另一條路：**錯誤是回傳值**，函式簽章上明寫，編譯器強制處理。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 錯誤表達 | `throw new Error(...)` | `return Err(...)` |
| 錯誤型別 | `unknown`（catch 拿到的） | 由 `Result<T, E>` 的 `E` 決定 |
| 函式簽章顯示 | **不顯示**（任何函式都可 throw） | `-> Result<T, E>` 明寫 |
| 強制處理 | 不強制（忘 try/catch 編譯也過） | **強制**（`#[must_use]`，忽略會警告） |
| 早返 | `try/catch` + rethrow | `?` 運算子（一字代替整個 try） |
| Null 處理 | `T \| undefined` / `T \| null` | `Option<T>`（同一個 `?` 也可用） |
| Promise rejection | 跟 throw 同管道 | `Result` 也能跨 async（第 26 章） |
| 全域 try / catch | `process.on('uncaughtException')` | panic hook：`std::panic::set_hook` |
| 區分「致命」「非致命」 | 自己約定（exception class） | **語言層**：`Result` vs `panic!` |
| 多種錯誤合一 | 自訂 class 繼承樹 | enum + `#[from]`（第 19 章 thiserror） |
| 加 context | `throw new Error("...", { cause: e })` | `.context("...")`（第 19 章 anyhow） |

### 程式碼對照

```ts
// TS — throw + try/catch
function loadCount(path: string): number {
  const s = fs.readFileSync(path, "utf-8");   // 可能 throw
  const n = parseInt(s.trim(), 10);
  if (Number.isNaN(n)) throw new Error("not a number");
  return n;
}

try {
  const n = loadCount("data.txt");
  console.log(n);
} catch (e) {
  console.error("failed:", e);
}
```

```rust
// Rust — Result + ?
fn load_count(path: &str) -> Result<u32, AppError> {
    let s = fs::read_to_string(path)?;          // io::Error 自動轉 AppError
    let n: u32 = s.trim().parse()?;             // ParseIntError 自動轉
    Ok(n)
}

match load_count("data.txt") {
    Ok(n)  => println!("{n}"),
    Err(e) => eprintln!("failed: {e}"),
}
```

`?` 運算子的等價 TS：

```ts
// TS 沒有 ?，要 try/catch + rethrow
function loadCount(path: string): number {
  let s: string;
  try { s = fs.readFileSync(path, "utf-8"); }
  catch (e) { throw new Error("read failed", { cause: e }); }
  // ...
}
```

```rust
// Rust 一個 ? 就解決
let s = fs::read_to_string(path)?;
```

### 心智模型差異

1. **錯誤是值，不是控制流**。TS `throw` 跳出整個 stack 直到 catch；Rust `Result` 是普通回傳值，沒有「跳」這件事——`?` 只是早返糖。後果：Rust 不會發生「上游忘 catch 結果程式炸了」這種事，編譯器強制每層決定要 propagate 還是處理。
2. **函式簽章是「可能會 fail」的契約**。TS 看簽章不知道函式會不會 throw（除非 `noImplicitAny` + 自家約定）；Rust 看到 `-> Result<T, E>` 就知道。寫 client 的人立刻知道要處理。
3. **`?` 替代 `try/catch`**。TS 串幾個可能 fail 的步驟要寫一連串 try 或 promise chain；Rust 每行加個 `?` 就完事，看起來像普通同步流程但語義保留錯誤路徑。
4. **panic 不等於 throw**。TS / Java exception 是日常控制流；Rust `panic!` 是「**程式有 bug**」的訊號，不該當控制流用。`catch_unwind` 存在但是非常少用（FFI 邊界、web server worker 隔離）。
5. **錯誤型別會「擴散」**。TS 寫 `catch (e: unknown)` 然後自己窄化；Rust 錯誤型別綁在 `Result<T, E>` 的 E——多種 error 合一要靠 enum + `From` impl（第 19 章用 `thiserror` 自動化）。一開始覺得繁瑣，但 caller 能 pattern match 處理特定錯誤。
6. **`Result<T, ()>` 是反 pattern**。TS 用 `boolean` 表示成功失敗常見；Rust 用 `Result<T, ()>`（成功 vs 沒理由的失敗）等同沒帶資訊——直接用 `Option<T>` 語意更乾淨。
7. **沒有「全域錯誤監聽」傾向**。Node 寫 `process.on('uncaughtException', ...)` 救命；Rust 鼓勵在邊界（main、handler）顯式收 `Result`，panic hook 是備案不是主線。

## 常見陷阱

1. **濫用 `unwrap`** — 寫得快，上線炸。code review 時看到 unwrap 就要問「為什麼一定不會 None?」。
2. **吞錯誤** — `let _ = result;` 讓編譯器閉嘴但錯誤消失。對 `Result` 必須 `?` 或 `match` 或 `unwrap`。
3. **error 型別膨脹** — 應用層用大 enum 收所有 error 沒問題；library 應該 expose 細緻型別。`thiserror` vs `anyhow` 取捨見第 19 章。
4. **`?` 不能在 closure 內隨便用** — closure 回傳型別要對得上。
5. **`Result<T, ()>` 反 pattern** — 用 `Option<T>` 替代，語意更清楚（成功 vs 沒值）。

## 練習

1. 寫 `divide(a: i64, b: i64) -> Result<i64, &'static str>`，b=0 回錯誤。
2. 把上題的 error 換成自訂 enum，加 `Overflow` variant 處理 `i64::MIN / -1` 的溢位。
3. 寫一個函式串 3 個會失敗的步驟（parse → lookup → compute），用 `?` 串起來。
4. 思考：為什麼 `?` 不能直接用在 `main()` 預設簽章（`fn main()`）？

## 延伸閱讀

- [The Rust Book — Ch 9 Errors](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Error Handling in Rust — Andrew Gallant](https://blog.burntsushi.net/rust-error-handling/)
