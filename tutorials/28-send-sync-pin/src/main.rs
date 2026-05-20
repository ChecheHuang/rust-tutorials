use std::rc::Rc;
use std::sync::Arc;
use std::thread;

// ── Send：能 move 到別的 thread ─────────────────────────────
fn send_ok() {
    let s = String::from("hello"); // String: Send
    thread::spawn(move || {
        println!("got: {s}");
    })
    .join()
    .unwrap();
}

// Rc 不是 Send，下面這段註解掉的會編譯失敗
// fn send_fail() {
//     let r = Rc::new(5);  // Rc 不是 Send
//     thread::spawn(move || {
//         println!("{r}");  // ERROR: `Rc<i32>` cannot be sent between threads
//     });
// }

// ── Sync：&T 能跨 thread share ──────────────────────────────
fn sync_ok() {
    let v = Arc::new(vec![1, 2, 3]); // Arc<Vec<T>>: Send + Sync
    let mut handles = vec![];
    for i in 0..3 {
        let v = Arc::clone(&v);
        handles.push(thread::spawn(move || {
            println!("thread {i}: {v:?}");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}

// ── 在 async 內持有非 Send 的東西，會讓 future 變非 Send ────
async fn async_with_rc() {
    let r = Rc::new(5);
    // 注意：跨 await 持有 Rc 會讓整個 future 變非 Send
    // tokio::spawn 需要 Send，下面這段註解掉
    tokio::task::yield_now().await;
    println!("rc = {r}");
}

#[tokio::main]
async fn main() {
    println!("=== Send ===");
    send_ok();

    println!("\n=== Sync ===");
    sync_ok();

    println!("\n=== async + Rc（同個 task 沒問題） ===");
    async_with_rc().await;

    // 但下面這行會編譯錯誤：
    // tokio::spawn(async_with_rc());  // ERROR: future is not Send

    // 改用 Arc 就行了
    println!("\n=== async + Arc（可 spawn） ===");
    tokio::spawn(async {
        let a = Arc::new(42);
        tokio::task::yield_now().await;
        println!("arc = {a}");
    })
    .await
    .unwrap();

    println!("\n=== Pin demo ===");
    pin_demo().await;
}

// ── Pin：阻止 self-referential 物件被搬動 ───────────────────
async fn pin_demo() {
    use std::pin::pin;

    let fut = async {
        let x = 5;
        tokio::task::yield_now().await; // 跨 await
        println!("x = {x}");
    };

    // pin! macro：在 stack 上 pin
    let mut fut = pin!(fut);
    fut.as_mut().await;
}
