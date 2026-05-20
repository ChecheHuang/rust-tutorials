use crate::{Color, Device};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// 效果定義（純資料）— 對應第 08 章 enum。
#[derive(Clone, Debug)]
pub enum Effect {
    /// 整片同色
    Static(Color),
    /// 兩色之間 sin 曲線呼吸
    Breathe { a: Color, b: Color, period_ms: u64 },
    /// 沿 LED 走一道彩虹
    Rainbow { period_ms: u64 },
    /// 亮一顆、跑一圈
    Comet { color: Color, period_ms: u64 },
}

/// 跑效果直到收到停止訊號。對應第 27 章 tokio + ch29 watch channel。
pub async fn run_effect(
    device: Arc<dyn Device>,
    effect: Effect,
    mut stop: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let n = device.info().led_count;
    let start = std::time::Instant::now();
    let mut tick = tokio::time::interval(Duration::from_millis(30));
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let t = start.elapsed().as_millis() as f32;
                let frame = render(&effect, n, t);
                device.set_leds(&frame).await?;
            }
            changed = stop.changed() => {
                if changed.is_err() || *stop.borrow() {
                    let off = vec![Color::BLACK; n];
                    device.set_leds(&off).await.ok();
                    return Ok(());
                }
            }
        }
    }
}

fn render(effect: &Effect, n: usize, t_ms: f32) -> Vec<Color> {
    match effect {
        Effect::Static(c) => vec![*c; n],
        Effect::Breathe { a, b, period_ms } => {
            let phase = (t_ms / *period_ms as f32 * std::f32::consts::TAU).sin() * 0.5 + 0.5;
            vec![a.lerp(*b, phase); n]
        }
        Effect::Rainbow { period_ms } => {
            (0..n).map(|i| {
                let phase = (t_ms / *period_ms as f32 + i as f32 / n as f32) % 1.0;
                Color::from_hsv(phase * 360.0, 1.0, 1.0)
            }).collect()
        }
        Effect::Comet { color, period_ms } => {
            let head = ((t_ms / *period_ms as f32 * n as f32) as usize) % n;
            (0..n).map(|i| {
                let dist = (i + n - head) % n;
                let fade = (1.0 - dist as f32 / 5.0).clamp(0.0, 1.0);
                color.lerp(Color::BLACK, 1.0 - fade)
            }).collect()
        }
    }
}
