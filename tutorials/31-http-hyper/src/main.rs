use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Post {
    #[serde(rename = "userId")]
    user_id: u32,
    id: u32,
    title: String,
    #[allow(dead_code)]
    body: String,
}

#[derive(Debug, Serialize)]
struct NewPost {
    title: String,
    body: String,
    user_id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("rust-tutorials/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // ── 1. GET 簡單字串 ─────────────────────────────────────
    let resp = client
        .get("https://httpbin.org/get?source=rust")
        .send()
        .await?;
    println!("status = {}", resp.status());
    println!("headers:");
    for (k, v) in resp.headers() {
        println!("  {k}: {v:?}");
    }
    let text = resp.text().await?;
    println!("body (first 200 chars):\n{}\n", &text[..text.len().min(200)]);

    // ── 2. GET JSON deserialize 成 struct ───────────────────
    let post: Post = client
        .get("https://jsonplaceholder.typicode.com/posts/1")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    println!("got post: {post:?}\n");

    // ── 3. POST JSON ─────────────────────────────────────────
    let new = NewPost {
        title: "rust rocks".into(),
        body: "hello from rust".into(),
        user_id: 1,
    };
    let resp = client
        .post("https://jsonplaceholder.typicode.com/posts")
        .json(&new)
        .send()
        .await?;
    println!("POST status = {}", resp.status());
    let echoed: serde_json::Value = resp.json().await?;
    println!("server echoed: {echoed}\n");

    // ── 4. 自訂 headers + query params ──────────────────────
    let resp = client
        .get("https://httpbin.org/headers")
        .header("X-Custom", "foo")
        .header("Authorization", "Bearer dummy-token")
        .query(&[("q", "rust"), ("limit", "10")])
        .send()
        .await?;
    let v: serde_json::Value = resp.json().await?;
    println!("echoed headers/query: {}\n", v);

    // ── 5. concurrent fetch ─────────────────────────────────
    let ids = [1u32, 2, 3, 4, 5];
    let futs = ids.iter().map(|&id| {
        let c = &client;
        async move {
            let p: Post = c
                .get(format!("https://jsonplaceholder.typicode.com/posts/{id}"))
                .send()
                .await?
                .json()
                .await?;
            Ok::<_, reqwest::Error>(p)
        }
    });
    let results: Vec<_> = futures::future::join_all(futs).await;
    println!("concurrent results:");
    for r in results {
        match r {
            Ok(p) => println!("  id={} title='{}'", p.id, p.title),
            Err(e) => println!("  err: {e}"),
        }
    }

    Ok(())
}
