# 故事版：開在大樓裡的小書店 🏢

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「小書店開在一棟大樓裡，想借用大樓的倉庫、廣播、留言板和門鈴」的故事，幫你「感覺」出 Tauri plugin、權限、檔案系統與 scope、dialog、通知、剪貼簿、global shortcut、tray icon、視窗管理、開機自啟、deep link 和平台專屬程式碼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 想用大樓的設施，要先辦兩件事

小書店的店長叫圓圓，她的書店開在一棟大樓的二樓。

大樓有很多好用的設施：地下室倉庫、全棟廣播、一樓公用留言板……可是管理處說：「想用哪一樣，你要做**兩件事**：」

1. **去領那樣設施的鑰匙**，並且在店門口登記「本店有用這個」。
2. **填一張許可單**，寫清楚你要用它的哪個功能。

圓圓一口氣領了七把鑰匙：倉庫、表單窗口、廣播、留言板、外部連結、門鈴暗號、每天自動開店。

> 這就是 **plugin 系統**：Tauri 2 把系統能力拆成 plugin。先 `cargo tauri add ...` 加依賴（領鑰匙），再在 `tauri::Builder` 上 `.plugin(tauri_plugin_fs::init())`、`.plugin(tauri_plugin_dialog::init())`……註冊（範例註冊了 fs、dialog、notification、clipboard-manager、shell、global-shortcut、autostart 七個）。最後在 `capabilities/default.json` 寫 `"fs:allow-read-text-file"` 這類權限（許可單）。忘了註冊會 `not found`，忘了權限會 `not allowed`。

---

## 倉庫只開你那一格

圓圓要把庫存表存進地下室倉庫。管理處給她的鑰匙，**只打得開「二樓書店」那一格**。

隔壁的咖啡店老闆跟她說：「你去申請萬能鑰匙啦，整個倉庫都能開，多方便！」圓圓搖頭：「萬一鑰匙被偷，**整棟大樓的東西**都會被搬光。」

> 這就是 **檔案系統與 scope**：前端可用 `readTextFile` / `writeTextFile` 搭配 `BaseDirectory.AppData` 等目錄；也可以像範例一樣自己寫 command：`save_file` 用 `tokio::fs::write`、`read_file` 用 `tokio::fs::read_to_string`，錯誤用 `map_err(|e| e.to_string())` 回給前端。fs plugin 預設只開特定目錄，`fs:scope` 千萬**不要**放 `**`。

---

## 大樓的標準表單窗口

客人想把書單「另存成一個檔案」。圓圓不用自己做一個選資料夾的畫面，她請客人到大樓的**標準表單窗口**：這個窗口每層樓都長得一樣，大家都會用，選好了窗口會告訴圓圓「客人選了這個位置」。

窗口有自己的辦事員，所以客人在窗口慢慢挑的時候，**書店照樣營業**。

> 這就是 **dialog plugin**：`open({ filters: [...] })`、`save({ defaultPath })`、`message(...)`、`confirm(...)`。對話框是 OS 原生的，跑在原生 thread，不會卡住 webview。

---

## 全棟廣播和公用留言板

新書到了，圓圓想讓整棟的人知道。她先問管理處：「我可以用廣播嗎？」得到同意後，廣播響起：「**二樓書店新書到貨！**」

客人想把一句書摘帶走，圓圓就把它**寫在大樓公用留言板**上，客人走到哪一層都能抄下來、貼到別的地方。

> 這就是 **通知** 和 **剪貼簿**：`isPermissionGranted()` / `requestPermission()` 後 `sendNotification({ title, body })`；剪貼簿用 `writeText` / `readText`，權限是 `clipboard-manager:allow-read-text` / `allow-write-text`。剪貼簿是系統共用的，所以讀它也要權限。

---

## 在大樓任何地方按暗號

圓圓跟店員約好一個**門鈴暗號**：「按住兩個鍵再按 M」。不管店員在頂樓吃午餐還是在地下室搬貨，只要有人按了這個暗號，書店的鈴就會響。

不過她要先確認：這個暗號**有沒有別的店已經在用**。有些大樓早就把這組暗號拿去當別的用途了。

> 這就是 **global shortcut**：範例用 `tauri_plugin_global_shortcut::Builder::new().build()` 註冊 plugin，再用 `app.global_shortcut().on_shortcut(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM), ...)` 綁定。即使視窗不在前景也會觸發。有些 OS 已經佔用某些組合鍵，要每個平台測。

---

## 門關了，門口還留一個服務鈴

晚上書店拉下鐵門，可是圓圓沒有回家，她在**門口掛了一個小服務鈴**。

客人一按鈴，就會跳出一張小卡片，上面兩個選項：「**開門**」或「**今天打烊**」。選「開門」，鐵門拉起來、燈打開，客人就能進來；選「打烊」，圓圓就真的回家了。

> 這就是 **tray icon**：範例在 `.setup(|app| ...)` 裡用 `MenuItem::with_id(app, "show", "Show", ...)` 和 `"quit"` 做選單，`TrayIconBuilder::new().menu(&menu).show_menu_on_left_click(true)` 建立托盤。`on_menu_event` 裡 `"quit" => app.exit(0)`；`"show"` 則 `app.get_webview_window("main")` 後 `w.show()` + `w.set_focus()`。`on_tray_icon_event` 收到 `TrayIconEvent::Click` 也會叫出主視窗。圖示放在 `src-tauri/icons/`。

---

## 隔一間小房間當辦公室

圓圓想要一間**設定辦公室**。她先去看看：「辦公室已經有了嗎？」有的話就直接走進去；沒有的話，就請大樓**隔出一間新房間**，門牌寫「Settings」，大小 600 × 400。

辦公室和店面是兩個獨立的房間，但看的是**同一本帳**。

> 這就是 **視窗管理**：`app.get_webview_window("settings")` 存在就 `set_focus()`，否則用 `tauri::WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into())).title("Settings").inner_size(600.0, 400.0).build()`。每個視窗是獨立的 webview，資料透過 `tauri::State` / IPC 共享。

---

## 每天大樓開門，書店自動開

圓圓不想每天早上親自來開店，她跟管理處登記：「**大樓一開門，就幫我把書店打開。**」

不同城市的大樓，登記的地方不一樣：有的寫在管理處的登記簿，有的貼在電梯旁的公告欄。

> 這就是 **auto-start**：`.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))`，前端用 `enable` / `disable` / `isEnabled`。Windows 寫 Run 機碼、macOS 建 LaunchAgent plist、Linux 放 autostart `.desktop`。最好第一次啟動先問使用者要不要開。

---

## 專屬門牌，從外面直接找上門

圓圓印了一種**專屬門牌**：「`mycoolapp://share?url=...`」。別人在網路上點到這個門牌，大樓就會**直接把他帶到二樓書店**，還順便告訴圓圓他想看哪一本書。

> 這就是 **deep link**：在 `tauri.conf.json` 的 `plugins.deep-link.schemes` 設 `["mycoolapp"]`，瀏覽器或 OS 就能用自訂 URL scheme 啟動 app，常用在 OAuth callback。

---

## 每個城市的分店規矩不一樣

圓圓後來在三個城市開了分店。有些事情**只有某個城市才需要做**：台北分店要在電梯貼標示、台中分店要在門口放鞋櫃。

她在員工手冊裡標註：「**以下這段只給台北分店看**」，其他分店的員工根本拿不到那幾頁。

還有一條：有些大樓規定，**某些事只能在一樓櫃台辦**，店員要記得跑一趟一樓。

> 這就是 **平台專屬程式碼**：`#[cfg(target_os = "windows")]`、`#[cfg(target_os = "macos")]` 在編譯時就只留下對應平台的程式。某些 Windows API 必須在 UI thread 執行，用 `app.run_on_main_thread` 送過去。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 領鑰匙 + 店門口登記 | 加依賴 + **`.plugin(xxx::init())`** |
| 許可單 | **`capabilities/*.json` 的 permissions** |
| 倉庫只開你那一格 | **fs scope**（別用 `**`） |
| 存庫存表 | `save_file` / `read_file`（`tokio::fs`） |
| 標準表單窗口 | **dialog plugin**（`open` / `save` / `confirm`） |
| 全棟廣播 | **notification plugin**（`sendNotification`） |
| 公用留言板 | **clipboard-manager plugin**（`writeText` / `readText`） |
| 門鈴暗號 | **global shortcut**（`on_shortcut`） |
| 門口的服務鈴 | **tray icon**（`TrayIconBuilder`） |
| 「開門 / 打烊」卡片 | `Menu` + `MenuItem::with_id` + `on_menu_event` |
| 鐵門拉起、燈打開 | `get_webview_window("main")` → `show()` + `set_focus()` |
| 隔一間設定辦公室 | **`WebviewWindowBuilder`** 開新視窗 |
| 大樓開門就自動開店 | **autostart plugin** |
| 專屬門牌 | **deep link**（自訂 URL scheme） |
| 只給台北分店看的那幾頁 | **`#[cfg(target_os = "...")]`** |
| 只能在一樓櫃台辦 | **`run_on_main_thread`** |

---

## 一句話記住

> **想用大樓的設施，先領鑰匙、再填許可單；倉庫只開自己那一格，打烊了門口留個服務鈴，暗號在哪都叫得到你——每個城市的規矩不同，就在手冊上標好「只給這家分店看」。**
