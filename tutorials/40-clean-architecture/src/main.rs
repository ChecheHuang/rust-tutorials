// 範例：訂單系統。分三層 — domain / application / infrastructure。
// 由 main（adapter 層）把所有東西組裝起來。

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::info;

// ════════════════════════════════════════════════════════════
// domain — 業務核心。不依賴任何框架、DB、IO。
// ════════════════════════════════════════════════════════════
mod domain {
    use super::DomainError;

    #[derive(Clone, Debug)]
    pub struct Order {
        pub id: u64,
        pub items: Vec<Item>,
        pub status: Status,
    }

    #[derive(Clone, Debug)]
    pub struct Item {
        pub sku: String,
        pub qty: u32,
        pub unit_price_cents: i64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Status {
        Pending,
        Paid,
        Cancelled,
    }

    impl Order {
        pub fn total_cents(&self) -> i64 {
            self.items
                .iter()
                .map(|it| it.unit_price_cents * it.qty as i64)
                .sum()
        }

        pub fn pay(&mut self) -> Result<(), DomainError> {
            match self.status {
                Status::Pending => { self.status = Status::Paid; Ok(()) }
                Status::Paid => Err(DomainError::AlreadyPaid),
                Status::Cancelled => Err(DomainError::Cancelled),
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("already paid")]
    AlreadyPaid,
    #[error("order cancelled")]
    Cancelled,
}

// ════════════════════════════════════════════════════════════
// application — use cases 與依賴的 port（trait）
// ════════════════════════════════════════════════════════════
#[async_trait]
trait OrderRepo: Send + Sync {
    async fn save(&self, o: domain::Order);
    async fn get(&self, id: u64) -> Option<domain::Order>;
}

#[async_trait]
trait PaymentGateway: Send + Sync {
    async fn charge(&self, cents: i64) -> anyhow::Result<String>; // 回 txn id
}

struct PayOrderUseCase {
    repo: Arc<dyn OrderRepo>,
    payment: Arc<dyn PaymentGateway>,
}

impl PayOrderUseCase {
    async fn execute(&self, order_id: u64) -> anyhow::Result<String> {
        let mut order = self.repo.get(order_id).await
            .ok_or_else(|| anyhow::anyhow!("not found"))?;
        order.pay()?;
        let txn = self.payment.charge(order.total_cents()).await?;
        self.repo.save(order).await;
        Ok(txn)
    }
}

// ════════════════════════════════════════════════════════════
// infrastructure — port 的具體實作
// ════════════════════════════════════════════════════════════
struct InMemoryOrderRepo(Mutex<HashMap<u64, domain::Order>>);

#[async_trait]
impl OrderRepo for InMemoryOrderRepo {
    async fn save(&self, o: domain::Order) {
        self.0.lock().await.insert(o.id, o);
    }
    async fn get(&self, id: u64) -> Option<domain::Order> {
        self.0.lock().await.get(&id).cloned()
    }
}

struct FakePaymentGateway;

#[async_trait]
impl PaymentGateway for FakePaymentGateway {
    async fn charge(&self, cents: i64) -> anyhow::Result<String> {
        info!(cents, "charging");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(format!("txn-{cents}"))
    }
}

// ════════════════════════════════════════════════════════════
// adapter / composition root — 在這裡把依賴接好
// ════════════════════════════════════════════════════════════
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let repo: Arc<dyn OrderRepo> = Arc::new(InMemoryOrderRepo(Mutex::new(HashMap::new())));
    let payment: Arc<dyn PaymentGateway> = Arc::new(FakePaymentGateway);

    repo.save(domain::Order {
        id: 1,
        items: vec![
            domain::Item { sku: "A".into(), qty: 2, unit_price_cents: 1500 },
            domain::Item { sku: "B".into(), qty: 1, unit_price_cents: 999 },
        ],
        status: domain::Status::Pending,
    }).await;

    let use_case = PayOrderUseCase {
        repo: repo.clone(),
        payment: payment.clone(),
    };
    let txn = use_case.execute(1).await?;
    info!(%txn, "paid");

    let order = repo.get(1).await.unwrap();
    info!(?order.status, total = order.total_cents(), "after");
    Ok(())
}
