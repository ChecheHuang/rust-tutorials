use std::fmt::Display;

// ── 定義 trait ──────────────────────────────────────────────
trait Greet {
    // required method（無 body，須實作）
    fn name(&self) -> String;

    // default method（可選擇 override）
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

// ── 實作 trait ──────────────────────────────────────────────
struct Person {
    name: String,
}

impl Greet for Person {
    fn name(&self) -> String {
        self.name.clone()
    }
}

struct Robot {
    id: u32,
}

impl Greet for Robot {
    fn name(&self) -> String {
        format!("Unit-{}", self.id)
    }

    // override default
    fn greet(&self) -> String {
        format!("BEEP BOOP. I am {}.", self.name())
    }
}

// ── trait 作為函式參數（impl Trait 與泛型形式） ─────────────
fn say_hi(g: &impl Greet) {
    println!("{}", g.greet());
}

fn say_hi_generic<T: Greet>(g: &T) {
    println!("{}", g.greet());
}

// ── trait object：動態 dispatch ─────────────────────────────
fn say_hi_dyn(g: &dyn Greet) {
    println!("{}", g.greet());
}

// ── 多 trait bound ──────────────────────────────────────────
fn print_and_greet<T: Greet + Display>(item: &T) {
    println!("display: {item}");
    println!("greet:   {}", item.greet());
}

impl Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Person({})", self.name)
    }
}

// ── associated type vs generic type ─────────────────────────
trait Container {
    type Item; // associated type
    fn first(&self) -> Option<&Self::Item>;
}

struct IntBox(Vec<i32>);
impl Container for IntBox {
    type Item = i32;
    fn first(&self) -> Option<&i32> {
        self.0.first()
    }
}

// ── supertrait ──────────────────────────────────────────────
trait Greet2: Display {
    // 要實作 Greet2 必先實作 Display
    fn shout(&self) -> String {
        format!("{}!!!", self).to_uppercase()
    }
}

impl Greet2 for Person {}

// ── blanket impl ────────────────────────────────────────────
trait ToShoutyString {
    fn to_shouty(&self) -> String;
}
// 對所有實作 Display 的型別 free 提供 to_shouty
impl<T: Display> ToShoutyString for T {
    fn to_shouty(&self) -> String {
        format!("{self}").to_uppercase()
    }
}

fn main() {
    let alice = Person {
        name: "Alice".into(),
    };
    let r2d2 = Robot { id: 42 };

    say_hi(&alice);
    say_hi(&r2d2);
    say_hi_generic(&alice);
    say_hi_dyn(&r2d2);

    // ── trait object：執行期切換型別 ────────────────────────
    let crowd: Vec<Box<dyn Greet>> = vec![
        Box::new(Person {
            name: "Bob".into(),
        }),
        Box::new(Robot { id: 99 }),
    ];
    for g in &crowd {
        println!("dyn dispatch: {}", g.greet());
    }

    // ── supertrait ──────────────────────────────────────────
    println!("shout: {}", alice.shout());

    // ── blanket impl ────────────────────────────────────────
    println!("blanket: {}", "hello".to_shouty()); // &str 也有 Display
    println!("blanket: {}", 42.to_shouty()); // i32 也有 Display

    // ── associated type ─────────────────────────────────────
    let ib = IntBox(vec![10, 20]);
    println!("first int: {:?}", ib.first());
}
