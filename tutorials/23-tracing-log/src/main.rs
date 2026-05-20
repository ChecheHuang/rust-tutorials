use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::EnvFilter;

#[instrument]
fn fetch_user(id: u64) -> Result<String, &'static str> {
    debug!("looking up user");
    if id == 0 {
        error!("invalid id");
        return Err("invalid id");
    }
    if id == 999 {
        warn!(id, "this user is special");
    }
    let name = format!("user-{id}");
    info!(%name, "found");
    Ok(name)
}

#[instrument(skip(items))]
fn process_batch(items: &[u64]) {
    info!(count = items.len(), "starting batch");
    for &id in items {
        match fetch_user(id) {
            Ok(_) => {}
            Err(e) => warn!(id, error = e, "fetch failed"),
        }
    }
    info!("batch done");
}

fn main() {
    // ── subscriber 設定（一次性）─────────────────────────────
    // RUST_LOG=info 或 RUST_LOG=ch23=debug 控制 level
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(true)
        .with_line_number(true)
        .init();

    info!("=== tracing demo ===");

    // 簡單 event
    let user = "alice";
    info!(user, "logged in");

    // 結構化欄位（key=value）
    info!(method = "GET", path = "/api/users", status = 200, "request handled");

    // span：一段「事件範圍」
    let span = tracing::info_span!("processing", request_id = "abc123");
    let _enter = span.enter();
    info!("inside span");
    fetch_user(7).ok();
    drop(_enter); // 離開 span
    info!("outside span");

    // 批次：#[instrument] 自動 span
    process_batch(&[1, 2, 0, 999, 3]);

    // 不同 level
    debug!("only visible at debug");
    info!("info");
    warn!("warn");
    error!("error");
}
