# 13. 生命週期

> 範圍：lifetime 標註、'static、HRTB、elision 規則、struct 內的引用

## 為什麼有 Lifetime

第 06 章學到：reference 不能 dangle。編譯器要能判斷「**這個 reference 是不是還活著**」。

```rust
let r;
{
    let x = 5;
    r = &x;
}
println!("{r}");   // ERROR: x dropped, r dangle
```

對 local 變數編譯器自己看得出來。但**跨函式邊界**怎麼辦？

```rust
fn longest(s1: &str, s2: &str) -> &str {
    if s1.len() > s2.len() { s1 } else { s2 }
}
```

回傳的 reference 是 `s1` 還是 `s2` 的？編譯器無法推斷——所以**強迫你標**：

```rust
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str { ... }
```

讀作：「`s1`、`s2`、回傳值，**至少**都活得跟 `'a` 一樣久」。

## Lifetime 是 generic 參數

`<'a>` 跟 `<T>` 是同一級的東西——都是 generic parameter，但 lifetime 不影響型別、不影響執行期、純粹是**編譯器約束**。

```rust
fn longest<'a, T>(s1: &'a T, s2: &'a T) -> &'a T where T: PartialOrd {
    ...
}
```

順序慣例：先 lifetime，後 type，最後 const。

## Lifetime elision — 不必標的情況

很多時候編譯器可推導，不必寫：

```rust
fn first_word(s: &str) -> &str { ... }
```

編譯器套三條規則：

1. **每個 input reference 各自一個 lifetime**：`fn f(x: &T, y: &U)` 視為 `fn f<'a, 'b>(x: &'a T, y: &'b U)`
2. **只有一個 input lifetime，則 output 跟它一樣**：`fn f(x: &T) -> &U` 視為 `fn f<'a>(x: &'a T) -> &'a U`
3. **method 中有 `&self`，output lifetime = `self` 的**：method 內回傳的 reference 跟 self 同期

三條都不命中 = 必須手寫標註。

## struct 持有 reference

```rust
struct Excerpt<'a> {
    part: &'a str,
}
```

宣告 struct 有一個 lifetime 參數 `'a`，所有 reference field 都活得跟 `'a` 一樣久。使用時：

```rust
let novel = String::from("...");
let first = novel.split('.').next().unwrap();
let e = Excerpt { part: first };    // 'a 自動推導為 novel 的 lifetime
// novel 必須活到 e 不再使用之後
```

**慣例**：如果 struct 多數時候要 own 資料而非借用，就用 `String` / `Vec` 而不是 `&str` / `&[T]`，省掉一堆 lifetime 標註。

## `'static` — 整個 program 的 lifetime

```rust
let s: &'static str = "I live forever";
fn forever() -> &'static str { "..." }
```

`'static` 表示「**這個 reference 永遠有效**」。常見來源：
- 字串字面值：`"..."` 是 `&'static str`，存在 binary 的 .rodata
- `Box::leak(Box::new(...))`：故意 leak 拿 `&'static`
- `lazy_static` / `OnceCell` 全域變數

```rust
fn need_static<T: 'static>(_t: T) { ... }
```

trait bound 中的 `T: 'static` 表示「T 內部不含任何 lifetime 比 'static 短的 reference」——也就是 T 可以活到 program 結束（或它本身是 owned，無 reference）。常見於 thread spawn、Box<dyn Trait + 'static>。

## HRTB — Higher-Ranked Trait Bound

```rust
fn apply<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> usize,
{
    ...
}
```

`for<'a>` 唸「對所有可能的 lifetime 'a」。

何時用：closure 接受 `&str` 但 closure 自己對所有 lifetime 都成立。99% 的 case 編譯器自動 elide，**寫 library 提供 callback API** 時偶爾需要顯式 HRTB。

## 不能 dangle 的編譯期保證

```rust
fn dangle() -> &String {
    let s = String::from("local");
    &s    // ERROR: cannot return reference to local
}
```

要回 owned：

```rust
fn no_dangle() -> String {
    String::from("safe")
}
```

或者讓 input 帶 lifetime 傳遞：

```rust
fn no_dangle<'a>(input: &'a str) -> &'a str { input }
```

## 對照 TypeScript

**TS 完全沒有對應概念**。TS 由 GC 管理物件存活、reference 永遠有效；Rust 借用必須證明在編譯期不會 dangle，所以引入 lifetime 標註讓你告訴編譯器「reference 之間的活存關係」。

### 對照表

| 情境 | TypeScript | Rust |
|---|---|---|
| 物件何時釋放 | GC 決定（不確定） | scope 結束（確定） |
| 函式回 reference | 永遠安全（GC keep alive） | 必須證明 reference 仍有效 |
| Dangling reference | **不可能**（GC 救你） | **編譯失敗** |
| 顯式生命週期 | 不需要（沒這概念） | `<'a>` 標註 |
| 物件「永遠活著」 | 不必想 | `'static` |
| 結構持有 reference | 永遠 OK | struct 必須帶 lifetime 參數 |
| 「外部資料還活著嗎」 | 從來不問 | 編譯器逼你回答 |
| 跨函式邊界 | reference 永遠 valid | 必須標 lifetime 或 elision 推導 |

### 程式碼對照

```ts
// TS — 編譯期不檢查 reference 存活
function makeRef(): { value: number } {
  const local = { value: 42 };
  return local;       // OK，GC 把 local 升上 heap
}
const r = makeRef();
console.log(r.value);   // 42，永遠 valid

// TS 隨意持 reference 到任何物件
class Excerpt {
  constructor(public part: string) {}    // 持 string 沒問題
}
const e = new Excerpt("hello");
```

```rust
// Rust — 編譯期阻止 dangling
fn make_ref() -> &String {              // ❌ 編譯失敗
    let local = String::from("local");
    &local         // local 在函式結束時 drop，&local 會 dangle
}

// 解法 1：回 owned
fn make_owned() -> String {
    String::from("safe")
}

// 解法 2：input lifetime 傳遞
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or(s)
}

// struct 持有 reference 要標 lifetime
struct Excerpt<'a> {
    part: &'a str,
}
// 使用時 Excerpt 不能比 part 指向的資料活得久
```

對應的 TS narrowing 也不是 lifetime：

```ts
// TS 的 narrowing 跟生命週期無關
function getName(u?: User): string {
  if (!u) return "anon";
  return u.name;     // narrowing 後 u 是 User
}
```

Rust 用 `Option<&User>` 加 match 解：

```rust
fn get_name<'a>(u: Option<&'a User>) -> &'a str {
    match u {
        Some(u) => &u.name,
        None    => "anon",     // &'static str，可 coerce
    }
}
```

### 心智模型差異

1. **TS / GC 語言「reference 永遠有效」的假設要丟掉**。Rust 把「這個 reference 還活著嗎」這個問題從 runtime 移到編譯期——你要回答它，或讓編譯器 elision 規則自動回答。
2. **大多時候不必親手寫 lifetime**。3 條 elision 規則 cover 80%+：(1) 每個 input reference 一個 `'_`；(2) 只有一個 input lifetime 時 output 跟它；(3) `&self` method 內 output 跟 self。看到 `fn first_word(s: &str) -> &str` 沒標 lifetime 是合法的——elision 推完。
3. **`'static` 不是「常數」**。是「**可以活到 program 結束**」。字串字面值 `"hi"` 是 `&'static str`（存 binary），不是「不可變」。`String::leak(s)` 也能拿 `&'static str` 但會 leak。
4. **struct 持 reference 是「染色」操作**。一個 `&'a str` field 會把 lifetime `'a` 染遍 struct 的所有 method 簽章。設計時思考：**真的需要借用，還是 own 一份就好**？多數 case 改用 `String` / `Vec` 省一堆 lifetime 標註。
5. **lifetime 是 generic 參數**。跟 `<T>` 同層級——`fn f<'a, T>(x: &'a T)` 兩種參數都有。lifetime 不影響 runtime / 不影響型別、純粹是編譯器約束。
6. **trait bound `T: 'static`**。常見於 `thread::spawn` 跟 `Box<dyn Trait>`。意思是「T 不含比 `'static` 短的 reference」——也就是 T 可以活到 program 結束（owned 型別都符合，含 reference 的不符合）。TS 沒對應，因為 thread 之間是序列化傳遞。
7. **「lifetime mismatch」訊息要慢慢讀**。Rust 編譯器會明確指出兩個 reference 的生命週期不夠 overlap。修法多半是：縮短較大的、延長較小的（不太可能）、或拿 ownership 不要 borrow。

## 常見陷阱

1. **想標 lifetime 但又不想標**：先試簡單寫法，編譯器訊息會建議要不要加 `<'a>`。
2. **struct 內 reference 散播 lifetime**：常常一個 reference field 拉著整個 struct API 標註。設計時想：**真的需要 borrow，還是 own 就好**？
3. **`'static` 不是「常數」**：是「可活到 program 結束」。`String::leak(s)` 也能拿 `&'static str` 但會洩漏。
4. **`'a: 'b`（lifetime bound）**：「`'a` 至少活到 `'b` 結束」。當 struct field 跨多個 lifetime 時用。
5. **`&'a mut self` 較少寫**：method 簽章通常是 `&mut self`，由 elision 推導 lifetime。

## 心智模型

```
't 大          't 小
'static        local
   ↑              ↓
程式結束         scope 結束
```

reference 的 lifetime 必須 ≤ 它指向資料的 lifetime。lifetime 標註只是**告訴編譯器你期望的關係**——編譯器負責驗證真實情況符合。

## 練習

1. 寫 `fn shorter<'a>(a: &'a str, b: &'a str) -> &'a str` 回較短的。
2. 修以下程式：
   ```rust
   fn f(s: &str) -> &str {
       let x = String::from(s);
       &x      // 為何失敗？
   }
   ```
3. 設計 `struct Iter<'a, T> { slice: &'a [T], pos: usize }` 並實作 `next() -> Option<&'a T>`。
4. 思考：為什麼 `Box<dyn Trait>` 跟 `Box<dyn Trait + 'static>` 預設是同一個？

## 延伸閱讀

- [The Rust Book — Ch 10.3 Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [Rustonomicon — Lifetimes](https://doc.rust-lang.org/nomicon/lifetimes.html)
