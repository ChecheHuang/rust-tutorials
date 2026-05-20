use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot, watch, Mutex, RwLock};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== mpsc: multi-producer, single-consumer ===");
    mpsc_demo().await;

    println!("\n=== oneshot: one-time send ===");
    oneshot_demo().await;

    println!("\n=== broadcast: 廣播給所有訂閱者 ===");
    broadcast_demo().await;

    println!("\n=== watch: 永遠看得到最新值 ===");
    watch_demo().await;

    println!("\n=== async Mutex / RwLock ===");
    locks_demo().await;
}

async fn mpsc_demo() {
    // 容量 = 8 的 channel
    let (tx, mut rx) = mpsc::channel::<i32>(8);

    // 三個 producer
    for i in 0..3 {
        let tx = tx.clone();
        tokio::spawn(async move {
            for j in 0..3 {
                let v = i * 10 + j;
                tx.send(v).await.unwrap();
            }
        });
    }
    drop(tx); // 所有 sender drop 後 rx 才會 None

    while let Some(msg) = rx.recv().await {
        println!("  got: {msg}");
    }
}

async fn oneshot_demo() {
    let (tx, rx) = oneshot::channel::<String>();
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        tx.send("done".into()).unwrap();
    });
    let result = rx.await.unwrap();
    println!("  oneshot result: {result}");
}

async fn broadcast_demo() {
    let (tx, _) = broadcast::channel::<&str>(16);

    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx1.recv().await {
            println!("  rx1 got: {msg}");
        }
    });
    tokio::spawn(async move {
        while let Ok(msg) = rx2.recv().await {
            println!("  rx2 got: {msg}");
        }
    });

    tx.send("hello").unwrap();
    tx.send("world").unwrap();
    sleep(Duration::from_millis(50)).await;
}

async fn watch_demo() {
    let (tx, mut rx) = watch::channel(0u64);

    tokio::spawn(async move {
        for i in 1..=5 {
            sleep(Duration::from_millis(20)).await;
            tx.send(i).unwrap();
        }
    });

    while rx.changed().await.is_ok() {
        let v = *rx.borrow();
        println!("  watch saw: {v}");
        if v >= 5 {
            break;
        }
    }
}

async fn locks_demo() {
    // async Mutex
    let m = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];
    for _ in 0..10 {
        let m = Arc::clone(&m);
        handles.push(tokio::spawn(async move {
            let mut g = m.lock().await;
            *g += 1;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    println!("  Mutex final = {}", *m.lock().await);

    // async RwLock
    let rw = Arc::new(RwLock::new(vec![1, 2, 3]));

    let r1 = Arc::clone(&rw);
    let h1 = tokio::spawn(async move {
        let g = r1.read().await; // 多個 reader 可同時
        println!("  reader: {g:?}");
    });

    let r2 = Arc::clone(&rw);
    let h2 = tokio::spawn(async move {
        let g = r2.read().await;
        println!("  reader: {g:?}");
    });

    h1.await.unwrap();
    h2.await.unwrap();

    {
        let mut g = rw.write().await; // 獨佔
        g.push(4);
    }
    println!("  after write: {:?}", *rw.read().await);
}
