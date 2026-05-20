# 53. USB HID 與硬體控制

> 範圍：HID 協定、`hidapi` crate、enumerate、input/output report、Tauri 整合、平台權限

## HID 是什麼

USB **Human Interface Device** — 鍵盤、滑鼠、遊戲手把、繪圖板，**還有 RGB 控制器**、巨集鍵盤、IoT dongle 也常走 HID（因為 HID 在所有 OS 上不必裝 driver）。

裝置會宣告 **HID Report Descriptor**，描述：
- input report：裝置 → 主機（按鍵、感測）
- output report：主機 → 裝置（LED 控制、回饋）
- feature report：雙向設定

對 RGB 控制這類產品，**廠商通常自訂 vendor-specific 報告**，沒公開規格——逆向工程居多（OpenRGB 就是這樣積攢）。

## 環境

### Windows
直接跑，可能需要 driver-less HID 即可。某些 keyboard 在 raw HID interface 抓不到 → 廠商 driver 鎖死。

### macOS
有 IOKit。沒特權但有限制：keyboard / mouse 的 input 走 OS 獨佔，第三方拿不到；但廠商一般會額外暴露 vendor HID interface（usage page 0xFF00+）給控制軟體用。

### Linux
要 udev rule 讓 user 拿得到 device：

```
# /etc/udev/rules.d/99-myapp.rules
SUBSYSTEM=="hidraw", MODE="0660", GROUP="plugdev", TAG+="uaccess"
KERNEL=="hidraw*", ATTRS{idVendor}=="1532", MODE="0660", GROUP="plugdev"
```

reload：`sudo udevadm control --reload && sudo udevadm trigger`

## 找你的裝置

```bash
cargo run --bin list-devices
```

```
VID    PID    Manufacturer         Product                Usage(Page/Id)
0x1532 0x022f Razer                Razer Huntsman Mini    0x1/0x6
0x1532 0x022f Razer                Razer Huntsman Mini    0xc/0x1
0x1532 0x022f Razer                Razer Huntsman Mini    0xff00/0x1   ← vendor
```

**同一台裝置會列出多個 entry** — 每個 interface 一個。要找 usage page **0xFF00 以上**（廠商自訂），通常是控制 endpoint。

## 開裝置讀 report

```rust
let api = HidApi::new()?;
let device = api.open(0x1532, 0x022f)?;
let mut buf = [0u8; 64];
let n = device.read_timeout(&mut buf, 1000)?;
```

`read_timeout(ms)`：
- 1000 → 等 1 秒
- 0 → 非阻塞
- -1 → 永久等

write：
```rust
let mut report = vec![0u8; 65];
report[0] = 0;            // Report ID（0 if numbered，否則特定 ID）
report[1..].copy_from_slice(&payload);
device.write(&report)?;
```

⚠️ **Windows / Linux 行為差異**：
- Windows：第一個 byte 必須是 Report ID（即使 device 不用 numbered report，也要寫 0）
- Linux：直接寫 payload，沒 Report ID prefix
- macOS：跟 Windows 同
- `hidapi` 包了統一界面，但對「report size」會 padding 到裝置宣告的 size

## Feature report（設定）

讀寫 feature：

```rust
let mut buf = vec![0u8; 65];
buf[0] = report_id;
device.get_feature_report(&mut buf)?;

device.send_feature_report(&buf)?;
```

多數 RGB 控制協定走 feature report — 不污染 input stream。

## 用 Tauri 整合

```rust
use tauri::{AppHandle, Manager};

#[tauri::command]
async fn set_rgb(r: u8, g: u8, b: u8) -> Result<(), String> {
    let api = HidApi::new().map_err(|e| e.to_string())?;
    let dev = api.open(0x1532, 0x022f).map_err(|e| e.to_string())?;

    let mut buf = vec![0u8; 65];
    buf[0] = 0;
    // 這裡填裝置特定 protocol
    buf[1] = 0x01;       // command
    buf[2] = r; buf[3] = g; buf[4] = b;
    dev.write(&buf).map_err(|e| e.to_string())?;
    Ok(())
}
```

長 connection：開一個背景 task 持有 `HidDevice`、用 mpsc 收 command，避免每次 invoke 都 re-open。

## hotplug

`hidapi` 沒原生 hotplug API → 定期 enumerate + diff：

```rust
let mut prev: HashSet<(u16,u16,String)> = HashSet::new();
loop {
    let api = HidApi::new()?;
    let now: HashSet<_> = api.device_list()
        .map(|d| (d.vendor_id(), d.product_id(), d.path().to_string_lossy().to_string()))
        .collect();
    for new in now.difference(&prev) { /* plugged in */ }
    for gone in prev.difference(&now) { /* unplugged */ }
    prev = now;
    sleep(1 sec).await;
}
```

Linux 想精準用 `udev`、Windows 用 `RegisterDeviceNotification`。

## 逆向工程提示

要解 RGB / 巨集裝置自訂協定：
1. **Wireshark + USBPcap**（Windows）抓官方軟體跟裝置的封包
2. **macOS 用 PacketLogger**（Apple developer tool）
3. **`wireshark + usbmon`**（Linux）
4. 拿封包對照軟體操作，逐步推算 command opcode

⚠️ **法律 / 倫理**：個人玩 OK；reverse 後**公開**可能違反 DMCA / 廠商 EULA。OpenRGB 這類 community 專案運作多年但風險仍在。

## 常見陷阱

1. **權限拒絕** — Linux 沒 udev rule、macOS 沒 input monitor 權限。
2. **Report ID prefix** — 寫忘了補 0、或多補了 byte，平台行為不同。
3. **buffer 大小** — 寫太短裝置忽略；寫太長部分平台直接拒。
4. **read_timeout(0) busy loop** — CPU 100%；用 timeout 或開 thread 阻塞。
5. **多 interface 同 VID/PID** — open 開錯了拿不到資料。用 `path()` 區分。
6. **interface number** — 某些 device 多 interface（gaming）、特定 interface 才接受 control command。
7. **裝置斷線後 device handle 失效** — 偵測 read error，重新 enumerate。
8. **Concurrent open** — 多 process 同時開可能其中一個失敗；用 lock。

## 練習

1. 跑 `list-devices`，找出你 PC 上所有 HID 裝置。
2. 對某個遊戲手把跑 `read-keyboard`，按鈕、搖桿動值看 report 變化。
3. 用 Wireshark 抓官方 RGB 軟體 1 秒封包，找 control command。
4. 把 hotplug 邏輯包成 Tauri event：emit `device:connected` / `device:disconnected`。

## 延伸閱讀

- [hidapi-rs](https://docs.rs/hidapi)
- [USB HID 1.11 spec](https://www.usb.org/sites/default/files/hid1_11.pdf)
- [OpenRGB protocol docs](https://gitlab.com/CalcProgrammer1/OpenRGB)
