// 列出所有 HID 裝置 — keyboard / mouse / RGB controller / 遊戲手把 ...
//
// macOS / Windows 不需特權；Linux 需要 udev rule（見 README）。

use anyhow::Result;
use hidapi::HidApi;

fn main() -> Result<()> {
    let api = HidApi::new()?;

    println!("{:<6} {:<6} {:<40} {:<25} {}",
        "VID", "PID", "Manufacturer", "Product", "Usage(Page/Id)");
    println!("{}", "─".repeat(120));

    for d in api.device_list() {
        println!(
            "{:04x}   {:04x}   {:<40} {:<25} {:#x}/{:#x}",
            d.vendor_id(),
            d.product_id(),
            d.manufacturer_string().unwrap_or("").chars().take(40).collect::<String>(),
            d.product_string().unwrap_or("").chars().take(25).collect::<String>(),
            d.usage_page(),
            d.usage(),
        );
    }
    Ok(())
}
