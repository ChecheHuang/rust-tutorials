# 05. Ownership

> 範圍：move 語意、Copy vs Clone、Drop、函式參數移動

## 為什麼有 Ownership

C/C++ 的問題：`free` 太早 = use-after-free；忘 `free` = leak；同個 buffer 兩處 free = double-free。GC 語言（Go / Java）用 runtime 解決，代價是 STW、難預測延遲、無法精準控制資源。

Rust 走第三條路：**所有權系統 + 編譯期檢查**——零 runtime 成本、自動 RAII、記憶體安全。代價是要學三條規則。

## 三條規則（必背）

1. 每個值有**唯一一個** owner。
2. 同一時刻只能有**一個** owner。
3. owner 離開 scope 時，值被 drop。

這三條配上「**move 語意**」就是整套系統的核心。

## Move：所有權轉移

```rust
let s1 = String::from("hello");
let s2 = s1;
println!("{s1}");  // ERROR: borrow of moved value
```

對 `String` 這類**擁有 heap 配置的型別**，`let s2 = s1` 不是複製內容——是把 ownership 從 `s1` **移到** `s2`。`s1` 之後不可用。

底層：`String` 是 (ptr, len, cap) 的 stack 三元組。move 只複製這個三元組，**不複製** heap 上的字串內容。原本指向 heap 的 `s1` 被標記為無效，避免兩個 owner 都認為自己負責 free。

## Copy：bitwise 複製

```rust
let n1 = 5;
let n2 = n1;
println!("{n1} {n2}");  // OK
```

對**完全在 stack 上**的小型別（`i32`、`f64`、`bool`、`char`、tuple of Copy 等），複製 stack 上的 bytes 就完事——`n1` 不會失效。

實作 `Copy` trait 的型別有 Copy 語意。Copy 是 **opt-in**：

```rust
#[derive(Copy, Clone)]
struct Point { x: i32, y: i32 }
```

什麼能 / 不能 `Copy`：
- ✅ 純 stack 資料（無 heap 配置、無 `Drop`）
- ❌ `String`、`Vec`、`Box`（會造成 double-free 風險）
- ❌ 任何 impl `Drop` 的型別（與 Drop 不可共存）

## Clone：顯式 deep copy

需要實際複製內容時：

```rust
let s3 = String::from("world");
let s4 = s3.clone();
```

`Clone` 是 **opt-in、顯式呼叫**。哲學上 Rust 不希望「不小心做了昂貴的 deep copy」——必須打 `.clone()` 提醒讀者。

`Clone` 是 `Copy` 的 supertrait（impl Copy 必先 impl Clone），但反之不然。

## 函式參數

```rust
let s = String::from("hi");
takes_ownership(s);
// println!("{s}");  // ERROR
```

傳值給函式 = move（或 copy，視型別）。要保留 owner 的兩個選擇：
1. `s.clone()` — 顯式複製
2. **借用** `&s` — 不轉移 ownership（下一章重點）

回傳值也是轉 ownership：

```rust
fn gives_ownership() -> String {
    String::from("gifted")
}
let s = gives_ownership();  // s 是新 owner
```

## Drop trait — 自動清理

owner 離開 scope，`Drop::drop` 自動執行：

```rust
struct LoudGuard(&'static str);

impl Drop for LoudGuard {
    fn drop(&mut self) {
        println!("dropping {}", self.0);
    }
}
```

範圍：
- block 結束 → drop block 內所有 local
- 函式結束 → drop 函式內所有 local
- struct drop → 遞迴 drop 所有 field

**順序**：variables 以**宣告反序** drop（先宣告的後 drop）；struct field 以**宣告順序** drop。

這就是 **RAII** —— lock guard、file handle、TCP socket 都靠 `Drop` 自動釋放。`std::sync::MutexGuard`、`std::fs::File` 都是經典例子。

## 顯式 drop

```rust
drop(big);
```

`std::mem::drop` 並不特殊——就是個 `fn drop<T>(_: T) {}`，吃進 ownership 立即讓它離開 scope。當你想**提早**釋放資源（特別是 lock）時用。

> 注意：`drop` 函式名與 `Drop::drop` method 同名，但你**不能直接呼叫** `obj.drop()`，必須用 `std::mem::drop(obj)`。

## 與 GC 語言對比

| 情境 | Rust | GC 語言 |
|------|------|---------|
| 何時釋放 | scope 結束（確定性） | GC 決定（不確定） |
| 釋放成本 | 編譯期決定，無 runtime 開銷 | runtime GC 暫停 |
| 循環引用 | 不可能（單一 owner） | GC 自動處理 |
| Resource handle | RAII 自動 | 需 try-with / using / defer |

## 對照 TypeScript

**這是 Rust 跟 TS 心智模型差最大的一章**。TS（與 JS）整套運行在 GC 上，記憶體是後台自動回收的——你**從來不需要思考「誰擁有這份資料」**。Rust 把這件事擺到語言核心，所有變數宣告同時就是 ownership 宣告。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 記憶體回收 | GC（runtime 後台執行） | RAII（編譯期決定 drop 點） |
| 變數賦值給另一個變數 | 物件 = 兩個 reference 指同物件（aliasing） | **move**（原變數失效）或 **copy**（依型別） |
| 多個變數指向同一物件 | 預設行為（reference 語意） | 預設**不行**（破壞 ownership 規則） |
| 深複製 | 自己手寫 / `structuredClone(x)` | `x.clone()`（必須**顯式**呼叫） |
| 「不小心 deep copy」 | 不太會（reference 是預設） | 不會（一定要打 `.clone()`） |
| 「不小心 alias」 | 常發生（mutate 一個影響另一個） | 編譯期就被擋下 |
| 資源清理 | `try/finally`、`using`（TC39 stage 3） | `Drop` trait 自動觸發 |
| 函式傳參 | 物件傳 reference、primitives 傳 value | **依型別**：`Copy` → copy，否則 move（**消耗**原 owner） |
| 提早釋放 | `obj = null`（祈禱 GC 收） | `drop(obj)`（**確定**立刻釋放） |
| 循環引用 | 兩個物件互指，現代 GC 處理 | **不可能**（單一 owner 規則） |

### 程式碼對照

```ts
// TS — reference 語意
const a = { name: "alice" };
const b = a;             // b 跟 a 指同一個物件
b.name = "bob";
console.log(a.name);     // "bob" — a 也被改了
console.log(b.name);     // "bob" — 兩個都可用

function consume(o: { name: string }) { /* ... */ }
consume(a);
console.log(a.name);     // 仍可用，a 還活著（GC 還沒收）
```

```rust
// Rust — move 語意
let a = String::from("alice");
let b = a;                     // ownership 轉到 b
// println!("{a}");            // ERROR: borrow of moved value
println!("{b}");                // OK

fn consume(s: String) { /* ... */ }
let a = String::from("alice");
consume(a);
// println!("{a}");            // ERROR: a 被 move 走了
```

要在 Rust 達到 TS 那種「兩處都可用」的效果，**得明確選一個**：

```rust
// 方案 A：clone（兩份獨立資料）
let a = String::from("alice");
let b = a.clone();
// 兩個都可用，但是「深複製」過——改 b 不影響 a

// 方案 B：借用（一份資料、多處唯讀引用，第 06 章）
let a = String::from("alice");
let b = &a;
// 兩個都可讀，b drop 後 a 仍正常

// 方案 C：Rc / Arc（多 owner refcount，第 14 章）
use std::rc::Rc;
let a = Rc::new(String::from("alice"));
let b = a.clone();              // 只增 refcount，不複製內容
```

### 心智模型差異

1. **「賦值」這個動作意義變了**。TS 寫 `let b = a` 是「**多一個指向同物件的別名**」；Rust 寫 `let b = a` 是「**把 ownership 從 a 搬到 b，a 死掉**」。把 TS 直覺帶來會處處撞牆。
2. **`clone` 不是免費**。Rust 不像 TS 預設拿 reference——預設是 move，要 alias 就必須**明示**自己要付什麼代價：`.clone()`（深複製）、`&`（借用）、`Rc/Arc`（refcount）。寫久了會發現這逼你思考「**我到底想要什麼資料共享模型**」。
3. **資源釋放是確定的**。TS GC 沒承諾何時收 `obj`；Rust 在 scope 結束**那一行**就 drop，且 drop 順序固定（local 反向、struct field 正向）。檔案、鎖、socket 都靠這個——對應 TS 要寫 `try/finally`。
4. **沒有「null pointer / dangling pointer」這類 runtime 錯誤**。TS 物件被誤 mutate、JS 已釋放 array 還拿到舊 reference——這些在 Rust 是**編譯失敗**而非 runtime bug。
5. **Copy vs reference 由型別決定，不是「primitive vs object」**。TS 規則是「primitives by value、objects by reference」；Rust 規則是「`impl Copy` → by value、否則 by move」。`i32` 是 Copy（行為像 TS primitive），`String` 不是 Copy（行為像 TS object 但**沒有 alias**）。
6. **GC 不存在**。沒有 STW、沒有不確定的延遲、也沒有「fight the GC」這種事——代價是要學 ownership。下章的借用是讓你**不必 clone 就能讓資料被多處讀**的核心工具。

## 常見陷阱

1. **`println!("{s1}")` 認不出 move** — 編譯器訊息會告訴你 line N moved，line M used。
2. **trying to `.clone()` 大型 vec / map 解決問題** — 多半是設計錯了，應該借用（下章）。
3. **在 `for x in v` 迴圈後 `v` 不可用** — `for x in v` 會 `IntoIterator::into_iter(v)`，move 整個 vec。要保留用 `for x in &v`。
4. **`Copy` 與 `Drop` 互斥** — 加了 `impl Drop` 後不能 derive Copy。
5. **`std::mem::drop` ≠ Drop trait** — 別混淆。前者是普通函式，後者是 trait method。

## 練習

1. 寫一個函式 `take_then_return(s: String) -> String`，吃進並回傳同一個 String（轉一圈）。
2. 把以下 code 改成不 panic 也不 ERROR：
   ```rust
   let v = vec![1, 2, 3];
   for x in v { println!("{x}"); }
   println!("{:?}", v);
   ```
3. 寫一個 `Guard` struct，建立時印 "acquired"，drop 時印 "released"。在 main 裡開兩個 nested block 觀察 drop 順序。
4. 思考：為什麼 `String` 不是 `Copy`，但 `&str` 是？

## 延伸閱讀

- [The Rust Book — Ch 4 Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rustonomicon — Ownership](https://doc.rust-lang.org/nomicon/ownership.html)
