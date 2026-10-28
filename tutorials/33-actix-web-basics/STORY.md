# 故事版：巷口早餐店的點餐櫃台 🥪

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「巷口早餐店開門營業、客人來點餐」的故事，幫你「感覺」出 `HttpServer`、`App`、用 `#[get]` / `#[post]` 註冊路由、`Path` / `Query` / `Json` 這些 extractor、共享狀態 `Data`、`Responder`、自訂錯誤 `ResponseError`、`wrap` 中介層是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 早餐店要開門了

巷口有一家**阿芳早餐店**。每天早上六點，阿芳把鐵門拉起來，在門口掛上「**營業中**」的牌子，告訴整條街：「我在 3000 號門牌，可以來點餐囉！」

店裡有好幾個**櫃台**，每個櫃台站著一位店員，客人一多就分散到不同櫃台，大家一起忙。

> 這就是 **`HttpServer`**：`HttpServer::new(...).bind(("0.0.0.0", 3000))?.run().await` 就是「拉鐵門、掛牌子、開始營業」。actix-web 預設會開**和 CPU 核心數一樣多的 worker**，就像好幾個櫃台同時接客。整個程式由 `#[actix_web::main]` 啟動，它用的是 actix 自己的 runtime（底層是 tokio），不是 `#[tokio::main]`。

---

## 每個櫃台都照同一本手冊佈置

阿芳寫了一本**佈置手冊**：「櫃台上要擺菜單、要貼點餐規則、要放收銀機。」每開一個新櫃台，店員就**照手冊從頭佈置一次**。

> 這就是 **`App::new()` 和傳給 `HttpServer::new` 的 closure**：`HttpServer::new(move || App::new()...)` 裡的那個 closure 就是佈置手冊，**每個 worker 都會各自執行一次**，佈置出自己的 `App`。

---

## 菜單上的小牌子

櫃台上的菜單每一行都掛著一個**小牌子**：

- 「**看全部會員**」→ 找店員小明
- 「**看某一位會員**」→ 找店員小華
- 「**辦新會員卡**」→ 找店員小美
- 「**退會員卡**」→ 找店員阿德

牌子是**直接掛在店員胸前**的。阿芳只要把這些店員「排上班」，客人看到牌子就知道要找誰。

> 這就是 **attribute macro 註冊路由**：`#[get("/users")] async fn list_users`、`#[get("/users/{id}")] async fn get_user`、`#[post("/users")] async fn create_user`、`#[delete("/users/{id}")] async fn delete_user`。牌子掛在函式身上，再用 `App::new().service(list_users)` 把它「排上班」。這是 actix 和 axum 最大的差別：axum 是在 `Router::new().route(...)` 那邊統一寫，actix 是牌子掛在每個 handler 身上。想把一群店員放在同一區，可以用 `web::scope("/api/v1")`。

---

## 客人遞來的三種紙條

客人點餐時，會用不同方式告訴店員要什麼：

1. 指著菜單上的**號碼**說：「我要看 **1 號**會員。」
2. 順口補一句：「**最多給我看 2 個**就好。」
3. 遞上一張**填好的申請表**，上面寫著名字和信箱。

店員不用自己去猜，櫃台旁邊有個**小幫手**，會先把號碼、補充說明、申請表都整理好，直接放在店員手上。

> 這就是 **extractor**：`Path<u64>` 從網址 `/users/1` 抓出 `1`；`Query<ListParams>` 從 `?limit=2` 抓出 `limit`；`Json<CreateUser>` 把請求 body 的 JSON 變成 struct。小幫手整理好才交給 handler。拿裡面的值可以用 `id.into_inner()`，或直接像 `payload.name`、`q.limit` 這樣用（deref）。如果申請表太大（`Json` 預設上限約 256K），小幫手會直接擋下來，要放寬得設定 `JsonConfig`。

---

## 大家共用一本會員簿

全店只有**一本會員簿**，放在店中央。每個櫃台都放了一張「**會員簿在中央桌上**」的指路卡。

有一次新來的店員自作主張，**在自己的櫃台各買了一本新的空白會員簿**。結果 A 櫃台辦的會員，B 櫃台查不到——因為大家寫在不同本子上！

阿芳說：「會員簿要在**開店前**就準備好**一本**，每個櫃台拿的只是**指路卡的影印本**。」

還有一條規則：同一時間只能**一個人翻**會員簿，要寫字的人先把簿子**拿起來**，寫完再放回去。

> 這就是 **共享狀態 `Data<AppState>`**：`Data::new(AppState { ... })` 必須在 `HttpServer::new` 的 closure **外面**建好，closure 裡用 `.app_data(state.clone())` 放進去。`Data<T>` 裡面包的就是 `Arc<T>`，clone 的只是指路卡，大家指向同一本簿子。如果在 closure **裡面** `Data::new`，每個 worker 就各有一本，資料不會共享。會員簿本身是 `Mutex<HashMap<u64, User>>`，`s.users.lock().unwrap()` 就是「把簿子拿起來」。

---

## 端給客人的東西可以長得不一樣

店員端出去的東西五花八門：

- 有時只是一句「**早安，歡迎光臨！**」
- 有時是一張**會員清單**。
- 辦好新卡時，會說「**辦好了！**」再附上新卡。
- 退卡成功時，只**點個頭**，什麼也不遞。

不管是什麼，只要是「**可以端出去給客人的東西**」就行。

> 這就是 **`Responder`**：`root()` 回傳字串 `"Hello, actix-web!"`、`list_users` 回傳 `Json(all)`，`create_user` 回傳 `HttpResponse::Created().json(user)`（201），`delete_user` 回傳 `HttpResponse::NoContent().finish()`（204）。它們的回傳型別都寫 `impl Responder`，意思是「任何能變成 HTTP 回應的東西」，對應 axum 的 `impl IntoResponse`。

---

## 「查無此人」的標準卡片

有客人說：「我要看 **99 號**會員。」小華翻遍會員簿都找不到。

阿芳早就印好了一張**標準卡片**，上面寫著「**查無此人**」，背面印著「代號 404」。小華只要把卡片遞出去就好，不用每次自己想怎麼說。

阿芳還提醒：「卡片上只寫『查無此人』，**別把會員簿裡亂七八糟的內部筆記抄給客人看**。」

> 這就是 **自訂錯誤 `ResponseError`**：`AppError::NotFound` 實作了 `ResponseError`，`status_code()` 回 `StatusCode::NOT_FOUND`，`error_response()` 回 `{"error": "not found"}`。`get_user` 回傳 `Result<Json<User>, AppError>`，`.ok_or(AppError::NotFound)` 一失敗，actix 就自動把它變成 404 回應。小心：預設會用 `Display` 的文字，別把 SQL 錯誤之類的內部訊息洩漏給 client。

---

## 門口的登記簿

早餐店門口還坐著**記錄員小芬**。每位客人進門，她就寫下「幾點幾分、點了什麼」；客人離開時，再寫「花了多久、結果如何」。店員完全不用管這件事。

如果門口坐了好幾個人，**最後一個被安排坐下的，會坐在最外面**，客人進門時最先碰到他。

> 這就是 **`App::wrap(middleware)`**：範例用 `.wrap(tracing_actix_web::TracingLogger::default())` 幫每個請求記 log。多個 `wrap` 時，**最後 wrap 的最先看到請求**。想自己寫 middleware 要實作 `Transform` + `Service`，比較麻煩，第 37 章會跟 tower 比較。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 拉鐵門、掛營業牌 | `HttpServer::new(...).bind(...).run().await` |
| 好幾個櫃台 | 多個 **worker**（預設 = CPU 核心數） |
| 開店的人 | `#[actix_web::main]`（actix runtime） |
| 每個櫃台照手冊佈置 | `HttpServer::new` 的 closure，**每個 worker 跑一次** |
| 佈置好的櫃台 | `App::new()` |
| 掛在店員胸前的小牌子 | `#[get("/users")]`、`#[post(...)]`、`#[delete(...)]` |
| 把店員排上班 | `.service(list_users)` |
| 同一區的店員 | `web::scope("/api/v1")` |
| 菜單號碼 | `Path<u64>` |
| 順口補充「最多 2 個」 | `Query<ListParams>` |
| 填好的申請表 | `Json<CreateUser>` |
| 申請表太大被擋 | `JsonConfig` 的 body 上限 |
| 全店唯一的會員簿 | `Data<AppState>`（內含 `Arc`） |
| 指路卡影印本 | `state.clone()` + `.app_data(...)` |
| 各櫃台自己買新簿子 | 在 closure **內** `Data::new` → 狀態沒共享 |
| 一次只能一個人翻 | `Mutex` + `.lock()` |
| 端給客人的任何東西 | `impl Responder` |
| 「辦好了！」/ 點個頭 | `HttpResponse::Created()` / `NoContent()` |
| 「查無此人」標準卡片 | `AppError::NotFound` + `ResponseError` → 404 |
| 門口的記錄員 | `.wrap(TracingLogger::default())` |

---

## 一句話記住

> **actix-web 就是一家早餐店：`HttpServer` 開門、每個櫃台照 closure 手冊佈置 `App`、牌子 `#[get]` 掛在店員身上；會員簿 `Data` 要在開店前準備好一本，大家拿的只是指路卡。**
