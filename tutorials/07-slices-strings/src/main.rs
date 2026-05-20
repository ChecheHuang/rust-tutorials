fn main() {
    // ── &str vs String ───────────────────────────────────────
    let literal: &str = "hello"; // 存在 binary .rodata，'static
    let owned: String = String::from("hi"); // heap 配置
    let slice: &str = &owned; // String → &str（auto-deref）
    let part: &str = &owned[0..1];
    println!("{literal} {owned} {slice} '{part}'");

    // ── String 修改 ──────────────────────────────────────────
    let mut s = String::from("hello");
    s.push(' ');
    s.push_str("world");
    s += "!"; // 等同 push_str
    println!("{s}");

    // ── &[T] vs Vec<T> ───────────────────────────────────────
    let arr = [10, 20, 30, 40, 50];
    let slice: &[i32] = &arr[1..4]; // &[20, 30, 40]
    println!("slice = {slice:?}, len = {}", slice.len());

    let v: Vec<i32> = vec![1, 2, 3];
    let v_slice: &[i32] = &v[..]; // 整段
    println!("v_slice = {v_slice:?}");

    // ── &str 子切片 ──────────────────────────────────────────
    let s = String::from("hello world");
    let hello = &s[..5];
    let world = &s[6..];
    println!("'{hello}' '{world}'");

    // ── UTF-8 邊界（小心！）──────────────────────────────────
    let zh = String::from("你好"); // 6 bytes（每字 3 bytes UTF-8）
    println!("len (bytes) = {}", zh.len());
    println!("chars count = {}", zh.chars().count());
    println!("first char = {:?}", zh.chars().next());
    let prefix = &zh[..3]; // "你"，正好在字元邊界
    println!("prefix = '{prefix}'");
    // let bad = &zh[..1];  // PANIC: byte index 1 is not a char boundary

    // ── 三種把 &str 轉 String ────────────────────────────────
    let a: String = "hi".to_string();
    let b: String = String::from("hi");
    let c: String = "hi".to_owned();
    println!("equal: {} {} {}", a == b, b == c, a == c);

    // ── 常用 method ──────────────────────────────────────────
    let csv = "a,b,c,d";
    let parts: Vec<&str> = csv.split(',').collect();
    println!("split = {parts:?}");

    let n: i32 = "42".parse().unwrap();
    println!("parse = {n}");

    let replaced = "foo bar".replace("foo", "baz");
    println!("replace = {replaced}");

    let trimmed = "   spaces   ".trim();
    println!("trim = '{trimmed}'");

    // ── &str 作為函式參數（通用） ────────────────────────────
    let owned_string = String::from("hello world");
    let literal = "lit";
    println!("first1 = {}", first_word(&owned_string));
    println!("first2 = {}", first_word(literal));
}

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
