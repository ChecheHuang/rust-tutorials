// Redis Streams demo — 持久訊息 + consumer group。
// docker run --rm -p 6379:6379 redis:7
// REDIS_URL=redis://127.0.0.1:6379 cargo run --bin ch44-redis-stream

use anyhow::Result;
use redis::AsyncCommands;
use redis::streams::{StreamReadOptions, StreamReadReply};
use tracing::info;

const STREAM: &str = "tasks";
const GROUP: &str = "workers";
const CONSUMER: &str = "w-1";

async fn ensure_group(conn: &mut redis::aio::ConnectionManager) -> Result<()> {
    // 用 MKSTREAM 確保 stream 存在
    let _: Result<String, _> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(STREAM)
        .arg(GROUP)
        .arg("$")
        .arg("MKSTREAM")
        .query_async(conn)
        .await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let url = std::env::var("REDIS_URL").unwrap_or_default();
    if url.is_empty() {
        eprintln!("沒設 REDIS_URL");
        return Ok(());
    }
    let client = redis::Client::open(url)?;
    let mut conn = redis::aio::ConnectionManager::new(client).await?;
    ensure_group(&mut conn).await?;

    // producer：丟 5 條訊息
    let mut p = conn.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            let _: String = p
                .xadd(STREAM, "*", &[("job", format!("work-{i}"))])
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    });

    // consumer：讀 5 條
    let opts = StreamReadOptions::default()
        .group(GROUP, CONSUMER)
        .count(2)
        .block(2000);
    let mut received = 0;
    while received < 5 {
        let reply: StreamReadReply = conn
            .xread_options(&[STREAM], &[">"], &opts)
            .await?;
        for key in reply.keys {
            for entry in key.ids {
                info!(id = %entry.id, fields = ?entry.map, "consumed");
                let _: i64 = conn.xack(STREAM, GROUP, &[entry.id]).await?;
                received += 1;
            }
        }
    }
    Ok(())
}
