// CLI demo：用 FakeDevice 跑效果，方便沒硬體也能驗證骨架。
//
// rgb-cli list
// rgb-cli static fake:DemoKeyboard 255 0 0
// rgb-cli rainbow fake:DemoKeyboard 3000
// rgb-cli stop fake:DemoKeyboard

use anyhow::Result;
use clap::{Parser, Subcommand};
use rgb_core::{effects::Effect, Color, DeviceManager};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "rgb-cli")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 列出所有 device（demo 用 Fake）
    List,
    /// 靜態色：rgb-cli static <id> R G B
    Static {
        id: String,
        r: u8,
        g: u8,
        b: u8,
    },
    /// 彩虹：rgb-cli rainbow <id> [period_ms]
    Rainbow {
        id: String,
        #[arg(default_value_t = 3000)]
        period_ms: u64,
    },
    /// 呼吸：rgb-cli breathe <id> R G B
    Breathe {
        id: String,
        r: u8,
        g: u8,
        b: u8,
        #[arg(default_value_t = 2000)]
        period_ms: u64,
    },
    /// 停效果
    Stop { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let mgr = Arc::new(DeviceManager::new());
    mgr.populate_demo().await;

    match cli.cmd {
        Cmd::List => {
            for info in mgr.list().await {
                println!("[{}] {} ({:?}) — {} LEDs",
                    info.id, info.name, info.kind, info.led_count);
            }
        }
        Cmd::Static { id, r, g, b } => {
            mgr.start_effect(&id, Effect::Static(Color::new(r, g, b))).await?;
            println!("static {r} {g} {b} on {id} (Ctrl+C to quit)");
            tokio::signal::ctrl_c().await?;
            mgr.stop_effect(&id).await;
        }
        Cmd::Rainbow { id, period_ms } => {
            mgr.start_effect(&id, Effect::Rainbow { period_ms }).await?;
            println!("rainbow period={period_ms}ms on {id} (Ctrl+C to quit)");
            tokio::signal::ctrl_c().await?;
            mgr.stop_effect(&id).await;
        }
        Cmd::Breathe { id, r, g, b, period_ms } => {
            mgr.start_effect(&id, Effect::Breathe {
                a: Color::new(r, g, b),
                b: Color::BLACK,
                period_ms,
            }).await?;
            println!("breathe on {id}");
            tokio::signal::ctrl_c().await?;
            mgr.stop_effect(&id).await;
        }
        Cmd::Stop { id } => {
            mgr.stop_effect(&id).await;
            println!("stopped {id}");
        }
    }
    Ok(())
}
