use std::fmt;

// ── named struct ────────────────────────────────────────────
#[derive(Debug, Clone)]
struct Point {
    x: i32,
    y: i32,
}

// ── tuple struct ────────────────────────────────────────────
#[derive(Debug)]
struct Rgb(u8, u8, u8);

// ── unit struct ─────────────────────────────────────────────
struct Marker;

// ── enum：可帶不同 variant 形狀 ─────────────────────────────
#[derive(Debug)]
enum Shape {
    Circle { radius: f64 },
    Rect(f64, f64),
    Triangle(f64, f64, f64),
    Empty,
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rect(w, h) => w * h,
            Shape::Triangle(a, b, c) => {
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
            Shape::Empty => 0.0,
        }
    }
}

impl Point {
    // associated function（無 self） = 工廠 / static method
    fn origin() -> Self {
        Self { x: 0, y: 0 }
    }

    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    // method（有 self） = instance method
    fn translate(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    fn distance_to(&self, other: &Point) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    // ── struct method ────────────────────────────────────────
    let mut p = Point::new(3, 4);
    p.translate(1, 1);
    println!("{p}"); // Display
    println!("{p:?}"); // Debug

    let origin = Point::origin();
    println!("distance to origin = {:.3}", p.distance_to(&origin));

    // ── struct update syntax ────────────────────────────────
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 99, ..p1 }; // 其他 field 從 p1 複製/move
    println!("p2 = {p2}");

    // ── tuple struct ─────────────────────────────────────────
    let color = Rgb(255, 128, 0);
    println!("{color:?} → r={} g={} b={}", color.0, color.1, color.2);

    // ── unit struct（常用於 marker / 0-size type） ───────────
    let _m = Marker;

    // ── enum + match ─────────────────────────────────────────
    let shapes = [
        Shape::Circle { radius: 1.0 },
        Shape::Rect(3.0, 4.0),
        Shape::Triangle(3.0, 4.0, 5.0),
        Shape::Empty,
    ];
    for s in &shapes {
        println!("{s:?} → area = {:.3}", s.area());
    }

    // ── Option<T>：null 的型別安全版本 ───────────────────────
    let some: Option<i32> = Some(7);
    let none: Option<i32> = None;
    println!("some = {some:?}, none = {none:?}");

    // 處理方式
    println!("unwrap_or: {}", none.unwrap_or(-1));
    println!("map: {:?}", some.map(|x| x * 2));
    println!("and_then: {:?}", some.and_then(|x| if x > 0 { Some(x) } else { None }));

    // ── pattern destructure ─────────────────────────────────
    let p = Point { x: 10, y: 20 };
    let Point { x, y } = p; // destructure
    println!("destructured: x={x}, y={y}");

    let (a, b) = (1, 2); // tuple destructure
    println!("a={a}, b={b}");
}
