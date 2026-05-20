use serde::{Deserialize, Serialize};
use std::path::Path;

/// 使用者 profile：每個 device 一個效果設定 + 巨集綁定。
/// 對應第 24 章 config + 第 34 章 serde。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub device_effects: Vec<DeviceEffect>,
    pub macros: Vec<MacroBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceEffect {
    pub device_id: String,
    pub effect: EffectSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectSpec {
    Static { rgb: [u8; 3] },
    Breathe { a: [u8; 3], b: [u8; 3], period_ms: u64 },
    Rainbow { period_ms: u64 },
    Comet { rgb: [u8; 3], period_ms: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MacroBinding {
    pub trigger: String,        // 例如 "ctrl+shift+m"
    pub actions: Vec<MacroAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MacroAction {
    KeyPress { key: String, ms: u64 },
    Delay { ms: u64 },
    Text { text: String },
    Command { command: String, args: Vec<String> },
}

impl Profile {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&s)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let s = toml::to_string_pretty(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }
}

impl From<EffectSpec> for crate::effects::Effect {
    fn from(s: EffectSpec) -> Self {
        use crate::Color;
        match s {
            EffectSpec::Static { rgb } => Self::Static(Color::new(rgb[0], rgb[1], rgb[2])),
            EffectSpec::Breathe { a, b, period_ms } => Self::Breathe {
                a: Color::new(a[0], a[1], a[2]),
                b: Color::new(b[0], b[1], b[2]),
                period_ms,
            },
            EffectSpec::Rainbow { period_ms } => Self::Rainbow { period_ms },
            EffectSpec::Comet { rgb, period_ms } => Self::Comet {
                color: Color::new(rgb[0], rgb[1], rgb[2]),
                period_ms,
            },
        }
    }
}
