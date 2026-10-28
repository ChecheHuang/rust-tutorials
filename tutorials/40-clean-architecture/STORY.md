# 故事版：蛋糕訂做工作室的分工 🎂

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「蛋糕訂做工作室裡，師傅、店長、店員、老闆娘各管各的事」的故事，幫你「感覺」出 Clean Architecture 的三層（domain / application / infrastructure）、業務不變式、use case、port（trait）、adapter、composition root、依賴方向、`Arc<dyn Trait>` 與泛型、用 fake 測試，以及別過度抽象。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 為什麼要分工

甜甜蛋糕工作室一開始只有老闆娘一個人：她接電話、記訂單、收錢、做蛋糕，**全部混在一起**。

後來工作室要換一家刷卡公司，老闆娘發現：刷卡的事跟做蛋糕的規矩**纏在一起**，改一個地方，其他地方跟著壞。

她決定好好分工：**最重要的規矩**交給師傅；**做事的流程**交給店長；**外面的雜事**交給店員。

> 這就是 **為什麼分層**：讓核心業務邏輯**不依賴外部世界**。換 DB、換金流、升級框架時，不用動到業務規則；業務規則也能在不起 DB、不連網路的情況下單元測試。

---

## 師傅的規矩本

**美玲師傅**有一本**規矩本**，寫的都是跟蛋糕本身有關的事：

- 一張訂單裡有好幾樣品項，每樣有**編號、數量、單價**。
- 算總價：每樣「單價 × 數量」加起來。價錢一律用「**分**」來算，不用小數點，才不會算錯錢。
- 訂單一開始是「**等待付款**」。付了錢變成「**已付款**」。
- **已經付過的，不能再付一次**；**已經取消的，也不能付**。

美玲師傅**從不接電話、不碰收銀機、不管訂單記在哪本簿子**。她只管這些規矩對不對。

> 這就是 **domain 層**：`mod domain` 裡的 `Order`、`Item`（`sku`、`qty`、`unit_price_cents`）、`Status { Pending, Paid, Cancelled }`。`total_cents()` 用 `i64` 的「分」計算。`pay(&mut self)` 守住**業務不變式**：`Pending` → `Paid`，`Paid` 回 `DomainError::AlreadyPaid`，`Cancelled` 回 `DomainError::Cancelled`。domain **沒有 `async fn`、不依賴 sqlx / serde / tokio**，也不 `unwrap()` / `panic!`。

---

## 店長的工作流程

**阿哲店長**負責「客人要付款」這件事的**流程**：

1. 先去找出這張訂單。找不到就說「查無此單」。
2. 拿去給美玲師傅確認：「這張可以付嗎？」師傅說不行，就停下來。
3. 照總價**收錢**，拿到一張收據號碼。
4. 把更新後的訂單**收好**。
5. 把收據號碼交給客人。

阿哲自己**不決定**能不能付——那是師傅的規矩。他只是**按順序把事情串起來**。

> 這就是 **use case**：`PayOrderUseCase::execute(order_id)`。`self.repo.get(order_id).await.ok_or_else(...)?` 找訂單；`order.pay()?` 呼叫 domain 規則；`self.payment.charge(order.total_cents()).await?` 收錢；`self.repo.save(order).await` 存回去；回傳 `txn`。use case 只負責**編排**，不寫業務規則本身。

---

## 牆上的職缺說明書

阿哲的流程裡需要兩種人幫忙，但他**不在乎是誰**。他在牆上貼了兩張**職缺說明書**：

- 「**保管訂單的人**：要會『收好一張訂單』、會『用編號找出訂單』。」
- 「**收錢的人**：要會『收一筆錢』，然後給我一張收據號碼。」

只要符合說明書，**誰來上班都可以**。

> 這就是 **port（trait）**：application 層定義 `trait OrderRepo: Send + Sync { async fn save(...); async fn get(&self, id: u64) -> Option<Order>; }` 和 `trait PaymentGateway: Send + Sync { async fn charge(&self, cents: i64) -> anyhow::Result<String>; }`。`PayOrderUseCase` 的欄位是 `Arc<dyn OrderRepo>` 和 `Arc<dyn PaymentGateway>`，只認說明書、不認人。trait 裡要用 `async fn` 又要當 `dyn`，目前仍常用 `#[async_trait]`。

---

## 真正來上班的店員

現在來應徵的有兩位：

- **小珍**：用一本**筆記本**記訂單，一次只讓一個人翻。
- **阿弟**：還在實習，拿的是一台**練習用的玩具收銀機**。按下去會「嗶——」等一下下，然後吐出一張寫著「txn-金額」的收據。

以後工作室變大，可以換成**真正的檔案櫃**和**真正的刷卡機**——只要他們也符合職缺說明書，阿哲的流程**一個字都不用改**。

> 這就是 **infrastructure / adapter**：`InMemoryOrderRepo(Mutex<HashMap<u64, Order>>)` 實作 `OrderRepo`；`FakePaymentGateway` 實作 `PaymentGateway`，`sleep` 50ms 後回 `format!("txn-{cents}")`。實務上會換成 `PostgresOrderRepo`、`StripeGateway`，對外的 axum handler、CLI 也屬於這一層。

---

## 開店前的排班表

每天早上開店前，**老闆娘**拿出排班表：「今天保管訂單的是**小珍**，收錢的是**阿弟**。」然後把他們介紹給阿哲。

整間工作室**只有老闆娘知道**今天上班的是誰。阿哲只知道「有人會保管訂單、有人會收錢」。

今天的第一張單是 1 號：A 蛋糕 2 個、每個 1500 分，B 蛋糕 1 個 999 分，總共 **3999 分**。阿哲跑完流程，拿到收據「**txn-3999**」，訂單變成「已付款」。

> 這就是 **composition root**：`main` 裡 `let repo: Arc<dyn OrderRepo> = Arc::new(InMemoryOrderRepo(...))`、`let payment: Arc<dyn PaymentGateway> = Arc::new(FakePaymentGateway)`，再組成 `PayOrderUseCase { repo: repo.clone(), payment: payment.clone() }`。**只有 main 知道具體型別**，要換實作只改 main。`use_case.execute(1)` 之後，`repo.get(1)` 看到 `status` 是 `Paid`。

---

## 誰認識誰

工作室有一條鐵律：

- 美玲師傅**不認識**阿哲，也不認識小珍、阿弟。
- 阿哲認識師傅的規矩本，但**不認識**小珍、阿弟，只認識職缺說明書。
- 老闆娘**誰都認識**，因為她負責排班。

認識的方向**永遠是往裡面**，從來沒有反過來。

> 這就是 **依賴方向**：adapter → application → domain，**永遠不反向**。domain 不知道有 use case，use case 不知道有 `InMemoryOrderRepo`。Hexagonal、Onion、Clean 名字不同，核心都是「依賴向內」。

---

## 練習日

新人訓練那天，老闆娘不想真的刷客人的卡。她請**假的保管員**和**假的收銀機**來上班，讓阿哲把流程跑一遍，看看「付完款訂單會不會變成已付款」。

沒有真的檔案櫃、沒有真的刷卡機，**一眨眼就練習完了**。

> 這就是 **用 fake 測試**：寫一個 `FakeRepo(Mutex<Vec<Order>>)` 實作 `OrderRepo`，在 `#[tokio::test]` 裡組出 `PayOrderUseCase`，呼叫 `execute(1)` 後 `assert_eq!(..., Status::Paid)`。完全不起 DB、不連網路，毫秒級完成。

---

## 職缺說明書 vs 指定人選

職缺說明書的好處是**誰都能來**，但每次要找人時，阿哲得先看一下「今天是誰？」，多花一點點時間。

如果某個位置**永遠是同一個人**、又要做得飛快，老闆娘可以乾脆在開店時就**把那個人的名字寫死**在流程裡，省下確認的時間——只是要換人時就得重寫一份流程。

> 這就是 **`Arc<dyn Trait>` vs 泛型**：`struct UseCase { repo: Arc<dyn OrderRepo> }` 靈活，但呼叫要經過 vtable；`struct UseCase<R: OrderRepo> { repo: R }` 在編譯期就決定具體型別（monomorphization），零成本但每種實作會產生一份程式碼。CRUD 業務用 `Arc<dyn>` 簡單；hot path 用泛型。

---

## 別把工作室搞得太複雜

老闆娘也犯過錯：

- 她曾經幫**每個人**都寫一張職缺說明書，連「只會有一個人做、也不會換人」的工作都寫。結果牆上貼滿紙，誰也看不懂。
- 她曾經在師傅的規矩本裡寫「外送平台的表格要怎麼填」，結果換外送平台時，規矩本也得跟著改。
- 有個店員嫌麻煩，**自己判斷**「這張可以付」就直接收錢，跳過了店長和師傅。

> 這就是 **常見陷阱**：**過度抽象**——只在要 mock 或要替換實作時才抽 trait；**domain 依賴框架**——`Order` 上別 `#[derive(Serialize)]`，另外做 `OrderDto`；**adapter 漏 logic**——handler 直接寫業務邏輯、繞過 use case；還有 use case 做太多事變成 god class，要拆小。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 美玲師傅的規矩本 | **domain 層**（`mod domain`） |
| 訂單、品項、狀態 | `Order`、`Item`、`Status` |
| 用「分」算總價 | `total_cents()`（`i64`） |
| 已付不能再付 | `pay()` → `DomainError::AlreadyPaid` |
| 師傅不接電話、不碰收銀機 | domain 無 `async`、無 IO、無框架依賴 |
| 阿哲店長的流程 | **use case** `PayOrderUseCase::execute` |
| 牆上的職缺說明書 | **port**：`trait OrderRepo`、`trait PaymentGateway` |
| 「誰來都可以」 | `Arc<dyn OrderRepo>`（trait object） |
| 小珍的筆記本 | `InMemoryOrderRepo`（`Mutex<HashMap>`） |
| 阿弟的玩具收銀機 | `FakePaymentGateway`（回 `txn-{cents}`） |
| 真正的檔案櫃 / 刷卡機 | `PostgresOrderRepo` / `StripeGateway` |
| 老闆娘的排班表 | **composition root**（`main`） |
| 認識的方向永遠往內 | adapter → application → domain |
| 練習日的假店員 | **fake** + `#[tokio::test]` |
| 名字寫死在流程裡 | **泛型** `UseCase<R: OrderRepo>` |
| 牆上貼滿說明書 | **過度抽象** |
| 規矩本寫外送表格 | domain `derive(Serialize)` → 要分 DTO |
| 店員自己決定收錢 | adapter 繞過 use case |

---

## 一句話記住

> **師傅只管規矩（domain），店長只管流程並認職缺說明書（use case + trait），店員照說明書做事（adapter），只有老闆娘排班時知道是誰（main）——認識的方向永遠往內。**
