// 簡化 CQRS + Event Sourcing 範例：銀行帳戶。
//
// 寫端：command → aggregate 驗證 → 產生 events → append 到 event store
// 讀端：subscriber 收 events 更新 read model（查詢用快照）

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::info;
use uuid::Uuid;

// ════════════════════════════════════════════════════════════
// 共用型別
// ════════════════════════════════════════════════════════════
type AccountId = Uuid;

#[derive(Debug, Error)]
enum DomainError {
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("account not found")]
    NotFound,
    #[error("already opened")]
    AlreadyOpened,
}

// ════════════════════════════════════════════════════════════
// commands（意圖）
// ════════════════════════════════════════════════════════════
#[derive(Debug)]
enum Command {
    Open { id: AccountId, owner: String },
    Deposit { id: AccountId, cents: i64 },
    Withdraw { id: AccountId, cents: i64 },
}

// ════════════════════════════════════════════════════════════
// events（事實）
// ════════════════════════════════════════════════════════════
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    Opened { id: AccountId, owner: String, at: DateTime<Utc> },
    Deposited { id: AccountId, cents: i64, at: DateTime<Utc> },
    Withdrawn { id: AccountId, cents: i64, at: DateTime<Utc> },
}

// ════════════════════════════════════════════════════════════
// aggregate — 從 events 重建狀態 + 處理 command
// ════════════════════════════════════════════════════════════
#[derive(Clone, Debug, Default)]
struct Account {
    opened: bool,
    balance_cents: i64,
}

impl Account {
    fn apply(&mut self, ev: &Event) {
        match ev {
            Event::Opened { .. } => { self.opened = true; }
            Event::Deposited { cents, .. } => { self.balance_cents += cents; }
            Event::Withdrawn { cents, .. } => { self.balance_cents -= cents; }
        }
    }

    fn handle(&self, cmd: &Command) -> Result<Vec<Event>, DomainError> {
        match cmd {
            Command::Open { id, owner } => {
                if self.opened { return Err(DomainError::AlreadyOpened); }
                Ok(vec![Event::Opened {
                    id: *id, owner: owner.clone(), at: Utc::now(),
                }])
            }
            Command::Deposit { id, cents } => {
                if !self.opened { return Err(DomainError::NotFound); }
                Ok(vec![Event::Deposited { id: *id, cents: *cents, at: Utc::now() }])
            }
            Command::Withdraw { id, cents } => {
                if !self.opened { return Err(DomainError::NotFound); }
                if self.balance_cents < *cents { return Err(DomainError::InsufficientFunds); }
                Ok(vec![Event::Withdrawn { id: *id, cents: *cents, at: Utc::now() }])
            }
        }
    }
}

// ════════════════════════════════════════════════════════════
// event store — append-only log
// ════════════════════════════════════════════════════════════
#[async_trait]
trait EventStore: Send + Sync {
    async fn append(&self, id: AccountId, events: Vec<Event>);
    async fn load(&self, id: AccountId) -> Vec<Event>;
}

struct InMemoryStore {
    log: RwLock<HashMap<AccountId, Vec<Event>>>,
    bus: broadcast::Sender<Event>,
}

#[async_trait]
impl EventStore for InMemoryStore {
    async fn append(&self, id: AccountId, events: Vec<Event>) {
        let mut log = self.log.write().await;
        for e in &events {
            let _ = self.bus.send(e.clone());
        }
        log.entry(id).or_default().extend(events);
    }
    async fn load(&self, id: AccountId) -> Vec<Event> {
        self.log.read().await.get(&id).cloned().unwrap_or_default()
    }
}

// ════════════════════════════════════════════════════════════
// 寫端 dispatcher
// ════════════════════════════════════════════════════════════
async fn dispatch(store: &dyn EventStore, cmd: Command) -> Result<(), DomainError> {
    let id = match &cmd {
        Command::Open { id, .. } | Command::Deposit { id, .. } | Command::Withdraw { id, .. } => *id,
    };

    let history = store.load(id).await;
    let mut agg = Account::default();
    for ev in &history { agg.apply(ev); }

    let new_events = agg.handle(&cmd)?;
    store.append(id, new_events).await;
    Ok(())
}

// ════════════════════════════════════════════════════════════
// 讀端：projection（query 用的 view）
// ════════════════════════════════════════════════════════════
#[derive(Clone, Debug, Default)]
struct BalanceView {
    map: HashMap<AccountId, (String, i64)>, // id → (owner, balance)
}

async fn run_projection(
    mut rx: broadcast::Receiver<Event>,
    view: Arc<RwLock<BalanceView>>,
) {
    while let Ok(ev) = rx.recv().await {
        let mut v = view.write().await;
        match ev {
            Event::Opened { id, owner, .. } => {
                v.map.insert(id, (owner, 0));
            }
            Event::Deposited { id, cents, .. } => {
                if let Some(x) = v.map.get_mut(&id) { x.1 += cents; }
            }
            Event::Withdrawn { id, cents, .. } => {
                if let Some(x) = v.map.get_mut(&id) { x.1 -= cents; }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (bus, _) = broadcast::channel(100);
    let store: Arc<dyn EventStore> = Arc::new(InMemoryStore {
        log: RwLock::new(HashMap::new()),
        bus: bus.clone(),
    });

    let view = Arc::new(RwLock::new(BalanceView::default()));
    tokio::spawn(run_projection(bus.subscribe(), view.clone()));

    let alice = Uuid::new_v4();
    dispatch(&*store, Command::Open { id: alice, owner: "alice".into() }).await?;
    dispatch(&*store, Command::Deposit { id: alice, cents: 10_000 }).await?;
    dispatch(&*store, Command::Withdraw { id: alice, cents: 3_000 }).await?;
    match dispatch(&*store, Command::Withdraw { id: alice, cents: 999_999 }).await {
        Err(e) => info!(%e, "expected error"),
        _ => unreachable!(),
    }

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    println!("\n=== event log ===");
    for ev in store.load(alice).await {
        println!("  {ev:?}");
    }
    println!("\n=== read model ===");
    let v = view.read().await;
    for (id, (owner, bal)) in &v.map {
        println!("  {owner} ({id}) → {bal} cents");
    }
    Ok(())
}
