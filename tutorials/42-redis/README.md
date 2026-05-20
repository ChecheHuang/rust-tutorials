# 42. Redis

> 範圍：connection manager、常用指令、cache aside、rate limit、pub/sub、Lua

## crate 選擇

| crate | 特點 |
|-------|------|
| **redis** | 官方推薦，async + sync、cluster 支援 |
| **fred** | 全 async、設計現代、自動 pipelining |
| **rustis** | 純 async tokio |

新專案多選 `redis` 或 `fred`。

## Connection 管理

```rust
let client = redis::Client::open("redis://127.0.0.1:6379")?;
let mut conn = redis::aio::ConnectionManager::new(client).await?;
```

`ConnectionManager` 自動重連、有 backoff。不要為每 request 新開 connection。

連線池版本：`deadpool-redis` 或 `bb8-redis`，適合需要多 connection 並發場景。

## 常用 commands（用 `AsyncCommands` trait）

```rust
let _: () = conn.set("k", "v").await?;
let v: String = conn.get("k").await?;
let _: () = conn.set_ex("k", "v", 60).await?;         // TTL 60s
let _: () = conn.expire("k", 30).await?;              // 重設 TTL
let n: i64 = conn.incr("counter", 1).await?;          // atomic
let _: () = conn.lpush("queue", "task").await?;       // list
let _: () = conn.sadd("set", "member").await?;        // set
let _: () = conn.hset("h", "field", "val").await?;    // hash
let exists: bool = conn.exists("k").await?;
let _: i64 = conn.del("k").await?;
```

回傳型別用 turbofish 標：`conn.get::<_, Option<String>>(key).await?`，或 binding 寫死。

## Cache-aside pattern

```
request → 查 cache
            ├ hit → return
            └ miss → 查 DB → 寫 cache（TTL）→ return
```

```rust
async fn get_user(conn, id) {
    if let Some(s) = conn.get(&format!("user:{id}")).await? {
        return parse(s);
    }
    let user = db.fetch_user(id).await?;
    conn.set_ex(&format!("user:{id}"), serialize(&user), 60).await?;
    Ok(user)
}
```

⚠️ **stale-while-revalidate** / cache stampede：高並發 miss 時所有 request 同時打 DB。對策：
- 加 lock（SETNX + TTL）
- 用 `singleflight` 風格 in-process dedupe
- 提前刷新（TTL 80% 時開背景更新）

## Rate limit

### Fixed window

```rust
let n: i64 = conn.incr(&format!("rl:{ip}"), 1).await?;
if n == 1 { conn.expire(&format!("rl:{ip}"), 10).await?; }
return n <= LIMIT;
```

簡單但邊界 burst 風險。

### Sliding log

把每次 request timestamp 推 sorted set，刪舊的，看當前數量。較精準但成本高。

### Token bucket

複雜但平滑——多用 Lua script 原子化：

```lua
-- KEYS[1] = bucket key
-- ARGV[1] = now, ARGV[2] = rate, ARGV[3] = burst
...
```

## Pub/Sub

```rust
let mut pubsub = client.get_async_pubsub().await?;
pubsub.subscribe("events").await?;
let mut stream = pubsub.on_message();
while let Some(msg) = stream.next().await {
    let payload: String = msg.get_payload()?;
}
```

注意：
- pub/sub 是 **fire-and-forget**，下線就丟訊息
- 不保證順序、不保證送達
- 持久訊息要用 **Streams**（XADD / XREAD）或專業 MQ（第 44 章）

## Redis Streams（取代 pub/sub for 持久訊息）

```rust
let _: String = redis::cmd("XADD").arg("stream").arg("*")
    .arg("field").arg("value").query_async(&mut conn).await?;

// consumer group
redis::cmd("XGROUP").arg("CREATE").arg("stream").arg("grp").arg("$").arg("MKSTREAM")
    .query_async(&mut conn).await?;
```

## 分散式鎖（Redlock）

```
SET resource_name unique_id NX PX 30000
```

`NX` = 只在不存在時設、`PX` = millisecond TTL。**unique_id** 是 client 自己生的，釋放鎖時用 Lua script 比對，避免誤刪別人的鎖。

⚠️ Redlock 在跨多 Redis 節點下有爭議，討論可見 Martin Kleppmann vs antirez 的辯論。**單機 Redis 加 fencing token** 對絕大多數場景已足夠。

## Lua scripting

當需要**原子化多 command**：

```rust
let script = redis::Script::new(r#"
    if redis.call("GET", KEYS[1]) == ARGV[1] then
        return redis.call("DEL", KEYS[1])
    end
    return 0
"#);
let res: i64 = script.key("lock").arg(token).invoke_async(&mut conn).await?;
```

## 常見陷阱

1. **沒設 TTL** — cache 累積成記憶體墓場。預設給 TTL。
2. **大 key** — 單 key 幾 MB 會 block；拆分 / 用 hash field。
3. **`KEYS *`** — 生產禁用，阻塞整個 server；用 `SCAN`。
4. **redis as DB** — RDB / AOF 不是強耐久；重要資料還是要 source of truth 在 DB。
5. **連線數爆掉** — 多 worker × 多 client = N×M 條 connection。用 connection manager / pool。
6. **JSON 存值 vs hash field** — 整個取整個寫的場景用 JSON string；部分讀寫用 hash。
7. **`MULTI`/`EXEC`** — Redis 的 transaction 不是 SQL transaction，是「打包執行不被插隊」，沒有回滾。

## 練習

1. 把第 38 章 JWT 加 blacklist：登出後把 `jti` 寫進 Redis with TTL。
2. 寫一個 token bucket rate limit（Lua script 版本）。
3. 用 Streams 做 webhook delivery queue，含 consumer group + retry。
4. 比較 cache-aside 與 write-through 在你 app 的取捨。

## 延伸閱讀

- [redis-rs](https://docs.rs/redis)
- [Redis docs](https://redis.io/docs/)
- [Distributed locks with Redis](https://redis.io/docs/manual/patterns/distributed-locks/)
