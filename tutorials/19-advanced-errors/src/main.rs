use anyhow::{Context, Result};
use std::fs;
use thiserror::Error;

// ── thiserror：library 端的具體錯誤型別 ─────────────────────
#[derive(Debug, Error)]
pub enum DbError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("user {id} not found")]
    NotFound { id: u64 },
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required field: {0}")]
    Missing(&'static str),

    #[error("parse error")]
    Parse(#[from] std::num::ParseIntError),
}

// ── library 端：用 thiserror 暴露具體錯誤型別 ───────────────
fn fetch_user(id: u64) -> Result<String, DbError> {
    if id == 0 {
        return Err(DbError::NotFound { id });
    }
    if id == 999 {
        return Err(DbError::Connection("timed out".into()));
    }
    Ok(format!("user-{id}"))
}

fn load_port() -> Result<u16, ConfigError> {
    let s = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let port: u16 = s.parse()?; // ParseIntError → ConfigError
    Ok(port)
}

// ── application 端：用 anyhow 收齊一切錯誤 + 加 context ────
fn application_main() -> Result<()> {
    let port = load_port().context("failed to load port")?;
    println!("port = {port}");

    let user = fetch_user(0).context("during initial fetch")?;
    println!("user = {user}");
    Ok(())
}

// ── 串多層 context ─────────────────────────────────────────
fn load_config(path: &str) -> Result<String> {
    let bytes = fs::read(path)
        .with_context(|| format!("reading config from '{path}'"))?;
    let s = String::from_utf8(bytes)
        .with_context(|| format!("config file '{path}' contains invalid UTF-8"))?;
    Ok(s)
}

fn main() {
    // ── library 風格 ────────────────────────────────────────
    match fetch_user(7) {
        Ok(u) => println!("ok: {u}"),
        Err(e) => println!("err: {e}"),
    }

    let err = fetch_user(0).unwrap_err();
    println!("matchable error: {err:?}");
    if matches!(err, DbError::NotFound { id: 0 }) {
        println!("  → caller can pattern-match on specific variant");
    }

    // ── application 風格：anyhow ────────────────────────────
    println!("\n--- anyhow chain demo ---");
    match application_main() {
        Ok(_) => println!("OK"),
        Err(e) => {
            // anyhow 的 Display：印頂層；Debug：印整條 chain
            println!("Error: {e}");
            println!("\nDebug (full chain):\n{e:?}");

            // 走 chain
            println!("\nWalk chain:");
            for (i, cause) in e.chain().enumerate() {
                println!("  {i}: {cause}");
            }
        }
    }

    // ── multi-level context ────────────────────────────────
    println!("\n--- multi-level context ---");
    if let Err(e) = load_config("/nonexistent.toml") {
        for (i, c) in e.chain().enumerate() {
            println!("  {i}: {c}");
        }
    }
}
