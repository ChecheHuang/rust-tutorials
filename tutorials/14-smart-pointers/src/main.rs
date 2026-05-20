use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

// ── 遞迴型別需要 Box ────────────────────────────────────────
#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

fn main() {
    // ── Box<T>：唯一擁有者、heap 配置 ────────────────────────
    let boxed: Box<i32> = Box::new(42);
    println!("box: {boxed}, deref: {}", *boxed);

    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
    println!("recursive list: {list:?}");

    // ── Rc<T>：單執行緒共享所有權（refcount） ───────────────
    let a = Rc::new(String::from("shared"));
    let b = Rc::clone(&a);
    let c = Rc::clone(&a);
    println!("Rc count = {}", Rc::strong_count(&a)); // 3
    println!("a={a} b={b} c={c}");
    drop(b);
    drop(c);
    println!("after drop, count = {}", Rc::strong_count(&a)); // 1

    // ── RefCell<T>：runtime borrow check（內部可變） ────────
    let cell = RefCell::new(5);
    *cell.borrow_mut() += 10;
    println!("cell = {}", cell.borrow());

    // 跑 panic 演示：兩個 borrow_mut 會 panic
    let result = std::panic::catch_unwind(|| {
        let c = RefCell::new(0);
        let _b1 = c.borrow_mut();
        let _b2 = c.borrow_mut(); // PANIC: already mutably borrowed
    });
    println!("double borrow_mut panicked? {}", result.is_err());

    // ── Rc<RefCell<T>>：共享 + 可變（單執行緒） ─────────────
    let shared_mut = Rc::new(RefCell::new(vec![1, 2, 3]));
    let other = Rc::clone(&shared_mut);
    shared_mut.borrow_mut().push(4);
    other.borrow_mut().push(5);
    println!("shared+mut = {:?}", shared_mut.borrow());

    // ── Arc<T>：跨執行緒共享（atomic refcount） ─────────────
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];
    for i in 0..3 {
        let d = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            println!("thread {i}: {d:?}");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // ── Arc<Mutex<T>>：跨執行緒共享 + 可變 ──────────────────
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            *c.lock().unwrap() += 1;
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("final count = {}", *counter.lock().unwrap());

    // ── Cow：clone-on-write ─────────────────────────────────
    let s1 = process("hello"); // 不變動 → 借用，零 alloc
    let s2 = process("hello world contains spaces"); // 變動 → 配置新 String
    println!("s1 (borrowed): {s1}");
    println!("s2 (owned):    {s2}");
}

fn process(s: &str) -> Cow<str> {
    if s.contains(' ') {
        // 需要修改 → 取得 owned
        Cow::Owned(s.replace(' ', "_"))
    } else {
        // 不需修改 → 借用
        Cow::Borrowed(s)
    }
}
