# 故事版：機器人玩具和萬用對講機 🤖

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「小莉用一台萬用對講機，跟玩具箱裡的機器人們說話」的故事，幫你「感覺」出 USB HID、report descriptor、input / output / feature report、enumerate、VID / PID 與 usage page、`read_timeout`、Report ID、平台權限、Tauri 整合、hotplug 和逆向工程。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 所有玩具都會的通用語言

小莉有一大箱機器人玩具：會走路的鍵盤機器人、會滾來滾去的滑鼠機器人、全身會發光的彩燈機器人……它們來自不同的工廠。

神奇的是，它們都會說**同一種通用語言**。所以小莉不用替每個玩具買專屬的遙控器，**一台萬用對講機**就能跟全部的玩具說話。

> 這就是 **USB HID（Human Interface Device）**：鍵盤、滑鼠、手把，還有 RGB 控制器、巨集鍵盤都常走 HID，因為**所有 OS 都不用另外裝 driver**。Rust 用 `hidapi` crate 當萬用對講機，入口是 `HidApi::new()`。

---

## 每個玩具身上的說明牌

每個機器人背後都貼著一塊**說明牌**：

- 「我會**告訴你**什麼」：例如哪個按鈕被按了。
- 「你可以**叫我**做什麼」：例如把眼睛燈打開。
- 「我的**設定**」：例如亮度幾格，可以讀也可以改。

不過彩燈機器人的說明牌上，有一段寫著工廠自己的**密語**，外面的人看不懂。想知道怎麼叫它變色，只能自己研究。

> 這就是 **HID Report Descriptor** 和三種 report：**input report**（裝置 → 主機）、**output report**（主機 → 裝置）、**feature report**（雙向設定）。RGB 這類產品多半用廠商自訂（vendor-specific）的格式，沒有公開規格。

---

## 點名：箱子裡有誰？

小莉第一件事是**點名**。她按下對講機的「全部報上名來」，每個玩具都回答：

「我是 1532 號工廠、022f 號型號，叫 Huntsman Mini，我這個耳朵是一般按鍵用的。」

奇怪的是，鍵盤機器人**報了三次名**——因為它有**三隻耳朵**：一隻聽一般按鍵、一隻聽音量鍵，第三隻是**專門聽工廠密語**的。小莉要控制燈光，就得找那隻密語耳朵。

> 這就是 **enumerate**：`list_devices.rs` 用 `api.device_list()` 逐一印出 `vendor_id()`（VID）、`product_id()`（PID）、`manufacturer_string()`、`product_string()`、`usage_page()` / `usage()`。同一台裝置每個 **interface** 會列一筆；**usage page 0xFF00 以上**是廠商自訂，通常就是控制用的那個。

---

## 打開頻道，聽它說話

小莉選好鍵盤機器人，跟對講機說：「**接通 1532 號工廠、022f 號型號。**」

接通之後，她拿出一個能裝 64 顆豆子的小碗，對機器人說：「**有話就說，我最多等你一秒。**」

- 一秒內機器人說了話 → 碗裡裝了幾顆豆子，小莉一顆一顆念出來。
- 一秒內什麼都沒說 → 碗是空的，她就再問一次。

她試過「完全不等、一直問」：對講機忙到發燙，什麼事都做不了。也試過「永遠等下去」：機器人不說話，她就一直呆站著。

> 這就是 **開裝置與讀 report**：`read_keyboard.rs` 從環境變數讀 `VID` / `PID`（`u16::from_str_radix(..., 16)`），`api.open(vid, pid)` 打開裝置，再 `device.read_timeout(&mut buf, 1000)` 讀進 `[0u8; 64]`。回傳 `n == 0` 代表逾時沒資料就 `continue`，否則把 `buf[..n]` 用 `{b:02x}` 印成十六進位。timeout：`1000` 等一秒、`0` 非阻塞（小心 busy loop 吃滿 CPU）、`-1` 永久等。

---

## 說話前，先喊頻道號碼

小莉要叫彩燈機器人變紅色。她發現在某些城市，對講機規定：**每句話的第一個字一定要先喊「頻道號碼」**，就算只有一個頻道，也要先喊「零」。可是在另一個城市，直接講內容就好。

還有：話**太短**機器人會裝沒聽到，**太長**有些對講機會直接拒絕。

> 這就是 **Report ID 與 buffer 大小**：寫入時 `report[0] = 0`（Report ID），後面接 payload，再 `device.write(&report)`。Windows / macOS 要這個前綴、Linux 的 hidraw 行為不同，`hidapi` 統一了介面但仍要注意。長度要符合裝置宣告的 report size。

---

## 改設定，走專用的小門

調整亮度這種「設定」，小莉不在大家都聽得到的頻道上喊，而是走機器人背後的**小門**，悄悄塞一張設定卡進去，也能從小門拿出目前的設定卡來看。

這樣一般的按鍵訊息不會被設定的話塞滿。

> 這就是 **feature report**：`device.get_feature_report(&mut buf)` 讀、`device.send_feature_report(&buf)` 寫，`buf[0]` 一樣是 report ID。多數 RGB 協定走 feature report，不污染 input stream。

---

## 有些房間要鑰匙才進得去

小莉去表哥家玩，表哥家的規矩不一樣：

- 有的家裡，對講機**直接就能用**。
- 有的家裡，鍵盤和滑鼠說的話**被大人收走了**，只有密語耳朵還能聽。
- 有的家裡，要先在門口**貼一張通行證**，寫明「小莉可以進玩具房」，然後請大人**重新讀一次門口的規定**。

> 這就是 **平台權限**：Windows 多半直接可用；macOS 的鍵盤 / 滑鼠 input 被 OS 獨佔，但廠商通常另外暴露 vendor interface；Linux 要寫 udev rule（`SUBSYSTEM=="hidraw", MODE="0660", ...`），再 `sudo udevadm control --reload && sudo udevadm trigger`。權限不足時 `open` 會失敗。

---

## 請一位專職的傳話員

小莉後來把對講機接到她的**桌上控制台**，上面有顏色按鈕。

一開始，她每按一次按鈕，就**重新接通一次**機器人，再講話、再掛斷，慢得要命。後來她請了一位**專職傳話員**：傳話員一直握著對講機不放，小莉把「變紅色」「變藍色」寫成紙條放進傳話員的信箱，傳話員一張一張照著講。

> 這就是 **Tauri 整合**：最簡單的寫法是在 `#[tauri::command] async fn set_rgb(r, g, b)` 裡 `HidApi::new()` → `open` → `write`；但長期使用應該開一個背景 task **持有 `HidDevice`**，用 **mpsc channel** 接收指令，避免每次 invoke 都重開裝置。

---

## 誰進來了？誰走了？

萬用對講機有個缺點：機器人被拔走或新接上時，**它不會主動告訴你**。

所以小莉每隔一秒就**重新點名一次**，拿這次的名單跟上次比：「上次沒有、這次有」就是新來的；「上次有、這次沒有」就是被拔走的。被拔走的機器人，原本接通的頻道也**失效了**，要重新點名、重新接通。

> 這就是 **hotplug（輪詢 + diff）**：`hidapi` 沒有原生 hotplug，就定期 enumerate，把 `(vendor_id, product_id, path)` 放進 `HashSet`，用 `now.difference(&prev)` 找新裝置、`prev.difference(&now)` 找拔掉的。同 VID/PID 的多個 interface 靠 `path()` 區分。要更精準，Linux 用 `udev`、Windows 用 `RegisterDeviceNotification`。

---

## 偷聽官方遙控器說什麼

彩燈機器人的密語沒人知道。小莉想了一個辦法：在官方遙控器和機器人之間放一台**錄音機**。她按一下官方遙控器的「紅色」，錄音機錄到一串聲音；再按「藍色」，又錄到一串。比對幾次，就猜出哪幾個字代表顏色。

媽媽提醒她：「自己在家玩可以，但把別人工廠的密語**到處公開**，可能會惹上麻煩喔。」

> 這就是 **逆向工程**：Windows 用 **Wireshark + USBPcap**、macOS 用 **PacketLogger**、Linux 用 **Wireshark + usbmon** 抓官方軟體和裝置之間的封包，對照操作推出 command opcode。個人使用 OK，公開可能違反 DMCA / EULA。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 所有玩具都會的通用語言 | **USB HID**（免 driver） |
| 萬用對講機 | **`hidapi`**（`HidApi::new()`） |
| 背後的說明牌 | **HID Report Descriptor** |
| 機器人告訴你 / 你叫它做 | **input report** / **output report** |
| 背後的小門 | **feature report**（`get_feature_report` / `send_feature_report`） |
| 點名 | **enumerate**：`api.device_list()` |
| 工廠號碼、型號 | **VID / PID** |
| 好幾隻耳朵 | 多個 **interface** |
| 密語耳朵 | **usage page ≥ 0xFF00**（vendor-specific） |
| 接通頻道 | `api.open(vid, pid)` |
| 最多等一秒 | **`read_timeout(&mut buf, 1000)`** |
| 64 顆豆子的小碗 | `[0u8; 64]` buffer |
| 先喊頻道號碼「零」 | **Report ID**：`report[0] = 0` |
| 門口的通行證 | Linux **udev rule** |
| 專職傳話員 + 信箱 | 背景 task 持有 `HidDevice` + **mpsc** |
| 每秒重新點名比對 | **hotplug**：輪詢 + `HashSet::difference` |
| 錄音機偷聽 | **Wireshark + USBPcap / usbmon** |

---

## 一句話記住

> **HID 是所有玩具都會的通用語言：先點名找到那隻密語耳朵，接通後最多等一下再聽；說話前記得先喊頻道號碼，設定走小門，請一位傳話員一直握著對講機——玩具被拔走它不會說，要自己每秒重新點名。**
