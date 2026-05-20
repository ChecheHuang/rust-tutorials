# 40. Clean Architecture

> 範圍：分層、port-adapter、trait object 作為 port、composition root

## 為什麼分層

不是為了「看起來專業」，而是讓**核心業務邏輯不依賴外部世界**：
- 換 DB 不必改 use case
- 業務邏輯可以單元測試（不起 DB / 網路）
- 框架升級不會傷到 domain

## 三層

```
┌─────────────────────────────────────────┐
│           adapter / composition         │   ← main、HTTP、CLI、gRPC handler
│   把 use case + port 實作組裝起來         │
├─────────────────────────────────────────┤
│             application                 │   ← use cases + ports（trait）
│   PayOrderUseCase、OrderRepo trait      │
├─────────────────────────────────────────┤
│               domain                    │   ← 純邏輯，零依賴
│   Order、Item、Status、業務不變式         │
└─────────────────────────────────────────┘
依賴方向：adapter → application → domain（永遠不反向）
```

### domain 層

只放**業務概念與不變式**：

```rust
pub struct Order { pub id: u64, pub items: Vec<Item>, pub status: Status }
impl Order {
    pub fn pay(&mut self) -> Result<(), DomainError> { ... }   // 不變式：已付不能再付
}
```

**禁止**：
- `async fn`（IO 是別層的事）
- 依賴 sqlx / serde / tokio
- `unwrap()`、`panic!`

### application 層

定義 **port**（trait）+ **use case**（聚合 ports 達成一個業務動作）：

```rust
#[async_trait]
trait OrderRepo: Send + Sync {
    async fn save(&self, o: Order);
    async fn get(&self, id: u64) -> Option<Order>;
}

struct PayOrderUseCase {
    repo: Arc<dyn OrderRepo>,
    payment: Arc<dyn PaymentGateway>,
}

impl PayOrderUseCase {
    async fn execute(&self, id: u64) -> Result<String> {
        let mut o = self.repo.get(id).await.ok_or(...)?;
        o.pay()?;                                   // 呼叫 domain 規則
        let txn = self.payment.charge(o.total_cents()).await?;
        self.repo.save(o).await;
        Ok(txn)
    }
}
```

### infrastructure / adapter 層

實作 ports（PostgresOrderRepo、StripePaymentGateway）+ 對外（axum handler、CLI command）。

## composition root

整個 app **只有 main 知道具體型別**：

```rust
fn main() {
    let repo: Arc<dyn OrderRepo> = Arc::new(PostgresOrderRepo::new(pool));
    let payment: Arc<dyn PaymentGateway> = Arc::new(StripeGateway::new(key));
    let use_case = PayOrderUseCase { repo, payment };
    serve(use_case);
}
```

換 fake / mock 只改 main，其他層不動。

## Rust 風的特殊考量

### `async_trait` 還是必要

trait 想有 `async fn` 目前最好仍用 `#[async_trait]`，雖然 Rust 1.75+ 有 native AFIT，但 `dyn Trait` + async 還不完全乾淨。

### `Arc<dyn Trait>` vs 泛型

```rust
// dyn 版本：靈活但有 vtable 開銷
struct UseCase { repo: Arc<dyn OrderRepo> }

// 泛型版本：零成本但 monomorphic
struct UseCase<R: OrderRepo> { repo: R }
```

實務：CRUD 業務用 `Arc<dyn>` 簡單；hot path / 多型不要的場景用泛型。

### 避免 Java 風過度抽象

不要為每個 struct 寫 interface：

```rust
// ❌ 沒必要
trait UserService { fn get(&self) -> User; }
struct UserServiceImpl;

// ✅ 直接 struct
struct UserService;
impl UserService { fn get(&self) -> User { ... } }
```

只在「**要 mock**」或「**要替換實作**」才抽 trait。

## 跟 Hexagonal、Onion 的差異

差別不大：
- **Hexagonal**：強調 port-adapter
- **Onion**：強調同心圓層次
- **Clean**：Robert Martin 整合版

實務只要做到「依賴向內」就夠。

## 測試策略

```rust
struct FakeRepo(Mutex<Vec<Order>>);
impl OrderRepo for FakeRepo { ... }

#[tokio::test]
async fn pay_changes_status() {
    let repo = Arc::new(FakeRepo::default());
    let use_case = PayOrderUseCase { repo: repo.clone(), payment: Arc::new(FakePayment) };
    use_case.execute(1).await.unwrap();
    assert_eq!(repo.get(1).await.unwrap().status, Status::Paid);
}
```

完全不起 DB、網路，毫秒級。

## 常見陷阱

1. **過度抽象** — 寫一堆只有一個實作的 trait，徒增複雜度。需要替換才抽。
2. **domain 依賴 framework** — domain struct 上 `#[derive(Serialize)]` 就把它跟 serde 綁死。要分 `OrderDto`。
3. **use case 變肥** — 一個 use case 做太多事 = god class；拆細粒度。
4. **`Arc<dyn>` 過用** — 全部 trait object 會犧牲效能，可測但跑不快。
5. **adapter 漏 logic** — handler 直接寫業務邏輯，繞過 use case。

## 練習

1. 加一個 `CancelOrderUseCase`，含「已付不能取消」規則。
2. 為 `PayOrderUseCase` 寫 mock-based 單元測試。
3. 把 `InMemoryOrderRepo` 替換成 sqlx 版本，看主程式以外有沒有要改。
4. 加 `EventBus` port，付款成功發 `OrderPaid` 事件。

## 延伸閱讀

- [The Clean Architecture — Uncle Bob](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Hexagonal Architecture — Cockburn](https://alistair.cockburn.us/hexagonal-architecture/)
