---
name: rust-chapter-quiz
description: Rust 語言教學章節互動測驗工具。針對 rust-tutorials 專案的 54 堂課與 2 個 capstone，透過問答形式測試使用者對特定章節的掌握程度，答錯時立即講解直到完全理解。使用時機：使用者說「測驗」、「quiz」、「考考我」、「我想測試第 X 章」、「幫我出題」，或任何要測試 Rust 教學章節知識的場景。
---

# Rust Chapter Quiz

針對 `tutorials/` 下的 54 堂 Rust 課程與 `capstones/` 下的 2 個專案進行互動式知識測驗。

## 互動方式：一律用一般訊息 + 數字作答

**這個 skill 完全不使用 `AskUserQuestion`。** 所有互動（選模式、選章節、出題、總結後的下一步）都用一般 assistant 訊息呈現，選項一律標成 `1`、`2`、`3`、`4`，請使用者直接回覆數字。

理由：

- 題幹常含多行 Rust 程式碼，互動卡片對程式碼排版與語法高亮支援不穩定，容易擠成一團
- 一般訊息可以完整呈現 fenced code block（含語言標註），題目品質與可讀性最重要
- 統一用數字作答，使用者心智負擔最小，流程一致

呈現選項的固定格式：

```
選項：

1. ...
2. ...
3. ...
4. ...

請直接回答 1、2、3 或 4。
```

每次出題或要使用者選擇時，都用上述格式，並等使用者回覆數字後再繼續。

## 程式碼呈現規則

題目程式碼**一律**以帶語言標註的 markdown fenced code block 輸出到對話訊息，不要塞進單行文字裡：

- Rust 程式碼用 ` ```rust `
- `Cargo.toml` 用 ` ```toml `
- shell 指令用 ` ```bash `
- 錯誤訊息 / 輸出用 ` ```text `

不要用無標註的裸 code fence，會失去語法高亮。短程式碼（1–3 行）與長程式碼（4 行以上、含函式體 / borrow / scope / 多段比較）都走同一條通道——直接貼在題目訊息裡。

## 測驗流程

### 第一步：選擇模式

用一般訊息輸出：

```
請選擇測驗模式：

1. 學習模式（推薦）：答錯立即講解，確認理解後才繼續下一題，共 5 題
2. 考試模式：全部 5 題做完後再統一講解錯誤，模擬實際考試
3. 快速測驗：只出 3 題，快速確認核心概念

請直接回答 1、2 或 3。
```

### 第二步：選擇章節大類

8 個 Part 先分成 4 群，請使用者選群（也接受直接輸入章節編號）：

```
要測驗哪個範圍的章節？也可直接輸入章節編號（如「14」）：

1. Part 1–3：語言核心 (01–25)：所有權、借用、型別系統、生命週期、trait、智慧指標、巨集、unsafe、模組、測試、cargo、tracing、config、benchmark
2. Part 4–5：非同步 + Web (26–39)：async/await、tokio、Send/Sync/Pin、channel、stream、http、axum、actix、serde、sqlx、SeaORM、middleware、JWT、WebSocket
3. Part 6–7：架構 + 部署 (40–50)：Clean Arch、DI、Redis、gRPC、MQ、CQRS、Docker、CI/CD、K8s、Prometheus/OTel、profiling/resilience
4. Part 8 + Capstones：桌面與專案 (51–54、cap1、cap2)：Tauri 基礎、系統整合、hidapi、跨平台打包；CLI 工具、RGB 控制器

請直接回答 1、2、3、4，或輸入章節編號。
```

使用者選了某群後，再列出該群下的具體章節（每次最多列 4 個常見章節，其餘讓使用者直接輸入編號）。例：

```
Part 1–3 範圍，請選擇章節（或直接輸入其他編號，如「14」）：

1. 05 所有權：ownership、move、clone、Copy trait
2. 06 借用：& vs &mut、borrow checker、reborrow
3. 13 生命週期：'a 標註、elision、'static
4. 其他章節 (01–04, 07–25)：請直接輸入章節編號，如「14」

請回答 1、2、3，或輸入章節編號。
```

各群建議首頁顯示的章節：

- 第二群（Part 4–5）：26 async-await、32 axum-basics、35 sqlx-postgres、38 jwt-auth
- 第三群（Part 6–7）：40 clean-architecture、43 grpc-tonic、48 kubernetes、50 profiling-resilience
- 第四群：51 tauri-basics、53 hidapi-usb、cap1 cli-tool、cap2 rgb-controller

詳細章節目錄與核心考點見 `references/chapters.md`。

### 第三步：讀取章節內容

選定章節（如 14）後讀取：

```
tutorials/14-smart-pointers/README.md
tutorials/14-smart-pointers/src/main.rs   （一併讀取以掌握實際程式碼）
```

Capstone：

```
capstones/01-cli-tool/README.md + src/main.rs (+ src/search.rs 等核心模組)
capstones/02-rgb-controller/README.md + src/lib.rs + src/device.rs + src/effects.rs
```

路徑格式：`tutorials/{XX-目錄名稱}/README.md`，目錄名稱對照見 `references/chapters.md`。

### 第四步：設計並出題

根據模式決定題數：學習 / 考試模式 5 題，快速測驗 3 題。

難度排序：概念題（第 1–2 題）→ 應用題（第 3–4 題）→ 陷阱或進階題（第 5 題）

#### ⛔ 絕對禁止：題目不完整就丟選項

**在請使用者作答前，必須先把題目完整呈現在對話訊息裡。** 這是不可妥協的硬規則。

具體禁止的情況：

1. **程式碼缺失** — 選項在問「哪一行會編譯失敗 / 哪個呼叫會 panic」，但程式碼裡根本沒有那些行 / 呼叫
2. **指代不明** — 用「上方程式碼」「下面這段」但實際上方對話訊息裡沒有對應的 code block
3. **情境只寫一半** — 例如題幹說「比較 A 寫法和 B 寫法」但只給了 A、沒給 B
4. **缺輸入** — 題目要算輸出結果，但沒給輸入值
5. **預期被推測** — 用「請腦補 c1、c2、c3 都呼叫 run_twice」這種「使用者自己想像」的方式提問

**自檢清單**（輸出題目後、等使用者作答前，先回頭讀一次自己剛輸出的訊息）：

- [ ] 題目敘述本身是否能獨立回答？（不靠想像 / 不靠記憶之前的對話）
- [ ] 所有選項提到的程式碼片段 / 函式呼叫 / 變數，是否都出現在訊息裡的 code block？
- [ ] 如果選項在問「哪一行」「哪個呼叫」，那些行 / 呼叫是否確實在 code block 裡？
- [ ] 程式碼有沒有完整到能直接編譯 / 推理出結果？（必要的 use、main 函式、函式定義是否都齊）

**任何一項打不了勾就回去補完題目，不要先丟選項。**

寧可訊息冗長、寧可重貼，也不要讓使用者看著殘缺的題目去猜選項。出題時把自己當成第一次看到題目的使用者，反問「我有沒有足夠資訊回答？」

#### ⛔ 絕對禁止：選項裡沒有正確答案

**輸出選項前，必須先在心裡（或暫存紙上）解一次題，確認正確答案確實對應到某個選項。**

常見出包情境：

- 選項描述跟實際程式碼行為不一致（例：選項說「(c)(d) 不能編譯」，但其實只有 (c) 不能）
- 出於對某個 closure / 型別行為的誤判，把錯誤答案標成「正確選項」
- 4 個選項都不完全正確，使用者被迫選「最接近的」

**自檢清單**：

- [ ] 我能不能用一句話講出正確答案？這句話是否**精確命中**某個選項（不是「比較像 2」這種模糊）？
- [ ] 對於程式碼題，我有沒有把每行 closure 的 trait / 借用方式真的推導一次，而不是憑印象？
- [ ] 干擾選項裡的「常見誤解」是真的會被誤判的情況，還是我自己也搞混了？

正解和選項對不起來，就**改選項描述、不是改正解**。寧可這題出得簡單，也不要出錯題誤導使用者。

Rust 章節常出的題型重點：

- **所有權 / 借用**：哪段 code 編譯不過？為什麼？
- **trait**：dyn vs impl、orphan rule、blanket impl、trait object 安全性
- **lifetime**：'a 該標哪裡、elision 規則
- **async**：`.await` 在 hold lock 時的 deadlock 風險、Send bound
- **錯誤**：`?` 對 Result/Option、`thiserror` vs `anyhow` 使用時機
- **效能 / 安全陷阱**：`unwrap` / `panic` 的場合、`unsafe` 何時才合理

#### 純概念題範例

```
【第 1 題 / 共 5 題】以下關於 Rust 智慧指標的描述，哪個正確？

1. Rc<T> 是執行緒安全的引用計數
2. RefCell<T> 把借用檢查移到 runtime
3. Box<T> 一定要 unsafe 才能解引用
4. Arc<RefCell<T>> 是跨 thread 共享可變狀態的標準解

請直接回答 1、2、3 或 4。
```

#### 短程式碼題範例（1–3 行）

```
【第 2 題 / 共 5 題】以下程式碼執行後，哪個敘述正確？
```

````
```rust
let x = 5;
let y = x;
println!("{x} {y}");
```
````

```
1. 編譯成功，因為 i32 是 Copy
2. 編譯失敗，因為 x 已 move 到 y
3. 執行時 panic
4. 必須改成 x.clone() 才能通過

請直接回答 1、2、3 或 4。
```

#### 長程式碼題範例（4 行以上 / 含函式體 / borrow / scope / 多段比較）

先輸出題目敘述 + 帶語言標註的 fenced code block，再列數字選項：

````
【第 3 題 / 共 5 題】以下 Rust 程式碼的編譯結果為何？

```rust
fn main() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    let r3 = &mut s;
    println!("{}, {}, {}", r1, r2, r3);
}
```

選項：

1. 編譯成功，輸出 "hello, hello, hello"
2. 編譯失敗：不能在有 immutable 借用存活時建立 mutable 借用
3. 編譯失敗：String 不能借用
4. runtime panic：borrow rules violation

請直接回答 1、2、3 或 4。
````

#### 多段比對題範例（選項本身是不同程式碼，需要左右對照）

當題目是「下列哪個寫法才是 idiomatic Rust」這類**選項之間需要視覺化比較**的場合，把每段程式碼分別放進各自的 code block：

````
【第 4 題 / 共 5 題】下列哪個寫法符合 Rust 慣例（idiomatic）地把 Vec<i32> 轉成總和？

1. 用 for 迴圈累加
```rust
let mut sum = 0;
for n in &v {
    sum += n;
}
sum
```

2. 用 iter().sum()
```rust
v.iter().sum::<i32>()
```

3. 用 fold
```rust
v.iter().fold(0, |acc, x| acc + x)
```

4. 用 reduce
```rust
v.iter().copied().reduce(|a, b| a + b).unwrap_or(0)
```

請直接回答 1、2、3 或 4。
````

### 第五步：判斷答案與回饋

使用者回覆數字後判分。

**答對**：

- 讚美 + 1–2 句說明為何正確
- 學習模式：立即繼續下一題；考試模式：記錄，全部結束後統整

**答錯（學習模式）**：

- 說明答錯，給出正確答案與詳細解釋（含程式碼範例）
- Rust 特別著重「為什麼編譯器這樣判斷」、「ownership 規則的直觀理解」
- 出一道等效題（同概念換角度）確認理解，一樣用數字選項格式：

  ```
  確認理解：{同概念換角度的題目}

  1. ...
  2. ...
  3. ...
  4. ...

  請直接回答 1、2、3 或 4。
  ```

- 再次答錯時：再解釋一次後直接繼續下一題（避免無限迴圈）

**答錯（考試模式）**：記錄錯誤，不立即說明，全部結束後統整講解。

### 第六步：章節總結

```
本章測驗完成！
得分：{X}/{Y}（學習模式以第一次答對計）

{答錯概念列表（如有）}

接下來要做什麼？

1. 繼續選其他章節：回到章節選擇
2. 重測本章：換一批題目再測一次
3. 建議下一步：根據答錯點推薦下個該複習的章節
4. 結束：離開測驗

請直接回答 1、2、3 或 4。
```

考試模式在此處逐一講解錯題後再顯示上述選項。

「建議下一步」：根據答錯點映射到相依章節，例如：

- 答錯 13 lifetime → 建議重溫 05、06
- 答錯 32 axum extractor → 建議 12 trait、26 async-await
- 答錯 28 Send/Sync → 建議 14 smart-pointers、26 async-await

## 題目設計原則

- 優先測該章節**最核心概念**，非邊緣案例
- 4 個選項的干擾選項要選**常見誤解**或**從其他語言帶來的錯誤直覺**（Rust 特別容易被 C++/Java/Go 背景的人誤判）
- 程式碼題目貼近 `tutorials/` 實際程式碼風格
- **程式碼題比重**：5 題中至少 2–3 題應為程式碼閱讀／比對題（Rust 章節的重點就是「會不會看 ownership/lifetime/編譯器訊息」，純概念題撐不起測驗）。純概念章節（如 03 變數、22 cargo）可降至 1 題
- **程式碼呈現規則**：題目程式碼一律以帶語言標註的 markdown code block 輸出（`rust`/`toml`/`bash`/`text`）。詳見第四步的範例
- 對 async / unsafe / lifetime 這三大難點要特別出陷阱題
- 跨章節題（例如「ch29 channel + ch28 Send」結合）可以在進階題使用

## 不要做的事

- **絕對不要**在題目程式碼 / 情境不完整的情況下就丟選項（詳見第四步的「絕對禁止」段落與自檢清單）
- **絕對不要**使用 `AskUserQuestion`；所有互動一律用一般訊息 + 數字作答
- 不要出「定義型」死背題（例：「Rust 是不是系統語言？」）
- 不要出**沒有絕對對錯**的設計風格題（除非該章 README 明確表態）
- 不要把所有 4 個選項都做成「看起來都對」——一定要有 2 個明顯錯誤、1 個近似誘餌
- 不要在 Capstone 出語法題——那層應該考架構決策與權衡
