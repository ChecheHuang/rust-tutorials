//! rgb-core: device 抽象 + 效果引擎。
//!
//! 設計分層（呼應第 40 章 Clean Architecture）：
//!
//! ```text
//!   adapter (Tauri commands / CLI)
//!     ↓ Arc<DeviceManager>
//!   application (DeviceManager, EffectEngine)
//!     ↓ Arc<dyn Device>
//!   domain (Device trait, Color, Effect)
//!     ↓
//!   infrastructure (RazerHuntsman, FakeDevice, ...)
//! ```

pub mod color;
pub mod device;
pub mod effects;
pub mod manager;
pub mod profile;

pub use color::Color;
pub use device::{Device, DeviceInfo, DeviceKind};
pub use manager::DeviceManager;
pub use profile::Profile;
