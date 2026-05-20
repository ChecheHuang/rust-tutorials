// 跑之前：
//   docker run --rm -p 6379:6379 redis:7
//   REDIS_URL=redis://127.0.0.1:6379 cargo run

use anyhow::Result;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    user_id: u64,
    name: String,
}

async fn cache_aside(conn: &mut redis::aio::ConnectionManager) -> Result<()> {
    let key = "user:42";
    // 1. try get
    let cached: Option<String> = conn.get(key).await?;
    if let Some(s) = cached {
        let sess: Session = serde_json::from_str(&s)?;
        info!(?sess, "cache hit");
        return Ok(());
    }

    // 2. miss → 假裝 DB query
    info!("cache miss, fetching from db");
    let sess = Session { user_id: 42, name: "alice".into() };

    // 3. write back with TTL
    let _: () = conn.set_ex(key, serde_json::to_string(&sess)?, 60).await?;
    Ok(())
}

async fn counter_atomic(conn: &mut redis::aio::ConnectionManager) -> Result<()> {
    let _: () = conn.del("api:hits").await?;
    let mut last: i64 = 0;
    for _ in 0..5 {
        last = conn.incr("api:hits", 1).await?;
    }
    info!(last, "incr counter");
    Ok(())
}

async fn rate_limit(conn: &mut redis::aio::ConnectionManager, ip: &str) -> Result<bool> {
    // 簡單 fixed window rate limit
    let key = format!("rl:{ip}");
    let n: i64 = conn.incr(&key, 1).await?;
    if n == 1 {
        let _: () = conn.expire(&key, 10).await?; // 10s window
    }
    Ok(n <= 5)
}

async fn pubsub_demo() -> Result<()> {
    let url = std::env::var("REDIS_URL")?;
    let client = redis::Client::open(url)?;
    let pubsub_conn = client.get_async_pubsub().await?;
    let publish_client = client.clone();

    let mut pubsub = pubsub_conn;
    pubsub.subscribe("events").await?;

    let publisher = tokio::spawn(async move {
        let mut c = publish_client.get_multiplexed_async_connection().await.unwrap();
        for i in 0..3 {
            let _: () = redis::cmd("PUBLISH")
                .arg("events")
                .arg(format!("hello-{i}"))
                .query_async(&mut c)
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    use futures::StreamExt;
    let mut stream = pubsub.on_message();
    for _ in 0..3 {
        let msg = stream.next().await.unwrap();
        let payload: String = msg.get_payload().unwrap();
        info!(%payload, channel = msg.get_channel_name(), "received");
    }
    publisher.await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let url = std::env::var("REDIS_URL").unwrap_or_default();
    if url.is_empty() {
        eprintln!("沒設 REDIS_URL；範例僅作展示。");
        eprintln!("docker run --rm -p 6379:6379 redis:7");
        eprintln!("REDIS_URL=redis://127.0.0.1:6379 cargo run");
        return Ok(());
    }

    let client = redis::Client::open(url.clone())?;
    let mut conn = redis::aio::ConnectionManager::new(client).await?;

    println!("=== cache aside ===");
    cache_aside(&mut conn).await?;
    cache_aside(&mut conn).await?; // 第二次應 hit

    println!("\n=== atomic counter ===");
    counter_atomic(&mut conn).await?;

    println!("\n=== rate limit ===");
    for i in 0..7 {
        let ok = rate_limit(&mut conn, "1.2.3.4").await?;
        println!("req {} → {}", i, if ok { "PASS" } else { "BLOCKED" });
    }

    println!("\n=== pub/sub ===");
    pubsub_demo().await?;
    Ok(())
}
