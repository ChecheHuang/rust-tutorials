use futures::stream::{self, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() {
    println!("=== 基本：iter → stream ===");
    let mut s = stream::iter(vec![1, 2, 3, 4, 5]);
    while let Some(x) = s.next().await {
        println!("got: {x}");
    }

    println!("\n=== stream adapter ===");
    let total: i32 = stream::iter(1..=10)
        .map(|x| x * 2)
        .filter(|x| futures::future::ready(*x > 5))
        .fold(0, |acc, x| async move { acc + x })
        .await;
    println!("sum = {total}");

    println!("\n=== buffered: 並發 fetch，限制 N 個 ===");
    let urls = vec!["a", "b", "c", "d", "e"];
    let results: Vec<String> = stream::iter(urls)
        .map(|u| async move { fake_fetch(u).await })
        .buffer_unordered(3) // 同時最多 3 個並發
        .collect()
        .await;
    println!("results: {results:?}");

    println!("\n=== mpsc 轉 Stream ===");
    let (tx, rx) = mpsc::channel::<i32>(4);
    tokio::spawn(async move {
        for i in 1..=5 {
            tx.send(i).await.unwrap();
            sleep(Duration::from_millis(20)).await;
        }
    });

    let mut stream = ReceiverStream::new(rx);
    while let Some(x) = stream.next().await {
        println!("from channel stream: {x}");
    }

    println!("\n=== take / take_while / skip_while ===");
    let v: Vec<i32> = stream::iter(1..=20)
        .take(5) // 前 5
        .collect()
        .await;
    println!("take(5): {v:?}");

    let v: Vec<i32> = stream::iter(1..=20)
        .take_while(|x| futures::future::ready(*x < 7))
        .collect()
        .await;
    println!("take_while<7: {v:?}");

    println!("\n=== chunks_timeout ===");
    let (tx, rx) = mpsc::channel(16);
    tokio::spawn(async move {
        for i in 1..=10 {
            tx.send(i).await.unwrap();
            sleep(Duration::from_millis(5)).await;
        }
    });

    let mut s = ReceiverStream::new(rx).ready_chunks(4);
    while let Some(chunk) = s.next().await {
        println!("chunk: {chunk:?}");
    }
}

async fn fake_fetch(url: &str) -> String {
    sleep(Duration::from_millis(30)).await;
    format!("body-from-{url}")
}
