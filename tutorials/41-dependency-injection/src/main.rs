// 兩種 DI 風格：
//   1. manual：自己在 main 組 Arc，最 Rust idiomatic
//   2. shaku：宣告式 DI container，類似 Java spring 但編譯期解析
//
// 結論：99% 情況用 manual。DI container 帶入更多抽象，
// Rust 編譯期類型系統已經幫你檢查很多東西。

use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

// ── ports ───────────────────────────────────────────────────
#[async_trait]
trait Mailer: Send + Sync {
    async fn send(&self, to: &str, body: &str) -> anyhow::Result<()>;
}

#[async_trait]
trait Clock: Send + Sync {
    fn now_unix(&self) -> u64;
}

// ── 實作 ────────────────────────────────────────────────────
struct StdoutMailer;
#[async_trait]
impl Mailer for StdoutMailer {
    async fn send(&self, to: &str, body: &str) -> anyhow::Result<()> {
        info!("[mail] to={to} body={body}");
        Ok(())
    }
}

struct SystemClock;
impl Clock for SystemClock {
    fn now_unix(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// ── use case：構造函式注入 ───────────────────────────────────
struct NotifyUseCase {
    mailer: Arc<dyn Mailer>,
    clock: Arc<dyn Clock>,
}

impl NotifyUseCase {
    fn new(mailer: Arc<dyn Mailer>, clock: Arc<dyn Clock>) -> Self {
        Self { mailer, clock }
    }
    async fn notify(&self, to: &str, msg: &str) -> anyhow::Result<()> {
        let now = self.clock.now_unix();
        self.mailer.send(to, &format!("[{now}] {msg}")).await
    }
}

// ── 風格 1：manual wiring ───────────────────────────────────
async fn manual_demo() -> anyhow::Result<()> {
    let mailer: Arc<dyn Mailer> = Arc::new(StdoutMailer);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let use_case = NotifyUseCase::new(mailer, clock);
    use_case.notify("alice@x.com", "manual wiring works").await
}

// ── 風格 2：shaku container ─────────────────────────────────
use shaku::{module, Component, HasComponent, Interface};

trait MailerSvc: Interface {
    fn send_sync(&self, to: &str, body: &str);
}

#[derive(Component)]
#[shaku(interface = MailerSvc)]
struct PrintMailer;

impl MailerSvc for PrintMailer {
    fn send_sync(&self, to: &str, body: &str) {
        info!("[shaku-mail] to={to} body={body}");
    }
}

trait Notifier: Interface {
    fn notify(&self, to: &str, msg: &str);
}

#[derive(Component)]
#[shaku(interface = Notifier)]
struct NotifierImpl {
    #[shaku(inject)]
    mailer: Arc<dyn MailerSvc>,
}

impl Notifier for NotifierImpl {
    fn notify(&self, to: &str, msg: &str) {
        self.mailer.send_sync(to, msg);
    }
}

module! {
    AppModule {
        components = [PrintMailer, NotifierImpl],
        providers = [],
    }
}

fn shaku_demo() {
    let module = AppModule::builder().build();
    let notifier: &dyn Notifier = module.resolve_ref();
    notifier.notify("bob@x.com", "shaku container works");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    println!("=== manual wiring ===");
    manual_demo().await?;
    println!("\n=== shaku container ===");
    shaku_demo();
    Ok(())
}
