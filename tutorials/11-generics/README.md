# 11. 泛型

> 範圍：函式 / struct / enum 泛型、where clause、monomorphization、const generics

## 為什麼要泛型

寫過幾次 `largest_i32`、`largest_f64`、`largest_str` 就會煩——程式碼幾乎一樣。泛型 = 寫**一次**，編譯器在使用點**產生多份**特化版本。

```rust
fn largest<T: PartialOrd + Copy>(slice: &[T]) -> T {
    let mut max = slice[0];
    for &x in slice.iter().skip(1) {
        if x > max { max = x; }
    }
    max
}
```

`<T>` 宣告型別參數，`: PartialOrd + Copy` 是 trait bound（要求 T 必須實作這些 trait）。

## 三個泛型 site

### 泛型函式

```rust
fn print<T: Debug>(x: T) {
    println!("{x:?}");
}
```

### 泛型 struct / enum

```rust
struct Pair<A, B> {
    first: A,
    second: B,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 泛型 impl

```rust
impl<A: Debug, B: Debug> Pair<A, B> {
    fn show(&self) { println!("{:?} {:?}", self.first, self.second); }
}

// 只在特定具現化時才有的 method
impl Pair<i32, i32> {
    fn sum(&self) -> i32 { self.first + self.second }
}
```

第二個 impl 沒有 `<A, B>`，因為它**只**對 `Pair<i32, i32>` 有效。

## `where` clause

trait bound 多了寫得長，用 `where` 拉到下面：

```rust
fn process<T, U>(a: T, b: U) -> T
where
    T: Debug + Clone + Default,
    U: AsRef<str> + Send + Sync + 'static,
{
    ...
}
```

風格慣例：3 個以上 bound 或單個 bound 很長就用 `where`。

## 多重 trait bound

```rust
fn add_two<T: Add<Output = T> + Copy>(a: T, b: T) -> T { a + b }
```

- `T: Add<Output = T>` 要求 T 實作 `Add` 且 `+` 結果還是 T
- `+ Copy` 因為 `a + b` 會消耗 a、b，加 Copy 讓編譯器允許

## Monomorphization — Rust 泛型的代價/好處

```rust
let x = largest(&[1, 2, 3]);          // T = i32
let y = largest(&[1.0, 2.0, 3.0]);    // T = f64
```

編譯器**在 binary 內產出兩份特化**：`largest_i32` 與 `largest_f64`。

優點：
- 零 runtime 開銷（沒有 vtable、沒有 boxing）
- 編譯器可 inline、特化
- 跟手寫專用版本一樣快

缺點：
- binary 變大（每個具現化都是一份程式碼）
- 編譯變慢
- 錯誤訊息較長（要包到使用點才看得到問題）

如果你想要動態 dispatch（節省 binary、執行期切換型別），用 trait object（`Box<dyn Trait>`），下一章介紹。

## 預設 trait bound

很多時候要的 trait bound 多到痛苦。常見組合：

| 場景 | 常見 bound |
|------|------------|
| 印 / log | `Debug` |
| 對比 / 排序 | `PartialOrd` + `Ord` |
| 雜湊（key） | `Hash + Eq` |
| 跨 thread | `Send + Sync + 'static` |
| iterate | `IntoIterator` |
| 可複製 | `Clone` 或 `Copy` |

## `Default` trait

```rust
let zero: i32 = Default::default();
let empty: String = String::default();
let v: Vec<i32> = Vec::default();
```

`Default::default()` 由型別 / 變數宣告決定回哪種。標準函式庫絕大多數型別都實作。`#[derive(Default)]` 自動產生（field 全用 `Default::default()`）。

## const generics

型別參數可以是 **const 值**（不只是型別）：

```rust
fn print_array<const N: usize>(arr: &[i32; N]) {
    println!("len {}: {:?}", N, arr);
}

print_array(&[0; 5]);   // N = 5
print_array(&[0; 3]);   // N = 3
```

用途：陣列長度作為型別、矩陣維度、buffer size。最常見 case：寫泛型陣列 method。

## 慣例與風格

- 型別參數命名：`T`、`U`、`V`（單字母）或 `Item`、`Key`、`Output`（有意義）
- 簽章按 `<lifetime, type, const>` 順序排
- bound 多就 `where`
- struct generic 不要超過 4 個型別參數（讀者撐不住）

## 對照 TypeScript

泛型是 TS 跟 Rust 表面**最像**的特性之一，但底層機制完全不同：TS 泛型在 runtime 被擦除（type erasure），Rust 泛型在編譯期被 **monomorphize**——每個具現化產生獨立的特化版本。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 宣告 | `function f<T>(x: T)` | `fn f<T>(x: T)` |
| 多型參 | `<T, U>` | `<T, U>` |
| 約束 | `<T extends Foo>` | `<T: Foo>` 或 `where T: Foo` |
| 多重約束 | `<T extends Foo & Bar>` | `<T: Foo + Bar>` |
| Runtime 存在 | 擦除（`typeof T` 不存在） | **編譯期**保留，影響 codegen |
| 同一函式多次 call | 一份 code（generics 是文件） | **N 份**特化（每個 T 一份） |
| Default 參數 | `<T = string>` | `<T = i32>`（限定地方） |
| Const 參數 | 沒有 | `<const N: usize>`（const generics） |
| 條件型別 | `T extends U ? X : Y` | trait 特化 / blanket impl（無一對一） |
| infer | `T extends Array<infer U> ? U : never` | associated type / 第 13 章 GAT |
| 範例：泛型容器 | `Box<T>` / `Array<T>` | `Vec<T>` / `Box<T>` |
| 範例：泛型結果 | `Result<T, E>`（自訂） | `Result<T, E>`（內建） |

### 程式碼對照

```ts
// TS — 泛型函式
function largest<T>(arr: T[], cmp: (a: T, b: T) => number): T {
  return arr.reduce((m, x) => cmp(x, m) > 0 ? x : m);
}
const n = largest([3, 1, 4, 1, 5], (a, b) => a - b);   // 5
const s = largest(["b", "a", "c"], (a, b) => a.localeCompare(b));   // "c"
// 兩次都跑同一份 code（泛型只是型別檢查）
```

```rust
// Rust — 泛型 + trait bound + monomorphization
fn largest<T: PartialOrd + Copy>(slice: &[T]) -> T {
    let mut max = slice[0];
    for &x in slice.iter().skip(1) {
        if x > max { max = x; }
    }
    max
}
let n = largest(&[3, 1, 4, 1, 5]);    // T = i32
let s = largest(&["b", "a", "c"]);    // T = &str
// 編譯產生兩份特化：largest_i32、largest_str
```

```ts
// TS — interface 約束
interface HasId { id: number }
function findById<T extends HasId>(items: T[], id: number): T | undefined {
  return items.find(x => x.id === id);
}
```

```rust
// Rust — trait 約束
trait HasId { fn id(&self) -> u64; }
fn find_by_id<T: HasId>(items: &[T], id: u64) -> Option<&T> {
    items.iter().find(|x| x.id() == id)
}
```

### 心智模型差異

1. **TS 泛型在 runtime 不存在；Rust 泛型在 binary 內展開**。後果一：Rust 不能寫 `if T == String` 這種 runtime check（要用 trait 系統做）；後果二：泛型函式被多種 T 用過，binary 變大（每份特化獨立 code）；後果三：泛型函式跟手寫專用版本**一樣快**（無 boxing、無 vtable）。
2. **trait bound 是 Rust 的 `extends`**。TS `<T extends Foo>` 限制 T 必須有 Foo 的結構；Rust `<T: Foo>` 限制 T 必須**實作**了 Foo trait——名義型別檢查。結構像不夠，要顯式 `impl Foo for MyType`。
3. **多重約束用 `+` 不是 `&`**。TS：`T extends Foo & Bar`（intersection type）；Rust：`T: Foo + Bar`（trait bound 列表）。
4. **`Copy` / `Clone` 在 TS 沒對應**。TS 物件預設 reference 拿來拿去，泛型不必想；Rust 因為 ownership，泛型函式內想複製 `T` 必須 `T: Copy` 或 `T: Clone`。`fn duplicate<T: Clone>(x: &T) -> T { x.clone() }`。
5. **dispatch 模式由你選**。TS 永遠是 dynamic dispatch（runtime virtual call）；Rust 給你兩種：泛型 `<T: Trait>` 是 static（每份特化）、`dyn Trait` 是 dynamic（vtable）。一般預設 static，需要存到 `Vec<Box<dyn Trait>>` 才用 dynamic（第 12 章）。
6. **const generics 是 TS 沒有的維度**。`fn print<const N: usize>(arr: &[i32; N])` 把陣列長度當型別參數帶——TS 只能用 tuple type `[number, number, number]` 寫死。
7. **錯誤訊息更長**。TS 泛型錯誤通常一行訊息；Rust 泛型錯誤可能展開到 trait bound 鏈，初看嚇人但訊息實際很細。`cargo check` 時讀完通常能定位。

## 常見陷阱

1. **不熟 trait bound 訊息** — 編譯器說「`T` doesn't implement `Display`」，補 `T: Display`。
2. **`T: Copy` vs `T: Clone`** — Copy 嚴格，Clone 寬鬆但需顯式 `.clone()`。能用 Clone 就用 Clone。
3. **`impl<T> Foo for Bar<T>` 與 `impl Foo for Bar<i32>` 衝突** — Rust 不允許同時存在（重疊 impl），要用 specialization（unstable）或 trait 設計避開。
4. **binary 體積爆炸** — 大型泛型 function 被多處呼叫時，每個具現化都產一份。用 `dyn` 或把核心抽離成非泛型 inner function。
5. **error 訊息太長** — 把 trait bound 寫進 type alias 或 trait alias 助讀（trait alias 仍不穩定，用 sealed trait pattern）。

## 練習

1. 寫泛型 `swap<T>(a: &mut T, b: &mut T)`。
2. 寫 `Stack<T>` 含 `push`、`pop`、`peek`。
3. 寫泛型 `mean<T: Sum + Div<f64, Output = T> + Copy>(items: &[T]) -> T`（提示：可能需要更多 bound）。
4. 比較這兩種寫法在 binary size 與 runtime 的差別：
   ```rust
   fn render<T: Display>(x: T) { ... }
   fn render(x: &dyn Display) { ... }
   ```

## 延伸閱讀

- [The Rust Book — Ch 10 Generics](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [Rust Reference — Generic params](https://doc.rust-lang.org/reference/items/generics.html)
