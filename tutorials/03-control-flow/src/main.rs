fn main() {
    // ── if 是 expression，可賦值 ─────────────────────────────
    let n = 5;
    let parity = if n % 2 == 0 { "even" } else { "odd" };
    println!("{n} is {parity}");

    // ── loop 可以 break 帶值 ─────────────────────────────────
    let mut i = 0;
    let answer = loop {
        i += 1;
        if i * i > 100 {
            break i;
        }
    };
    println!("first n where n*n>100: {answer}");

    // ── while ────────────────────────────────────────────────
    let mut s = String::from("hello");
    while s.len() < 10 {
        s.push('!');
    }
    println!("{s}");

    // ── for over iterator ────────────────────────────────────
    for x in 1..=3 {
        print!("{x} ");
    }
    println!();

    for (i, c) in "abc".chars().enumerate() {
        println!("{i}: {c}");
    }

    // ── match：必須窮盡 ──────────────────────────────────────
    let dir = "north";
    let (dx, dy) = match dir {
        "north" => (0, 1),
        "south" => (0, -1),
        "east" => (1, 0),
        "west" => (-1, 0),
        _ => (0, 0), // catch-all
    };
    println!("{dir} → ({dx}, {dy})");

    // ── match guard ──────────────────────────────────────────
    let opt: Option<i32> = Some(7);
    let described = match opt {
        Some(n) if n > 5 => format!("big {n}"),
        Some(n) => format!("small {n}"),
        None => "nothing".to_string(),
    };
    println!("{described}");

    // ── tuple destructure + range pattern ────────────────────
    let point = (3, -1);
    let region = match point {
        (0, 0) => "origin",
        (x, 0) if x > 0 => "positive x-axis",
        (0, y) if y > 0 => "positive y-axis",
        (x, y) if x > 0 && y > 0 => "Q1",
        (1..=10, 1..=10) => "small Q1",
        _ => "elsewhere",
    };
    println!("region: {region}");

    // ── @ binding：邊配對邊綁名 ──────────────────────────────
    let n = 7;
    match n {
        x @ 1..=5 => println!("small ({x})"),
        x @ 6..=10 => println!("medium ({x})"),
        x => println!("other ({x})"),
    }

    // ── if let：只 care 一個 arm ─────────────────────────────
    let some_val: Option<i32> = Some(42);
    if let Some(x) = some_val {
        println!("got {x}");
    }

    // ── while let：直到 None ─────────────────────────────────
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        print!("{top} ");
    }
    println!();

    // ── let else (Rust 1.65+)：早返 ──────────────────────────
    let Some(parsed) = "42".parse::<i32>().ok() else {
        eprintln!("parse failed");
        return;
    };
    println!("parsed: {parsed}");

    // ── labeled break / continue ─────────────────────────────
    let pair = 'outer: loop {
        for x in 1..10 {
            for y in 1..10 {
                if x * y == 42 {
                    break 'outer (x, y);
                }
            }
        }
        break (0, 0);
    };
    println!("42 = {} * {}", pair.0, pair.1);
}
