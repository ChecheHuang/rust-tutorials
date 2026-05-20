use crate::device::{Device, FakeDevice};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

/// 管理所有 device + 各 device 上跑的效果 task。
pub struct DeviceManager {
    devices: RwLock<HashMap<String, Arc<dyn Device>>>,
    /// device_id → stop sender，發訊號讓對應效果 task 收尾
    effect_stops: RwLock<HashMap<String, watch::Sender<bool>>>,
}

impl Default for DeviceManager {
    fn default() -> Self { Self::new() }
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(HashMap::new()),
            effect_stops: RwLock::new(HashMap::new()),
        }
    }

    /// 註冊裝置（運行時動態加，例如 hotplug）
    pub async fn register(&self, dev: Arc<dyn Device>) {
        let id = dev.info().id.clone();
        self.devices.write().await.insert(id, dev);
    }

    pub async fn list(&self) -> Vec<crate::DeviceInfo> {
        self.devices.read().await.values()
            .map(|d| d.info().clone())
            .collect()
    }

    pub async fn get(&self, id: &str) -> Option<Arc<dyn Device>> {
        self.devices.read().await.get(id).cloned()
    }

    /// 啟動 device 上的效果。若已有效果在跑，先停舊的。
    pub async fn start_effect(&self, id: &str, effect: crate::effects::Effect) -> anyhow::Result<()> {
        self.stop_effect(id).await;

        let device = self.get(id).await
            .ok_or_else(|| anyhow::anyhow!("device not found: {id}"))?;
        let (stop_tx, stop_rx) = watch::channel(false);
        self.effect_stops.write().await.insert(id.into(), stop_tx);

        tokio::spawn(async move {
            if let Err(e) = crate::effects::run_effect(device, effect, stop_rx).await {
                tracing::error!(?e, "effect task failed");
            }
        });
        Ok(())
    }

    pub async fn stop_effect(&self, id: &str) {
        if let Some(tx) = self.effect_stops.write().await.remove(id) {
            let _ = tx.send(true);
        }
    }

    /// 用 FakeDevice 灌兩個 demo device。
    pub async fn populate_demo(&self) {
        self.register(Arc::new(FakeDevice::new("DemoKeyboard", 87))).await;
        self.register(Arc::new(FakeDevice::new("DemoMouse", 7))).await;
    }
}
