// 韌性模式範例：
//   1. timeout
//   2. retry with exponential backoff + jitter
//   3. circuit breaker（closed / open / half-open）
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

// ── 3. circuit breaker ─────────────────────────────────────
//
//   Closed ──連續 N 次失敗──▶ Open ──冷卻時間到──▶ HalfOpen
//     ▲                        ▲                    │
//     └──────── 探測成功 ───────┼────────────────────┤
//                              └──── 探測失敗 ──────┘
//
// HalfOpen 只放「一個」探測請求過去，其他請求照樣快速失敗，
// 避免後端剛恢復就被整批流量再打掛。
#[derive(Debug, Clone, Copy)]
enum BreakerState {
    Closed { failures: u64 },
    Open { until: std::time::Instant },
    /// 已放出一個探測請求，結果回來前其他請求一律拒絕
    HalfOpen,
}

#[derive(Clone)]
struct Breaker {
    // 只在 await 之外短暫持有，用 std Mutex 即可（第 29 章）
    state: Arc<std::sync::Mutex<BreakerState>>,
    threshold: u64,
    cooldown: Duration,
}

impl Breaker {
    fn new(threshold: u64, cooldown: Duration) -> Self {
        Self {
            state: Arc::new(std::sync::Mutex::new(BreakerState::Closed { failures: 0 })),
            threshold,
            cooldown,
        }
    }

    async fn call<F, Fut, T>(&self, op: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // 1. 決定這個請求能不能過
        {
            let mut st = self.state.lock().unwrap();
            match *st {
                BreakerState::Closed { .. } => {}
                BreakerState::Open { until } if std::time::Instant::now() < until => {
                    return Err(anyhow::anyhow!("circuit open"));
                }
                BreakerState::Open { .. } => {
                    // 冷卻結束：自己當探測請求
                    *st = BreakerState::HalfOpen;
                    info!("circuit HALF-OPEN, probing");
                }
                BreakerState::HalfOpen => {
                    return Err(anyhow::anyhow!("circuit half-open, probe in flight"));
                }
            }
        }

        // 2. 真的去打後端（不持有鎖）
        let res = op().await;

        // 3. 依結果更新狀態
        let mut st = self.state.lock().unwrap();
        match (&res, *st) {
            (Ok(_), BreakerState::HalfOpen) => {
                *st = BreakerState::Closed { failures: 0 };
                info!("probe ok, circuit CLOSED");
            }
            (Ok(_), _) => *st = BreakerState::Closed { failures: 0 },
            (Err(_), BreakerState::HalfOpen) => {
                *st = BreakerState::Open { until: std::time::Instant::now() + self.cooldown };
                warn!("probe failed, circuit OPEN again");
            }
            (Err(_), BreakerState::Closed { failures }) => {
                let n = failures + 1;
                if n >= self.threshold {
                    *st = BreakerState::Open { until: std::time::Instant::now() + self.cooldown };
                    warn!(failures = n, cooldown_ms = self.cooldown.as_millis(), "circuit OPEN");
                } else {
                    *st = BreakerState::Closed { failures: n };
                }
            }
            (Err(_), BreakerState::Open { .. }) => {}
        }
        res
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
    // 連續失敗 3 次 → OPEN，之後 2 次直接被拒（沒打後端）
    for i in 0..5 {
        let r: Result<()> = cb.call(|| async {
            Err(anyhow::anyhow!("fail {i}"))
        }).await;
        info!(i, ?r, "call");
    }
    // 冷卻後第一個請求當探測：這次故意失敗 → 再 OPEN
    tokio::time::sleep(Duration::from_millis(600)).await;
    let r: Result<()> = cb.call(|| async { Err(anyhow::anyhow!("still down")) }).await;
    info!(?r, "probe #1");
    // 再冷卻一次，探測成功 → CLOSED，之後恢復正常
    tokio::time::sleep(Duration::from_millis(600)).await;
    let r = cb.call(|| async { Ok::<_, anyhow::Error>("recovered") }).await;
    info!(?r, "probe #2");
    let r = cb.call(|| async { Ok::<_, anyhow::Error>("normal") }).await;
    info!(?r, "after close");

    // rate limit demo
    println!("\n=== rate limit (5/s) ===");
    rate_limited_demo().await;
    Ok(())
}
