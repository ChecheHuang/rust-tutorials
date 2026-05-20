fn main() {
    println!("=== Cargo Advanced Demo ===");

    // ── 透過 cfg 切換 feature ───────────────────────────────
    #[cfg(feature = "fancy")]
    println!("fancy feature is ON");

    #[cfg(not(feature = "fancy"))]
    println!("fancy feature is OFF");

    #[cfg(feature = "extra")]
    println!("extra feature is ON");

    // ── 編譯期環境變數（從 build.rs 或 Cargo） ──────────────
    println!("package name: {}", env!("CARGO_PKG_NAME"));
    println!("package version: {}", env!("CARGO_PKG_VERSION"));
    println!("rustc version: {}", env!("CARGO_PKG_RUST_VERSION"));

    // ── cfg target ──────────────────────────────────────────
    #[cfg(target_os = "windows")]
    println!("on Windows");

    #[cfg(target_os = "linux")]
    println!("on Linux");

    #[cfg(target_os = "macos")]
    println!("on macOS");

    #[cfg(debug_assertions)]
    println!("DEBUG build");
    #[cfg(not(debug_assertions))]
    println!("RELEASE build");

    println!("\n試試：");
    println!("  cargo run                          # 預設 features");
    println!("  cargo run --no-default-features    # 關閉 fancy");
    println!("  cargo run --features extra         # 加上 extra");
    println!("  cargo run --release                # release profile");
}
