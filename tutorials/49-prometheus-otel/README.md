# 49. Prometheus 與 OpenTelemetry

> 範圍：三大觀測 pillar、metrics-rs、Prometheus exporter、tracing → OTLP、Grafana 串接

## 三 pillar

```
metrics    → 計數、長條圖（多少 / 多快）       Prometheus
logs       → 結構化事件（發生了什麼）           Loki / ELK
traces     → request 跨服務的旅程              Tempo / Jaeger
```

實務上做好其中**兩個**就已經比多數團隊強了。

## metrics 概念

### 四種類型

| Type | 用途 | Rust API |
|------|------|----------|
| counter | 只增不減（請求數、錯誤數） | `counter!(...).increment(1)` |
| gauge | 上下浮動（連線數、queue 長度） | `gauge!(...).set(n)` |
| histogram | 分布（latency） | `histogram!(...).record(t)` |
| summary | 分布（client-side quantile，避免） | — |

**Rust 生態**用 `metrics` crate 為 facade（類似 `tracing` 之於 log），exporter 換不同後端。

```rust
use metrics::{counter, histogram};
counter!("http_requests_total", "route" => "/").increment(1);
histogram!("http_request_duration_seconds", "path" => "/").record(elapsed);
```

## Prometheus exporter

```rust
let recorder = PrometheusBuilder::new().install_recorder()?;
// /metrics 路由
async fn metrics() -> String { recorder.render() }
```

curl `localhost:8080/metrics` 拿到：

```
# HELP http_requests_total
# TYPE http_requests_total counter
http_requests_total{route="/"} 5

# HELP http_request_duration_seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{path="/",le="0.005"} 4
...
```

K8s 加 `ServiceMonitor`（kube-prometheus）讓 Prometheus 自動 scrape。

## 命名約定

```
<namespace>_<subsystem>_<name>_<unit>

http_request_duration_seconds  ✅  含單位
http_request_time              ❌  單位不明
```

unit 必加（`_seconds` / `_bytes` / `_total`）。

## Label cardinality 是雷區

```rust
// ❌ 每個 user_id 一條 series → 1M user = 1M series → 爆炸
counter!("api_calls", "user_id" => user_id).increment(1);

// ✅ 用低基數標籤
counter!("api_calls", "endpoint" => "/users", "method" => "GET").increment(1);
```

實務經驗：**任何 label 值的可能性 > 100 就要警惕**。

## tracing → OpenTelemetry

`tracing` 是 Rust 標準的 span / event 函式庫。透過 `tracing-opentelemetry` 把 span 送 OTLP collector：

```rust
let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(...)
    .install_batch(runtime::Tokio)?;

let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
tracing_subscriber::registry()
    .with(fmt_layer)
    .with(otel_layer)
    .init();
```

之後業務 code 完全不變：

```rust
#[tracing::instrument]
async fn handle(req: Request) {
    info!("processing");
}
```

span 自動產生、附上 attribute、trace id 跨服務傳遞。

## Trace context 跨服務

HTTP header 帶 `traceparent: 00-<trace-id>-<span-id>-01`：

```
service A → header → service B → header → service C
                    │                     │
                    └─── 同一 trace 樹 ───┘
```

`tower-http::trace` 或自寫 middleware 注入 / propagate。

## OTLP collector

```
your-app ─OTLP─→ otel-collector ─→ Tempo / Jaeger
                              └─→ Prometheus
                              └─→ Loki
```

collector 集中處理 sampling、redaction、轉發。`otel/opentelemetry-collector-contrib` image。

## Sampling

每 request 開 span 對效能有影響——大量場景要 sampling：

```rust
opentelemetry_sdk::trace::Config::default()
    .with_sampler(Sampler::TraceIdRatioBased(0.1)) // 10%
```

策略：
- **head sampling**：開始就決定（簡單但可能漏 error trace）
- **tail sampling**：collector 端看完整 trace 後決定（保留錯誤、慢請求）

## Grafana 串接

```
Prometheus → metrics
Tempo      → traces（OTLP 接收）
Loki       → logs（fluent-bit / promtail）
```

Grafana 在 UI 把三者連起來：**從 latency spike → drill 到對應 trace → 看 log 上下文**。

## Histogram bucket 選擇

```rust
PrometheusBuilder::new()
    .set_buckets(&[
        0.005, 0.01, 0.025, 0.05, 0.1,
        0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ])?
```

預設 bucket 不一定符合你 SLO。`P95 < 200ms` 的服務就要在 50-500ms 區段密集。

進階：**native histogram**（Prometheus 2.40+）省 cardinality + 自動 bucket。

## 結構化 log 也算觀測

`tracing` 寫 log 帶 trace id：

```
2026-05-20T10:00:00Z INFO request{trace_id=abc method=GET path=/api/users} processed in 23ms
```

ELK / Loki 都能查 `trace_id=abc` 跳到 Tempo 看 trace。

## 常見陷阱

1. **高 cardinality label** — 把 user_id / request_id 放 label 直接打爆 Prom。
2. **`info!()` 在 hot path** — JSON formatter 慢；hot loop 用 `debug!` + 不開。
3. **沒設 service name** — trace 上 service 都叫 unknown_service。`OTEL_SERVICE_NAME=ch49`。
4. **histogram bucket 沒對齊 SLO** — P99 計算錯誤。
5. **OTLP collector 沒設 timeout** — collector 慢拖累 app；用 batch + drop on full。
6. **/metrics 不要鎖認證** — Prom scrape 進不去；要用網路層隔離。
7. **`tracing-subscriber` init 兩次** — panic 或全靜默。
8. **sampling 太低看不到問題** — error 路徑強制 100% 採樣（tail sampling 解）。

## 練習

1. 加 RED metric（Rate / Error / Duration）每個 endpoint。
2. 寫 docker-compose 起 Prometheus + Grafana + Tempo，端點看 dashboard。
3. 改 sampling 策略：對 5xx response 100% 取樣，其他 10%。
4. 比較 `tracing::instrument` 自動 span 跟手動 `Span::current()` 寫法。

## 延伸閱讀

- [metrics-rs](https://docs.rs/metrics)
- [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry)
- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)
- [Prometheus best practices](https://prometheus.io/docs/practices/naming/)
