# 44. 訊息佇列

> 範圍：NATS / Redis Streams / Kafka / RabbitMQ 取捨、producer / consumer、consumer group、at-least-once

## 為什麼要 MQ

- **解耦**：生產者不知道誰會處理
- **削峰填谷**：瞬間流量寫進 queue，慢慢消化
- **重試**：失敗訊息可以放回 / DLQ
- **fan-out**：一條事件多個訂閱者

## 主流選項

| 系統 | 模型 | Rust crate | 強項 |
|------|------|------------|------|
| **NATS / JetStream** | pub/sub、persistent stream | `async-nats` | 輕量、低延遲、好用 |
| **Redis Streams** | log + consumer group | `redis` | 已有 Redis 就免新元件 |
| **Kafka** | partitioned log | `rdkafka`（librdkafka C 包） | 高吞吐、生態最大 |
| **RabbitMQ** | AMQP（exchange / queue / routing） | `lapin` | 複雜 routing、傳統企業 |
| **NSQ** | pub/sub | `nsq` | 簡單 |

**新專案建議**：先看是否能用 NATS（JetStream）+ Redis Streams。Kafka 適合「日吞吐億條」級。

## NATS：核心模型

```
publish ─→ subject ─→ subscribe
              ↑
              通配符 events.> events.* 等
```

範例 subject：`events.user.created` / `events.order.paid`。subscriber 可以訂 `events.user.*` 或 `events.>`。

```rust
let client = async_nats::connect("nats://127.0.0.1:4222").await?;
let mut sub = client.subscribe("events.>").await?;
client.publish("events.user.created", body.into()).await?;

while let Some(msg) = sub.next().await {
    // msg.subject, msg.payload
}
```

### JetStream（持久）

純 NATS pub/sub 不持久。要持久 + ack / replay 用 **JetStream**：

```rust
let js = async_nats::jetstream::new(client);
js.create_stream(jetstream::stream::Config {
    name: "ORDERS".into(),
    subjects: vec!["orders.>".into()],
    ..Default::default()
}).await?;

let consumer = stream.create_consumer(...).await?;
let mut msgs = consumer.messages().await?;
while let Some(msg) = msgs.next().await {
    let m = msg?;
    process(&m).await?;
    m.ack().await?;
}
```

### Request/Reply

NATS 內建 RPC 模式：

```rust
let resp = client.request("svc.foo", payload).await?;
```

server 端 subscribe + 用 `msg.reply` publish 回去。

## Redis Streams

Redis 5.0+ 提供類 Kafka 的 log + consumer group：

```rust
conn.xadd("tasks", "*", &[("job", "work-1")]).await?;   // 寫
conn.xread_options(&["tasks"], &[">"], &opts).await?;   // 讀
conn.xack("tasks", "workers", &[id]).await?;            // ack
```

優勢：用既有 Redis，沒新元件。劣勢：規模上限不如 Kafka、需要自己處理 DLQ。

## Consumer group（at-least-once）

```
stream/topic ──┬─→ consumer-1（讀 partition A）
               ├─→ consumer-2（讀 partition B）
               └─→ consumer-3（讀 partition C）
```

每條訊息 group 內**只有一個** consumer 拿到。配合 **ack**：
- 處理成功 → ack
- 失敗 / timeout → 訊息會被重新派送
- 多次失敗 → DLQ

**at-least-once 重要前提**：consumer 必須做**冪等**處理。重複收到不能造成副作用變動兩次。

## 三種 delivery semantics

| 語意 | 意思 | 怎麼達成 |
|------|------|---------|
| at-most-once | 可能丟 | publish 後不 ack |
| at-least-once | 不丟可能重 | ack + idempotent consumer |
| exactly-once | 不重不丟 | 需要 transactional outbox + 去重，極難純 MQ 達成 |

Kafka 有 exactly-once API 但限制多。**實務多採 at-least-once + idempotency**。

## Outbox pattern（與 DB 一致性）

問題：DB 寫成功但訊息發送失敗（或反之）→ 資料不一致。

解：
```
1. transaction 內：寫業務表 + 寫 outbox 表
2. commit
3. 背景 worker：讀 outbox → publish → 標記已發
```

訊息系統可重新嘗試，DB 是 source of truth。

## Kafka 概念對照

| Kafka 名詞 | 對應 |
|------------|------|
| topic | 類似 NATS subject / Redis stream |
| partition | 分片，順序保證在 partition 內 |
| consumer group | 同上 |
| offset | 消費位置 |
| key | 決定 partition（同 key 同 partition） |

Rust 客戶端 `rdkafka` 需要 librdkafka C 函式庫；交叉編譯有點麻煩，但效能極佳。

## RabbitMQ 概念

```
publisher → exchange ──routing key──→ queue ──→ consumer
```

exchange 類型：direct / topic / fanout / headers。比 NATS / Kafka 更彈性也更複雜。

## 常見陷阱

1. **at-most-once 當持久** — pub/sub 不 ack 模式會丟訊息。
2. **consumer 沒做冪等** — at-least-once 重送會產生重複資料。
3. **訊息順序假設** — 跨 partition 沒順序保證；同 key 才有。
4. **大 payload** — MQ 都不適合大 blob；用 S3 + 訊息只帶 URL。
5. **沒 DLQ** — 毒訊息卡住整個 consumer。
6. **客戶端 buffer** — `async-nats` 的 channel buffer 滿了會 backpressure；要設合理大小。
7. **跨服務 schema 演進** — 加欄位用 backward-compatible 策略（protobuf / Avro / JSON Schema）。

## 練習

1. 用 NATS 寫 producer / consumer，consumer 做冪等（用 message id 去重）。
2. 把 Redis Streams 範例加 DLQ：超過 3 次失敗的訊息搬到 `tasks:dlq`。
3. 評估你的場景該選哪個 MQ：寫一個對比表。
4. 用 outbox pattern + transaction 解決 DB / MQ 一致性。

## 延伸閱讀

- [async-nats](https://docs.rs/async-nats)
- [NATS JetStream](https://docs.nats.io/nats-concepts/jetstream)
- [Redis Streams](https://redis.io/docs/data-types/streams/)
- [rdkafka](https://docs.rs/rdkafka)
