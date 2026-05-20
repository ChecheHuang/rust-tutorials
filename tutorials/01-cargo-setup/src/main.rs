fn main() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    println!("Hello from {name} v{version}!");
    println!("Edition: 2021");
    println!("Run `rustc --version` to see your toolchain.");
}
