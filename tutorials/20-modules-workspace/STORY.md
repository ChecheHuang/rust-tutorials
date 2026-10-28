# 故事版：聯合校區裡的學校與教室 🏫

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「好幾間學校共用一個校區，每間學校裡有很多教室和辦公室」的故事，幫你「感覺」出 workspace、crate、module 三層結構，`mod` 與拆檔、`pub` / `pub(crate)` / private、`use` 與 `pub use`（re-export）、bin 與 lib、`[workspace.dependencies]`、跨 crate 依賴。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 校區、學校、教室

小禾讀的學校在一個**聯合校區**裡。校區裡有好幾間學校：國小、國中、還有一間出版教科書的編輯部。

每間學校裡有很多**教室和辦公室**，像「教務處」、「教務處裡的密碼保管室」。每個房間裡放著各種**東西**：作業本、印章、課表。

校區管理處說：「每間學校**各自開學、各自畢業**；校區只是幫大家管共用的東西。」

> 這就是 **三層結構**：**workspace**（校區）用一個 root `Cargo.toml` 管多個 crate；**crate**（學校）是獨立的編譯單位；**module**（教室）是 crate 內部的命名空間，裡面放 `fn`、`struct`、`const` 等 items。module 只存在於編譯期。

---

## 隔一間教室，要先在平面圖上登記

教務處想要一間「**密碼保管室**」。最快的方法是在教務處裡**用隔板直接隔出一間**。

後來保管室東西越來越多，乾脆**搬到隔壁大樓**。不過搬過去之後，教務處的**平面圖上一定要寫一行**：「這裡有一間密碼保管室」，不然大家根本不知道它存在，就算大樓蓋好了也沒人會進去。

> 這就是 **`mod`**：可以內嵌寫 `mod auth { pub mod password { ... } }`，也可以拆檔：`mod auth;` 會去找 `src/auth.rs`（新風格）或 `src/auth/mod.rs`（舊風格），子 module 放在 `src/auth/password.rs`。**`mod` 是宣告結構**，忘了寫，檔案就不會被編譯進來。

---

## 門口的牌子

每間房間門口都有一塊牌子：

- **沒掛牌子**：只有**這間房間裡的人**能用裡面的東西。像數學科辦公室裡的「加一小幫手」，只有數學老師能叫他。
- **「歡迎參觀」**：誰都可以進來，連校區外的人都行。
- **「本校師生限定」**：只要是這間學校的人就能進，別的學校不行。
- **「限樓上辦公室」**：只開放給管這間房的上一層辦公室。

數學科對外只開放「**乘二服務**」，但乘二服務在裡面偷偷叫了加一小幫手幫忙。外面的人只看得到結果，叫不到小幫手。

> 這就是 **visibility**：預設 private，`math::private_helper` 只在 `math` 裡用得到，`main` 裡呼叫會編譯錯誤；`pub fn double` 對外公開，內部呼叫 `private_helper(x) * 2`。`pub(crate) struct Session` 只在本 crate 可見；`pub(super)` 只給 parent module。慣例是先用 `pub(crate)`，真的需要才升 `pub`——`pub` 等於對外承諾。

---

## 叫外號比較快

每次要去密碼保管室拿東西，小禾都要說一大串：「**教務處的密碼保管室的雜湊印章**」。

老師說：「你在自己的筆記本第一頁寫一行『**雜湊印章 = 教務處的密碼保管室的雜湊印章**』，之後直接說『雜湊印章』就好。」

不過這個外號**只寫在小禾自己的筆記本裡**，別人不知道；而且寫外號**不會蓋出新房間**，房間還是要先在平面圖登記。

> 這就是 **`use`**：`use auth::password::hash;` 之後 `hash("via use")` 跟 `auth::password::hash("p@ss")` 是同一個東西；`use auth::{Session, login};` 一次引入多個；`as` 可以取別名。`use` 只是**縮短路徑**、只在目前 scope 有效，不會宣告 module。

---

## 一樓服務台

校外家長常常迷路：「報名要去哪棟哪樓哪間？」

學校乾脆在**一樓服務台**放一個窗口：「**報名請在這裡辦**。」實際上服務台把事情轉給教務處處理，但家長只要記得一樓服務台就好。

學校裡面房間怎麼重新安排都沒關係，服務台的窗口不變，家長就不會迷路。

> 這就是 **re-export `pub use`**：`pub use auth::login as public_login;` 讓外面用 `public_login("carol")`（或 `your_crate::login`）就能叫到，不用知道內部路徑。library 常在 crate root 用 `pub use crate::network::client::Client;` 把深層結構攤平成扁平 API。

---

## 有開學典禮的學校、出版教科書的編輯部

校區裡的學校分兩種：

- **一般學校**：有**開學典禮**，典禮一開始，大家就開始上課。
- **編輯部**：沒有典禮，也沒有學生；它**只出版教材**，給其他學校拿去用。

有些學校**兩種身分都有**：自己出教材，也自己開學用。甚至一間學校可以有好幾個不同的開學典禮，例如早上部和夜間部。

> 這就是 **bin crate vs lib crate**：有 `fn main` 的 `src/main.rs` 是 binary；`src/lib.rs` 是 library。一個 package 可以同時有兩者，binary 用 `your_crate::...` 呼叫 lib。多個 binary 放 `src/bin/server.rs`、`src/bin/client.rs`，用 `cargo run --bin server` 選。另外還有給 C 用的 `cdylib` / `staticlib`，以及 `proc-macro`。

---

## 校區管理處的共用規定

校區管理處發了一張**共用規定**：「全校區都用同一版的課綱、同一個學年。」各學校的規章只要寫「**課綱：照校區規定**」，就不用每間自己抄一份。

所有學校也共用**同一個大倉庫**，做好的教具放在一起，別間學校要用就不用再做一次。管理處一聲令下「全部考試」，每間學校一起考。

> 這就是 **workspace**：root `Cargo.toml` 寫 `[workspace] members = [...]`、`[workspace.package] edition = "2021"`，member 寫 `edition.workspace = true`、`version.workspace = true`（本章的 `Cargo.toml` 就是這樣）。所有 member 共用一個 `target/`，`cargo test` 一次跑全部。

---

## 統一採購

以前國小買 A 牌粉筆、國中買 B 牌，倉庫裡同一種東西放了**兩套**，又佔位置又浪費錢。

現在管理處統一列一張**採購清單**：「粉筆一律買這個牌子、這個版本。」各學校只要寫「粉筆：**照校區採購**」。

> 這就是 **`[workspace.dependencies]`**：root 寫 `serde = "1"`、`tokio = { version = "1", features = ["full"] }`，member 寫 `serde.workspace = true`。統一版本，避免同一個 crate 編兩份。

---

## 向外借東西、獨立的分校

學校需要的東西可以從三個地方來：

- **大型書局**：大家都在那裡買。
- **某個人的倉庫**：指定地址和哪一櫃去拿。
- **校區裡的隔壁學校**：走過去拿就好，不用上架到書局。

另外，校區裡有幾間比較大的**實驗分校**，管理處說：「你們**自己管自己**，有自己的規章、自己的倉庫，不要算進校區。」而校區管理處本身**沒有學生**，只負責管理。

> 這就是 **跨 crate 依賴**：crates.io（`serde = "1"`）、git（`{ git = "...", branch = "main" }`）、本地 path（`{ path = "../internal" }`）。本系列 root 用 `exclude` 讓 `capstones/` 有自己的 `Cargo.toml` 與 `target/`；root 只有 `[workspace]` 沒有 `[package]`，叫做 **virtual manifest**。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 聯合校區 | **workspace** |
| 一間學校 | **crate** |
| 教室、辦公室 | **module** |
| 房間裡的作業本、印章 | items（`fn`、`struct`、`const`） |
| 隔板隔出的房間 | 內嵌 `mod auth { ... }` |
| 搬到隔壁大樓＋平面圖登記 | `mod auth;` + `auth.rs` / `auth/password.rs` |
| 沒掛牌子 | **private**（預設） |
| 「歡迎參觀」 | **`pub`** |
| 「本校師生限定」 | **`pub(crate)`** |
| 「限樓上辦公室」 | **`pub(super)`** |
| 加一小幫手 | `math::private_helper` |
| 筆記本裡的外號 | **`use`**（只縮短路徑） |
| 一樓服務台 | **`pub use`**（re-export） |
| 有開學典禮的學校 | **bin crate**（`main.rs`） |
| 編輯部 | **lib crate**（`lib.rs`） |
| 早上部、夜間部 | `src/bin/*.rs` 多個 binary |
| 校區共用規定 | `[workspace.package]` + `edition.workspace = true` |
| 共用大倉庫 | 共享 `target/` |
| 統一採購清單 | **`[workspace.dependencies]`** |
| 書局 / 某人倉庫 / 隔壁學校 | crates.io / git / `path` 依賴 |
| 自己管自己的實驗分校 | `exclude` 的 capstones |
| 沒有學生的管理處 | virtual manifest |

---

## 一句話記住

> **workspace 是校區、crate 是學校、module 是教室：房間要先登記（`mod`）才存在，門口牌子（`pub`、`pub(crate)`、private）決定誰能進，外號（`use`）只是叫起來比較短，一樓服務台（`pub use`）讓外人不迷路，校區統一採購（`[workspace.dependencies]`）大家用同一套。**
