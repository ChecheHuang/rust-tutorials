use std::time::{Duration, Instant};
use tokio::task;
use tokio::time::sleep;

#[tokio::main(worker_threads = 4)]
async fn main() {
    println!("Tokio runtime: 4 worker threads");
    println!("current thread: {:?}\n", std::thread::current().id());

    // ── spawn：把 task 派給 runtime（可能跨 thread） ────────
    let h = tokio::spawn(async {
        sleep(Duration::from_millis(50)).await;
        println!("spawn: hello from {:?}", std::thread::current().id());
        42
    });
    let v = h.await.unwrap();
    println!("got value: {v}\n");

    // ── 大量 task：spawn 1000 個 ────────────────────────────
    let t0 = Instant::now();
    let mut handles = vec![];
    for i in 0..1000 {
        handles.push(tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            i
        }));
    }
    let mut sum = 0u64;
    for h in handles {
        sum += h.await.unwrap() as u64;
    }
    println!("1000 tasks sum = {sum}, took {:?}\n", t0.elapsed());

    // ── spawn_blocking：CPU bound 工作 ──────────────────────
    let h = task::spawn_blocking(|| {
        // 模擬 CPU 重活
        (1..=10_000_000u64).sum::<u64>()
    });
    let big_sum = h.await.unwrap();
    println!("blocking sum = {big_sum}\n");

    // ── yield_now：禮讓 scheduler ───────────────────────────
    tokio::spawn(yield_demo());
    sleep(Duration::from_millis(50)).await;

    // ── current_thread runtime（single-threaded） ──────────
    // 適合 GUI / embedded / 已在另一個 thread 內想跑 async
    println!("\nstarting current_thread runtime...");
    let local_rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    local_rt.block_on(async {
        println!("inside current_thread, thread = {:?}", std::thread::current().id());
    });
    println!("current_thread done");

    // ── task::JoinSet：管理一群 task ────────────────────────
    println!("\nJoinSet demo:");
    let mut set = task::JoinSet::new();
    for i in 1..=5 {
        set.spawn(async move {
            sleep(Duration::from_millis((6 - i) * 20)).await;
            i
        });
    }
    while let Some(res) = set.join_next().await {
        println!("  finished: {:?}", res.unwrap());
    }
}

async fn yield_demo() {
    for i in 0..5 {
        println!("yield_demo iter {i}");
        task::yield_now().await; // 主動讓出
    }
}
