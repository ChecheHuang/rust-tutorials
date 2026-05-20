use crate::Color;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceKind {
    Keyboard,
    Mouse,
    Mousepad,
    Headset,
    Generic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub kind: DeviceKind,
    /// LED 個數（zone 或單顆）
    pub led_count: usize,
}

/// 統一 device 抽象——對 RGB 與巨集硬體一視同仁。
/// 對應第 12 章 trait + 第 40 章 port。
#[async_trait]
pub trait Device: Send + Sync {
    fn info(&self) -> &DeviceInfo;

    /// 設整顆裝置同色（最簡單的 path）。
    async fn set_all(&self, color: Color) -> anyhow::Result<()>;

    /// 設每顆 LED 不同色。長度必須等於 `led_count`。
    async fn set_leds(&self, colors: &[Color]) -> anyhow::Result<()>;

    /// 預設效果（裝置韌體內建的 wave / breathe 等），key 由廠商定義。
    /// 無需支援的可以回 NotSupported error。
    async fn set_firmware_effect(&self, _name: &str) -> anyhow::Result<()> {
        anyhow::bail!("firmware effects not supported by {}", self.info().name);
    }
}

// ── 範例 1：FakeDevice ──────────────────────────────────────
/// 沒有硬體時可用——測試友好、stdout 印 LED 狀態。
pub struct FakeDevice {
    info: DeviceInfo,
    state: Mutex<Vec<Color>>,
}

impl FakeDevice {
    pub fn new(name: &str, led_count: usize) -> Self {
        Self {
            info: DeviceInfo {
                id: format!("fake:{name}"),
                name: name.into(),
                vendor: "fake".into(),
                kind: DeviceKind::Generic,
                led_count,
            },
            state: Mutex::new(vec![Color::BLACK; led_count]),
        }
    }
}

#[async_trait]
impl Device for FakeDevice {
    fn info(&self) -> &DeviceInfo { &self.info }

    async fn set_all(&self, color: Color) -> anyhow::Result<()> {
        let mut s = self.state.lock().unwrap();
        for c in s.iter_mut() { *c = color; }
        tracing::info!(?color, name = %self.info.name, "set_all");
        Ok(())
    }

    async fn set_leds(&self, colors: &[Color]) -> anyhow::Result<()> {
        if colors.len() != self.info.led_count {
            anyhow::bail!("expected {} leds, got {}", self.info.led_count, colors.len());
        }
        let mut s = self.state.lock().unwrap();
        s.copy_from_slice(colors);
        Ok(())
    }
}

// ── 範例 2：HidDevice 骨架（actor pattern） ────────────────
/// 真實 HID 裝置——具體 protocol 因廠商而異。
///
/// `hidapi::HidDevice` 對某些平台不是 Send，且寫入是阻塞 IO；
/// 正確做法：把 device 鎖在專屬 thread 內，主 async code
/// 透過 mpsc 送 command 進去。這裡示意 actor pattern 骨架。
pub struct HidRgbDevice {
    info: DeviceInfo,
    /// 把 colors 送進 IO thread；IO thread 用 framer 包成 HID frame。
    tx: tokio::sync::mpsc::Sender<Vec<Color>>,
}

impl HidRgbDevice {
    /// 啟動一個 thread 持有 `device`，從 channel 拉 frame 出來寫。
    pub fn spawn(
        info: DeviceInfo,
        device: hidapi::HidDevice,
        framer: impl Fn(&[Color]) -> Vec<u8> + Send + 'static,
    ) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<Color>>(16);
        std::thread::spawn(move || {
            while let Some(colors) = rx.blocking_recv() {
                let frame = framer(&colors);
                if let Err(e) = device.write(&frame) {
                    tracing::warn!(?e, "hid write failed");
                }
            }
        });
        Self { info, tx }
    }
}

#[async_trait]
impl Device for HidRgbDevice {
    fn info(&self) -> &DeviceInfo { &self.info }

    async fn set_all(&self, color: Color) -> anyhow::Result<()> {
        let colors = vec![color; self.info.led_count];
        self.set_leds(&colors).await
    }

    async fn set_leds(&self, colors: &[Color]) -> anyhow::Result<()> {
        if colors.len() != self.info.led_count {
            anyhow::bail!("expected {} leds, got {}", self.info.led_count, colors.len());
        }
        self.tx
            .send(colors.to_vec())
            .await
            .map_err(|_| anyhow::anyhow!("device IO thread closed"))?;
        Ok(())
    }
}
