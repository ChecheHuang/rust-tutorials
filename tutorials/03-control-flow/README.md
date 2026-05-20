# 03. 控制流與 Pattern Matching

> 範圍：if / else、loop / while / for、match、if let、while let、`@` binding、let else

## 核心觀念：一切是 expression

C / Java 把控制流當 statement（不回值），Rust 把它們當 **expression**——`if`、`match`、`loop` 都能賦值給變數。

```rust
let parity = if n % 2 == 0 { "even" } else { "odd" };
```

這代替了三元運算子（Rust **沒有** `?:`），也讓 functional 風格更自然。

## `loop` — 可 break 帶值

```rust
let answer = loop {
    i += 1;
    if i * i > 100 { break i; }
};
```

`loop` 表示「無條件無限」，比 `while true` 對編譯器更友善（unreachable analysis 更準）。

## `while` / `for`

- `while cond { ... }` — 一般迴圈
- `for x in iter { ... }` — 迭代任何 `IntoIterator`

```rust
for x in 1..=3 { ... }              // 1, 2, 3
for (i, c) in "abc".chars().enumerate() { ... }
```

`1..3` 不含 3、`1..=3` 含 3。Rust **沒有** C 風 `for(i=0;i<n;i++)`，因為靠 iterator + range 就夠。

## `match` — 必須**窮盡**

```rust
let (dx, dy) = match dir {
    "north" => (0, 1),
    "south" => (0, -1),
    "east"  => (1, 0),
    "west"  => (-1, 0),
    _ => (0, 0),
};
```

- 每個 arm 是 `pattern => expr,`
- 編譯器**強制**窮盡所有可能（漏 case 編譯失敗）
- `_` 是 wildcard catch-all
- 比起 `switch`，沒有 fallthrough

### Guard（守衛條件）

```rust
match opt {
    Some(n) if n > 5 => ...,
    Some(n)          => ...,
    None             => ...,
}
```

`if` guard 在 pattern 匹配後額外檢查。注意 guard 失敗時編譯器**不會**回頭重試，要小心 ordering。

### 常見 pattern

```rust
1                        // 字面值
1..=10                   // range（含）
x                        // 綁名
_                        // wildcard
(x, y)                   // tuple
Point { x, y }           // struct destructure
Point { x, .. }          // 部分 destructure
Some(x)                  // enum variant
Some(_) | None           // or pattern
arr @ [first, ..]        // @ binding + slice pattern
```

### `@` binding — 邊配對邊綁名

```rust
match n {
    x @ 1..=5 => println!("small ({x})"),
    x @ 6..=10 => println!("medium ({x})"),
    x => println!("other ({x})"),
}
```

想知道「**屬於哪個範圍**」同時「**拿到具體值**」就用 `@`。

## `if let` / `while let` — match 的單臂簡寫

只在意一個 pattern 時：

```rust
if let Some(x) = some_val {
    println!("got {x}");
}
```

迴圈版：

```rust
while let Some(top) = stack.pop() {
    process(top);
}
```

`while let` 是處理 stack/queue 的標準模式。

## `let else` (Rust 1.65+) — 早返

```rust
let Some(parsed) = "42".parse::<i32>().ok() else {
    return;  // 或 panic!, break, continue
};
// 這行之後 parsed 是 i32
```

避免 `if let` 巢狀過深。else block **必須** diverge（return / break / panic）。

## labeled break / continue

```rust
'outer: loop {
    for x in 1..10 {
        for y in 1..10 {
            if x * y == 42 {
                break 'outer (x, y);
            }
        }
    }
}
```

從多層巢狀直接跳出，比設 flag 乾淨。label 用單引號開頭。

## 為什麼沒有 `?:` 三元

`if-else` 本身就是 expression，三元只是 noise。寫起來：

```rust
// 取代 cond ? a : b
let x = if cond { a } else { b };
```

差異不大但更可讀。

## 對照 TypeScript

最大共通點：Rust 的 `match` 跟 TS 在 union / discriminated union 上的 narrowing 都用 pattern matching 思路，但 Rust **編譯期強制窮盡**而 TS narrowing 是型別系統推導的副產品。最大差異：Rust 控制流結構**都是 expression**，TS 的 `if` / `switch` 是 statement。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| `if` 回值 | statement，無回值；要值用 `?:` | expression：`let x = if c { a } else { b };` |
| 三元 | `cond ? a : b` | 沒有，用 `if-else` expression |
| `switch` | fallthrough（除非 break） | `match` **無 fallthrough**，每 arm 獨立 |
| 窮盡檢查 | discriminated union + `never` 手動湊 | `match` **預設強制**窮盡 |
| 預設 case | `default:` | `_` |
| range pattern | 無（要手寫 `if`） | `1..=10` 直接 match |
| `if let` | TS 4.9 `narrowing` + 解構 | 內建：`if let Some(x) = opt { ... }` |
| labeled break | `outer: for ... break outer;` | `'outer: loop ... break 'outer;` |
| for-each | `for (const x of arr)` / `arr.forEach` | `for x in v` |
| C 風 for | `for (let i=0; i<n; i++)` | 沒有，用 `for i in 0..n` |
| 早返 + 解構 | 自己寫 if + return | `let Some(x) = opt else { return; };` |

### 程式碼對照

```ts
// TS — discriminated union 配 switch
type Shape =
  | { kind: "circle"; r: number }
  | { kind: "rect"; w: number; h: number };

function area(s: Shape): number {
  switch (s.kind) {
    case "circle": return Math.PI * s.r ** 2;
    case "rect":   return s.w * s.h;
    default:
      const _exhaustive: never = s;   // 漏 case 才能編譯失敗
      throw new Error();
  }
}
```

```rust
// Rust — enum 配 match
enum Shape {
    Circle { r: f64 },
    Rect   { w: f64, h: f64 },
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => std::f64::consts::PI * r * r,
        Shape::Rect { w, h } => w * h,
        // 漏 case 直接編譯失敗，不需 never trick
    }
}
```

```ts
// TS 三元
const parity = n % 2 === 0 ? "even" : "odd";
```

```rust
// Rust if 是 expression
let parity = if n % 2 == 0 { "even" } else { "odd" };
```

### 心智模型差異

1. **TS 的 `if (truthy)`**（`0`、`""`、`null`、`undefined`、`NaN` 都 falsy）→ Rust **嚴格**：`if` 條件必須是 `bool`，沒有 falsy 概念。`if some_number` 編譯失敗，要寫 `if x != 0`。
2. **窮盡檢查是「免費」的**。TS 要靠 `const _: never = x` 這種 boilerplate 才能在漏 case 時被擋；Rust `match` 預設就強制——加 enum variant 後**所有相關 match 都會編譯失敗**直到你補上，重構安全感極強。
3. **沒有 `switch` fallthrough**。TS / C 那個「忘寫 break 串到下一個 case」的 bug 在 Rust 不存在。要多 case 走同邏輯用 or-pattern：`Some(1) | Some(2) => ...`。
4. **控制流 = 值**。TS 要拿 if 的結果只能三元；Rust `if`、`match`、`loop`、block 全都會回值，整段程式能維持 expression-oriented 風格，少很多 mutable variable。
5. **`let else` 是 TS 沒有的 idiom**。TS 寫 `if (!x) return; const y = x.foo;`（搭配 narrowing）；Rust `let Some(y) = x.foo else { return; };` 一行解構 + early return，比 TS narrowing 更明顯。
6. **range pattern**。TS `switch` 一個 case 對一個值；Rust `match` 一個 arm 可以對應一段範圍（`1..=10 => ...`），數值分類更乾淨。

## 常見陷阱

1. **忘 `_` catch-all** — 配對 `&str`、`i32` 這類「無限可能」型別時，沒寫 `_` 編譯失敗。
2. **match arm 漏逗號 vs 必要逗號** — block arm（`=> { ... }`）可省逗號，expression arm 不可省。Rustfmt 會幫你補。
3. **`if let` 不會強制窮盡** — 用 `if let` 時沒處理 `None` 編譯**不會**警告，習慣要 `else`。
4. **range pattern 與 range expression 不同** — `match` 內的 `1..=3` 是 pattern；`for x in 1..=3` 是 expression。語法相似但底層機制不同。
5. **guard 不算 exhaustiveness 證明** — `Some(n) if n > 0` 不會被視為涵蓋所有 `Some`，編譯器仍要你補 `Some(n) =>`。

## 練習

1. 寫 `fizzbuzz(1..=20)`，用 match 把 `(i % 3 == 0, i % 5 == 0)` tuple 配對到輸出。
2. 解析 `"+3"`、`"-7"`、`"5"`：用 match 把 sign 與絕對值拆出（用 `chars().next()`）。
3. 把以下 `if let` 巢狀重寫為 `let else`：
   ```rust
   if let Some(a) = x {
       if let Some(b) = a.field {
           process(b);
       }
   }
   ```

## 延伸閱讀

- [The Rust Book — Ch 6 Match](https://doc.rust-lang.org/book/ch06-02-match.html)
- [Rust Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html)
