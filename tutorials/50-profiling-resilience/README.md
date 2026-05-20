# 50. Profiling 與韌性

> 範圍：tokio-console、flamegraph、pprof、timeout / retry / circuit breaker / rate limit

## 兩個獨立但相關的主題

1. **profiling**：找慢、找 leak、找 deadlock
2. **resilience**：對外部失敗保護自己 + 防止連環崩潰

## Profiling

### 1. tokio-console — 看 async runtime

```toml
[dependencies]
tokio = { version = "1", features = ["full", "tracing"] }
console-subscriber = "0.4"
```

```rust
console_subscriber::init();
```

跑 `tokio-console` 連到 app：看到每個 task、wait time、resource、blocking 操作。

**找問題的場景**：
- task 多但沒進度（被 mutex 卡）
- 某 task wake 太頻繁
- channel 滿了沒人 consume

### 2. flamegraph — 看 CPU hot spot

```bash
cargo install flamegraph
cargo flamegraph --bin myapp
# 產生 flamegraph.svg
```

Linux 用 perf，macOS 用 dtrace。SVG 是互動式——點某層往下看。

**讀 flamegraph**：
- 寬 = CPU 時間占比
- 高 = call stack 深
- 找又寬又靠頂的 → hot function

### 3. pprof-rs / cargo-pgo

```rust
let guard = pprof::ProfilerGuardBuilder::default()
    .frequency(1000)
    .build()?;
// ... 跑業務 ...
let report = guard.report().build()?;
let file = std::fs::File::create("flamegraph.svg")?;
report.flamegraph(file)?;
```

優勢：嵌進 binary，runtime 取 sample → /debug/pprof endpoint。

### 4. heaptrack / valgrind massif — 找 mem leak

Rust 安全模式下不易 leak（除非 `Box::leak` 或 Rc cycle）。但**過量 alloc**（短生命物件）會拖速度。

- `heaptrack` 是 Linux 上最方便
- `dhat-rs` crate 內嵌測量 alloc / dealloc

### 5. cargo bench + criterion

第 25 章詳細，這裡略。

### 6. PGO（Profile-Guided Optimization）

`cargo-pgo` 自動化兩階段 build：
1. profile build → 跑代表性 workload
2. 用 profile data 重 build → 10–20% 提速常見

值得在 latency-critical service 試。

## Resilience 四件套

### 1. timeout（永遠要設）

```rust
let res = tokio::time::timeout(Duration::from_secs(2), call()).await;
```

**沒 timeout 的 IO 等於放火**：一個慢 backend 拖垮整個 service。

實務分層：
- HTTP client：connect timeout 1s、request timeout 5s
- DB query：5s（OLTP）/ 60s（reporting）
- background task：依業務

### 2. retry + backoff + jitter

```rust
for attempt in 1..=max {
    match op().await {
        Ok(v) => return Ok(v),
        Err(_) => {
            let wait = base_ms * 2u64.pow(attempt) + jitter();
            sleep(wait).await;
        }
    }
}
```

關鍵：
- **exponential backoff**：50ms → 100 → 200 → 400 → ...
- **jitter**：±50% 隨機，避免多 client 同步 retry（thundering herd）
- **只 retry idempotent 操作**：GET 安全，POST 危險（除非 idempotency key）
- **5xx + timeout retry**，4xx 不要 retry（除了 429 + Retry-After）

### 3. circuit breaker

```
closed   → 正常通過，計失敗
  ↓ N 次失敗
open     → 直接拒絕，不打後端
  ↓ 冷卻時間
half-open → 放少量探測
  ↓ 成功
closed
```

**避免雪崩**：後端炸了，client 還拼命 retry → 後端更炸。breaker 開啟後快速失敗，給後端喘息。

Rust 生態：`failsafe`、`tower::limit`、自寫（本範例）。

### 4. rate limit

兩端：
- **server 端**：限制每 client 速率（governor + tower）
- **client 端**：限制自己對外速率（避免被對方 rate limit）

```rust
let quota = Quota::per_second(NonZeroU32::new(100).unwrap());
let lim = RateLimiter::direct(quota);
lim.until_ready().await;        // 阻塞到有 token
```

`governor` 用 token bucket，記憶體高效。

## tower 的整合

tower 有現成的 layer：

```rust
ServiceBuilder::new()
    .layer(TimeoutLayer::new(Duration::from_secs(2)))
    .layer(RetryLayer::new(MyPolicy))
    .layer(RateLimitLayer::new(100, Duration::from_secs(1)))
    .service(client);
```

任何 tower service 都套得上 — 包括 reqwest、hyper、tonic。

## bulkhead pattern

把資源分桶：DB pool 一份給寫、一份給讀；不同 endpoint 用不同 semaphore。**一個 endpoint 慢爆不至於拖垮全部**。

```rust
let read_sem = Arc::new(Semaphore::new(50));
let permit = read_sem.acquire().await?;
// query...
```

## graceful degradation

外部依賴掛了，**回退到較簡版**而非整個壞：
- 推薦系統掛 → 回熱門 default 列
- cache 掛 → 直接查 DB（慢但能用）
- 翻譯 service 掛 → 顯示原文

寫程式時就要思考：「這個失敗該 fail-fast 還是 fallback？」

## 觀察 + alert

profiling 是事後、resilience 是事前 — **observability**（第 49 章）告訴你哪裡需要加。

關鍵指標：
- **error rate** > 1%：alert
- **P99 latency** > SLO：alert
- **circuit open**：alert（後端有事）
- **retry rate** > 10%：alert（也是後端有事）

## 常見陷阱

1. **沒 timeout 的 reqwest** — 預設沒 timeout，連到掛掉的 server 永遠等。
2. **retry 沒 jitter** — 故障恢復時 thundering herd。
3. **retry 非 idempotent** — 重複扣款。
4. **breaker 太敏感** — 偶發失敗就斷，用戶受影響。`failure_rate` 而非 `failure_count`。
5. **rate limit token 不足初始 burst** — 啟動瞬間擋住所有請求；用 burst 容量。
6. **tokio-console 上 production** — 有 overhead，預設只在 dev profile 開。
7. **profile 用錯 workload** — 用測試 load profile 出來的熱點跟生產不同。
8. **flamegraph 抓不到 async** — 跨 await 的 stack 斷掉；用 `tokio-console` 補。

## 練習

1. 寫一個對外 client：含 timeout + retry + breaker，模擬 backend 掛 5 秒，觀察行為。
2. 用 `tokio-console` 找一個故意 deadlock 的 task。
3. 用 `cargo flamegraph` profile 第 25 章的 sum benchmark，比較三種寫法的 hot spot。
4. 設計 graceful degradation：cache 掛時的 fallback 路徑。

## 延伸閱讀

- [tokio-console](https://github.com/tokio-rs/console)
- [flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [Release It! — Michael Nygard](https://pragprog.com/titles/mnee2/release-it-second-edition/)
- [Tower retry / timeout layers](https://docs.rs/tower)
