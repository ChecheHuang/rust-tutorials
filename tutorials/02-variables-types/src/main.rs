fn main() {
    // ── let / mut ────────────────────────────────────────────
    let x = 5;
    // x = 6;          // ERROR: cannot assign twice to immutable variable
    let mut y = 5;
    y = 6;
    println!("x={x}, y={y}");

    // ── shadowing (新 binding，型別可變) ──────────────────────
    let z = "5";
    let z: i32 = z.parse().unwrap();
    let z = z * 2;
    println!("z={z}"); // 10

    // ── 基本型別 ──────────────────────────────────────────────
    let a: i8 = -128;
    let b: u32 = 4_000_000_000;
    let c: f64 = 3.14;
    let d: bool = true;
    let e: char = '我'; // char 是 4 bytes，Unicode scalar
    println!("a={a} b={b} c={c} d={d} e={e}");

    // ── 型別推導與 turbofish ──────────────────────────────────
    let v = vec![1, 2, 3]; // Vec<i32>
    let v2: Vec<u8> = vec![1, 2, 3];
    let parsed = "42".parse::<i64>().unwrap(); // turbofish
    println!("v={v:?} v2={v2:?} parsed={parsed}");

    // ── 整數溢位（顯式語意） ─────────────────────────────────
    let max = u8::MAX;
    let wrapped = max.wrapping_add(1); // 0
    let checked = max.checked_add(1); // None
    let saturating = max.saturating_add(1); // 255
    let (overflowed, did_overflow) = max.overflowing_add(1);
    println!(
        "max={max} wrap={wrapped} check={checked:?} sat={saturating} \
         over=({overflowed}, {did_overflow})"
    );

    // ── as 轉型 ──────────────────────────────────────────────
    let big: i32 = 300;
    let small: u8 = big as u8; // 截斷：300 % 256 = 44
    let f: f32 = big as f32;
    println!("big={big} small={small} f={f}");

    // ── 安全轉型用 TryFrom ────────────────────────────────────
    let safe = u8::try_from(big); // Err(TryFromIntError)
    println!("try_from(300_i32 -> u8) = {safe:?}");

    // ── const ────────────────────────────────────────────────
    const MAX_USERS: u32 = 1_000;
    println!("MAX_USERS={MAX_USERS}");

    // ── unit type ────────────────────────────────────────────
    let unit: () = ();
    println!("unit={unit:?}");
}
