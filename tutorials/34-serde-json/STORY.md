# 故事版：小鎮郵局的打包與拆包 📦

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「小鎮郵局幫大家打包、寄送、拆包」的故事，幫你「感覺」出 `Serialize` / `Deserialize`、`#[serde(default)]`、`skip_serializing_if`、`rename_all`、enum 的 `tag`、`flatten`、`serde_with` 的 `DisplayFromStr`、`validator`、動態 `Value` 是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 打包員和拆包員

小鎮郵局裡有兩位老員工：**打包員阿伯**和**拆包員阿姨**。

小芸想把她的「會員卡」寄給遠方的朋友。可是會員卡上有照片、有號碼、有名字，**不能直接塞進郵筒**。阿伯會把每一項東西整理好，**一格一格寫進標準的寄件單**，裝進紙箱寄出去。

遠方的郵局收到箱子，阿姨照著寄件單，**把東西一樣一樣擺回原本的樣子**，朋友就拿到一張一模一樣的會員卡。

> 這就是 **`Serialize` / `Deserialize`**：`#[derive(Serialize, Deserialize)] struct User` 讓 `User` 會「被打包」和「被拆包」。`serde_json::to_string_pretty(&u)` 是阿伯打包成 JSON；`serde_json::from_str::<User>(raw)` 是阿姨拆包回 struct。

---

## 同一套整理方法，可以換不同的箱子

阿伯整理東西的方法是**固定的**：先列出有哪些格子、每格放什麼。至於最後要裝**紙箱、木箱還是塑膠袋**，交給不同的包裝機就好。

> 這就是 **serde 的模型**：`Serialize` / `Deserialize` **跟格式無關**。同一個 struct 可以輸出成 JSON、TOML、YAML、bincode……只要換不同的 serializer（包裝機），struct 本身不用改。這一章用的包裝機是 `serde_json`。

---

## 寄件單上沒寫的格子

有一天來了一個箱子，寄件單上只寫了「號碼 2、名字 bob、信箱 b@x.com」，**「是否啟用」那一格空著沒寫**。

阿姨不慌不忙：「沒寫的話，就**照預設值填**——是否啟用：**否**。」

反過來，阿伯打包時發現小芸的會員卡**沒有照片**。他不會在寄件單上寫「照片：無」，而是**乾脆不印那一格**，單子比較乾淨。

> 這就是 **`#[serde(default)]` 和 `skip_serializing_if`**：`active` 欄位加了 `#[serde(default)]`，JSON 缺這欄就給 `bool` 的預設值 `false`。`avatar` 加了 `#[serde(skip_serializing_if = "Option::is_none")]`，是 `None` 的時候序列化就**整個不輸出**。

---

## 外國郵局的寫字規矩

有些箱子要寄到國外。那邊的郵局規定：格子名稱**不能有底線**，要寫成「第一個字小寫、後面每個字開頭大寫」，例如 `requestId`、`isSuccess`。

阿伯不想把自己的筆記全部改寫，就在箱子上貼一張「**整箱照外國規矩寫**」的貼紙，打包時自動轉換。

> 這就是 **`#[serde(rename_all = "camelCase")]`**：`ApiResp` 的 Rust 欄位是 `request_id`、`is_success`，輸出 JSON 會變成 `requestId`、`isSuccess`。只改單一欄位名用 `#[serde(rename = "foo")]`。

---

## 箱子外面的分類標籤

郵局每天會收到好幾種**通知信**：「有人進門了」「有人離開了（附上原因）」「我還活著，打聲招呼」。

為了讓阿姨一眼就知道是哪一種，阿伯在每封信裡**第一行寫上分類**：`"type": "login"`、`"type": "logout"`、`"type": "heartbeat"`，後面才接內容。

> 這就是 **enum 的 internally tagged**：`#[serde(tag = "type", rename_all = "snake_case")] enum Event { Login{..}, Logout{..}, Heartbeat }`，輸出像 `{"type":"login","user":"alice"}`。預設（沒加 `tag`）是 externally tagged：`{"Login":{"user":"a"}}`；還有 adjacent：`#[serde(tag = "t", content = "c")]`。實務 API 最常用 internally tagged。

---

## 拆掉多餘的內盒

有一種箱子叫「**第幾頁的貨**」：外面寫「第 1 頁、每頁 10 件」，裡面還有一個**內盒**裝著貨。

收件的人抱怨：「每次都要多拆一層好麻煩！」阿伯就改成**把內盒拆掉**，貨直接跟頁碼放在同一層。

> 這就是 **`#[serde(flatten)]`**：`Page<T>` 的 `data` 欄位加了 `flatten`，裡面的欄位會被「攤平」到外層，跟 `page`、`size` 平起平坐，不會多一層巢狀。注意：`flatten` 跟 `deny_unknown_fields` 不能一起用。

---

## 用文字寫的金額

隔壁銀行寄來的箱子很奇怪：金額不是寫數字 `123.45`，而是寫成**一串文字** `"123.45"`，還加了引號。

阿姨本來只會拆「數字格子」，看到文字就卡住。後來郵局買了一台**小翻譯機**，專門把「寫成文字的數字」翻回真正的數字。

> 這就是 **`serde_with` 的 `DisplayFromStr`**：`Money` 加上 `#[serde_as]`，`amount` 欄位標 `#[serde_as(as = "DisplayFromStr")]`，就能把 `"123.45"` 字串解析成 `f64`。這招也常用在 `u64` 太大、JS 端會丟精度時，改成字串傳。

---

## 檢查員把關

有人寄來一張「**報名表**」：信箱寫「not-email」、密碼只有 5 個字、年齡填 5 歲。

阿姨拆得開，可是**檢查員**看了直搖頭：「信箱格式不對、密碼至少要 8 個字、年齡要 18 到 120 歲。」他一次把**所有問題列成清單**退回去，而不是看到第一個錯就停。

> 這就是 **`validator`**：`#[derive(Validate)] struct SignUp` 搭配 `#[validate(email)]`、`#[validate(length(min = 8))]`、`#[validate(range(min = 18, max = 120))]`，呼叫 `bad.validate()` 會回傳包含多個錯誤的結果。實務上在 handler 收到 `Json<T>` 後立刻 `.validate()?`，失敗就回 400。拆得開（反序列化成功）不代表內容合理，兩件事要分開做。

---

## 萬用收納袋

偶爾會收到**不知道裡面裝什麼**的箱子，沒有寄件單格式可以對照。

阿姨就把東西全倒進一個**萬用收納袋**，袋子上有很多小口袋，可以用「名字口袋」、「標籤口袋第 0 個」這樣一層層去翻。想找「a」這一格，翻到了就拿出來，翻不到就當作 0。

可是阿伯提醒：「**萬用袋不要天天用**。有固定格式的就照寄件單拆，才不會拿錯東西都不知道。」

> 這就是 **動態 `serde_json::Value`**：用 `json!({...})` 建立，用 `v["name"]`、`v["tags"][0]` 存取；`partial.get("a").and_then(|v| v.as_i64()).unwrap_or(0)` 是「翻得到就拿、翻不到給預設」。`Value` 會失去型別安全，建議頂層還是用 struct，只把未知欄位收進 `HashMap<String, Value>`。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 打包員阿伯 | `Serialize` / `serde_json::to_string` |
| 拆包員阿姨 | `Deserialize` / `serde_json::from_str` |
| 固定的整理方法，箱子可換 | serde 模型**與格式無關**（JSON / TOML / ...） |
| 沒寫的格子照預設填 | `#[serde(default)]` |
| 沒照片就不印那一格 | `skip_serializing_if = "Option::is_none"` |
| 「整箱照外國規矩寫」貼紙 | `#[serde(rename_all = "camelCase")]` |
| 信裡第一行的分類 | `#[serde(tag = "type")]`（internally tagged） |
| 拆掉多餘的內盒 | `#[serde(flatten)]` |
| 文字寫的金額 + 翻譯機 | `serde_with` 的 `DisplayFromStr` |
| 檢查員列出所有問題 | `validator` 的 `#[derive(Validate)]` + `.validate()` |
| 萬用收納袋 | `serde_json::Value` / `json!` |
| 一層層翻口袋 | `v["tags"][0]`、`.get("a").and_then(...)` |

---

## 一句話記住

> **serde 就是郵局的打包員和拆包員：`Serialize` 打包、`Deserialize` 拆包，屬性是貼在箱子上的規矩貼紙；拆得開還要讓 `validator` 檢查，萬用袋 `Value` 能不用就不用。**
