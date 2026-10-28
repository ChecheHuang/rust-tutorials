# 故事版：用餐區和廚房之間的出菜口 🍳

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「小餐館前面是用餐區、後面是廚房，中間只靠一個出菜口溝通」的故事，幫你「感覺」出 Tauri 的架構、`#[tauri::command]`、`invoke`、參數與回傳、async command、`State`、event、安全模型，以及跟 Electron 的取捨。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 前面漂亮，後面能幹

小美和阿傑開了一家小餐館。

- **前面的用餐區**歸小美管：桌布、燈光、菜單排版，全都要好看。客人只會待在這裡。
- **後面的廚房**歸阿傑管：切菜、炒菜、算帳，真正的功夫都在這裡。

兩個區域中間隔著一道牆，牆上只開了一個**出菜口**。客人要什麼，小美把單子從出菜口遞進去；阿傑做好，再從出菜口遞出來。

> 這就是 **Tauri 的架構**：前端（HTML / JS / Vue / React）跑在 **webview** 裡，是用餐區；**Rust core** 是廚房，放你的業務邏輯；兩邊透過 **IPC**（出菜口）溝通。

---

## 自己蓋裝潢，還是借大樓現成的

隔壁的大餐廳（Electron）很豪華：它**自己帶了一整套裝潢**搬進來，每一家分店長得一模一樣，可是搬家要用大卡車，店也特別佔空間。

小美的餐館選擇**借用大樓本來就有的裝潢**（Tauri）。行李只有一個小背包，開店又快又省地方。缺點是：不同城市的大樓，裝潢**長得有一點點不一樣**——台北那棟的窗戶是方的，台中那棟是圓的。所以小美開新分店前，一定**親自去每一棟看一次**。

> 這就是 **Tauri vs Electron**：Electron 內嵌一整份 Chromium（binary 約 150 MB），Tauri 用**系統 webview**（約 10 MB、記憶體也省）。代價是渲染各平台略有差異：Windows 是 WebView2、macOS 是 WKWebView、Linux 是 WebKitGTK，所以每個 OS 都要實機測。

---

## 菜單上有的菜，才點得到

阿傑會做很多事，但**不是每一件都讓客人點**。他會挑幾道菜，在旁邊畫一顆星，然後把這張「**可點菜單**」貼在出菜口。

小美遞單子進來時，阿傑只看這張表：表上有的就做，沒有的就說「沒有這道」。

> 這就是 **`#[tauri::command]` + `generate_handler!`**：範例裡 `greet`、`do_async_work`、`read_secret` 都標了 `#[tauri::command]`（畫星星），再用 `.invoke_handler(tauri::generate_handler![greet, do_async_work, read_secret])` 貼到出菜口。沒註冊的函式，前端叫不到。

---

## 遞單子進去，等盤子出來

客人說：「我要一份打招呼套餐，名字寫 Bob。」

小美寫好單子遞進去：「**greet，名字：Bob**」，然後就去招呼別桌，**不在出菜口前面乾等**。一會兒鈴聲響起，盤子出來了，上面放著一張紙條：「Hello, Bob!」，旁邊還寫著「這個名字有 3 個字」。

盤子一定要用**大家都認得的標準盤**裝，小美才端得出去。

> 這就是 **`invoke` 和回傳值**：前端 `await invoke('greet', { name: 'Bob' })` 拿回一個 Promise。`greet` 回傳 `GreetResponse { message, length }`，其中 `length` 是 `name.chars().count()`。回傳型別要 **`serde::Serialize`**（標準盤），才能變成 JSON 送回前端。

---

## 兩邊講話習慣不同，出菜口自動翻譯

小美寫單子習慣寫「**welcomeMessage**」（駝峰寫法），阿傑在廚房習慣寫「**welcome_message**」（底線寫法）。

還好出菜口有個小翻譯員，會**自動把參數名字換成對方習慣的寫法**。不過翻譯員只翻「欄位名字」，如果小美在單子內容裡自己寫了一串字，翻譯員不會動。

> 這就是 **參數命名自動轉換**：JS 端用 camelCase、Rust 端用 snake_case，Tauri 會自動轉換參數名稱；但你手寫的字串內容不會被轉。

---

## 燉湯要慢慢來，出菜口不能堵住

有一道「慢燉湯」要燉好幾秒。

如果阿傑**站在出菜口前面守著湯鍋**，其他單子就全部遞不進來。所以阿傑把湯放到爐上慢慢燉，自己繼續處理別的單，湯好了再從出菜口送出去。

要是湯燉失敗了，阿傑不會整個人倒在地上（那樣整家店都卡住），而是遞出一張**「抱歉，今天燉不出來」的說明卡**，小美就能好好跟客人解釋。

> 這就是 **async command + `Result`**：`do_async_work(seconds)` 是 `async fn`，用 `tokio::time::sleep` 等待，不阻塞 webview；**不要**用 `std::thread::sleep`。回傳 `Result<String, String>`，`Err` 在前端會變成 `invoke` 丟出的錯誤。Rust 端 panic 不會好好告訴前端，所以 command 裡要用 `Result`。

---

## 廚房牆上的公告板

開店那天，阿傑在廚房牆上釘了一塊**公告板**，寫著「歡迎光臨！」。之後任何一道菜需要這句話，廚師抬頭看公告板就好，不用每次重新寫。

如果公告板上是會一直改的東西，例如「今天賣出幾份」，大家要改之前得先**拿那支唯一的粉筆**，免得兩個人同時改、數字亂掉。

> 這就是 **`State`**：`.manage(AppConfig { welcome_message: "Welcome to Tauri!".into() })` 把資料交給 Tauri 管理，`read_secret(state: tauri::State<AppConfig>)` 就能讀到。會變的 state 要用 `Mutex<T>` / `RwLock<T>` 包起來（那支粉筆）；async command 裡如果要跨 `await` 持有鎖，要用 `tokio::sync::Mutex`。

---

## 廚房也會主動廣播

有一次客人點了一百個水餃。阿傑每包好十個，就對著出菜口的**小喇叭**喊一聲：「進度 10！」「進度 20！」……

小美在外面**豎起耳朵聽**，把進度條一格一格往前推。不想聽了，就把耳朵關上。

> 這就是 **event**：Rust 端用 `app.emit("progress", i)` 廣播（`AppHandle` 可以帶進背景 task），前端 `listen('progress', e => ...)` 收聽，回傳的 `un()` 用來取消。前端往 Rust 也能 emit，但通常直接 `invoke` 比較直白。

---

## 客人不能自己走進廚房

隔壁大餐廳以前發生過一件事：門沒關好，客人**直接走進廚房**亂開瓦斯。

小美的餐館規定很嚴：**預設什麼都不准**。連「借用廚房的剪刀」都要事先寫在一張**許可清單**上，清單上沒有的，一律擋下來。

> 這就是 **Tauri 的安全模型**：預設封閉，前端要用的能力必須在 `src-tauri/capabilities/default.json` 的 `permissions`（例如 `"core:default"`、`"shell:allow-open"`）裡明確開放。對比 Electron 早年 `nodeIntegration = true` 全開的災難。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 用餐區 | **前端**（webview 裡的 HTML / JS） |
| 廚房 | **Rust core** |
| 出菜口 | **IPC** |
| 自帶整套裝潢的大餐廳 | **Electron**（內嵌 Chromium） |
| 借大樓現成的裝潢 | **系統 webview**（WebView2 / WKWebView / WebKitGTK） |
| 畫星星的菜 | **`#[tauri::command]`** |
| 貼在出菜口的可點菜單 | **`generate_handler![...]`** + `invoke_handler` |
| 遞單子進去 | 前端 **`invoke('greet', { name })`** |
| 標準盤 | 回傳型別要 **`Serialize`** |
| 自動翻譯欄位名字 | camelCase ↔ snake_case 自動轉換 |
| 慢燉湯 | **async command**（`do_async_work`） |
| 「今天燉不出來」說明卡 | 回傳 **`Result<T, E>`**，別 panic |
| 廚房公告板 | **`.manage(...)`** + **`tauri::State<T>`** |
| 唯一的粉筆 | `Mutex` / `RwLock` |
| 小喇叭喊進度 | **`app.emit(...)`** |
| 豎起耳朵聽 | 前端 **`listen(...)`** |
| 許可清單 | **capabilities / permissions** |

---

## 一句話記住

> **前端是漂亮的用餐區、Rust 是能幹的廚房，兩邊只靠出菜口說話：畫了星星又貼上菜單的菜才點得到，慢菜不堵出菜口、失敗遞說明卡，公共資料寫在公告板、改之前先拿粉筆，客人永遠進不了廚房。**
