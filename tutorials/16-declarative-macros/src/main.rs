// ── 最簡單的 macro ──────────────────────────────────────────
macro_rules! say_hello {
    () => {
        println!("Hello from macro!");
    };
}

// ── 帶參數的 macro ──────────────────────────────────────────
macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}

// ── 多重 pattern（type-overloaded） ─────────────────────────
macro_rules! debug {
    () => {
        eprintln!("[DEBUG] (no args)");
    };
    ($val:expr) => {
        eprintln!("[DEBUG] {} = {:?}", stringify!($val), $val);
    };
    ($($val:expr),+ $(,)?) => {
        $(debug!($val);)+
    };
}

// ── 變數參數（variadic） ─────────────────────────────────────
macro_rules! my_min {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        std::cmp::min($x, my_min!($($rest),+))
    };
}

// ── 不同 fragment specifier ─────────────────────────────────
macro_rules! create_struct {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug)]
        struct $name {
            $($field: $ty),*
        }
    };
}

create_struct!(Point { x: i32, y: i32 });
create_struct!(Person { name: String, age: u32, });

// ── 為型別 impl trait ───────────────────────────────────────
macro_rules! impl_display_via_debug {
    ($name:ty) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

#[derive(Debug)]
struct Wrapped(i32);
impl_display_via_debug!(Wrapped);

// ── 小型 DSL：類似 hashmap! ─────────────────────────────────
macro_rules! map {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($key, $val);)*
        m
    }};
}

fn main() {
    say_hello!();

    println!("square(5) = {}", square!(5));
    println!("square(2 + 3) = {}", square!(2 + 3)); // = (2+3) * (2+3) = 25 ✓

    debug!();
    debug!(42);
    debug!(1 + 1);
    debug!("hello", 3.14, [1, 2, 3]);

    println!("min = {}", my_min!(3, 1, 4, 1, 5, 9, 2, 6));

    let p = Point { x: 1, y: 2 };
    let person = Person {
        name: "Alice".into(),
        age: 30,
    };
    println!("{p:?}");
    println!("{person:?}");

    println!("{}", Wrapped(99));

    let scores = map! {
        "alice" => 95,
        "bob" => 80,
        "carol" => 88,
    };
    println!("scores = {scores:?}");

    // 內建 macro 也是 macro_rules! 或 procedural macro
    let v = vec![1, 2, 3];
    let repeat = vec![0; 5];
    println!("vec! examples: {v:?} {repeat:?}");
}
