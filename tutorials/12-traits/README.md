# 12. Traits

> 範圍：trait 定義 / 實作、default method、associated type、trait object、orphan rule、blanket impl

## Trait = 「能做什麼」的合約

```rust
trait Greet {
    fn name(&self) -> String;            // required
    fn greet(&self) -> String {          // default
        format!("Hello, {}!", self.name())
    }
}
```

- 沒 body 的 method = **required**，每個實作型別必須提供
- 有 body 的 method = **default**，可選擇 override

跟 Java interface 比的差別：default method **不只是糖**——可以基於其他 method 寫複雜邏輯（標準函式庫 `Iterator` 就有上百個 default method）。

## 實作 trait

```rust
struct Person { name: String }

impl Greet for Person {
    fn name(&self) -> String { self.name.clone() }
}
```

## 在 API 上接受 trait

三種寫法，**語意一樣**（除了 dyn）：

```rust
// 1. impl Trait — 靜態 dispatch，編譯期決定
fn say_hi(g: &impl Greet) { println!("{}", g.greet()); }

// 2. 泛型 + bound — 同上，更明顯
fn say_hi<T: Greet>(g: &T) { println!("{}", g.greet()); }

// 3. dyn Trait — 動態 dispatch，runtime vtable
fn say_hi(g: &dyn Greet) { println!("{}", g.greet()); }
```

| | `impl Trait` / 泛型 | `dyn Trait` |
|---|---|---|
| Dispatch | static（編譯期 inline） | dynamic（vtable lookup） |
| Binary size | 每具現一份 | 一份 |
| Runtime cost | 零 | indirect call |
| 可放 `Vec` | ❌（每個元素須同型別） | ✅ |
| 可作為 function 回傳 | `impl Trait` 可，泛型不可（無具體型別） | ✅ |
| 物件安全限制 | 無 | 有（見下） |

## Trait object — `dyn Trait`

```rust
let crowd: Vec<Box<dyn Greet>> = vec![
    Box::new(Person { ... }),
    Box::new(Robot { ... }),
];
for g in &crowd { g.greet(); }
```

存在 heap、有 vtable，能在同一個 collection 裝**不同具體型別**。

### Object safety

不是所有 trait 都能變成 `dyn Trait`。**Object-safe** 的條件主要是：
- 所有 method 不能是 generic（`fn foo<T>(&self, t: T)` 不行）
- 不能用 `Self` 作為非 receiver 的型別（如回傳 `Self` 不行）
- `Sized` 不可作為 supertrait

違反時編譯器會明確告訴你。要修：把問題 method 加 `where Self: Sized`，這樣只能透過泛型呼叫，但 trait 還能 dyn-ify。

## Associated type vs generic type

```rust
// 用 associated type
trait Container {
    type Item;
    fn first(&self) -> Option<&Self::Item>;
}

// 用 generic
trait Container<T> {
    fn first(&self) -> Option<&T>;
}
```

差別：
- associated type：**每個型別只能對應一個 `Item`**。`IntBox` 的 Item 永遠是 i32，不能同時實作兩個版本。
- generic type：**同一個型別可實作多種**。`MyContainer` 可以同時實作 `Container<i32>` 與 `Container<String>`。

`Iterator` 用 associated type（因為一個 iterator 只 yield 一種 item），`From<T>` 用 generic（一個型別可以從多種來源轉）。經驗法則：**選擇唯一就 associated type；可多就 generic**。

## Supertrait

```rust
trait Greet2: Display {
    // 實作 Greet2 必須先實作 Display
    fn shout(&self) -> String {
        format!("{}!!!", self).to_uppercase()
    }
}
```

「**A 是 B 的 supertrait**」= 實作 A 必先實作 B。default method body 可以用 supertrait 的 method。

## Blanket impl

```rust
impl<T: Display> ToShoutyString for T {
    fn to_shouty(&self) -> String { format!("{self}").to_uppercase() }
}
```

「**對所有實作了 Display 的型別，都自動提供 to_shouty**」。標準函式庫到處用：

```rust
impl<T> ToString for T where T: Display { ... }  // 為何所有 Display 型別都有 to_string
impl<T> From<T> for T { ... }                     // 任何型別 From 自己
```

## Orphan rule

「**至少 trait 或型別有一個是你 crate 的**」才能寫 impl。

```rust
// 你的 crate 內：
impl MyTrait for Vec<i32> { ... }    // ✅ trait 是你的
impl Display for MyStruct { ... }    // ✅ struct 是你的
impl Display for Vec<i32> { ... }    // ❌ 兩個都不是你的
```

設計目的：防 crate A 和 crate B 各自為 `Vec` impl 同一個 trait，導致下游程式不知該用誰。

繞過：用 **newtype pattern**：

```rust
struct MyVec(Vec<i32>);
impl Display for MyVec { ... }  // OK，MyVec 是你的
```

## `Self` vs `self`

- `Self`（大寫）= 「實作這個 trait 的型別」（型別層級）
- `self`、`&self`、`&mut self` = method receiver（值層級）

```rust
trait Builder {
    fn build(self) -> Self;  // ownership 收尾 builder
}
```

## 對照 TypeScript

Trait 表面像 TS `interface`，但**威力大很多**：default method 可以含實作、能為**別人的型別** impl、有 blanket impl、能變成 trait object（runtime polymorphism）。TS interface 只表達 shape，trait 表達能力 + 行為。

### 對照表

| 概念 | TypeScript | Rust |
|---|---|---|
| 定義 | `interface Greet { greet(): string }` | `trait Greet { fn greet(&self) -> String; }` |
| 預設實作 | 沒有（要 abstract class） | trait 內可寫 default body |
| 實作宣告 | `class Foo implements Greet` | `impl Greet for Foo` |
| 多重 | `implements A, B` | `impl A for Foo` + `impl B for Foo` 多個 block |
| 結構/名義 | **結構**（同形狀自動符合） | **名義**（必須顯式 impl） |
| 為別人型別加方法 | declaration merging / prototype 修改 | `impl Trait for ExternalType`（**孤兒規則**限制） |
| 動態 dispatch | 永遠是 | `dyn Trait`（顯式選） |
| 靜態 dispatch | 沒有 | `impl Trait` / `<T: Trait>`（預設選） |
| Associated type | `interface I { type A; ... }` 沒有；用 generic | `type Item` |
| Supertrait | `interface B extends A` | `trait B: A` |
| 物件安全 | 沒這概念 | 有限制（method 不能 generic、不能回 `Self`） |
| Method 衝突 | 看 lookup 順序 | fully-qualified syntax：`<T as Trait>::method(x)` |
| 自動 `to_string` | `obj.toString()` 來自 Object | 實作 `Display` 自動有 `to_string`（blanket impl） |

### 程式碼對照

```ts
// TS — interface + class
interface Greet {
  name(): string;
  greet(): string;
}

class Person implements Greet {
  constructor(private n: string) {}
  name(): string { return this.n; }
  greet(): string { return `Hello, ${this.name()}!`; }   // 沒 default 只能自己寫
}

const crowd: Greet[] = [new Person("alice"), new Person("bob")];
for (const g of crowd) console.log(g.greet());
```

```rust
// Rust — trait + impl + default method
trait Greet {
    fn name(&self) -> String;
    fn greet(&self) -> String {                  // default
        format!("Hello, {}!", self.name())
    }
}

struct Person { n: String }
impl Greet for Person {
    fn name(&self) -> String { self.n.clone() }
    // greet 用 default
}

let crowd: Vec<Box<dyn Greet>> = vec![
    Box::new(Person { n: "alice".into() }),
    Box::new(Person { n: "bob".into() }),
];
for g in &crowd { println!("{}", g.greet()); }
```

為別人的型別加方法（**TS 做不到的事**）：

```rust
// 為標準函式庫的 i32 加 method
trait Shoutable { fn shout(&self) -> String; }
impl Shoutable for i32 {
    fn shout(&self) -> String { format!("{self}!!!") }
}
println!("{}", 42.shout());   // "42!!!"
```

### 心智模型差異

1. **trait > interface 的能力差**。TS interface 純粹是「shape 描述」，runtime 不存在。Rust trait 是真的「能力契約」：default method 可寫複雜邏輯（標準 `Iterator` trait 有上百個 default method）、blanket impl 為**所有**滿足某條件的型別自動加方法、可變成 trait object 做 runtime polymorphism。
2. **名義 vs 結構**。TS 兩個 interface 形狀相同就互通；Rust 必須**顯式** `impl Trait for Type`。多了一個動作但好處：能精確控制誰實作了什麼，且能在編譯後從型別推回所有實作（refactoring 友好）。
3. **`dyn Trait` 跟 TS 用法不同**。TS 永遠 dynamic，跟 interface 變數沒區別。Rust 預設 static（`fn f<T: Trait>(t: T)`，每份特化），要 dynamic 必須顯式 `Box<dyn Trait>` 或 `&dyn Trait`——後者才能塞進同個 `Vec`。設計時要選邊。
4. **`impl Trait` vs `dyn Trait` 語意不同**。`fn f() -> impl Trait` = 「我回**一種**實作 trait 的具體型別，編譯期決定」；`fn f() -> Box<dyn Trait>` = 「runtime 才知道是哪種」。回 closure 必用前者，runtime 切換型別必用後者。
5. **孤兒規則（orphan rule）**。Rust 限制：「trait 跟型別至少一個是你 crate 的」才能寫 impl。防止兩個 lib 各自為 `Vec` 實作同個 trait 衝突。TS 寫 `Array.prototype.foo = ...` 隨意——但這也是 TS 生態的混亂來源。
6. **blanket impl 是「全網覆蓋」**。Rust：`impl<T: Display> ToString for T` 讓**所有** `Display` 型別自動獲得 `to_string`。TS 用 declaration merging 也能擴 prototype，但無條件約束、容易污染。
7. **associated type 不是 generic**。TS 沒對應；Rust `Iterator::Item` 表示「**每個** Iterator 實作只能對一種 Item」。設計選擇：唯一對應用 associated type、可多對應用 generic（如 `From<T>`）。

## 常見陷阱

1. **`&dyn Trait` 的物件安全錯誤** — error 訊息會說「the trait cannot be made into an object」。看具體哪個 method 違反。
2. **trait 衝突的 method 名** — `s.iter()` 在多個 trait 都有時，用 fully-qualified syntax：`Trait::method(&value)`。
3. **`From` 與 `Into` 互推** — impl `From<A> for B` 自動產生 `impl Into<B> for A`，反之不行。一般只實作 From。
4. **default method 不知道**：impl trait 時忘 override 高效版本（如 `Iterator::size_hint`），用 default 慢版。
5. **dyn 不能 `Self`** — trait method 回 `Self` 就無法 dyn 化（不知道 Self 大小）。

## 練習

1. 寫 `trait Animal { fn sound(&self) -> &str; }`，實作 `Dog`、`Cat`、`Cow`，丟進 `Vec<Box<dyn Animal>>` iterate。
2. 設計 `trait Shape { fn area(&self) -> f64; fn perimeter(&self) -> f64; }`，給 `Circle`、`Square` 實作。
3. 寫 blanket impl：對所有 `Iterator<Item = i32>` 提供 `sum_squares()` method。
4. 思考：為什麼 `Vec<Box<dyn FnOnce()>>` 編譯失敗，但 `Vec<Box<dyn Fn()>>` OK？

## 延伸閱讀

- [The Rust Book — Ch 10.2 Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)
