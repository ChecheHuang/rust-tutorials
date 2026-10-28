# 故事版：海邊旅館的櫃台 🏨

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「海邊小旅館開門營業，客人來櫃台辦各種事」的故事，幫你「感覺」出 axum 的 `Router`、handler、extractor（`Path` / `Query` / `Json` / `State`）、`IntoResponse` 與狀態碼、共享狀態 `AppState`、自訂錯誤，以及 middleware layer 和 `axum::serve`。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 大廳的指示牌

海邊有一間小旅館，經理叫**莉莉**。客人一走進大廳，就會看到一塊大大的**指示牌**：

- 「**/**」→ 櫃台打招呼
- 「**/users**」→ 住客登記處：**看名單**往左、**新登記**往右
- 「**/users/房號**」→ 某一間房：**查詢**往左、**退房**往右

同一個地方，客人想做的事不一樣，就走到不同的服務員面前。

> 這就是 **`Router`**：`Router::new().route("/", get(root)).route("/users", get(list_users).post(create_user)).route("/users/:id", get(get_user).delete(delete_user))`。路徑 + HTTP 方法（GET / POST / DELETE）決定由哪個 handler 處理。`:id` 是路徑參數。

---

## 每個服務員只做一件事

指示牌後面站著好幾位**服務員**，每個人只負責一件事：

- 打招呼的服務員只會說一句：「**Hello, axum!**」
- 看名單的服務員把住客名單拿給你。
- 新登記的服務員幫你寫進名冊。

服務員都很專心，他們不用管客人是從哪個門進來的，只管把自己那件事做好。

> 這就是 **handler**：一個 `async fn`，回傳某個 `impl IntoResponse`。最簡單的 `async fn root() -> &'static str { "Hello, axum!" }` 就是一個 handler。handler 是普通函式，沒有 macro 魔法。

---

## 服務員一看就知道要什麼

莉莉的服務員有個特殊本領：他們會在**名牌**上寫好「我需要什麼」，旅館就自動把東西準備好交給他。

- 查詢房間的服務員名牌寫著：「**我需要房號**」→ 旅館從客人說的「/users/1」抓出 **1** 給他。
- 看名單的服務員寫著：「**我需要附註**」→ 客人說「/users?limit=2」，旅館就把「最多 2 位」交給他。
- 新登記的服務員寫著：「**我需要填好的登記表**」→ 旅館把客人帶來的表格交給他。

服務員**不用自己去翻客人的包包**，只要名牌寫對，東西就到手。

> 這就是 **extractor**：handler 的參數型別決定要從 request 抽出什麼。`Path(id): Path<u64>` 取路徑參數；`Query(p): Query<ListParams>` 取 query string（`limit: Option<usize>`）；`Json(payload): Json<CreateUser>` 把 body 反序列化成 `CreateUser`。參數順序大多自由，axum 看型別決定。

---

## 登記表只有一份，要最後才拿

有一天，新來的服務員把名牌寫成：「我**先**需要登記表，**再**需要名冊。」結果旅館根本不讓他上班。

莉莉解釋：「客人的**登記表只有一份**，拿走了就沒了。所以要把它放在**最後**一個拿，其他東西都準備好了才拆開表格。」

> 這就是 **吃 body 的 extractor 只能有一個，而且要放最後**：`create_user(State(s): State<AppState>, Json(payload): Json<CreateUser>)` 是對的。把 `Json` 放在 `State` 前面會編譯失敗，錯誤訊息通常又長又難讀。

---

## 大家共用的住客名冊

櫃台後面有一本**住客名冊**，每位服務員都要用。莉莉沒有給每個人一本副本，而是給每人一張**「名冊在哪裡」的卡片**；卡片可以隨便複印，指的都是同一本。

名冊旁邊還有一把**小鎖**：同一時間只能一個人翻開來寫，免得兩個人同時寫在同一行。

旅館剛開幕時，名冊裡已經有一位客人：**1 號 alice**。

> 這就是 **`State` 與 `AppState`**：`#[derive(Clone)] struct AppState { users: Arc<Mutex<HashMap<u64, User>>> }`，用 `.with_state(state)` 交給 router，handler 用 `State(s): State<AppState>` 拿。`AppState` 必須 `Clone`，所以重的東西用 `Arc` 包（複印卡片，不複印名冊）；`Mutex` 是那把小鎖，`s.users.lock().unwrap()` 才能讀寫。

---

## 回覆客人的各種方式

服務員回覆客人，有好幾種方式，旅館都會幫忙包裝好：

- 說一句話：「Hello, axum!」→ 普通的口頭回覆。
- 遞一份**整理好的名單**→ 用標準格式印出來。
- 新登記完成：蓋上「**201 已建立**」章，再附上新住客的資料。
- 查不到房號：只回一張「**404 查無此房**」的卡片。
- 退房成功：只蓋一個「**204 已完成，沒有其他內容**」的章。

> 這就是 **`IntoResponse`**：`&'static str` 變成 200 + 純文字；`Json<Vec<User>>` 變成 200 + JSON；`(StatusCode::CREATED, Json(user))` 自訂狀態碼；`get_user` 回 `Result<Json<User>, StatusCode>`，找不到用 `.ok_or(StatusCode::NOT_FOUND)`；`delete_user` 直接回 `StatusCode::NO_CONTENT` 或 `NOT_FOUND`。

---

## 旅館自己的道歉卡

莉莉覺得每次都只回「404」太冷淡，於是設計了一套**道歉卡**：

- 「找不到」→ 404 卡，上面寫 `not found`。
- 「表格填錯」→ 400 卡。
- 「我們這邊出問題」→ 500 卡。

服務員只要說「這是找不到的情況」，旅館就自動遞出對應的卡片。

> 這就是 **自訂錯誤 → `IntoResponse`**：定義 `enum AppError { NotFound, BadInput(String), Internal }`，替它 `impl IntoResponse`，在 `into_response` 裡轉成 `(StatusCode, Json(...))`。handler 回 `Result<T, AppError>`，錯誤就會自動變成漂亮的 HTTP 回應。

---

## 門口的警衛和登記簿

旅館大門有一位**警衛**，每個進出的客人他都會記在**登記簿**上：幾點進來、要去哪、多久離開。

莉莉還可以再加一層：門口的**接待員**，負責回答「外地人可以進來嗎？」

這些人都站在**服務員外面**一層一層包著：客人進來先經過最外層，回覆出去時再反過來經過他們。

> 這就是 **middleware（tower `Layer`）**：`.layer(TraceLayer::new_for_http()).layer(CorsLayer::permissive())`。request 從外層流進去、response 從內層流出來，**最後加的 layer 在最外層**。本章程式用 `tracing_subscriber::fmt::init()` 和 `info!(id, "user created")` 記錄日誌。

---

## 開門營業

一切準備好，莉莉走到門口，**在 3000 號門牌掛上「營業中」**，然後一直站在那裡迎接客人，直到打烊。

客人陸續上門：

- 「/」→ Hello, axum!
- 「/users」→ 看到 alice。
- 帶著登記表來新增 bob → 拿到「201 已建立」和 2 號房。
- 退掉 2 號房 → 拿到「204」。

> 這就是 **`TcpListener` + `axum::serve`**：`tokio::net::TcpListener::bind("0.0.0.0:3000").await` 掛上門牌，`axum::serve(listener, app).await` 開始營業。axum 建立在 **hyper + tokio + tower** 上，所以上一章自己開櫃台的苦工都不用做了。用 `curl localhost:3000/users` 就能試。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 海邊小旅館 | **axum 應用程式** |
| 大廳指示牌 | **`Router`** + `.route(...)` |
| 往左查詢、往右退房 | **HTTP method**（`get`、`post`、`delete`） |
| 「/users/房號」 | 路徑參數 `/users/:id` |
| 只做一件事的服務員 | **handler**（`async fn`） |
| 名牌寫著「我需要什麼」 | **extractor** |
| 我需要房號 | **`Path<u64>`** |
| 我需要附註 | **`Query<ListParams>`** |
| 我需要填好的登記表 | **`Json<CreateUser>`** |
| 登記表只有一份，最後拿 | body extractor **只能一個、放最後** |
| 「名冊在哪裡」的卡片 | **`State<AppState>`**（`Clone` + `Arc`） |
| 名冊旁的小鎖 | **`Mutex`** |
| 各種回覆方式 | **`IntoResponse`** |
| 201 / 404 / 204 的章 | `StatusCode::CREATED` / `NOT_FOUND` / `NO_CONTENT` |
| 旅館的道歉卡 | 自訂 **`AppError` + `impl IntoResponse`** |
| 門口警衛和登記簿 | **middleware**（`TraceLayer`） |
| 一層一層包著服務員 | tower **`Layer`** 的洋蔥順序 |
| 在 3000 號門牌掛上營業中 | `TcpListener::bind` + **`axum::serve`** |

---

## 一句話記住

> **axum 就是一間有指示牌（`Router`）的旅館：每位服務員（handler）只要在名牌上寫好需要什麼（`Path`、`Query`、`State`、最後才拿 `Json`），旅館就自動準備好；回覆用各種章（`IntoResponse` + `StatusCode`），大家共用的名冊靠卡片和小鎖（`Arc<Mutex<...>>`）。**
