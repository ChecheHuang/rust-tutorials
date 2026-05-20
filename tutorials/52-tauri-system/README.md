# 52. Tauri 系統整合

> 範圍：plugin 系統、檔案 / dialog / 通知 / 剪貼簿、tray icon、global shortcut、window 管理、auto-start

## Plugin 系統

Tauri 2 把大多數系統能力拆成 plugin（v1 是 allowlist 內建）。要用先安裝 + 註冊 + permission。

```bash
# Rust 端
cargo tauri add fs dialog notification clipboard-manager shell global-shortcut

# 或手動 cargo add
```

`Cargo.toml`：
```toml
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
...
```

註冊：
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    ...
```

權限：`src-tauri/capabilities/default.json`
```json
{
  "permissions": [
    "core:default",
    "fs:allow-read-text-file",
    "dialog:allow-open"
  ]
}
```

## 檔案系統

JS 端：
```js
import { readTextFile, writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs'
const txt = await readTextFile('config.toml', { baseDir: BaseDirectory.AppConfig })
await writeTextFile('notes.txt', 'hi', { baseDir: BaseDirectory.AppData })
```

或寫 Rust command 用 `tokio::fs`：
```rust
#[tauri::command]
async fn save_file(path: String, contents: String) -> Result<(), String> {
    tokio::fs::write(&path, contents).await.map_err(|e| e.to_string())
}
```

⚠️ **scope**：plugin 預設只允許特定目錄。要放開：`capabilities/*.json` 內 `fs:scope`。**生產不要 allow `**`**。

## Dialog（檔案 / 開啟對話框）

```js
import { open, save, message, confirm } from '@tauri-apps/plugin-dialog'

const file = await open({ multiple: false, filters: [{ name: 'JSON', extensions: ['json'] }] })
const path = await save({ defaultPath: 'export.json' })
await message('Done!', { kind: 'info' })
const ok = await confirm('Delete?', { kind: 'warning' })
```

對話框跑在原生 thread，不會卡 webview。

## 系統通知

```js
import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification'

if (!(await isPermissionGranted())) await requestPermission()
sendNotification({ title: 'Build done', body: 'Compile success in 3.4s' })
```

注意：
- macOS 需要在 Info.plist 加 NSUserNotification 描述
- Windows 11 Action Center / Linux libnotify 都會用

## 剪貼簿

```js
import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager'
await writeText('copied!')
const t = await readText()
```

權限要 `clipboard-manager:allow-read-text` / `allow-write-text`。

## Global shortcut

```rust
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

app.global_shortcut()
    .on_shortcut(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM), |_app, _sh, _ev| {
        // 觸發
    })?;
```

或 JS 端用 plugin API 註冊。

⚠️ **平台衝突**：Ctrl+Shift+M 在某些 OS 已被佔用；test 各 OS。

## Tray icon

```rust
TrayIconBuilder::new()
    .menu(&menu)
    .on_menu_event(|app, ev| match ev.id.as_ref() { ... })
    .on_tray_icon_event(|tray, ev| match ev { ... })
    .build(app)?;
```

加圖示在 `src-tauri/icons/`。

實務：
- macOS 用 template image（自動配 light/dark）
- Windows 16×16 / 32×32 ICO
- Linux StatusNotifier protocol（KDE / GNOME 部分支援）

## Window 管理

```rust
// 從 command 內
#[tauri::command]
fn open_settings(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_focus();
    } else {
        tauri::WebviewWindowBuilder::new(&app, "settings", tauri::WebviewUrl::App("settings.html".into()))
            .title("Settings")
            .inner_size(600.0, 400.0)
            .build().unwrap();
    }
}
```

多 window 各自獨立 webview，state 透過 `tauri::State` / IPC 共享。

## Auto-start

```rust
.plugin(tauri_plugin_autostart::init(
    MacosLauncher::LaunchAgent,
    None,
))
```

```js
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
```

- Windows：寫 Run 機碼
- macOS：建 LaunchAgent plist
- Linux：autostart .desktop

## Deep link（自訂 URL scheme）

```toml
# tauri.conf.json
{
  "plugins": {
    "deep-link": {
      "schemes": ["mycoolapp"]
    }
  }
}
```

用 `mycoolapp://path?x=1` 從瀏覽器 / OS 啟動你的 app。OAuth callback、登入跳轉常用。

## OS-specific code

```rust
#[cfg(target_os = "windows")]
fn pin_to_taskbar() { ... }

#[cfg(target_os = "macos")]
fn set_dock_icon() { ... }
```

或用 plugin 抽象掉細節。

## 常見陷阱

1. **plugin 沒註冊** — JS 呼叫 `not found`；忘了 `.plugin(...)`。
2. **permission 沒開** — invoke 失敗 `not allowed`；對應到 capabilities。
3. **fs scope 過寬** — `**` 把整顆硬碟 expose，極危險。
4. **windows 視窗 thread** — 某些 Windows API 必須在 UI thread；用 `app.run_on_main_thread`。
5. **tray icon Linux** — KDE / GNOME 對 StatusNotifier 支援度差異，要 fallback。
6. **deep link iOS / Android** — 須在 platform config 加 universal link / app link。
7. **macOS notarization** — 必要，否則 Gatekeeper 擋。
8. **screen capture / accessibility** — macOS 跟 Wayland 需要額外 permission flow。

## 練習

1. 寫一個剪貼簿管家：偵測剪貼簿變化，存 history list，shortcut 叫出來。
2. tray icon 加「最近 5 個檔案」submenu。
3. 接 deep link `mycoolapp://share?url=...` 開新 window 顯示。
4. 加 auto-start，且第一次啟動詢問使用者要不要開機自啟。

## 延伸閱讀

- [Tauri Plugins](https://tauri.app/v1/api/js/)
- [Tauri 2 Permissions](https://tauri.app/security/permissions/)
