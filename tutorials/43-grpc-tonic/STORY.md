# 故事版：玩具工廠和玩具店的專用訂貨單 🧸

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「玩具店跟玩具工廠用專用訂貨單溝通」的故事，幫你「感覺」出 gRPC、`.proto`、`build.rs` 產生程式碼、unary 和 server stream、`Status`、interceptor、HTTP/2、TLS 和健康檢查是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 以前：隨便寫的紙條

小鎮上有一間**玩具店**，老闆是小美；鎮外有一間**玩具工廠**，廠長是大熊叔叔。

以前小美要訂貨，就寫一張**隨手紙條**：「大熊叔叔你好～這週想要泰迪熊大概十隻左右，謝謝！」紙條字很多、格式每次都不一樣，大熊叔叔常常要猜：「『大概十隻』到底是幾隻？」

這種紙條很適合**貼在店門口給客人看**，大家都看得懂；但店和工廠之間每天來回幾百張，就太慢、太容易誤會了。

> 這就是 **REST/JSON vs gRPC**：JSON 是人看得懂的文字，適合對外 API、瀏覽器；gRPC 用二進位的 protobuf、走 HTTP/2，適合**內部 service 之間**。常見組合是「對外 REST + 對內 gRPC」。

---

## 先一起訂好訂貨單的格式

於是小美和大熊叔叔坐下來，**先**一起設計一張**專用訂貨單**：

- 這張單叫「**打招呼申請單**」，第 1 格寫「名字」。
- 回覆叫「**打招呼回條**」，第 1 格寫「回覆的話」。
- 工廠提供兩種服務：「**回一張回條**」，以及「**連續寄好幾張回條**」。

每一格都有**固定的編號**。以後就算多加新格子，舊的編號也不會亂掉。

> 這就是 **`.proto` 檔**：`proto/greeter.proto` 裡用 `syntax = "proto3"; package greeter;` 定義 `service Greeter`，有 `rpc SayHello (HelloRequest) returns (HelloReply)` 和 `rpc SayHelloStream (HelloRequest) returns (stream HelloReply)`；`message HelloRequest { string name = 1; }` 的 `= 1` 就是那格的編號。gRPC 是 **schema 先寫**，不像 OpenAPI 常常事後補。

---

## 印刷機自動印出兩本簿子

格式定好之後，大熊叔叔把它丟進一台**印刷機**。印刷機一下就印出兩本簿子：

- 一本給**工廠**：上面已經印好「收到申請單時，你要填哪些東西」。
- 一本給**玩具店**：上面已經印好「要寄單時，照這格式填就對了」。

兩本簿子來自**同一份格式**，所以絕對對得上。格式一改，重新印一次就好。

> 這就是 **`build.rs` + `tonic_build::compile_protos("proto/greeter.proto")`**：`cargo build` 時自動把 `.proto` 轉成 Rust 程式碼放在 `OUT_DIR`。server 和 client 都用 `tonic::include_proto!("greeter")` 引入，名字要和 `package greeter;` 對上。工廠那本是 `greeter_server::{Greeter, GreeterServer}`，店那本是 `greeter_client::GreeterClient`。記得加 `cargo:rerun-if-changed`，格式改了才會自動重印。

---

## 工廠照簿子把工作做好

大熊叔叔照工廠那本簿子，寫下自己的做法：

「收到打招呼申請單 → 拆開信封、看名字 → 寫一張回條：『**Hello, 名字！**』寄回去。」

工廠的大門開在**50051 號**碼頭，等著訂貨單送上門。

> 這就是 **實作 service**：`impl Greeter for MyGreeter` 裡的 `say_hello(&self, req: Request<HelloRequest>) -> Result<Response<HelloReply>, Status>`。`req.into_inner().name` 是拆信封拿內容，`Response::new(HelloReply { ... })` 是包好回條。`Server::builder().add_service(GreeterServer::new(MyGreeter)).serve(addr)` 在 `0.0.0.0:50051` 開門。

---

## 一張單，一張回條

小美這邊，用**店的那本簿子**連上工廠：「喂，50051 號碼頭嗎？」

她填一張申請單：名字寫「world」，寄出去。很快收到一張回條：「Hello, world!」

一來一回，乾淨俐落。

> 這就是 **unary RPC**：client 用 `GreeterClient::connect("http://127.0.0.1:50051")` 連線，`client.say_hello(Request::new(HelloRequest { name: "world".into() }))` 送一個 request、拿一個 response，再 `resp.into_inner().message` 讀內容。

---

## 一張單，一箱一箱送來

有時候小美想要的不只一張回條。她改用「**連續寄好幾張**」那個服務。

工廠收到單子後，開始用**輸送帶**送：第 0 張、第 1 張……每隔一下就送一張，總共 5 張。輸送帶上**最多只放得下 4 張**，如果小美那頭還沒拿走，工廠就先等一下，不會讓回條堆到掉滿地。

小美站在輸送帶尾巴，**來一張拿一張**，直到輸送帶說「沒了」。

> 這就是 **server streaming**：`say_hello_stream` 裡建 `tokio::sync::mpsc::channel(4)`，`tokio::spawn` 一個 task 迴圈 `tx.send(Ok(HelloReply { ... }))` 五次；`ReceiverStream::new(rx)` 包成 stream，型別是 `SayHelloStreamStream`（`Pin<Box<dyn Stream<...> + Send>>`）。buffer 大小 4 就是 **backpressure**。client 端用 `while let Some(reply) = stream.message().await?` 一個個收。另外還有 client stream 和 bidi stream，共四種風格。

---

## 退件單上寫的是名字，不是數字

有一次小美忘了蓋店章，工廠把單子**退回來**，上面寫著：「**未驗證身分（unauthenticated）**：沒有店章。」

小美一開始以為退件單會跟郵局一樣寫「401」，結果工廠的退件單用的是**自己的一套名字和號碼**。

> 這就是 **`Status`**：gRPC 的錯誤型別，有具名的 status code、訊息和 details。`Status::unauthenticated("no token")` 對應的是 gRPC 的 16，**不是 HTTP 的 401**。

---

## 工廠門口的警衛

後來工廠在門口請了一位**警衛**。每張訂貨單進門前，警衛都先翻一下信封上的**店章欄**：

- 有店章 → 放行，送進工廠。
- 沒店章 → 當場退件，連廠長都不用驚動。

小美那邊也請了一位小幫手，**每張單子寄出前自動蓋好店章**。

> 這就是 **interceptor**：`fn auth_intercept(req: Request<()>) -> Result<Request<()>, Status>` 檢查 `req.metadata().get("authorization")`，沒有就回 `Status::unauthenticated`。用 `GreeterServer::with_interceptor(svc, auth_intercept)` 掛上去；client 端也能用 interceptor 自動帶 token。tonic 底層是 tower service，所以也能 `.layer(TraceLayer::new_for_grpc())`、`TimeoutLayer` 這些 tower-http 的 layer。

---

## 一條路上好多台車一起跑

店和工廠之間只鋪了**一條路**，但這條路有**很多車道**。好幾張訂貨單、好幾條輸送帶可以**同時**在上面跑，不用一台車等前一台車回來才出發。

而且訂貨單是**壓得很扁的密碼格子**，不是長長的信，一台車能載好多張。

> 這就是 **HTTP/2 multiplex + protobuf 二進位編碼**：一條連線上同時跑多個 stream；protobuf 比 JSON 小又快。注意預設訊息上限 4MB，太大要調 `max_decoding_message_size(...)`。

---

## 上鎖的信封和服務台

正式營運後，大熊叔叔又加了三件事：

1. **上鎖的信封**：路上的人看不到內容，也冒充不了工廠。
2. **門口的「營業中」燈**：別人一看就知道工廠今天有沒有開工。
3. **服務目錄**：有人來問「你們工廠有哪些服務？」，工廠能直接列給他看。

還有一條規矩：**訂貨單格式改了，店和工廠都要一起換新簿子**，不然一邊寫第 3 格、一邊根本沒有第 3 格。

> 這就是 **TLS、健康檢查、reflection**：`Identity::from_pem(cert, key)` + `ServerTlsConfig::new().identity(identity)` 開 TLS（很多 proxy 不支援明文 h2c）；`tonic-health` 提供 `grpc.health.v1.Health`；`tonic-reflection` 讓 `grpcurl ... list` 查服務。proto 一改 client / server 要同步，CI 要鎖 `.proto`。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 隨手紙條 | **REST / JSON** |
| 專用訂貨單 | **gRPC + protobuf** |
| 先一起設計的訂貨單格式 | **`.proto`**：`proto/greeter.proto` |
| 格子的固定編號 | **field number** `string name = 1;` |
| 印刷機 | **`build.rs`** + `tonic_build::compile_protos` |
| 把印好的簿子拿進來 | **`tonic::include_proto!("greeter")`** |
| 工廠的做法 | **`impl Greeter for MyGreeter`** |
| 拆信封 / 包回條 | `req.into_inner()` / `Response::new(...)` |
| 50051 號碼頭 | `Server::builder().add_service(...).serve(addr)` |
| 店的那本簿子 | **`GreeterClient::connect(...)`** |
| 一張單一張回條 | **unary**：`say_hello` |
| 輸送帶一張張送 | **server stream**：`say_hello_stream` + `ReceiverStream` |
| 輸送帶最多放 4 張 | **`mpsc::channel(4)` backpressure** |
| 具名的退件單 | **`Status`**（`Status::unauthenticated` = 16） |
| 門口警衛 | **interceptor**：`with_interceptor` |
| 一條路很多車道 | **HTTP/2 multiplex** |
| 上鎖的信封 | **TLS**：`ServerTlsConfig` |
| 營業中燈 / 服務目錄 | **`tonic-health`** / **`tonic-reflection`** |

---

## 一句話記住

> **gRPC 就是先一起設計好專用訂貨單（`.proto`），讓印刷機（`build.rs`）印出兩邊對得上的簿子；一張單換一張回條是 unary，輸送帶一張張送是 stream，門口警衛（interceptor）先檢查店章——格式一改，兩邊要一起換新簿子。**
