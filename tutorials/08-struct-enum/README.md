# 08. Struct 與 Enum

> 範圍：struct / tuple struct / unit struct、enum、Option<T>、derive

## Struct — 三種形態

### Named field struct

```rust
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 3, y: 4 };
println!("{}, {}", p.x, p.y);
```

最常見的形態，類似其他語言的 record / class fields。

### Tuple struct

```rust
struct Rgb(u8, u8, u8);
let c = Rgb(255, 0, 0);
let (r, g, b) = (c.0, c.1, c.2);
```

無 field 名，用 `.0` `.1` 存取。適合「**型別化的 wrapper**」（newtype pattern）：

```rust
struct Meters(f64);
struct Kilograms(f64);
// 編譯期區分，避免單位混淆
```

### Unit struct

```rust
struct Marker;
```

零 size、無 field。常用於：
- 作為 trait impl 的「標記型別」
- generic phantom type
- 與外部系統互動的 sentinel

## Method 與 Associated Function

```rust
impl Point {
    fn origin() -> Self { Self { x: 0, y: 0 } }   // associated（無 self）
    fn new(x: i32, y: i32) -> Self { Self { x, y } }

    fn translate(&mut self, dx: i32, dy: i32) {    // method（&mut self）
        self.x += dx;
        self.y += dy;
    }
}

let p = Point::origin();        // 用 :: 呼叫 associated
let mut p2 = Point::new(1, 2);
p2.translate(3, 4);             // 用 . 呼叫 method
```

`Self`（大寫）= 「**當前 impl block 的型別**」，等同寫 `Point`，但 refactor 時不必改。

### `self` 參數的三種形式

| 簽章 | 意義 | 何時用 |
|------|------|--------|
| `fn m(self, ...)` | 取得 ownership | 想消費 instance（builder 收尾、轉型） |
| `fn m(&self, ...)` | 共享借用 | 唯讀方法（getter、計算） |
| `fn m(&mut self, ...)` | 獨佔借用 | 修改方法 |

## Struct Update Syntax

```rust
let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 99, ..p1 };   // 其他 field 從 p1
```

`..p1` 意思是「**剩下的 field 從 p1 取**」。注意：如果取到的 field 不是 `Copy`，會 **move**。

## Enum — 真正的 sum type

```rust
enum Shape {
    Circle { radius: f64 },          // struct-like variant
    Rect(f64, f64),                  // tuple-like variant
    Triangle(f64, f64, f64),
    Empty,                            // unit variant
}
```

跟 C enum 不同——Rust enum **每個 variant 可以帶不同形狀的資料**。這就是 sum type / tagged union / discriminated union。

```rust
match shape {
    Shape::Circle { radius } => ...,
    Shape::Rect(w, h) => ...,
    Shape::Empty => ...,
    _ => ...,
}
```

`match` 強制窮盡——加 variant 後所有 match 點都會被編譯器揪出來補。**重構安全感的核心**。

## `Option<T>` — 沒有 null 的 nullable

```rust
enum Option<T> {
    Some(T),
    None,
}
```

Rust **沒有 null**。可能不存在的值用 `Option<T>` 表達：

```rust
fn find(items: &[i32], target: i32) -> Option<usize> {
    items.iter().position(|&x| x == target)
}

match find(&[1, 2, 3], 2) {
    Some(i) => println!("at {i}"),
    None => println!("not found"),
}
```

### Option 常用方法

```rust
opt.unwrap()                  // panic if None（用於確定不會 None）
opt.unwrap_or(default)        // None → default
opt.unwrap_or_else(|| ...)    // None → 動態計算
opt.expect("msg")             // unwrap + 自訂 panic 訊息
opt.map(|x| x * 2)            // Some(x) → Some(x*2)，None → None
opt.and_then(|x| ...)         // Some(x) → 函式回傳，None → None（flatten）
opt.or(Some(other))           // None → Some(other)
opt.is_some() / opt.is_none()
opt.ok_or(err)                // Some(x) → Ok(x)，None → Err(err)
```

## `#[derive(...)]`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Point { x: i32, y: i32 }
```

讓編譯器自動生成常見 trait 實作：

| Trait | 用途 |
|-------|------|
| `Debug` | `{:?}` formatting |
| `Clone` | `.clone()` deep copy |
| `Copy` | bitwise 複製語意（須先 Clone） |
| `PartialEq` / `Eq` | `==` `!=` |
| `PartialOrd` / `Ord` | `<` `>` `<=` `>=` |
| `Hash` | 可放 `HashMap` / `HashSet` key |
| `Default` | `Self::default()` |

derive 是 procedural macro，第 17 章詳述如何寫自己的。

## Destructure

```rust
let p = Point { x: 10, y: 20 };
let Point { x, y } = p;          // OK
let Point { x: a, y: b } = p;    // 重新命名
let Point { x, .. } = p;          // 略過 y
let (a, b) = (1, 2);              // tuple destructure
```

也可以在 function 簽章直接 destructure：

```rust
fn dist(Point { x, y }: &Point) -> f64 {
    ((x * x + y * y) as f64).sqrt()
}
```

## 對照 TypeScript

這是 TS 跟 Rust **觀念最接近**的章節。TS 強在 structural typing + discriminated union，Rust 強在名義型別 + 真正的 sum type。`Option<T>` 對應 TS 的 `T | undefined`，但 Rust 強迫你 pattern match 才能拆。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| Plain object | `interface User { id: number; name: string }` | `struct User { id: u32, name: String }` |
| 型別系統 | **structural**（同形狀互通） | **nominal**（同 field 不同名也不通） |
| Tuple | `[number, number]` | `(i32, i32)` 或 tuple struct |
| Discriminated union | `{ kind: "x" } \| { kind: "y" }` | `enum Shape { X, Y { ... } }` |
| Tagged union 標記 | 自己選欄位（`kind` / `type`） | enum variant 本身 |
| 列舉常數 | `enum Color { Red, Blue }` | `enum Color { Red, Blue }` |
| 列舉帶資料 | TS enum **不行**，要用 union | `enum Msg { Quit, Move { x: i32, y: i32 } }` |
| Nullable | `T \| undefined` / `T \| null` | `Option<T>`（**只一個**） |
| Optional field | `name?: string` | `name: Option<String>` |
| 拆解 | `const { x, y } = p;` | `let Point { x, y } = p;` |
| 部分拆解 | `const { x, ...rest } = p;` | `let Point { x, .. } = p;`（剩下 ignore） |
| 自動產生方法 | `Object.assign({}, defaults, x)` / decorators | `#[derive(Clone, Debug, PartialEq, ...)]` |
| 印 console | `console.log(obj)` 物件展開 | `println!("{:?}", obj)` 需 `Debug` derive |
| 兩個型別不混淆 | TS structural：同形狀互通 | newtype：`struct UserId(u32)` 絕不會跟 `OrderId(u32)` 互通 |

### 程式碼對照

```ts
// TS — discriminated union + null
interface User { id: number; name: string }
type Shape =
  | { kind: "circle"; r: number }
  | { kind: "rect"; w: number; h: number };

function find(users: User[], id: number): User | undefined {
  return users.find(u => u.id === id);
}

const u = find(users, 1);
if (u !== undefined) {
  console.log(u.name);    // narrowing 後可用
}
```

```rust
// Rust — enum + Option
struct User { id: u32, name: String }

enum Shape {
    Circle { r: f64 },
    Rect   { w: f64, h: f64 },
}

fn find(users: &[User], id: u32) -> Option<&User> {
    users.iter().find(|u| u.id == id)
}

match find(&users, 1) {
    Some(u) => println!("{}", u.name),
    None    => println!("not found"),
}
```

Newtype 對照：

```ts
// TS — branded type（需要技巧才能避免互通）
type UserId  = number & { __brand: "UserId" };
type OrderId = number & { __brand: "OrderId" };
const u: UserId = 1 as UserId;
const o: OrderId = u;          // 編譯失敗（brand 不同）
```

```rust
// Rust — newtype 直接拿來用
struct UserId(u32);
struct OrderId(u32);
let u = UserId(1);
// let o: OrderId = u;         // 編譯失敗，型別不同
```

### 心智模型差異

1. **`Option<T>` 不是 nullable**。TS `User | undefined` narrowing 後就能用 `.name`，但 `undefined` / `null` 仍是兩個不同 falsy 值的歷史包袱。Rust 只有一個 `None`，且**必須**用 `match` / `if let` / `?` / `unwrap` 拆——少了「忘記 null check」的整類 bug。
2. **enum 真的是 sum type**。TS enum 是常數列舉（Java 風），帶資料要用 union；Rust enum 一開始就是 ADT（algebraic data type），variant 可帶不同形狀的資料。`Result<T, E>`、`Option<T>` 都是普通 enum，不是內建 hack。
3. **nominal vs structural**。TS `interface A { x: number }` 跟 `interface B { x: number }` **互通**（structural）；Rust `struct A { x: i32 }` 跟 `struct B { x: i32 }` **絕不互通**。一開始覺得啰嗦，但對大型 codebase 是好事——`UserId` 跟 `OrderId` 不會誤接。
4. **`#[derive(...)]` 比 TS decorators 強**。TS 印物件靠 `console.log` runtime introspect；Rust 必須 `#[derive(Debug)]` 才能 `{:?}` 印——但 derive 本身是 procedural macro（第 17 章），可以做任意編譯期生成。`serde::Serialize`、`Clone`、`PartialEq` 都用同套機制。
5. **沒有可選 field**。TS `name?: string` 一筆寫完；Rust 要寫 `name: Option<String>`，存取要 `if let Some(n) = &u.name`。多一步，但「**到底有沒有值**」永遠明確。
6. **method 用 `impl` 區塊定義**。TS class 把資料 + 方法綁在一起；Rust 把資料定義（struct）與方法定義（impl block）**分開**——同一個 struct 可以有多個 impl block、跨檔案組織、甚至為別人的型別 impl trait（孤兒規則限制，第 12 章）。
7. **builder 取代命名參數**。TS 寫 `new Foo({ name, age })` 一行解決；Rust 沒有命名參數，常見做法是 `Foo::builder().name(...).age(...).build()`（`derive_builder` 或 `bon` crate 幫你生）。

## 常見陷阱

1. **enum size = 最大 variant + tag** — 大 variant 拖累整體 size。考慮 `Box<LargeVariant>` 減 size。
2. **`#[derive(Copy)]` 需要所有 field 都 `Copy`** — 含 `String` 的 struct 不能 derive Copy。
3. **`Self` vs 型別名** — `Self` refactor-safe；型別名顯式但須同步改。
4. **`impl Point` 跨多個 impl block** — Rust 允許多個 `impl Point { ... }`，可分檔組織。
5. **struct field 預設 private** — 跨 module 要 `pub field: ...`。

## 練習

1. 設計 `enum Json { Null, Bool(bool), Num(f64), Str(String), Array(Vec<Json>), Object(HashMap<String, Json>) }`，寫 `fn pretty(j: &Json) -> String`。
2. 寫 `Rectangle::new(w, h)`、`area`、`is_square`，使用 `&self`。
3. 用 newtype 包 `u32` 成 `UserId`，使其無法直接與 `OrderId(u32)` 互換。
4. 把以下 Option 鏈用 `?` 簡化（提示：在回傳 Option 的函式內）：
   ```rust
   user.address.and_then(|a| a.city).and_then(|c| c.zip)
   ```

## 延伸閱讀

- [The Rust Book — Ch 5 Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [The Rust Book — Ch 6 Enums](https://doc.rust-lang.org/book/ch06-00-enums.html)
