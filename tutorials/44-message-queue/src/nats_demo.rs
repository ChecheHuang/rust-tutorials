// 跑前先啟動 NATS：
//   docker run --rm -p 4222:4222 nats:latest
//   NATS_URL=nats://127.0.0.1:4222 cargo run --bin ch44-nats

use anyhow::Result;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    kind: String,
    payload: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let url = std::env::var("NATS_URL").unwrap_or_default();
    if url.is_empty() {
        eprintln!("沒設 NATS_URL，範例僅作展示。");
        eprintln!("docker run --rm -p 4222:4222 nats:latest");
        return Ok(());
    }

    let client = async_nats::connect(&url).await?;

    // ── 1. pub/sub ──────────────────────────────────────────
    let mut sub = client.subscribe("events.>").await?;
    let pub_client = client.clone();
    tokio::spawn(async move {
        for i in 0..3 {
            let e = Event {
                kind: format!("kind-{i}"),
                payload: format!("hello-{i}"),
            };
            let body = serde_json::to_vec(&e).unwrap();
            pub_client.publish(format!("events.kind{i}"), body.into()).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    });

    for _ in 0..3 {
        if let Some(msg) = sub.next().await {
            let e: Event = serde_json::from_slice(&msg.payload)?;
            info!(subject = %msg.subject, ?e, "received");
        }
    }

    // ── 2. request/reply ────────────────────────────────────
    let svc_client = client.clone();
    tokio::spawn(async move {
        let mut req_sub = svc_client.subscribe("svc.ping").await.unwrap();
        while let Some(msg) = req_sub.next().await {
            if let Some(reply) = msg.reply {
                svc_client.publish(reply, "pong".into()).await.unwrap();
            }
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let resp = client.request("svc.ping", "hello?".into()).await?;
    info!(reply = ?String::from_utf8_lossy(&resp.payload), "got reply");

    Ok(())
}
