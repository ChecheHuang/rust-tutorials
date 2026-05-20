fn main() {
    // ── move 語意（非 Copy 型別）─────────────────────────────
    let s1 = String::from("hello");
    let s2 = s1; // s1 的 ownership 移到 s2
                 // println!("{s1}"); // ERROR: s1 已 move
    println!("after move: {s2}");

    // ── Copy 型別：bitwise 複製，不 move ─────────────────────
    let n1 = 5;
    let n2 = n1; // n1 仍可用
    println!("Copy: {n1} {n2}");

    // ── Clone：顯式 deep copy ────────────────────────────────
    let s3 = String::from("world");
    let s4 = s3.clone();
    println!("Clone: {s3} {s4}");

    // ── 傳給函式 = move ──────────────────────────────────────
    let s = String::from("ownership");
    takes_ownership(s);
    // println!("{s}"); // ERROR

    let n = 42;
    makes_copy(n);
    println!("after makes_copy, n still here: {n}");

    // ── 回傳 = 轉移 ownership ────────────────────────────────
    let gifted = gives_ownership();
    println!("got gift: {gifted}");

    // ── explicit drop ────────────────────────────────────────
    let big = String::from("big buffer");
    drop(big);
    // println!("{big}"); // ERROR

    // ── Drop trait：自訂解構 ─────────────────────────────────
    println!("\n--- Drop ordering ---");
    let _outer = LoudGuard("outer");
    {
        let _inner = LoudGuard("inner");
        println!("inside block");
    } // inner drops here
    println!("after block (outer still alive)");
} // outer drops here

fn takes_ownership(s: String) {
    println!("function got: {s}");
} // s drops here

fn makes_copy(n: i32) {
    println!("function got copy: {n}");
}

fn gives_ownership() -> String {
    String::from("gifted")
}

struct LoudGuard(&'static str);

impl Drop for LoudGuard {
    fn drop(&mut self) {
        println!("  dropping {}", self.0);
    }
}
