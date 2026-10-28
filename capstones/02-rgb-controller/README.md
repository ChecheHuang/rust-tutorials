# Capstone 2 — RGB / 巨集控制器

跨品牌、跨平台的 RGB 燈光與巨集統一控制工具，靈感來自 [OpenRGB](https://openrgb.org/)。整合 Part 1–7 主線 + Part 8 桌面開發，是整套教材最綜合的 capstone。

> 🧸 想先用故事直覺理解這個專案？看 [STORY.md](./STORY.md)（小學生都懂的比喻版）。

> 上層規劃見 [../../ROADMAP.md](../../ROADMAP.md)。

## 範圍

1. **裝置偵測** — 啟動時 enumerate 已連接的 USB HID 裝置，識別已支援的品牌 / 型號
2. **RGB 控制** — 純色、漸層、波浪、reactive（隨按鍵亮起）、自訂動畫
3. **巨集** — 錄製鍵盤輸入序列，綁定到指定按鍵，可設定延遲與重複（範例目前只有資料模型，player／錄製留給你實作，見下方「你要補的部分」）
4. **設定持久化** — 每個裝置一個 profile（TOML / JSON），支援匯出 / 匯入
5. **系統整合** — 系統托盤常駐、開機自啟、最小化到托盤
6. **跨平台** — Windows、macOS、Linux 三平台都能 build + 簽章 + 發佈

## 學習目標

- **Tauri IPC**：前端（Vue/React）↔ Rust core 的雙向通訊
- **`hidapi`**：讀取 input report、發送 output report、處理 feature report
- **Trait abstraction**：定義 `Device` trait，讓多個廠商 driver 共用一致介面
- **Trait object dispatch**：執行期透過 `Arc<dyn Device>` 處理動態 device list（用 `Arc` 而非 `Box`：同一個 device 要同時給 manager 和背景的效果 task 持有）
- **Async 事件流**：USB 裝置插拔事件 → `tokio::broadcast` → 前端通知
- **設定持久化**：`serde` + `directories` crate（取得跨平台設定目錄）
- **跨平台簽章**：Windows code sign（`signtool`）、macOS notarize（`xcrun notarytool`）、Linux AppImage

## 架構草圖

```
┌─────────────────────────────────────────────────────┐
│  前端 (Vue + TS)                                      │
│  - Device list / Color picker / Macro editor          │
└────────────────┬─────────────────────────────────────┘
                 │ Tauri IPC (invoke / event)
┌────────────────▼─────────────────────────────────────┐
│  Tauri Core (Rust)                                    │
│  ┌──────────────────────────────────────────────┐    │
│  │  commands.rs  (Tauri command handlers)        │    │
│  ├──────────────────────────────────────────────┤    │
│  │  device_manager.rs                            │    │
│  │   - enumerate / hotplug watcher               │    │
│  │   - HashMap<id, Arc<dyn Device>>              │    │
│  ├──────────────────────────────────────────────┤    │
│  │  drivers/                                     │    │
│  │   - razer.rs   (impl Device)                  │    │
│  │   - corsair.rs (impl Device)                  │    │
│  │   - logitech.rs (impl Device)                 │    │
│  ├──────────────────────────────────────────────┤    │
│  │  config.rs  (TOML 持久化)                     │    │
│  │  macro_engine.rs                              │    │
│  └──────────────────────────────────────────────┘    │
└────────────────┬─────────────────────────────────────┘
                 │ hidapi
┌────────────────▼─────────────────────────────────────┐
│  USB HID 裝置（鍵盤 / 滑鼠 / 燈條）                     │
└──────────────────────────────────────────────────────┘
```

## 前置章節

| 必須 | 章節 |
|------|------|
| 必須 | Part 1–3 全部（語言核心 + 工程化基礎） |
| 必須 | 26–29（async / Tokio / Send-Sync / channels） |
| 必須 | 34 serde-json（設定序列化） |
| 必須 | Part 8 全部（Tauri + hidapi + 打包） |
| 建議 | Capstone 1 先做完（CLI 版的 RGB 工具當作熱身） |

## 建議實作節奏

| 階段 | 內容 |
|------|------|
| 1 | **CLI prototype**：先不上 Tauri，純 CLI 列出 HID 裝置、改一個鍵盤的單色 RGB |
| 2 | **Trait 抽象**：把單一 driver 抽出 `Device` trait，加第二個廠商驗證抽象正確 |
| 3 | **Tauri 殼**：搭 Vue + Tauri 框架，把 CLI 功能搬到 IPC command |
| 4 | **熱插拔**：用 `tokio::task` 偵測裝置變化，透過 event 通知前端 |
| 5 | **動畫**：實作 effect runner（漸層 / 波浪），跑在獨立 task |
| 6 | **巨集**：錄製 / 播放 / 綁定 |
| 7 | **持久化 + 托盤 + 自啟** |
| 8 | **跨平台簽章與發佈**（GitHub Releases 自動 build matrix） |

## 評估標準

- [ ] 至少支援 **2 個品牌** 的鍵盤 RGB 控制（可挑社群已逆向出 protocol 的型號）
- [ ] 至少 1 個巨集情境可用（錄製 → 綁定 → 觸發）
- [ ] 設定可正確持久化、重啟後恢復
- [ ] 偵測到裝置插拔時，UI 自動更新
- [ ] 三大平台都能 build + 簽章 + 安裝執行
- [ ] CI 自動產出 release artifact
- [ ] README 附 demo gif 與支援裝置清單

## 法律與道德提醒

- 與廠商 protocol 互通屬合理使用（reverse engineering for interoperability），但**不得**包含廠商私有韌體
- 直接寫 USB output report 有 brick 裝置風險，建議在便宜的測試鍵盤上開發
- 不要用偽造的 VID / PID 假冒原廠軟體

---

## 本目錄附帶的範例實作

### 目前提供

```
src/
  lib.rs          ← module 索引
  color.rs        ← RGB / HSV / lerp
  device.rs       ← Device trait + FakeDevice + HidRgbDevice actor 骨架
  effects.rs      ← Static / Breathe / Rainbow / Comet runner
  manager.rs      ← DeviceManager（註冊、列出、啟停效果）
  profile.rs      ← 使用者 profile（serde + TOML）+ 巨集綁定資料模型（只有資料結構，還不會播放）
  main.rs         ← rgb-cli：用 FakeDevice 驗證骨架
```

跑：

```bash
cd capstones/02-rgb-controller
cargo run -- list
cargo run -- rainbow fake:DemoKeyboard 3000
cargo run -- static fake:DemoMouse 0 200 255
```

效果指令會一直跑到你按 **Ctrl+C**。注意 `rgb-cli stop <id>` 目前只是 demo：每次執行 `rgb-cli` 都是新的 process、各自有一份 `DeviceManager`，所以在另一個終端機下 `stop` **停不到**正在跑的效果。要跨 process 控制，得有一個常駐的 manager（Tauri app 或 daemon）讓 CLI 去連，見下方第 8 點。

### 你要補的部分

1. **真實 HID driver**：實作具體廠商 framer（HID frame packing）並透過 `HidRgbDevice::spawn` 註冊
2. **Tauri shell**：`npm create tauri-app`，把 `DeviceManager` 用 `tauri::State<Arc<DeviceManager>>` 注入，commands 對應 list/set_color/start_effect/stop_effect
3. **Hotplug**：背景 task 每秒掃 `HidApi::new()`，diff 出 device 變化、emit `device:connected` / `device:disconnected`
4. **Macro engine**：定義 `MacroAction` 已在 `profile.rs`，需要實作 player（觸發 → key 模擬 → delay）；player 用 `enigo` crate 跨平台模擬鍵盤
5. **巨集錄製**：global shortcut 開錄製模式、用 `rdev` crate 監聽全域 key 事件記錄成 `Vec<MacroAction>`
6. **Tray + 自啟動**：見第 52 章
7. **打包簽章**：見第 54 章
8. **跨 process 控制**：讓 `rgb-cli stop` 真的能停掉別的 process 在跑的效果——做法是讓 manager 常駐（Tauri app 或 daemon），CLI 透過 local socket / named pipe 或 HTTP（第 32 章 axum）送指令過去

### 為什麼這樣切

- `Device` trait 是 **port**（第 40 章），廠商實作是 adapter，UI / CLI / Tauri 都用同一個 manager
- effect runner 跑在 tokio task、用 watch channel 停下（第 27、29 章）
- HID IO 是 blocking → actor pattern 隔離（一個 std::thread 持有 device + mpsc channel）
- profile 用 serde + TOML（第 24、34 章）
- 動畫 frame 每 30ms 一次 → tokio::interval（第 27 章）

### 進階挑戰

- effect graph：用戶連接多個效果節點（generator → blend → output），跑成自訂特效
- 跨裝置同步：Color picker 改色一次套到全部裝置
- audio reactive：抓 system audio peak，亮度跟著節奏
- web companion：把 manager 開 axum endpoint（第 32 章），手機網頁遙控
- ML 主題：用 `candle` 在 image 上抽 dominant color，動態套到燈光
