# 51. Tauri 入門

> 範圍：架構、`#[tauri::command]`、State、event、跟 Electron 的取捨

## Tauri vs Electron 一句話

| | Electron | Tauri |
|---|----------|-------|
| Backend | Node.js | Rust |
| Frontend renderer | 內嵌 Chromium | 系統 webview |
| Binary 大小 | ~150 MB | ~10 MB |
| 記憶體 | 高（每 window 一個 Chrome） | 低（共用系統 webview） |
| Web 渲染差異 | 一致（Chrome） | 系統各異（Edge / WebKit / WebKitGTK） |
| 學習曲線 | JS 友好 | 需要 Rust |

**選擇**：要極致小、安全、能寫 Rust → Tauri；要瀏覽器 100% 一致 / 純 JS team → Electron。

## 架構

```
┌──────────────────────────────────────┐
│  Frontend (HTML/JS/Vue/React/...)    │   ← 瀏覽器 webview
│        invoke('cmd', args)           │
└──────────────┬───────────────────────┘
               │ IPC
┌──────────────▼───────────────────────┐
│  Rust core                           │   ← 你的業務邏輯
│   #[tauri::command]                  │
│   State<T>                           │
│   emit / listen                      │
└──────────────────────────────────────┘
```

## 建專案

```bash
# 互動式
npm create tauri-app@latest
# 選 frontend：vanilla / react / vue / svelte / solid
# 選 package manager：npm / pnpm / yarn

# 跑 dev
cd app && npm install
npm run tauri dev
```

dev mode 下 frontend 用 vite hot reload、Rust 改了重 build。

## `#[tauri::command]` — 暴露 Rust 函式給前端

```rust
#[tauri::command]
fn greet(name: &str) -> GreetResponse { ... }

#[tauri::command]
async fn slow_op(ms: u64) -> Result<String, String> { ... }
```

註冊：
```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet, slow_op])
    .run(tauri::generate_context!())?;
```

前端：
```js
import { invoke } from '@tauri-apps/api/core'
const r = await invoke('greet', { name: 'Bob' })
// invoke 失敗 → throws；返回 Promise<T>
```

## 參數與回傳

- 參數名 **camelCase**（JS 端）vs **snake_case**（Rust 端）：Tauri 會自動轉。
- 回傳要 `serde::Serialize`。錯誤要 `Result<T, E>`，E 要 `Serialize`。
- async command 不阻塞 webview。

## State 共享

```rust
struct AppConfig { db_url: String }

tauri::Builder::default()
    .manage(AppConfig { db_url: env!("DB_URL").into() })
    ...

#[tauri::command]
fn config(state: tauri::State<AppConfig>) -> String {
    state.db_url.clone()
}
```

可變 state 用 `Mutex<T>` / `RwLock<T>` 包：

```rust
.manage(Mutex::new(Counter(0)))

#[tauri::command]
fn incr(state: State<Mutex<Counter>>) -> u32 {
    let mut c = state.lock().unwrap();
    c.0 += 1;
    c.0
}
```

## Event：雙向通訊

Rust → JS：
```rust
#[tauri::command]
fn start_job(app: tauri::AppHandle) {
    tokio::spawn(async move {
        for i in 0..100 {
            app.emit("progress", i).ok();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
}
```

JS：
```js
import { listen } from '@tauri-apps/api/event'
const un = await listen('progress', e => console.log(e.payload))
// 結束：un()
```

JS → Rust 也可以 emit，少用；通常用 invoke 更直白。

## Security model

Tauri 預設**封閉**：前端不能呼叫任何東西，要顯式 allow。

`src-tauri/capabilities/default.json`：
```json
{
  "permissions": ["core:default", "shell:allow-open"]
}
```

對應 Electron 的「nodeIntegration = true 全開」災難，Tauri 預設安全。

## webview 差異

| OS | webview |
|----|---------|
| Windows 10+ | WebView2（Chromium-based） |
| macOS | WKWebView（Safari WebKit） |
| Linux | WebKitGTK |

**測試策略**：每個 OS 都實機跑——尤其 CSS / JS API 細節差異。

## 跨平台 build

```bash
npm run tauri build               # 當前平台
# Windows: .msi + .exe
# macOS:   .dmg + .app
# Linux:   .deb / .AppImage / .rpm
```

跨編譯比較痛——常用 GitHub Actions matrix（見第 47 章）。

## debug

- Rust 端：正常 `println!` / `tracing`，dev mode 看到
- JS 端：webview devtools（dev 自動開、prod 預設關）
- 加 `tauri.conf.json` 的 `withGlobalTauri: true` 讓 `window.__TAURI__` 可探

## 常見陷阱

1. **command 參數命名** — Rust snake_case ↔ JS camelCase 自動轉，但 string literal 不會。
2. **跨 thread state** — `Mutex` / `RwLock` 必要；`std::sync::Mutex` 在 async command 內 hold 過 await point 會出事，async 路徑用 `tokio::sync::Mutex`。
3. **panic** — Rust 端 panic 不會直接告知前端，整個 webview 卡住。command 內用 `Result`，避免 panic。
4. **CORS / CSP** — Tauri 透過 `tauri://` 提供 asset，外部 fetch 要設 CSP / allowlist。
5. **bundle 失敗** — 缺 icon / signing cert；按 `tauri.conf.json` 一一補。
6. **macOS notarization** — production 要 Apple Developer 簽名 + notarize 才能跑。
7. **不要在 command 用 `std::thread::sleep`** — 阻塞 tokio runtime；用 `tokio::time::sleep`。
8. **大檔案傳輸** — IPC 不適合 100MB+ binary；用 file path + read directly。

## 練習

1. 跑 `npm create tauri-app`，把範例的 `greet` 改成回傳系統資訊（OS、CPU 核數、記憶體）。
2. 加一個長時間任務 command（5 秒），用 event 每秒推進度。
3. 用 `Mutex` 包一個 counter，前端兩個按鈕 +1 / -1。
4. 比較 dev mode vs `npm run tauri build` 的 binary 大小。

## 延伸閱讀

- [Tauri 2.0 Docs](https://tauri.app/)
- [Tauri vs Electron benchmarks](https://github.com/tauri-apps/benchmark_results)
