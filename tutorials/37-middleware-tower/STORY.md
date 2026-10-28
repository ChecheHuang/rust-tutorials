# 故事版：俄羅斯娃娃工坊的傳話遊戲 🪆

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「在一層套一層的俄羅斯娃娃裡傳紙條」的故事，幫你「感覺」出 tower 的 `Service` 和 `Layer`、`ServiceBuilder` 的順序、`middleware::from_fn`、`next.run(req)`、提早擋下（短路）、tower-http 現成的 layer、只套在部分路由的 middleware，以及 tower 和 actix middleware 的差別。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 會回話的娃娃

老吳師傅的工坊裡，做的不是普通娃娃，而是**會回話的娃娃**。

你從娃娃頭頂的小洞**塞一張紙條**進去，等一下，它就會從同一個洞**吐一張回信**出來。

學徒小柚問：「娃娃裡面在做什麼？」老吳說：「不重要。**只要是『收紙條、等一下、吐回信』的東西，都叫做會回話的娃娃**。」

小柚還發現，有禮貌的客人塞紙條之前，會先敲敲娃娃問：「**你現在有空嗎？**」

> 這就是 **tower 的 `Service` trait**：`call(&mut self, req) -> Self::Future`，收一個 request，回一個「之後會給你 `Result<Response, Error>`」的 Future。`poll_ready` 就是「你現在有空嗎？」。handler 是 Service，middleware 也是 Service——只要是 request → response 的東西都是。最裡面那顆最小的娃娃，就是我們的 handler，例如 `hello()` 回 `"Hello, middleware!"`。

---

## 套殼機

工坊裡有一台台**套殼機**。你把一顆娃娃放進去，機器就在它外面**再套一顆大娃娃**，吐出來的還是一顆「會回話的娃娃」，只是多了一層。

外層娃娃收到紙條時，可以**先看一看、加點東西**，再往裡面傳；裡面回信傳出來時，也可以**再加點東西**才吐出去。

> 這就是 **`Layer` trait**：`fn layer(&self, inner: S) -> Self::Service`，拿一個 Service，包成新的 Service。`A(B(C(handler)))` 這樣一層包一層，**最外面的 A 最先看到 request、最後碰到 response**。

---

## 照清單由外往內套

老吳給小柚一張**套殼清單**：

1. 最外層：**記錄員殼**
2. 第二層：**計時沙漏殼**
3. 第三層：**壓縮殼**
4. 第四層：**外地通行殼**

小柚問：「清單第一行是最外面還是最裡面？」老吳說：「**照清單從上往下，就是從外往內**，看了就懂，不用倒過來想。」

> 這就是 **`ServiceBuilder`**：範例的 `ServiceBuilder::new().layer(TraceLayer::new_for_http()).layer(TimeoutLayer::new(...)).layer(CompressionLayer::new()).layer(CorsLayer::new()...)`，**寫的順序就是由外到內**，比直接在 `Router` 上連續 `.layer()` 直覺。注意：在 `Router` 上連續 `.layer()` 時，**越後面加的越外層**。範例 `.layer(from_fn(timing)).layer(from_fn(request_id)).layer(middleware)`，所以 `middleware` 那一整組在最外面、`request_id` 次之、`timing` 最靠近 handler。

---

## 架上現成的殼

這四種殼都不用自己做，**工坊架上就有現成的**：

- **記錄員殼**：每張紙條經過都寫進記事本。
- **計時沙漏殼**：紙條進去就翻沙漏，**1 秒沙子漏完還沒回信，就直接吐一張「太慢了」**。
- **壓縮殼**：把回信摺得小小的，寄起來比較輕。
- **外地通行殼**：允許從別的城鎮寄來的紙條。

有一顆娃娃叫「**慢吞吞**」，每次都要想 2 秒才回信。結果沙漏殼等不及，直接吐出「太慢了」。

> 這就是 **tower-http 現成的 layer**：`TraceLayer`（tracing log）、`TimeoutLayer::new(Duration::from_secs(1))`（`/slow` 要睡 2 秒，所以回 **408**）、`CompressionLayer`（gzip）、`CorsLayer::new().allow_origin(Any).allow_methods(Any)`（CORS）。其他還有 `RequestBodyLimitLayer`、`SetResponseHeaderLayer`、`NormalizePathLayer`。注意 timeout 算的是**整個回應時間**，不是閒置時間。

---

## 自己捏一個殼

小柚想自己做殼。老吳說：「最簡單的方法，是寫一段**『我要做什麼』的步驟**，套殼機會幫你變成殼。」

小柚做了一個**計時殼**：

1. 紙條進來，按下碼錶。
2. **把紙條交給裡面那層**，等它回信。
3. 回信出來，停下碼錶，在本子上寫「這張紙條花了多久」。

她又做了一個**貼編號殼**：紙條進來時貼上一個編號貼紙，回信出去時，**也貼上同一個編號**，之後才找得到是哪一對。

> 這就是 **`middleware::from_fn`**：`async fn timing(req: Request, next: Next) -> Response` 裡，`let resp = next.run(req).await;` 就是「交給裡面那層」，前後可以做事（記 `start.elapsed()`）。`request_id` 在 `req.headers_mut()` 和 `resp.headers_mut()` 都插入 `x-request-id`。需要讀 state 的版本用 `from_fn_with_state`。比自己 `impl Service + Layer` 簡單很多，先用 `from_fn` 就對了。

---

## 沒有通行證就退回

小柚再做一個**守門殼**：紙條上要有「**通行證**」那一欄，沒有的話，守門殼**直接吐回一張「你沒有權限」**，根本不把紙條往裡面傳。裡面的娃娃從頭到尾**不知道有人來過**。

> 這就是 **短路（提早回應）**：`async fn auth(req: Request, next: Next) -> Result<Response, StatusCode>`，`headers().get(header::AUTHORIZATION).is_none()` 時直接 `return Err(StatusCode::UNAUTHORIZED)`（401），沒有呼叫 `next.run(req)`，handler 不會執行。

---

## 守門殼只套在祕密娃娃身上

工坊的大部分娃娃，誰都可以傳紙條。只有一區「**祕密娃娃**」需要通行證。

老吳說：「守門殼**只套在祕密區**，別套到全部。」而且要**先把祕密娃娃擺好，再套殼**——如果殼套完了才又擺一顆新娃娃進來，那顆新的就沒被套到。

> 這就是 **局部 middleware**：範例的 `let private = Router::new().route("/me", ...).layer(middleware::from_fn(auth));`，再用 `.nest("/private", private)` 掛上去。`/` 不用通行證，`/private/me` 沒帶 `authorization` header 就 401。**`.layer()` 只會套到在它之前加的 route**，之後才 `.route()` 的不會被包。

---

## 通用尺寸的殼

隔壁也有一家娃娃工坊。他們的殼**只能套在自家的娃娃上**。

老吳的殼卻是**通用尺寸**：不只自家娃娃，別家用同一種規格做的娃娃——不管是會打電話的、會寄信的——都套得上。

> 這就是 **tower 與 actix middleware 的差別**：tower 是**跨框架的抽象**，同一個 Layer 可以給 axum、tonic、hyper、reqwest 用。actix 的 middleware（`Transform` + `Service`，或 `actix_web_lab::from_fn`）只能 actix 用。現成集合上，tower-http 也比 actix 散落各處的 crate 完整。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 會回話的娃娃 | **`Service`**（`call(req) -> Future`） |
| 「你現在有空嗎？」 | `poll_ready` |
| 最裡面的小娃娃 | **handler**（`hello`、`slow`、`echo`） |
| 套殼機 | **`Layer`**（`layer(inner) -> Service`） |
| 一層包一層 | `A(B(C(handler)))` |
| 由外到內的套殼清單 | **`ServiceBuilder`** |
| 記錄員殼 | `TraceLayer::new_for_http()` |
| 1 秒沙漏殼 | `TimeoutLayer` → 408 |
| 壓縮殼 | `CompressionLayer` |
| 外地通行殼 | `CorsLayer` |
| 自己捏殼的步驟 | `middleware::from_fn(f)` |
| 交給裡面那層 | `next.run(req).await` |
| 計時殼 | `timing` |
| 進出都貼同一個編號 | `request_id`（`x-request-id` header） |
| 沒通行證就退回 | `auth` 回 `Err(StatusCode::UNAUTHORIZED)`（短路） |
| 守門殼只套祕密區 | 子 `Router` `.layer(...)` + `.nest("/private", ...)` |
| 先擺娃娃再套殼 | `.route()` 要在 `.layer()` **之前** |
| 只能套自家娃娃的殼 | actix 的 `Transform` + `Service` |
| 通用尺寸的殼 | tower Layer 可給 axum / tonic / hyper / reqwest |

---

## 一句話記住

> **middleware 就是俄羅斯娃娃的殼：每一層都是「收紙條、吐回信」的 Service，`Layer` 是套殼機，`next.run(req)` 是把紙條往裡傳；不想讓它進去，就別呼叫 `next`。**
