# 故事版：玩具工廠的魔法貼紙 🏭

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「玩具工廠裡，貼一張貼紙就有老師傅幫你把說明書寫好」的故事，幫你「感覺」出 derive macro、helper attribute、`thiserror` 的 `#[error]` 與 `#[from]`、attribute macro、function-like macro、`syn` + `quote`、為什麼 proc macro 要放在獨立 crate。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 貼一張貼紙，說明書就寫好了

小樂在玩具工廠打工。每做好一個新玩具，就要寫一份「**怎麼把玩具裝進紙箱寄出去**」的說明書。玩具一多，小樂寫到手痠。

工廠的老師傅說：「你在玩具盒上貼一張『**請幫我寫裝箱說明書**』的貼紙就好。**工廠開工之前**，我會先看過盒子裡有哪些零件，照著零件幫你寫好。」

隔天小樂一看，每個貼了貼紙的盒子旁邊，都多了一份寫得整整齊齊的說明書。

> 這就是 **derive macro**：`#[derive(Serialize, Deserialize)]` 貼在 `struct User` 上，編譯期 `serde` 會看 `id`、`name`、`bio`、`email` 這些欄位，**自動產生** `impl Serialize for User` 和 `impl Deserialize for User`。`serde_json::to_string_pretty(&u)` 用的就是這份自動寫好的說明書。

---

## 貼紙旁邊的小便條

有些玩具有特別的要求。

「自我介紹卡」如果沒附，小樂想要老師傅**放一張空白卡**就好；「聯絡信箱」如果沒有，就**乾脆不要印在箱子上**。

小樂在零件旁邊夾了小便條：「沒有就放空白的」、「沒有就跳過」。老師傅寫說明書時看到便條，就照著改寫法。

> 這就是 **helper attribute**：`#[serde(default)] bio` 和 `#[serde(skip_serializing_if = "Option::is_none")] email`。這些便條**只有 serde 的老師傅看得懂**，別的師傅看了也不知道是什麼意思——它們不是公開的 Rust 語法，是那個 derive 自己定義的。

---

## 從箱子裡拆回玩具

有一天退回來一個箱子，裡面只有「編號 2、名字 bob」，**沒有自我介紹卡**。

小樂照著說明書反過來拆：因為便條寫了「沒有就放空白的」，所以拆出來的玩具，自我介紹卡是空白的，沒有出錯。

> 這就是 **反序列化**：`serde_json::from_str(r#"{"id": 2, "name": "bob"}"#)` 得到 `User { bio: "", email: None, .. }`。`bio` 缺了用 `Default` 補，`email` 是 `Option` 缺了就是 `None`。

---

## 問題卡也能自動寫

工廠出包的時候，要填「**問題卡**」：零件找不到、第幾行說明看不懂、倉庫門打不開……每種問題的句子都不一樣。

老師傅說：「一樣啊，你在每種問題旁邊寫一句**範本句子**，像『第 {幾行} 看不懂：{原因}』，我幫你把『怎麼把問題卡唸出來』寫好。」

> 這就是 **`thiserror` 的 derive**：`#[derive(Debug, Error)] enum AppError`，每個 variant 上的 `#[error("parse error at line {line}: {msg}")]` 變成 `Display` 的內容。`println!("error: {err}")` 就印出 `parse error at line 42: unexpected token`。

---

## 自動轉接頭

倉庫那邊的問題卡長得跟工廠的**不一樣**。以前小樂每次都要手動抄一遍，改成工廠的格式。

老師傅在「倉庫問題」那一格貼了一個**轉接頭**：「以後倉庫的卡直接插進來，就會自動變成工廠的卡，而且**原本那張倉庫卡會夾在後面**，想看還看得到。」

> 這就是 **`#[from]`**：`Io(#[from] std::io::Error)` 讓 thiserror 自動產生 `impl From<std::io::Error> for AppError`，所以 `let wrapped: AppError = io.into();` 可以成立（`?` 也會自動轉）。同時它被設成 `source()`，`wrapped.source()` 拿得到原本的 io 錯誤。

---

## 最常用的貼紙早就有了

小樂發現，工廠門口本來就放著幾種**最常用的貼紙**：「可以把玩具描述出來」、「可以照樣複製一個」、「可以比較兩個一不一樣」。

他給「座標小卡」貼上這幾張，就能複製一張、拿來比，確定兩張一模一樣。

> 這就是 **內建 derive**：`#[derive(Debug, Clone, PartialEq, Eq, Hash)] struct Coord(i32, i32)`。`Debug`、`Clone`、`PartialEq` 也都是 derive——「看結構、生 impl」，跟 serde 是同一種東西。

---

## 另外兩種魔法：包裝紙和特製訂單

老師傅還會兩招：

- **包裝紙**：把一個整個做好的玩具（例如「開工儀式」）**整個包起來**，外面加一層別的東西。玩具本身也可以被改掉。
- **特製訂單機**：你寫一張訂單丟進去，機器**在開工前就檢查訂單對不對**，甚至打電話去倉庫確認有沒有這些零件。

> 這就是 **attribute macro** 與 **function-like macro**：`#[tokio::main]` 把 `async fn main` 包成同步的 runtime 入口；`#[tracing::instrument]` 幫函式包上 span。`sqlx::query!("SELECT ...")` 長得像 `macro_rules!`，但能在**編譯期連資料庫**驗證 SQL——宣告式 macro 做不到。

---

## 老師傅怎麼工作

小樂偷看老師傅工作：

1. 先把盒子裡的東西**拆成一張零件清單**（名字、有哪些欄位）。
2. 拿出一張**填空範本**，把名字填進去。
3. 把寫好的說明書交出去。

而且老師傅**不在工廠裡**，他在**隔壁的工坊**。因為說明書要在工廠開工**之前**就寫好，所以工坊一定要比工廠早開門。

> 這就是 **`syn` + `quote` + 獨立 crate**：輸入 `TokenStream` → `syn::parse_macro_input!(input as DeriveInput)` 變成 AST（零件清單）→ `quote! { impl #name { ... } }` 填範本 → 輸出 `TokenStream`。proc macro 必須在 `[lib] proc-macro = true` 的 crate，先編譯好給編譯器載入，所以才有 `serde` + `serde_derive` 兩個 crate 的分法。

---

## 偷看說明書、別亂請師傅

小樂想知道老師傅到底寫了什麼，就用一副**特殊眼鏡**，看見貼紙展開後的完整說明書。寫好的說明書是普通文字，**玩具出貨後，裡面完全看不到貼紙的痕跡**。

不過老師傅也提醒：「請我寫說明書很慢，也很貴。**只有一個玩具**的話，你自己寫就好。工廠裡一堆玩具都要同一種說明書時，才值得找我。」

> 這就是 **`cargo expand`** 與 **何時自己寫 proc macro**：展開後是純 Rust，執行時**零成本**；但 macro 會拖慢編譯、錯誤位置（span）難處理。先找現成 crate，同樣的 impl 重複寫 N 次才考慮自己寫，學的話走 proc-macro-workshop。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 「請幫我寫說明書」貼紙 | **derive macro** `#[derive(...)]` |
| 老師傅照零件寫的說明書 | 自動生成的 `impl Serialize for User` |
| 工廠開工之前就寫好 | **編譯期**展開 |
| 貼紙旁的小便條 | **helper attribute** `#[serde(default)]`、`skip_serializing_if` |
| 缺卡時放空白卡 | `Default` 補欄位、`Option` 為 `None` |
| 問題卡的範本句子 | `#[error("...")]` → `Display` |
| 自動轉接頭，原卡夾在後面 | `#[from]` → `From` impl + `source()` |
| 門口最常用的貼紙 | 內建 derive：`Debug`、`Clone`、`PartialEq` |
| 包裝紙 | **attribute macro** `#[tokio::main]` |
| 開工前檢查的特製訂單機 | **function-like macro** `sqlx::query!` |
| 零件清單 | `syn` 解析出的 AST（`DeriveInput`） |
| 填空範本 | `quote!` |
| 隔壁先開門的工坊 | **proc-macro crate**（`proc-macro = true`） |
| 特殊眼鏡 | `cargo expand` |
| 一個玩具別請師傅 | 別為一個 struct 寫 proc macro |

---

## 一句話記住

> **proc macro 就是工廠開工前的老師傅：你貼貼紙（`#[derive]`）、夾便條（helper attribute），他在隔壁工坊用 `syn` 拆零件、用 `quote!` 填範本，幫你寫好 impl；出貨時看不到貼紙，執行零成本，但請他很花編譯時間。**
