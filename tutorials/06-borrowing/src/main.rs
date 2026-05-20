fn main() {
    // ── 共享借用 &T：可同時多個 ──────────────────────────────
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("r1={r1} r2={r2}");
    println!("owner still valid: {s}");

    // ── 獨佔借用 &mut T：一次一個 ────────────────────────────
    let mut s2 = String::from("hi");
    let m = &mut s2;
    m.push_str(", world");
    println!("via &mut: {m}");

    // ── 借用規則：一寫多讀互斥（NLL 已優化 scope）─────────────
    let mut v = vec![1, 2, 3];
    let a = &v;
    let b = &v;
    println!("read: {a:?} {b:?}");
    // a, b 之後不再使用，編譯器知道借用結束
    let c = &mut v;
    c.push(4);
    println!("after mut: {c:?}");

    // ── 函式：借用而不取走 ───────────────────────────────────
    let s3 = String::from("borrow me");
    let len = calc_len(&s3);
    println!("'{s3}' len = {len}");

    let mut s4 = String::from("change me");
    append_hello(&mut s4);
    println!("after mutation: {s4}");

    // ── reborrow：&mut → &mut 鏈 ─────────────────────────────
    let mut s5 = String::from("reborrow");
    let outer = &mut s5;
    {
        let inner: &mut String = &mut *outer; // reborrow
        inner.push('!');
    } // inner 結束，outer 重新生效
    println!("via outer: {outer}");

    // ── 不能回傳 dangling reference ─────────────────────────
    let owned = give_owned();
    println!("safely owned: {owned}");

    // ── auto deref：method 透明 ──────────────────────────────
    let s6 = String::from("AUTO");
    print_lower(&s6); // 不必 .as_str()
}

fn calc_len(s: &String) -> usize {
    // 慣例上 prefer &str 而非 &String，下章詳述
    s.len()
}

fn append_hello(s: &mut String) {
    s.push_str(" hello");
}

fn give_owned() -> String {
    // 不能寫 -> &String，函式內 local 會在回傳後 drop
    String::from("safe")
}

fn print_lower(s: &str) {
    println!("lower: {}", s.to_lowercase());
}
