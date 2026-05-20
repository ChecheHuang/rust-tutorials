use serde::Deserialize;
use std::env;

// ── 設定資料結構 ────────────────────────────────────────────
#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
    #[serde(default)]
    features: FeatureFlags,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
    #[serde(default = "default_pool_size")]
    pool_size: u32,
}

fn default_pool_size() -> u32 {
    10
}

#[derive(Debug, Deserialize, Default)]
struct FeatureFlags {
    #[serde(default)]
    new_ui: bool,
    #[serde(default)]
    beta: bool,
}

// ── 手刻：file + env override ───────────────────────────────
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // 1. 從內嵌的 default TOML 開始
    const DEFAULT_TOML: &str = r#"
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgres://localhost/dev"
"#;

    let mut cfg: Config = toml::from_str(DEFAULT_TOML)?;

    // 2. 環境變數 override（簡單版）
    if let Ok(p) = env::var("APP_SERVER_PORT") {
        cfg.server.port = p.parse()?;
    }
    if let Ok(url) = env::var("DATABASE_URL") {
        cfg.database.url = url;
    }
    if env::var("APP_FEATURES_BETA").ok().as_deref() == Some("true") {
        cfg.features.beta = true;
    }

    Ok(cfg)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = load_config()?;
    println!("Loaded config:");
    println!("{cfg:#?}");

    println!("\n試試：");
    println!("  APP_SERVER_PORT=9000 cargo run");
    println!("  APP_FEATURES_BETA=true cargo run");
    println!("  DATABASE_URL=postgres://prod cargo run");

    Ok(())
}
