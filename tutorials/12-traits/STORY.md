# 故事版：童軍營的能力徽章 🏅

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「去童軍營考能力徽章」的故事，幫你「感覺」出 trait 定義與實作、required / default method、`impl Trait` 與 `dyn Trait`、associated type、supertrait、blanket impl、orphan rule 是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 「打招呼」徽章的考試規則

暑假，阿哲參加童軍營。營地裡有各式各樣的**能力徽章**，第一個是「**打招呼徽章**」。

徽章背面寫著考試規則：

1. **必考題**：「說出你的名字。」——每個人都要自己準備，沒有標準答案。
2. **送分題**：「打招呼。」——如果你沒準備，就用營地的**標準台詞**：「Hello，我是＿＿！」空格填你剛剛說的名字。

> 這就是 **trait**：一份「**能做什麼**」的合約。`trait Greet { fn name(&self) -> String; fn greet(&self) -> String { format!("Hello, {}!", self.name()) } }`。沒有內容的 `name` 是 **required method**，一定要自己寫；有內容的 `greet` 是 **default method**，不寫就用標準台詞。

---

## 阿哲和機器人都考過了

阿哲說出「我叫 Alice 隊的阿哲」——他沒準備自己的打招呼，就用標準台詞，**考過了**。

營地還有一台**巡邏機器人**，編號 42。它說名字時說「Unit-42」；打招呼時它不用標準台詞，而是用**自己的版本**：「BEEP BOOP. I am Unit-42.」也**考過了**。

重點是：兩個人都要**親自去報名考試**。光是「看起來好像會打招呼」不算數。

> 這就是 **實作 trait**：`impl Greet for Person { fn name(&self) ... }` 只寫必考題；`impl Greet for Robot` 連 `greet` 都自己寫，**覆寫** default。Rust 是**名義**的：型別要明確寫 `impl Greet for ...` 才算有這個能力，長得像不算。

---

## 營火晚會：事先排好的節目單

晚上營火晚會，主持人要請有徽章的人上台打招呼。

第一種主持方式：主持人**事先拿到節目單**，知道第一個上台的是阿哲、第二個是機器人。他就**替每個人各準備一張專屬的流程卡**，上台時直接照著念，不用停下來想。

> 這就是 **靜態分派**：`fn say_hi(g: &impl Greet)` 和 `fn say_hi_generic<T: Greet>(g: &T)` 意思一樣——**編譯期**就知道是哪個型別，替每種型別各產生一份程式碼（上一章的 monomorphization）。沒有額外成本。

---

## 營火晚會：臨時排隊的混合隊伍

第二天晚會比較隨性：大家**臨時排成一隊**，有人、有機器人，主持人事先不知道誰站第幾個。

規定是：每個人排隊時都要裝進一樣大小的**報名袋**，袋子上**別著一張說明卡**，寫著「請這位打招呼時，要按哪個按鈕」。

主持人走到每個人面前，**先翻一下說明卡**，再照著做。稍微慢一點點，但隊伍裡可以**混著不同的人**。

> 這就是 **trait object `dyn Trait`**：`let crowd: Vec<Box<dyn Greet>> = vec![Box::new(Person{..}), Box::new(Robot{..})];`。`Box` 是一樣大小的報名袋（不同型別大小不同，要放到 heap）；說明卡就是 **vtable**，執行期才查要呼叫誰的 `greet`。`fn say_hi_dyn(g: &dyn Greet)` 也是這種。

### 不是每種徽章都能用說明卡

有些徽章的考題是「**變成一個跟你一模一樣的新隊員**」。主持人只看說明卡，根本不知道你多高多重，沒辦法做。這種徽章就**不能**用在臨時隊伍裡。

> 這就是 **object safety**：回傳 `Self`、或 method 本身帶泛型的 trait，不能變成 `dyn Trait`，編譯器會說 *cannot be made into an object*。

---

## 兩個徽章一起要

「**舞台表演證**」要求：同時有「打招呼徽章」**和**「自我介紹名牌」。阿哲兩個都有，就可以上台：先秀名牌「Person(Alice)」，再打招呼。

> 這就是 **多重 trait bound**：`fn print_and_greet<T: Greet + Display>(item: &T)`。程式裡 `impl Display for Person` 就是那塊名牌。

---

## 收藏盒徽章：每個盒子只收一種

營地有「**收藏盒徽章**」。規則是：你的收藏盒要能「拿出第一件收藏品」，而且你要**先宣告這盒收的是什麼**。

阿哲的盒子宣告「**只收數字卡**」，那它就永遠只收數字卡，不能同時又宣告成「收貝殼」。

> 這就是 **associated type**：`trait Container { type Item; fn first(&self) -> Option<&Self::Item>; }`，`impl Container for IntBox { type Item = i32; ... }`。一個型別只能對應**一種** `Item`。如果想讓同一個型別有多種版本，就改用泛型 trait（像 `From<T>`）。

---

## 先有名牌，才能考「大聲喊」

「**大聲喊徽章**」的報名條件寫著：「**要先有自我介紹名牌，才能報名。**」因為考題是：「把你的名牌內容用全大寫喊出來，後面加三個驚嘆號！」

阿哲已經有名牌，報名後什麼都不用多準備，直接喊出「PERSON(ALICE)!!!」。

> 這就是 **supertrait**：`trait Greet2: Display { fn shout(&self) -> String { format!("{}!!!", self).to_uppercase() } }`。要實作 `Greet2` 必須先實作 `Display`，default method 裡就能用 `Display` 的能力。程式裡 `impl Greet2 for Person {}` 一行就搞定。

---

## 有名牌的，全部自動送徽章

營長宣布：「從今天起，**所有有自我介紹名牌的人**，都**自動**拿到『大寫徽章』！」

不只隊員，連營地裡寫著「hello」的木牌、寫著「42」的號碼牌，因為它們都**能被唸出來**，也全部自動有了大寫徽章。

> 這就是 **blanket impl**：`impl<T: Display> ToShoutyString for T`。所有實作 `Display` 的型別都自動有 `to_shouty()`，所以 `"hello".to_shouty()`、`42.to_shouty()` 都能用。標準函式庫裡所有 `Display` 型別都有 `to_string()`，也是這樣來的。

---

## 不能替別營的人頒別營的徽章

隔壁營地的小朋友來玩。阿哲想頒一個**隔壁營地的徽章**給**隔壁營地的人**——營長阻止：「徽章和人**至少要有一個是我們營的**，才能由我們頒。不然兩個營各頒一次，就會吵架。」

阿哲想到辦法：請那位朋友**穿上我們營的背心**，他就算是我們營的隊員了，可以頒。

> 這就是 **orphan rule** 和 **newtype**：trait 或型別至少一個是你 crate 的才能寫 `impl`。`impl Display for Vec<i32>` 不行，但 `struct MyVec(Vec<i32>); impl Display for MyVec` 可以。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 能力徽章的考試規則 | **trait** |
| 必考題「說出名字」 | **required method** `fn name(&self)` |
| 送分題的標準台詞 | **default method** `greet` |
| 機器人用自己的台詞 | **覆寫** default method |
| 要親自報名才算 | `impl Greet for Person`（名義型別） |
| 事先排好節目單、專屬流程卡 | **靜態分派** `&impl Greet` / `<T: Greet>` |
| 臨時混合隊伍 | **`Vec<Box<dyn Greet>>`** |
| 一樣大小的報名袋 | **`Box`** |
| 袋上的說明卡 | **vtable**（動態分派） |
| 「變成一模一樣的新隊員」不能用說明卡 | **object safety** |
| 兩個徽章一起要 | `T: Greet + Display` |
| 收藏盒先宣告只收一種 | **associated type** `type Item` |
| 先有名牌才能考大聲喊 | **supertrait** `trait Greet2: Display` |
| 有名牌的全部自動送徽章 | **blanket impl** `impl<T: Display> ... for T` |
| 徽章和人至少一個是我們營的 | **orphan rule** |
| 穿上我們營的背心 | **newtype pattern** |

---

## 一句話記住

> **trait 就是能力徽章：必考題自己寫、送分題有標準答案；事先知道是誰就用 `impl Trait`（快），隊伍裡混著不同人就用 `Box<dyn Trait>`（翻說明卡）；徽章和人至少一個要是自己家的。**
