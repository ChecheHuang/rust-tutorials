fn main() {
    // ── 基本函式 ──────────────────────────────────────────────
    println!("add(2, 3) = {}", add(2, 3));

    // 函式 body 最後一行是 expression（無分號），即回傳值
    println!("double_block(5) = {}", double_block(5));

    // ── 閉包：型別推導 ────────────────────────────────────────
    let add_one = |x| x + 1;
    println!("add_one(4) = {}", add_one(4));

    // 顯式標 fn pointer 型別
    let mul: fn(i32, i32) -> i32 = |a, b| a * b;
    println!("mul(3, 4) = {}", mul(3, 4));

    // ── 捕獲方式：by reference (Fn) ───────────────────────────
    let s = String::from("hello");
    let print_s = || println!("borrowed: {s}");
    print_s();
    print_s();
    println!("s 仍可用：{s}");

    // ── 捕獲：by mutable reference (FnMut) ────────────────────
    let mut count = 0;
    let mut bump = || {
        count += 1;
    };
    bump();
    bump();
    println!("count = {count}");

    // ── 捕獲：by move (FnOnce or Fn) ──────────────────────────
    let owned = String::from("owned");
    let take = move || println!("moved: {owned}");
    take();
    take(); // 仍可叫，因為 println! 用的是 &owned
            // println!("{owned}"); // ERROR: owned 已 move 進 closure

    // ── 三個 closure trait 對應的接受者 ───────────────────────
    apply_fn(|x| x + 1); // Fn: 唯讀，可重複呼叫
    let mut sum = 0;
    apply_fn_mut(|x| {
        sum += x;
        sum
    });
    println!("after FnMut: sum = {sum}");

    let s_for_once = String::from("consumed");
    apply_fn_once(move || drop(s_for_once)); // FnOnce: 只能叫一次

    // ── 高階函式（functional 風格） ───────────────────────────
    let nums = vec![1, 2, 3, 4, 5];
    let doubled_sum: i32 = nums.iter().map(|x| x * 2).sum();
    println!("doubled sum = {doubled_sum}");

    // ── 回傳 closure ──────────────────────────────────────────
    let adder = make_adder(10);
    println!("adder(5) = {}", adder(5));
}

fn add(a: i32, b: i32) -> i32 {
    a + b // 沒有分號 → 是 expression → 即回傳值
}

fn double_block(x: i32) -> i32 {
    // block 本身是 expression
    let doubled = {
        let temp = x * 2;
        temp + 0
    };
    doubled
}

fn apply_fn<F: Fn(i32) -> i32>(f: F) {
    println!("apply_fn({}) = {}", 10, f(10));
}

fn apply_fn_mut<F: FnMut(i32) -> i32>(mut f: F) {
    f(1);
    f(2);
    println!("apply_fn_mut done");
}

fn apply_fn_once<F: FnOnce()>(f: F) {
    f();
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
