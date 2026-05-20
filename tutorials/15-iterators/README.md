# 15. Iterator 與函式式風格

> 範圍：Iterator trait、adapter chain、collect、fold、lazy evaluation、自訂 iterator

## 核心：`Iterator` trait

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // ... 上百個 default method 全靠這個 next 撐
}
```

「**反覆 call next() 直到 None**」——就這麼簡單。整個 functional API 都是基於這個 trait 的 default method。

## 三種 iter 方式

```rust
let v = vec![1, 2, 3];

for x in v.iter()      { /* x: &i32 */ }
for x in v.iter_mut()  { /* x: &mut i32 */ }
for x in v.into_iter() { /* x: i32，v 被消耗 */ }
```

慣例：`for x in &v` ≡ `v.iter()`，`for x in &mut v` ≡ `v.iter_mut()`，`for x in v` ≡ `v.into_iter()`。

## Lazy — 不消費不執行

```rust
let it = nums.iter().map(|x| {
    println!("touched {x}");
    x * 2
});
// 此時 println 還沒跑！

let collected: Vec<i32> = it.collect();
// 現在才跑
```

`map` / `filter` / `take` / `skip` / `chain` 都是 lazy adapter，不會自己跑——直到 **consumer**（`collect`、`sum`、`for`、`count`、`fold` 等）才觸發。

這設計讓你 chain 100 個 adapter 不會多 100 個 intermediate `Vec`。

## 常用 adapter

| Adapter | 行為 |
|---------|------|
| `map(f)` | 每個 element 套 f |
| `filter(p)` | 留下 p(x) == true 的 |
| `filter_map(f)` | f 回 Option，None 跳過、Some 保留 |
| `take(n)` | 前 n 個 |
| `skip(n)` | 跳前 n 個 |
| `step_by(n)` | 每 n 個取一個 |
| `chain(other)` | 串接另一個 iterator |
| `zip(other)` | 配對成 tuple，short 那邊用完就停 |
| `enumerate()` | yield `(index, item)` |
| `rev()` | 反向（需 `DoubleEndedIterator`） |
| `cycle()` | 無限循環 |
| `flat_map(f)` | f 回 iterator，攤平 |
| `inspect(f)` | 副作用偷看（debug 用） |

## 常用 consumer

| Consumer | 行為 |
|----------|------|
| `collect::<C>()` | 蒐集成 collection |
| `sum() / product()` | 加總 / 連乘 |
| `max() / min()` | 最大/最小 → Option |
| `count()` | 計數 |
| `find(p)` | 第一個滿足 p 的 → Option |
| `any(p)` | 是否有滿足 p |
| `all(p)` | 是否全部滿足 p |
| `position(p)` | 第一個滿足 p 的 index → Option |
| `fold(init, f)` | 累積（reduce） |
| `for_each(f)` | 純副作用消費 |
| `last()` | 最後一個 → Option |

## `collect` 的型別魔法

```rust
let v: Vec<i32> = (1..=5).collect();
let s: String = ['a', 'b', 'c'].iter().collect();
let m: HashMap<i32, String> = (1..=3).map(|i| (i, i.to_string())).collect();
```

`collect` 是 generic over `FromIterator`。**目標型別告訴它要組成什麼**。turbofish 也可：

```rust
let v = (1..=5).collect::<Vec<i32>>();
```

特殊：`Iterator<Item = Result<T, E>>` 可以 collect 成 `Result<Vec<T>, E>`——任一個 Err 就早返：

```rust
let xs: Result<Vec<i32>, _> = ["1", "2", "3"].iter().map(|s| s.parse()).collect();
```

非常 idiomatic。

## `fold` — 萬能 reduce

```rust
let sum = (1..=10).fold(0, |acc, x| acc + x);
```

`fold(init, f)`：拿 init 開始，反覆 `acc = f(acc, x)`。所有 reduce、accumulate 的事情都能用 fold 寫。

`reduce` 是 `fold` 的特例（用第一個 element 作 init）：

```rust
let max = nums.iter().copied().reduce(i32::max);  // Option<i32>
```

## 自訂 Iterator

只要實作 `next`：

```rust
struct Fibonacci { a: u64, b: u64 }

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let cur = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(cur)
    }
}

let fibs: Vec<u64> = Fibonacci { a: 0, b: 1 }.take(10).collect();
```

實作 `Iterator` 後**免費**獲得 100+ 個 method（map、filter、collect、...）。

## 效能：跟手寫 loop 一樣快

```rust
// 這兩個 compile 出幾乎一模一樣的 assembly：
let sum: i32 = (0..1_000_000).map(|x| x * x).sum();

let mut sum = 0;
for x in 0..1_000_000 { sum += x * x; }
```

LLVM 對 iterator chain 的 inline / unroll / vectorize 非常成熟。**寫 functional 不會比 imperative 慢**。這是 Rust 引以為傲的「zero-cost abstraction」。

## `IntoIterator` — 為什麼 `for x in v` 直接工作

```rust
trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}
```

`for` 迴圈背後就是 `IntoIterator::into_iter`。`Vec<T>` 實作三次：
- `impl IntoIterator for Vec<T>` → `T`（消耗）
- `impl IntoIterator for &Vec<T>` → `&T`（=  `iter()`）
- `impl IntoIterator for &mut Vec<T>` → `&mut T`（= `iter_mut()`）

這就是 `for x in v` / `for x in &v` / `for x in &mut v` 都能 work 的原理。

## 對照 TypeScript

Iterator chain 是 TS 跟 Rust **最像**的功能之一——`.map().filter().reduce()` 直譯。差別只在 (1) Rust **lazy**、TS Array methods **eager**；(2) Rust 多了 `Iterator` trait 跟自訂 iterator 的標準路徑；(3) Rust iterator 跟手寫 loop **一樣快**（zero-cost）。

### 對照表

| 操作 | TypeScript（Array） | Rust（Iterator） |
|---|---|---|
| 變換每項 | `arr.map(f)` | `iter.map(f)` |
| 過濾 | `arr.filter(p)` | `iter.filter(p)` |
| 累積 | `arr.reduce(f, init)` | `iter.fold(init, f)` |
| 找第一個 | `arr.find(p)` | `iter.find(p)` → `Option` |
| 找 index | `arr.findIndex(p)` | `iter.position(p)` → `Option<usize>` |
| 是否有 | `arr.some(p)` | `iter.any(p)` |
| 是否全 | `arr.every(p)` | `iter.all(p)` |
| 加總 | `arr.reduce((a, b) => a+b, 0)` | `iter.sum()` |
| 計數 | `arr.length`（已知） | `iter.count()`（消耗） |
| 串接 | `arr.concat(other)` / `[...a, ...b]` | `a.chain(b)` |
| 配對 | 沒有原生 | `a.zip(b)` |
| 帶 index | `arr.entries()` / `arr.forEach((x, i) =>)` | `iter.enumerate()` |
| 攤平 | `arr.flatMap(f)` / `arr.flat()` | `iter.flat_map(f)` / `iter.flatten()` |
| 切片 | `arr.slice(0, n)` | `iter.take(n)` |
| 跳前 | `arr.slice(n)` | `iter.skip(n)` |
| 反向 | `[...arr].reverse()` | `iter.rev()` |
| 求值（消費） | 立刻執行 | **lazy**，需 `collect` / `sum` / for / 等才執行 |
| 蒐集成集合 | 已是 array | `iter.collect::<Vec<_>>()` / `<HashMap<_,_>>` |
| 並行版本 | 沒有（要 Worker） | `rayon::par_iter()` |

### 程式碼對照

```ts
// TS — 立即執行
const v = [1, 2, 3, 4, 5];
const result = v
  .filter(x => x % 2 === 0)
  .map(x => x * x)
  .reduce((a, b) => a + b, 0);
// 6 + 16 = 20

// 中間步驟在 runtime 真的產出陣列
const evens: number[] = v.filter(x => x % 2 === 0);   // [2, 4]
const squared: number[] = evens.map(x => x * x);      // [4, 16]
```

```rust
// Rust — lazy，沒中間 Vec
let v = vec![1, 2, 3, 4, 5];
let result: i32 = v.iter()
    .filter(|&&x| x % 2 == 0)
    .map(|&x| x * x)
    .sum();
// 20

// 編譯器把整條 chain 編成一個 loop，跟手寫一樣
let mut sum = 0;
for &x in &v {
    if x % 2 == 0 { sum += x * x; }
}
```

`Result` 集合的「short-circuit on Err」：

```ts
// TS — 用 try/catch 或 promise chain
const results: number[] = [];
try {
  for (const s of ["1", "2", "oops", "4"]) {
    const n = parseInt(s, 10);
    if (isNaN(n)) throw new Error("bad");
    results.push(n);
  }
} catch (e) { /* ... */ }
```

```rust
// Rust — collect 到 Result，第一個 Err 就早返
let results: Result<Vec<i32>, _> =
    ["1", "2", "oops", "4"].iter().map(|s| s.parse()).collect();
// results = Err(...)
```

### 心智模型差異

1. **lazy 是預設**。TS Array methods 每步都真實建陣列；Rust iterator chain `.filter().map()...` 在沒 consumer（`collect` / `sum` / `for`）前根本沒跑。`v.iter().map(|x| { println!("{x}"); x*2 })` 不會印任何東西。
2. **效能是 zero-cost**。TS 連 5 個 `.map().filter()...` 會建 5 個中間陣列、跑 5 個迴圈——一般沒事，hot path 痛。Rust LLVM 把整條 chain 編成一個 loop，跟手寫一樣快，可以放心 chain。
3. **三種 iterate**。TS `for (const x of arr)` 永遠借用、不消耗 arr；Rust 三種：`v.iter()`（`&T`）、`v.iter_mut()`（`&mut T`）、`v.into_iter()`（`T`、消耗 v）。`for x in v` 是 `into_iter`，會吃掉 v。
4. **`collect` 比 TS 強**。`collect::<Vec<_>>` / `collect::<HashMap<_,_>>` / `collect::<String>` / **`collect::<Result<Vec<_>, _>>` 自動 short-circuit on Err** — TS 沒對應，要手寫迴圈。
5. **`Option`、`Result` 也是 iterator**。`Some(3).iter()` 是 0 或 1 個元素的 iterator。常見技巧：`opt.iter().chain(other_iter)` 把 optional 跟一般集合合在一起。
6. **自訂 iterator 是「白給」一堆 method**。實作 `Iterator::next()` 後**免費**獲得 100+ 個 method。TS 寫 `for...of` 用的 Iterator protocol 也類似，但沒有 `.map().filter()` 這些方便方法（要自己 wrap 或用 lib）。
7. **並行很容易**。`use rayon::prelude::*; v.par_iter().map(...).sum()`——把 `iter` 換成 `par_iter` 直接平行化（thread pool 自動分配）。TS 對應的 Worker 要自己序列化、調度，差距很大。
8. **沒有 `.length`**。Rust iterator 不保證有大小（可能無限）。要長度：`v.len()`（對 `Vec` / `&[T]`）或 `iter.count()`（消耗 iterator）。

## 常見陷阱

1. **lazy 沒消費** — `nums.iter().map(...)` 沒 collect / for / sum，編譯器會警告 `unused must_use`。
2. **double iter 失敗** — 多數 iterator 不能複用，要 `clone()` iterator 或重新 `iter()`。
3. **`&&x` pattern** — `filter(|&&x| x > 0)` 因為 closure 拿 `&T`、filter 又包一層 `&` 得來。也可寫 `filter(|x| **x > 0)`。
4. **`size_hint`** — 自訂 iterator 多 impl 一個 `size_hint` 能讓 `collect` 預先 alloc。
5. **`for` 後 `v` 不可用** — `for x in v` 用 `into_iter`，consumed v；要保留用 `for x in &v`。

## 練習

1. 用 iterator chain 計算 1..=100 中**偶數的平方和**。
2. 把 `vec![1, 2, 3, 4, 5]` 轉成 `[(0,1),(1,2),(2,3),(3,4),(4,5)]`（enumerate）。
3. 寫 `fn pairs<T: Clone>(v: &[T]) -> Vec<(T, T)>` 回所有相鄰對。
4. 實作 `struct Counter { n: u32, max: u32 }`，當作 `Iterator<Item = u32>`，配合 `zip` 與 `filter` 算出某個結果（參考 Rust Book Ch 13）。
5. 用 `collect::<Result<Vec<_>, _>>()` 解析 `["1", "2", "oops", "4"]`，看會發生什麼。

## 延伸閱讀

- [`std::iter` 文件](https://doc.rust-lang.org/std/iter/)
- [The Rust Book — Ch 13 Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
