use std::fmt::Debug;
use std::ops::Add;

// ── 泛型函式 ────────────────────────────────────────────────
fn largest<T: PartialOrd + Copy>(slice: &[T]) -> T {
    let mut max = slice[0];
    for &x in slice.iter().skip(1) {
        if x > max {
            max = x;
        }
    }
    max
}

// where clause：較長的 bounds 寫這
fn print_all<T>(items: &[T])
where
    T: Debug + Clone,
{
    for x in items {
        println!("{x:?}");
    }
}

// ── 泛型 struct ─────────────────────────────────────────────
#[derive(Debug)]
struct Pair<A, B> {
    first: A,
    second: B,
}

impl<A: Debug, B: Debug> Pair<A, B> {
    fn show(&self) {
        println!("{:?} & {:?}", self.first, self.second);
    }
}

// impl 只在特定型別參數時提供 method
impl Pair<i32, i32> {
    fn sum(&self) -> i32 {
        self.first + self.second
    }
}

// ── 泛型 enum（標準函式庫的 Result 就是這樣） ───────────────
#[derive(Debug)]
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

// ── 泛型 method with multiple constraints ───────────────────
fn add_two<T: Add<Output = T> + Copy>(a: T, b: T) -> T {
    a + b
}

fn main() {
    // 各型別的同一函式
    let ints = vec![10, 25, 3, 40];
    let strs = vec!["banana", "apple", "cherry"];
    println!("largest int = {}", largest(&ints));
    println!("largest str = {}", largest(&strs));

    print_all(&[1, 2, 3]);
    print_all(&["a", "b"]);

    // 泛型 struct，兩個型別參數
    let p = Pair {
        first: 3,
        second: "hello",
    };
    p.show();

    let n = Pair {
        first: 10,
        second: 20,
    };
    println!("sum = {}", n.sum()); // 只在 i32 i32 時可用

    // 泛型 enum
    let tree = Tree::Node(
        Box::new(Tree::Leaf(1)),
        Box::new(Tree::Node(
            Box::new(Tree::Leaf(2)),
            Box::new(Tree::Leaf(3)),
        )),
    );
    println!("{tree:?}");

    // 多個 trait bound
    println!("add_two(3.5, 2.5) = {}", add_two(3.5, 2.5));
    println!("add_two(3i32, 4) = {}", add_two(3i32, 4));

    // ── Default trait 與 turbofish ──────────────────────────
    let zero: i32 = Default::default();
    let empty_string = String::default();
    let v: Vec<i32> = Vec::default();
    println!("defaults: {zero}, '{empty_string}', {v:?}");

    // const generics: 陣列長度作為型別參數
    let arr_5 = [0; 5];
    let arr_3 = [0; 3];
    print_array(&arr_5);
    print_array(&arr_3);
}

fn print_array<const N: usize>(arr: &[i32; N]) {
    println!("array of length {}: {:?}", N, arr);
}
