# 故事版：社區聖誕燈光秀的總控台 🌈

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「社區要辦聖誕燈光秀，各家借來的燈串牌子都不一樣，卻要用同一個總控台指揮」的故事，幫你「感覺」出 `rgb-core` 這整個專案是怎麼運作的——燈具為什麼都要學會同樣幾句話、排練用的紙板燈、真燈旁邊為什麼要站一位專屬燈工、燈光效果怎麼一格一格算出來、怎麼喊停，以及節目單怎麼存起來。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 各家燈串，牌子都不一樣

小光是社區燈光秀的總監。大家從家裡搬來各式各樣的燈：鍵盤形狀的燈牌、滑鼠形狀的小燈、桌墊燈、耳機燈……每一家工廠的開關、接頭、指令**全都不一樣**。

小光說：「我不想記一百種用法。**每一盞燈都要學會同樣幾句話**，學不會的就不能上台。」

這幾句話是：

1. 「**自我介紹**」：我是誰、哪家工廠做的、我是什麼形狀、身上有幾顆燈泡。
2. 「**全部變成這個顏色**」。
3. 「**每顆燈泡照這張清單換顏色**」——清單長度一定要剛好等於燈泡數。
4. 「**跑你工廠內建的花樣**」——這句可以不會，不會的就老實說「我不支援」。

> 這就是 **`Device` trait**（`src/device.rs`）：`fn info(&self) -> &DeviceInfo`、`async fn set_all(&self, color)`、`async fn set_leds(&self, colors: &[Color])`，以及有預設實作的 `set_firmware_effect`（預設 `anyhow::bail!("firmware effects not supported by ...")`）。`DeviceInfo { id, name, vendor, kind, led_count }`，`kind` 是 `enum DeviceKind { Keyboard, Mouse, Mousepad, Headset, Generic }`。trait 標了 `Send + Sync` 並用 `#[async_trait]`，才能在多個 task 之間共用。

---

## 顏色是三個數字

小光發給每個人一張**調色卡**：每個顏色都是「紅、綠、藍」三個 0 到 255 的數字。卡片上還印好了幾個常用色：黑、白、紅、綠、藍。

調色卡背面有兩個小技巧：

- **轉轉盤找顏色**：在一個色環上轉到幾度，就得到那個方向的顏色——0 度紅、120 度綠、240 度藍。
- **兩色中間調**：給兩個顏色和一個 0 到 1 的比例，就調出它們中間的顏色。比例 0.5 就是正中間。

> 這就是 **`src/color.rs`**：`struct Color { r, g, b }`（`u8`），常數 `Color::BLACK` / `WHITE` / `RED` / `GREEN` / `BLUE`。`Color::from_hsv(h, s, v)` 把色相轉 RGB；`lerp(self, other, t)` 先 `t.clamp(0.0, 1.0)` 再逐通道內插。測試 `hsv_round_trip_primary`、`lerp_endpoints` 驗證這兩個技巧。

---

## 排練用的紙板燈

真正的燈串還沒到，但小光想先排練。他做了兩盞**紙板燈**：一塊 87 格的「示範鍵盤」、一個 7 格的「示範滑鼠」。

紙板燈也會那幾句話：叫它變色，它就在本子上**記下每一格現在是什麼顏色**，順便大聲報告「我全部變紅了！」。清單長度不對，它一樣會拒絕：「我有 87 格，你只給我 80 個顏色。」

因為紙板燈會的話和真燈一模一樣，排練時怎麼指揮，正式演出就怎麼指揮。

> 這就是 **`FakeDevice`**：`state: Mutex<Vec<Color>>` 記錄每顆 LED，`set_all` 改完用 `tracing::info!` 印出來，`set_leds` 檢查 `colors.len() != led_count` 就 `bail!`。`DeviceManager::populate_demo()` 註冊 `FakeDevice::new("DemoKeyboard", 87)` 和 `FakeDevice::new("DemoMouse", 7)`，id 是 `fake:DemoKeyboard`、`fake:DemoMouse`。沒有硬體也能跑、也好測。

---

## 真燈旁邊，站一位專屬燈工

真的燈串很難搞：

- 它的開關**很慢**，扳一下要等它反應完才能扳下一次。
- 而且有些燈串**只認一個人**，換人碰它就不理你。

所以小光的做法是：每盞真燈旁邊都**站一位專屬燈工**，只有燈工能碰那盞燈。總控台想換顏色，就把一張「顏色清單」投進燈工面前的**小信箱**（最多放 16 張）。燈工一張一張拿出來，**翻譯成那家工廠聽得懂的暗號**，再扳開關。

扳失敗了，燈工會在日誌上記一筆「這次沒成功」，然後繼續處理下一張。如果哪天燈工不見了，信箱投不進去，總控台會收到「燈工已經離開」。

> 這就是 **`HidRgbDevice` 的 actor pattern**：`HidRgbDevice::spawn(info, device, framer)` 用 `std::thread::spawn` 開一條專屬 thread **持有 `hidapi::HidDevice`**（HID 寫入是阻塞 IO，而且某些平台的 handle 不是 `Send`）。總控台透過 `tokio::sync::mpsc::channel::<Vec<Color>>(16)` 送顏色，thread 用 `rx.blocking_recv()` 收，呼叫 `framer(&colors)` 轉成廠商的 HID frame 再 `device.write(&frame)`；失敗就 `tracing::warn!`。`set_leds` 送不出去時回 `"device IO thread closed"`。`set_all` 則是做一份 `vec![color; led_count]` 交給 `set_leds`。

---

## 四種燈光節目

小光準備了四種節目：

- **定色**：全部同一個顏色，一動也不動。
- **呼吸**：在兩個顏色之間**慢慢變亮、慢慢變暗**，像在呼吸。
- **彩虹**：每顆燈泡在色環上錯開一點點，整串看起來是一道**會流動的彩虹**。
- **彗星**：一顆最亮的燈泡**繞著跑**，後面拖著越來越暗的尾巴（大約五顆長）。

每一種節目都只要寫在紙上：「呼吸，紅色到黑色，兩秒一次」。紙上只是**說明**，還沒有真的動起來。

> 這就是 **`enum Effect`**（`src/effects.rs`）：`Static(Color)`、`Breathe { a, b, period_ms }`、`Rainbow { period_ms }`、`Comet { color, period_ms }`。它是**純資料**，`#[derive(Clone, Debug)]`，不含任何執行邏輯。

---

## 每 30 毫秒翻一頁

節目怎麼動起來？小光請了一位**翻頁員**。

翻頁員手上有一本**空白的動畫本**，旁邊放著一個**節拍器**，每 30 毫秒「滴」一下。每「滴」一聲，翻頁員就看看「節目開始到現在過了多久」，用計算機**算出這一頁每顆燈泡該是什麼顏色**，然後把整頁的顏色清單交給燈。

計算的方法很簡單：

- 呼吸：用一條上上下下的波浪曲線算出比例，再用調色卡的「兩色中間調」。
- 彩虹：每顆燈泡的位置不同，色環角度就錯開一點，時間一過，全部一起往前轉。
- 彗星：算出「頭」現在在哪一格，離頭越遠的越暗。

計算機**只負責算**，不碰燈；交給燈的事由翻頁員做。

> 這就是 **`run_effect` + `render`**：`run_effect(device, effect, stop)` 用 `tokio::time::interval(Duration::from_millis(30))` 當節拍器，每次 `tick` 用 `start.elapsed()` 算出 `t`，呼叫純函式 `render(&effect, n, t)` 產生 `Vec<Color>`，再 `device.set_leds(&frame).await?`。`Breathe` 用 `sin` 算 phase 再 `a.lerp(*b, phase)`；`Rainbow` 用 `(t / period + i / n) % 1.0` 乘 360 丟給 `from_hsv`；`Comet` 算 `head` 再依距離 `fade`。`render` 不做 IO，很好測。

---

## 一根舉起來就停的旗子

翻頁員一邊翻頁，一邊用眼角盯著一根**小旗子**。

旗子放下時，他就繼續翻；**旗子一舉起來**，他立刻停下，最後把所有燈泡**全部關黑**，然後下班。

如果拿旗子的人**整個不見了**（連旗子都帶走），翻頁員也知道「沒人會再管這場節目了」，一樣關燈下班。

> 這就是 **`tokio::select!` + `watch` channel**：迴圈裡同時等 `tick.tick()` 和 `stop.changed()`。`changed()` 回傳 `Err`（`Sender` 被 drop 了）或 `*stop.borrow()` 為 `true` 時，送出 `vec![Color::BLACK; n]` 後 `return Ok(())`。

---

## 總控台：名冊和旗子櫃

小光的**總控台**上有兩樣東西：

- 一本**燈具名冊**：每盞燈的編號對到那盞燈。很多人可以同時**翻看**名冊，但要**登記新燈**時，得等大家看完、一個人獨自寫。
- 一個**旗子櫃**：每盞正在跑節目的燈，櫃子裡都有一根對應的旗子。

小光要某盞燈換節目時，總控台會這樣做：

1. 先看旗子櫃裡有沒有那盞燈的旗子——有的話，**把它拿出來舉起**，舊節目停下。
2. 翻名冊找那盞燈；找不到就回報「**沒有這盞燈**」。
3. 做一根**新旗子**放進櫃子，然後派一位新的翻頁員去跑新節目。

要停某盞燈，就把它的旗子從櫃子拿出來、舉起來。

名冊和翻頁員手上拿的都是「這盞燈的**共用把手**」，不是燈本身。所以名冊和翻頁員可以**同時握著同一盞燈**，誰都不用把燈搶走。

> 這就是 **`DeviceManager`**（`src/manager.rs`）：`devices: RwLock<HashMap<String, Arc<dyn Device>>>` 是名冊，`effect_stops: RwLock<HashMap<String, watch::Sender<bool>>>` 是旗子櫃。`register` 用 `write()` 插入，`list` / `get` 用 `read()`。`start_effect` 先 `self.stop_effect(id)`，再 `get(id)`，找不到就 `"device not found: {id}"`；然後 `watch::channel(false)`、存入 sender、`tokio::spawn` 跑 `run_effect`。`stop_effect` 用 `remove(id)` 取出 sender 並 `send(true)`。`Arc<dyn Device>` 是 **trait object**：不同廠商的燈放在同一個 `HashMap` 裡，執行期才決定呼叫哪個實作。

---

## 節目單存成一本小冊子

燈光秀辦得很成功，大家希望明年照著再演一次。小光把今天的設定寫成一本**節目單小冊子**：

- 小冊子的名字。
- 每盞燈要演哪一種節目：每一段都先寫「**節目種類**」，再寫細節，例如「種類：呼吸，顏色 A：紅、顏色 B：黑，兩秒一次」。顏色在小冊子裡就寫成三個數字。
- 還有一頁「**快捷手勢**」：例如「按住三個鍵再按 M」，就自動做一串動作——按某個鍵、等一下、打一串字、執行某個指令。

明年拿出小冊子，就能把每一段**翻回**今天的節目。

不過「快捷手勢」那一頁目前**只寫了格式**，還沒有人負責真的去按鍵盤——那是之後要補的。

> 這就是 **`Profile`**（`src/profile.rs`）：`Profile { name, device_effects, macros }`，用 **serde + TOML** 的 `Profile::load` / `save`（`toml::from_str`、`toml::to_string_pretty`）。`EffectSpec` 用 `#[serde(tag = "type", rename_all = "snake_case")]`，序列化時多一個 `type = "breathe"` 欄位，顏色是 `[u8; 3]`。`impl From<EffectSpec> for Effect` 把存檔格式轉回執行用的 `Effect`。`MacroBinding { trigger, actions }` 與 `MacroAction`（`KeyPress`、`Delay`、`Text`、`Command`）目前只有**資料模型**，播放器要另外用 `enigo` 實作。

---

## 用對講機指揮排練

正式的總控台畫面還在做，小光先用一支**對講機**指揮排練。對講機上有幾個按鈕：

- 「**列出所有燈**」：把名冊唸一遍——編號、名字、形狀、幾顆燈泡。
- 「**定色**」「**彩虹**」「**呼吸**」：說出燈的編號和參數，節目就開始，一直跑到小光**按下結束鍵**，總控台才舉旗、關燈。

注意：每次拿起對講機都是**新開一場排練**，只有兩盞紙板燈；這場的旗子，管不到上一場的翻頁員。

> 這就是 **`src/main.rs`（`rgb-cli`）**：clap 的 `#[derive(Subcommand)] enum Cmd { List, Static, Rainbow, Breathe, Stop }`。啟動時 `Arc::new(DeviceManager::new())` + `populate_demo()`；效果類子命令呼叫 `mgr.start_effect(...)`，然後 `tokio::signal::ctrl_c().await?` 等 Ctrl+C，再 `mgr.stop_effect(&id)`。每次執行都是新的 process、新的 manager，所以獨立執行 `rgb-cli stop` 停不到別的 process 的效果。

---

## 一層一層的分工

小光把整個團隊畫成一張圖：

```
對講機 / 未來的總控台畫面        ← 誰在下指令
        ↓ 共用的總控台
總控台（名冊、旗子櫃、翻頁員）    ← 怎麼安排
        ↓ 共用的燈具把手
燈具的那幾句話、調色卡、節目     ← 規矩是什麼
        ↓
紙板燈、真燈 + 專屬燈工          ← 真正動手的
```

以後要換成漂亮的總控台畫面、要能**偵測有人新插上一盞燈**、要在螢幕角落留一個小圖示、要打包送到三個國家——上面那幾層都不用重寫，只要接在最上面或最下面。

> 這就是 **`src/lib.rs` 的分層**：adapter（Tauri commands / CLI）→ `Arc<DeviceManager>` → application（`DeviceManager`、effect engine）→ `Arc<dyn Device>` → domain（`Device` trait、`Color`、`Effect`）→ infrastructure（`FakeDevice`、`HidRgbDevice`、未來的廠商 driver）。`Device` 是 **port**，廠商實作是 **adapter**。接下來 Tauri 殼用 `tauri::State<Arc<DeviceManager>>`（第 51 章）、托盤與自啟（第 52 章）、hotplug 輪詢（第 53 章）、打包簽章（第 54 章）。

---

## 回到現實：故事 ↔ 程式碼對照表

| 故事裡的東西 | 真正的程式概念 | 在哪個檔案 |
|------|------|------|
| 每盞燈都要會的幾句話 | **`Device` trait**（`info` / `set_all` / `set_leds`） | `src/device.rs` |
| 「我不支援內建花樣」 | `set_firmware_effect` 的預設實作 `bail!` | `src/device.rs` |
| 自我介紹 | `DeviceInfo` + `DeviceKind` | `src/device.rs` |
| 調色卡 | `Color { r, g, b }` + 常數 | `src/color.rs` |
| 轉轉盤找顏色 | `Color::from_hsv` | `src/color.rs` |
| 兩色中間調 | `Color::lerp` | `src/color.rs` |
| 紙板燈 | **`FakeDevice`**（`Mutex<Vec<Color>>`） | `src/device.rs` |
| 清單長度不對就拒絕 | `colors.len() != led_count` → `bail!` | `src/device.rs` |
| 專屬燈工 | **actor**：專屬 `std::thread` 持有 `HidDevice` | `src/device.rs` |
| 燈工的小信箱（16 張） | `tokio::sync::mpsc::channel(16)` + `blocking_recv` | `src/device.rs` |
| 翻譯成工廠暗號 | `framer: impl Fn(&[Color]) -> Vec<u8>` | `src/device.rs` |
| 四種節目 | `enum Effect` | `src/effects.rs` |
| 節拍器每 30ms | `tokio::time::interval(30ms)` | `src/effects.rs` |
| 只算不碰燈的計算機 | 純函式 `render` | `src/effects.rs` |
| 翻頁員 | `run_effect` 跑在 `tokio::spawn` 的 task | `src/effects.rs`、`src/manager.rs` |
| 盯著小旗子 | `tokio::select!` + `watch::Receiver<bool>` | `src/effects.rs` |
| 停下前全部關黑 | `vec![Color::BLACK; n]` | `src/effects.rs` |
| 燈具名冊 | `RwLock<HashMap<String, Arc<dyn Device>>>` | `src/manager.rs` |
| 旗子櫃 | `RwLock<HashMap<String, watch::Sender<bool>>>` | `src/manager.rs` |
| 換節目先舉舊旗 | `start_effect` 先呼叫 `stop_effect` | `src/manager.rs` |
| 共用把手 | `Arc<dyn Device>`（trait object） | `src/manager.rs` |
| 兩盞排練紙板燈 | `populate_demo()` | `src/manager.rs` |
| 節目單小冊子 | `Profile` + serde + TOML | `src/profile.rs` |
| 每段先寫節目種類 | `#[serde(tag = "type")]` | `src/profile.rs` |
| 翻回今天的節目 | `impl From<EffectSpec> for Effect` | `src/profile.rs` |
| 快捷手勢那一頁 | `MacroBinding` / `MacroAction`（只有資料模型） | `src/profile.rs` |
| 排練用的對講機 | `rgb-cli`（clap `Subcommand`） | `src/main.rs` |
| 按結束鍵才關燈 | `tokio::signal::ctrl_c()` → `stop_effect` | `src/main.rs` |
| 一層一層的分工圖 | adapter → application → domain → infrastructure | `src/lib.rs` |

---

## 一句話記住

> **每盞燈都學會同樣幾句話，紙板燈陪你排練、真燈旁邊站一位專屬燈工；總控台翻名冊、派翻頁員每 30 毫秒算一頁顏色，想停就舉旗、關黑再下班——節目單存成小冊子，明年翻開就能重演。**
