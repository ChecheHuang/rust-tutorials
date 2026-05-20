use std::time::Duration;
use tokio::time::sleep;

// ── async fn ────────────────────────────────────────────────
async fn fetch_data(id: u64) -> String {
    println!("  fetch_data({id}) start");
    sleep(Duration::from_millis(200)).await; // 模擬 IO
    println!("  fetch_data({id}) done");
    format!("data-{id}")
}

async fn compute() -> i32 {
    sleep(Duration::from_millis(100)).await;
    42
}

#[tokio::main]
async fn main() {
    println!("=== Sequential（順序執行） ===");
    let t0 = std::time::Instant::now();
    let a = fetch_data(1).await;
    let b = fetch_data(2).await;
    println!("results: {a}, {b}, took {:?}", t0.elapsed());

    println!("\n=== Concurrent with join!（並發） ===");
    let t0 = std::time::Instant::now();
    let (a, b) = tokio::join!(fetch_data(3), fetch_data(4));
    println!("results: {a}, {b}, took {:?}", t0.elapsed());

    println!("\n=== spawn — 真正放到背景 ===");
    let t0 = std::time::Instant::now();
    let h1 = tokio::spawn(fetch_data(5));
    let h2 = tokio::spawn(fetch_data(6));
    let a = h1.await.unwrap();
    let b = h2.await.unwrap();
    println!("results: {a}, {b}, took {:?}", t0.elapsed());

    println!("\n=== select! — 哪個先完成走哪個 ===");
    let answer = tokio::select! {
        n = compute() => format!("compute returned {n}"),
        _ = sleep(Duration::from_millis(50)) => "timeout first".into(),
    };
    println!("{answer}");

    println!("\n=== async closure (move) ===");
    let prefix = String::from("hello");
    let say = async move {
        sleep(Duration::from_millis(10)).await;
        println!("{prefix} from async block");
    };
    say.await;

    println!("\n=== try_join! — 任一失敗即返 ===");
    let result: Result<_, &str> = tokio::try_join!(may_fail(false), may_fail(false));
    println!("both ok: {result:?}");

    let result: Result<_, &str> = tokio::try_join!(may_fail(false), may_fail(true));
    println!("one fails: {result:?}");
}

async fn may_fail(fail: bool) -> Result<&'static str, &'static str> {
    sleep(Duration::from_millis(20)).await;
    if fail {
        Err("oops")
    } else {
        Ok("ok")
    }
}
