// 韌性模式範例：
//   1. timeout
//   2. retry with exponential backoff + jitter
//   3. circuit breaker（簡化）
//   4. rate limit（client 端，governor）

use anyhow::Result;
use governor::{Quota, RateLimiter};
use rand::Rng;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

// ── 1. timeout ─────────────────────────────────────────────
async fn with_timeout<F, T>(fut: F, ms: u64) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match tokio::time::timeout(Duration::from_millis(ms), fut).await {
        Ok(r) => r,
        Err(_) => Err(anyhow::anyhow!("timeout {ms}ms")),
    }
}

// ── 2. retry + backoff + jitter ────────────────────────────
async fn retry<F, Fut, T>(mut op: F, max: u32) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut backoff_ms = 50u64;
    for attempt in 1..=max {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt == max => return Err(e),
            Err(e) => {
                let jitter: u64 = rand::thread_rng().gen_range(0..backoff_ms / 2);
                let wait = backoff_ms + jitter;
                warn!(attempt, ?e, wait_ms = wait, "retry");
                tokio::time::sleep(Duration::from_millis(wait)).await;
                backoff_ms = (backoff_ms * 2).min(2000);
            }
        }
    }
    unreachable!()
}

// ── 3. circuit breaker（簡化版） ───────────────────────────
#[derive(Clone)]
struct Breaker {
    fail_count: Arc<AtomicU64>,
    threshold: u64,
    open_until: Arc<tokio::sync::Mutex<Option<std::time::Instant>>>,
    cooldown: Duration,
}

impl Breaker {
    fn new(threshold: u64, cooldown: Duration) -> Self {
        Self {
            fail_count: Arc::new(AtomicU64::new(0)),
            threshold,
            open_until: Arc::new(tokio::sync::Mutex::new(None)),
            cooldown,
        }
    }

    async fn call<F, Fut, T>(&self, op: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        {
            let guard = self.open_until.lock().await;
            if let Some(t) = *guard {
                if std::time::Instant::now() < t {
                    return Err(anyhow::anyhow!("circuit open"));
                }
            }
        }
        match op().await {
            Ok(v) => {
                self.fail_count.store(0, Ordering::Relaxed);
                Ok(v)
            }
            Err(e) => {
                let n = self.fail_count.fetch_add(1, Ordering::Relaxed) + 1;
                if n >= self.threshold {
                    let mut g = self.open_until.lock().await;
                    *g = Some(std::time::Instant::now() + self.cooldown);
                    warn!(failures = n, cooldown_ms = self.cooldown.as_millis(), "circuit OPEN");
                }
                Err(e)
            }
        }
    }
}

// ── 4. rate limit（client 端） ─────────────────────────────
async fn rate_limited_demo() {
    let quota = Quota::per_second(NonZeroU32::new(5).unwrap());
    let lim = RateLimiter::direct(quota);
    let start = std::time::Instant::now();
    for i in 0..12 {
        lim.until_ready().await;
        info!(i, elapsed_ms = start.elapsed().as_millis() as u64, "tick");
    }
}

// ── fake unreliable service ────────────────────────────────
async fn flaky_call(attempt: u32) -> Result<&'static str> {
    if attempt < 3 {
        Err(anyhow::anyhow!("flaky"))
    } else {
        Ok("ok")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // timeout demo
    println!("=== timeout ===");
    let res = with_timeout(async {
        tokio::time::sleep(Duration::from_millis(300)).await;
        Ok::<_, anyhow::Error>("done")
    }, 100).await;
    info!(?res, "with_timeout 100ms");

    // retry demo
    println!("\n=== retry + backoff ===");
    let counter = Arc::new(AtomicU64::new(0));
    let counter2 = counter.clone();
    let r = retry(|| {
        let c = counter2.clone();
        async move {
            let n = c.fetch_add(1, Ordering::SeqCst) + 1;
            flaky_call(n as u32).await
        }
    }, 5).await?;
    info!(result = r, "retry success");

    // circuit breaker demo
    println!("\n=== circuit breaker ===");
    let cb = Breaker::new(3, Duration::from_millis(500));
    for i in 0..6 {
        let r: Result<()> = cb.call(|| async {
            Err(anyhow::anyhow!("fail {i}"))
        }).await;
        info!(i, ?r, "call");
    }

    // rate limit demo
    println!("\n=== rate limit (5/s) ===");
    rate_limited_demo().await;
    Ok(())
}
