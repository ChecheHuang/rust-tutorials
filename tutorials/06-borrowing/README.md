# 06. References 與 Borrowing

> 範圍：&T vs &mut T、借用規則、NLL、reborrow

## 為什麼要借用

第 05 章學到 move 後原 owner 就不能再用——但很多時候我們**只想暫時看一下** / **改一下**，不想轉移所有權。

```rust
let s = String::from("hi");
let n = s.len();   // 想看 length，不想消耗 s
```

借用（borrow）= 「暫時使用，不取走」。Rust 用 `&` 表示。

## 兩種借用

### `&T` — 共享借用（shared / immutable）

```rust
let s = String::from("hello");
let r1 = &s;
let r2 = &s;     // OK，可以同時有多個
println!("{r1} {r2} {s}");
```

特性：
- 同一時刻可同時存在**多個** `&T`
- 不能透過 `&T` 修改值
- 原 owner 不可同時用 `&mut`

### `&mut T` — 獨佔借用（exclusive / mutable）

```rust
let mut s = String::from("hi");
let m = &mut s;
m.push_str(" world");
```

特性：
- 同一時刻**只能存在一個** `&mut T`
- 持有 `&mut` 期間，**任何其他** `&` / `&mut` 都不能存在（也不能用 owner 本身）
- 原 owner 必須是 `mut`（`let mut s = ...`）

## 核心規則：**一寫多讀互斥**

> 同一個值，在任一個時刻：
> - 可以有任意多個 `&T`（讀者）
> - 或者只有一個 `&mut T`（寫者）
> - **不能同時有讀者和寫者**

這就是 Rust 防 data race 的根基。**編譯期**強制執行，不依賴 runtime check。

```rust
let mut v = vec![1, 2, 3];
let a = &v;
let m = &mut v;        // ERROR: 已有 &v
println!("{a:?}");
```

## NLL — Non-Lexical Lifetimes

舊版 Rust 用 lexical scope 判斷借用存活到 `}`。NLL 改成「**最後一次使用**」：

```rust
let mut v = vec![1, 2, 3];
let a = &v;
println!("{a:?}");      // a 最後使用
// a 在此後不存在
let c = &mut v;          // OK！
c.push(4);
```

這讓 borrow checker 友善很多。實務上**不必特別記**，照常寫，編譯器告訴你哪裡卡到。

## 借用給函式

```rust
fn calc_len(s: &String) -> usize {
    s.len()
}
let s = String::from("hi");
let n = calc_len(&s);
println!("{s} → {n}");  // s 仍可用
```

`&mut` 版本：

```rust
fn append_hello(s: &mut String) {
    s.push_str(" hello");
}
let mut s = String::from("hi");
append_hello(&mut s);
```

慣例：吃字串 prefer `&str` 而非 `&String`（更通用，下章詳述）。

## reborrow — `&mut` 鏈

```rust
let mut s = String::from("x");
let outer = &mut s;
{
    let inner: &mut String = &mut *outer;  // reborrow
    inner.push('!');
}
println!("{outer}");  // outer 又可用
```

`&mut *outer` 暫時把 outer 「借出」給 inner，inner drop 後 outer 重新生效。函式呼叫常隱式做這件事：

```rust
fn append(s: &mut String) { s.push('y'); }
let mut s = String::from("a");
let r = &mut s;
append(r);     // 等同 append(&mut *r)，編譯器自動 reborrow
r.push('z');   // r 仍可用
```

## 不能 dangling

```rust
fn dangle() -> &String {           // ERROR
    let s = String::from("local");
    &s    // s 結束時 drop，回傳的 reference 會 dangle
}
```

Rust **編譯期**就禁止這種寫法。要回傳擁有資料，就**回傳 owned 型別**（`String`）。或者要求 input lifetime（第 13 章）。

## auto deref：method 呼叫透明

```rust
let s = String::from("HELLO");
let s_ref: &String = &s;
println!("{}", s_ref.to_lowercase());  // 不必 (*s_ref).to_lowercase()
```

對 `&T` 呼叫 `T` 的 method，Rust 自動 deref。深度與規則見第 14 章 smart pointers。

## 對照 TypeScript

借用對 TS 開發者來說是**全新概念**——TS 永遠拿 reference，沒有「借出 vs 移交」的區分。這章是把第 05 章 ownership 之外的另一半 Rust 心智模型補上。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 取得 reference | 自動（物件預設 reference） | 顯式：`&x`、`&mut x` |
| 多個讀者 | 隨意 | 多個 `&T` OK |
| 多個寫者 | 隨意（race condition 沒人擋） | **絕對一次只有一個** `&mut T` |
| 讀寫並存 | 隨意 | **禁止**：有 `&T` 就不能有 `&mut T`（反之亦然） |
| Data race | runtime 才炸（Worker 也有） | **編譯期**擋下 |
| 不可變參數 | `Readonly<T>` 提示（不強制） | `&T` 強制不能修改 |
| 可變參數 | 直接傳，function 隨意改 | `&mut T` 顯式宣告 |
| 物件被別處 mutate | 常見 bug（`Object.freeze` 才擋） | 編譯期不可能（互斥規則） |
| 函式不影響 caller | 自己 deep clone 進去 | 拿 `&T` 借用，**不必複製** |
| dangling reference | GC 防住（永遠不 dangle） | 編譯期 lifetime 防住 |

### 程式碼對照

```ts
// TS — 函式可能偷偷改 caller 的物件
function appendIfShort(arr: number[]) {
  if (arr.length < 3) arr.push(99);   // 沒在簽章上提示！
}
const xs = [1, 2];
appendIfShort(xs);
// xs 現在是 [1, 2, 99]，呼叫端可能沒預期
```

```rust
// Rust — 簽章顯示是否可變
fn append_if_short(arr: &mut Vec<i32>) {
    if arr.len() < 3 {
        arr.push(99);
    }
}
let mut xs = vec![1, 2];
append_if_short(&mut xs);   // caller 必須打 &mut，看到就知道會被改
```

只讀範例：

```ts
// TS：函式承諾不改？看註解 / Readonly<T>，runtime 沒保證
function sum(arr: ReadonlyArray<number>): number {
  return arr.reduce((a, b) => a + b, 0);
}
```

```rust
// Rust：&[T] 直接禁止修改
fn sum(arr: &[i32]) -> i32 {
    arr.iter().sum()
}
```

### 心智模型差異

1. **「拿 reference」變成設計決策**。TS 寫 `function f(obj)` 不必想——本來就是 reference。Rust 要寫 `f(s: &str)` / `f(s: &mut String)` / `f(s: String)`，**三種都不同語意**。一開始煩，但簽章本身就告訴 caller「我會借、我會改、我會吃掉」。
2. **「一寫多讀互斥」是 data race 的解藥**。TS 在主執行緒沒 data race（單緒），但物件被多處同時 mutate 的邏輯 bug 很常見。Rust 編譯期擋下這類「同時兩處改一個東西」，連單緒也適用——副作用：直譯 JS 思路會處處撞牆，但寫久了會發現 bug 真的少很多。
3. **iterate 時不能 push 是同一條規則**。TS 寫 `arr.forEach(x => arr.push(x))` 不會編譯失敗（runtime 可能無限迴圈或行為怪異）。Rust `for x in &v { v.push(*x); }` 直接編譯失敗——`&v` 是 `&Vec<i32>` 借用、`push` 要 `&mut`，違反互斥。
4. **dangling 是不可能的**。TS 永遠不 dangle（GC 維持物件活著）；C++ 經典 dangling pointer bug 在 Rust 也是編譯失敗。代價是寫某些資料結構（雙向鏈表、graph）會痛，需要 `Rc/Arc + Weak`（第 14 章）。
5. **函式簽章顯示意圖**。看到 `fn render(d: &Doc)` 一眼知道是唯讀、`&mut Doc` 知道會改、`Doc` 知道會消耗。TS 純粹靠命名 / 文件約定。對大型 codebase 維護是質變。
6. **NLL 讓你不必想「lexical scope」**。Rust 編譯器算的是「**最後一次使用**」而非「**到 `}` 為止**」。實務上：照常寫，編譯器卡你才調整——多數時候直覺有效。

## 常見陷阱

1. **在 loop 內同時持 `&` 與 `&mut`**：
   ```rust
   let mut v = vec![1, 2, 3];
   for x in &v {           // &v
       v.push(*x);          // ERROR: v 已被借用
   }
   ```
   解法：先 collect 要新增的，再 push；或用 index loop。

2. **methods 隱含借用**：`s.method()` 等同 `Method::method(&s)`，可能借用 self；要修改需 `&mut self` 簽章 + `mut s`。

3. **編譯器訊息看 borrow span**：error 訊息會標出「borrow starts here / ends here」，照著看就能定位。

4. **慣例：函式參數 prefer borrow**：`fn process(s: &str)` 比 `fn process(s: String)` 更通用。除非函式真要取得所有權（如存到 struct）。

5. **`&mut Vec<T>` vs `&mut [T]`**：前者能 push/pop，後者只能修改元素內容。API 設計時想想 caller 要做什麼。

## 練習

1. 寫 `fn first_word(s: &str) -> &str` 回第一個單字，不要複製字串。
2. 修以下程式碼讓它編譯：
   ```rust
   let mut v = vec![1, 2, 3];
   let first = &v[0];
   v.push(6);
   println!("{first}");
   ```
3. 寫 `fn swap(a: &mut i32, b: &mut i32)`，呼叫 `swap(&mut x, &mut y)`。
4. 思考：為什麼編譯器**不**讓 `Vec::push` 在借用 element 期間執行？（提示：realloc）

## 延伸閱讀

- [The Rust Book — Ch 4.2 References & Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [NLL RFC 2094](https://rust-lang.github.io/rfcs/2094-nll.html)
