// 範例：開特定 VID/PID 裝置讀 HID report。
//
// 用 list-devices 先找到你的鍵盤 VID/PID，然後：
//   VID=1532 PID=022f cargo run --bin read-keyboard
//
// 注意：opening keyboard/mouse 可能會被 OS 拒絕（HID exclusivity）
// 或需要 root 權限。對 RGB / gaming device 一般 OK，因為它們暴露額外 interface。

use anyhow::{Context, Result};
use hidapi::HidApi;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let vid = u16::from_str_radix(
        &std::env::var("VID").context("set VID=<hex>")?, 16
    )?;
    let pid = u16::from_str_radix(
        &std::env::var("PID").context("set PID=<hex>")?, 16
    )?;

    let api = HidApi::new()?;
    let device = api.open(vid, pid)
        .with_context(|| format!("open VID={:04x} PID={:04x}", vid, pid))?;

    println!("Reading reports from {:?} (Ctrl+C to stop)", device.get_product_string()?);

    let mut buf = [0u8; 64];
    loop {
        let n = device.read_timeout(&mut buf, 1000)?;
        if n == 0 {
            continue;
        }
        print!("[{n:2} bytes]");
        for b in &buf[..n] {
            print!(" {b:02x}");
        }
        println!();
    }
}
