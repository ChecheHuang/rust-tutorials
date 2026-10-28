# 故事版：小鎮郵局的寄信與回信 ✉️

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「寫信去問問題、等對方回信」的故事，幫你「感覺」出 HTTP 的層次（reqwest / hyper / tokio）、重用 `Client`、GET 與狀態碼、headers、JSON 反序列化、`error_for_status`、POST JSON、自訂 header 和 query、同時寄好幾封信、hyper server，以及 HTTP/1、HTTP/2 和串流下載。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 一封信要經過好幾個人

小鎮上的**阿明**很喜歡寫信問問題。他寫好一封信之後，信會這樣旅行：

1. 阿明把問題講給**代書阿姨**聽。
2. 代書阿姨幫他寫成格式正確的信，交給**郵局**。
3. 郵局決定用什麼方式裝袋、編號。
4. 郵局交給**貨車**，貨車在**公路**上跑。

阿明只要跟代書阿姨講話就好，其他的都不用管。

> 這就是 **HTTP 的層次**：你的程式 → **`reqwest`**（高階 client，代書阿姨）→ **`hyper`**（底層 HTTP，郵局）→ `h2` / `h3`（協定實作）→ **`tokio`**（runtime + TCP，公路）。寫服務通常不直接碰 hyper，但整個 Rust web 生態都建在它上面。

---

## 請一位固定的郵差

阿明第一次寄信時，每封信都**重新找一位郵差**。每位新郵差都要先問路、認識門牌，送得很慢。

後來他**固定請老陳**當郵差。老陳記得每條路、跟收件人很熟，同一條路上的信還能**一趟一起送**。

阿明還跟老陳約好兩件事：信封上要寫**寄件人名字**「rust-tutorials/0.1」；如果**十天**都沒收到回信，就別等了。

> 這就是 **重用 `reqwest::Client`**：`reqwest::Client::builder().user_agent("rust-tutorials/0.1").timeout(Duration::from_secs(10)).build()?`。重用同一個 client 才有 **connection pool**、keep-alive、HTTP/2 multiplexing。每次都用 `reqwest::get()` 建新 client，會慢 10 倍以上。

---

## 寄出一封「請告訴我」的信

阿明寫了第一封信給 httpbin 先生：「請告訴我，你收到了什麼？」信封上加註「**source=rust**」。

回信來了。信封外面先蓋著一個**大印章**：「**200 — 一切順利**」。信封背面還有一堆小標籤：寄件日期、信紙種類、信有多長……

阿明拆開信，裡面是一大篇文字。他先讀前 200 個字就好。

> 這就是 **GET、status、headers、body**：`client.get("https://httpbin.org/get?source=rust").send().await?` 送出請求，`resp.status()` 是大印章，`resp.headers()` 是那些小標籤，`resp.text().await?` 才把信紙整篇讀出來。注意：讀 body 也要 `.await`，因為信紙可能還在路上。

---

## 照著格子填的回信

第二封信，阿明寄給 jsonplaceholder 小姐，問「第 1 號文章是什麼？」

她的回信是一張**填好格子的表格**：「作者編號」、「文章編號」、「標題」、「內容」。阿明家裡也有一張**一模一樣格子的空白表**，他把回信的內容一格一格抄過去，就整理好了。

只是對方表格上寫的是「userId」，阿明的表寫的是「user_id」。他在自己的表上貼一張小紙條：「**對方寫 userId 的那格，抄到這裡**。」

> 這就是 **JSON 反序列化**：`struct Post` 加上 `#[derive(Deserialize)]`，`.json().await?` 就把回應的 JSON 填進 struct。欄位名字不同時，用 `#[serde(rename = "userId")]` 對上 `user_id`。用不到的欄位 `body` 加了 `#[allow(dead_code)]`。

---

## 送到了，不代表成功

有一次阿明寄信問一個不存在的東西。回信**準時送到**，可是大印章上寫著「**404 — 查無此人**」。

阿明沒看印章就直接拆信，以為裡面是他要的答案，結果整理得一團亂。

從此他學會：**拆信之前先看印章**。印章是 400 以上的，就直接當作「失敗」處理。

> 這就是 **`error_for_status()`**：`send()` 收到 4xx / 5xx 時**仍然是 `Ok`**，因為信確實送到了。程式裡 `.send().await?.error_for_status()?.json().await?`，在解析前先把 status ≥ 400 轉成 `Err`。`reqwest::Error` 還有 `is_timeout()`、`is_connect()`、`is_decode()` 可以分辨錯誤種類。

---

## 寄出一張自己填的表格

這次換阿明**主動寄一份表格**過去：「我想發表一篇文章，標題『rust rocks』，內容『hello from rust』，作者 1 號。」

代書阿姨把阿明填好的表格**照格式謄寫**成對方看得懂的樣子再寄出。對方回信：「**201 — 已建立**」，並把收到的內容抄一份寄回來給他確認。

> 這就是 **POST JSON**：`struct NewPost` 加上 `#[derive(Serialize)]`，`client.post(url).json(&new).send().await?` 把 struct 轉成 JSON 當 body。回應可以用 `serde_json::Value` 接，不用事先定義 struct。如果是表單格式則用 `.form(&[...])`。

---

## 信封上的特別註記

有些收件人需要信封上寫特別的東西：

- 一張「**X-Custom: foo**」的貼紙。
- 一張「**通行證：Bearer dummy-token**」，證明阿明有資格問。
- 信封地址後面加註「**q=rust、limit=10**」，告訴對方要找什麼、最多給幾個。

> 這就是 **自訂 header 與 query**：`.header("X-Custom", "foo")`、`.header("Authorization", "Bearer dummy-token")` 加在 request headers；`.query(&[("q", "rust"), ("limit", "10")])` 會自動接到 URL 的 `?q=rust&limit=10`。

---

## 一次寄五封

阿明想知道第 1 到第 5 號文章。他沒有寄一封、等回信、再寄下一封，而是**五封一起寄出去**，然後等五封都回來。

有一封如果寄丟了，其他四封還是照樣收得到，他一封一封檢查：成功的印出標題，失敗的印出原因。

> 這就是 **並發請求**：`ids.iter().map(|&id| async move { ... })` 產生五個 Future，`futures::future::join_all(futs).await` 一起等。每個結果都是 `Result<Post, reqwest::Error>`，用 `match` 分開處理 `Ok(p)` 和 `Err(e)`。因為共用同一個 `client`，連線也會被重用。

---

## 自己開一間郵局櫃台

阿明長大後，想**自己開郵局櫃台**收別人的信。

他得自己做所有事：打開大門、**一直等有人來**、每來一個人就**派一位櫃員專門招呼他**，櫃員再照規則讀信、回信。

做了一陣子他就懂了：「難怪大家都去找**已經裝潢好的大郵局**。」

> 這就是 **用 hyper 寫 server**：`TcpListener::bind("0.0.0.0:3000")`，`loop { listener.accept().await? }` 接連線，每條連線 `tokio::spawn` 一個 task，用 `http1::Builder::new().serve_connection(io, service_fn(hello))` 處理。寫多了就會懂為什麼有 axum（下一章）。

---

## 一台車一封信 vs 一台車載很多封；大包裹分批送

舊的送信方式是**一台車一次只載一封**，下一封得等這台車回來。新的方式是**一台車同時載很多封**，各自編號，送到再分開。還有更新的方式，走的是**另一種路**，路況不好也不容易掉信。

如果收到的是一個**超大包裹**，阿明不會等整個包裹堆滿客廳才處理，而是**送來一箱就先搬進倉庫一箱**。

> 這就是 **HTTP/1.1 vs HTTP/2 vs HTTP/3 與串流回應**：HTTP/2 用 multiplexing 在一條連線上跑很多請求；HTTP/3 走 QUIC（UDP）。reqwest 會自動協商。下載大檔用 `while let Some(chunk) = resp.chunk().await?` 一塊一塊寫進檔案，不會把記憶體撐爆。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 代書阿姨 | **`reqwest`**（高階 HTTP client） |
| 郵局 | **`hyper`**（底層 HTTP） |
| 公路和貨車 | **`tokio`** runtime + TCP |
| 固定的郵差老陳 | 重用 **`reqwest::Client`**（connection pool） |
| 信封上的寄件人 / 十天沒回就算了 | `.user_agent(...)` / `.timeout(...)` |
| 寄出「請告訴我」的信 | **GET**：`client.get(url).send().await?` |
| 信封外的大印章 | **status code**（`resp.status()`） |
| 信封背面的小標籤 | **headers** |
| 拆開讀信紙 | `resp.text().await?` |
| 照格子抄的表格 | **`#[derive(Deserialize)]`** + `.json()` |
| 「userId 抄到這格」 | `#[serde(rename = "userId")]` |
| 拆信前先看印章 | **`error_for_status()`** |
| 自己填好寄出的表格 | **POST**：`.json(&new)` + `Serialize` |
| 信封上的貼紙和通行證 | `.header(...)` |
| 地址後的加註 | `.query(&[...])` |
| 五封一起寄 | **`futures::future::join_all`** |
| 自己開郵局櫃台 | **hyper server**（`serve_connection` + `service_fn`） |
| 一台車載很多封 | **HTTP/2 multiplexing** |
| 大包裹一箱一箱搬 | **串流回應** `resp.chunk()` |

---

## 一句話記住

> **寫 HTTP 就像寄信問問題：請一位固定的郵差（重用 `Client`），回信送到了先看印章（`error_for_status`），再照格子抄進自己的表（`Deserialize`）；想問很多問題就一起寄（`join_all`），想自己開櫃台才需要碰郵局本身（`hyper`）。**
