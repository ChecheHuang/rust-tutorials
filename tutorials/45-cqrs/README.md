# 45. CQRS 與 Event Sourcing

> 範圍：command / event 分離、aggregate、event store、projection、與 MQ 整合

## 兩個獨立但常配對的模式

| 模式 | 一句話 |
|------|--------|
| **CQRS**（Command-Query Responsibility Segregation） | 寫端跟讀端用不同模型 |
| **Event Sourcing** | 不存「當前狀態」，存「狀態變化的事件」 |

可以**只用 CQRS** 不用 ES、也可以**只用 ES** 不用 CQRS。組合起來常見因為很搭。

## 為什麼

- **讀寫負載差異大**：讀 100×，寫 1×。讀端做 denormalized view、寫端做業務驗證
- **完整審計**：事件即歷史，任何時點都能 replay 出當時狀態
- **時序分析**：「使用者過去 30 天變動過幾次密碼？」直接掃事件
- **bug 修復**：邏輯改了 → replay events → 重生 view

代價：**比 CRUD 複雜 3-5 倍**。

## 範例：銀行帳戶

### Commands（意圖）

```rust
enum Command {
    Open    { id: AccountId, owner: String },
    Deposit { id: AccountId, cents: i64 },
    Withdraw{ id: AccountId, cents: i64 },
}
```

Command 可能被**拒絕**（驗證失敗）。

### Events（事實）

```rust
enum Event {
    Opened    { id, owner, at },
    Deposited { id, cents, at },
    Withdrawn { id, cents, at },
}
```

Event 是**已發生**的事，永遠不會被改。append-only。

### Aggregate

```rust
struct Account { opened: bool, balance_cents: i64 }

impl Account {
    fn apply(&mut self, ev: &Event) { /* 純狀態變化 */ }
    fn handle(&self, cmd: &Command) -> Result<Vec<Event>, DomainError> {
        // 驗證業務規則 → 產生新事件
    }
}
```

`apply` 永遠不該失敗（事件已發生）；`handle` 才會驗證。

### 寫端流程

```
1. 收 command
2. load 該 aggregate 的所有歷史 events
3. fold apply 出當前狀態
4. handle(cmd) → 產生新 events（可能拒絕）
5. append events 到 store
6. publish events 到 bus
```

### 讀端流程

```
projection：subscribe events → 更新 read model 表
query：直接打 read model 表（不經 aggregate）
```

範例的 `BalanceView` 就是 read model；handler 收事件就 +/-。

## Event store 選擇

| 選項 | 說明 |
|------|------|
| **Postgres + 一張 events 表** | 最簡單、好上手 |
| **EventStoreDB** | 專門設計，subscription 強 |
| **Kafka** | 高吞吐、long-retention |
| **NATS JetStream** | 輕量、有 ack |

events 表 schema 大概：

```sql
CREATE TABLE events (
    id           BIGSERIAL PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    seq          BIGINT NOT NULL,
    type         TEXT NOT NULL,
    payload      JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (aggregate_id, seq)
);
```

`UNIQUE (aggregate_id, seq)` 是**樂觀並發**的關鍵：append 時帶當前 seq，重複會 fail → 重試。

## Snapshot 加速

aggregate 事件累積上千條 → load 慢。**定期 snapshot**：

```
load 流程：
  1. 取最新 snapshot（state + last_seq）
  2. 從 last_seq+1 開始 apply 後續 events
```

不是必須——只在效能瓶頸時才加。

## Projection 一致性

projection 是**最終一致**（eventually consistent）：
- 寫了一筆 → 讀端立刻查可能還沒看到
- 業務必須容忍這個延遲（通常毫秒級）

需要強一致場景（例如「下單後馬上看訂單頁」），有兩種解：
1. **read-your-own-writes**：寫完直接回 client，client 顯示樂觀 UI
2. **同步 projection**：寫事件後同 transaction 內也更新 read model（部分犧牲 CQRS 純度）

## Schema 演進

事件不能改。新欄位用：
- **upcaster**：load 舊事件時轉新格式
- **versioning**：`v1.UserCreated` → `v2.UserCreated`

**永遠不要 DELETE 或 UPDATE 舊事件**。

## 什麼場景不該用

- 純 CRUD 後台 → 過度設計
- 沒有審計 / replay 需求 → 收益微薄
- 團隊不熟事件思維 → 維護痛苦

**建議**：先寫 CRUD，**業務真的需要事件回溯時再轉**（資料遷移可寫成一次性 script）。

## Rust 生態

- **cqrs-es**：完整 framework，內建 Postgres / DynamoDB store
- **disintegrate** / **eventually**：較輕量
- **手寫**：本範例的方式，沒框架負擔

實務上手寫 + Postgres events 表 + projection worker 通常夠用。

## 常見陷阱

1. **修改舊事件** — 破壞 source of truth，禁止。
2. **projection 失敗沒重試** — read model 永遠停留在舊狀態。
3. **大事件** — 把整個 entity 序列化進事件 = 違反「事件記錄變化」原則；只記 delta。
4. **跨 aggregate transaction** — Event Sourcing 通常一筆 command 一個 aggregate；跨需要 saga / process manager。
5. **時鐘** — 事件 timestamp 不能依賴 client，server 寫入時填。
6. **GDPR / 刪除權** — append-only 跟「徹底刪除」矛盾；用 **crypto-shredding**（敏感欄位加密，丟 key = 失效）。

## 練習

1. 加 `Transfer` command（跨帳戶）：需要 saga 模式（兩個 aggregate 互動）。
2. 把 `InMemoryStore` 換成 Postgres events 表。
3. 加 snapshot：每 100 events 寫一份。
4. 加版本：`AccountOpenedV2` 多個 `email` 欄位，寫 upcaster。

## 延伸閱讀

- [Greg Young — CQRS Documents](https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf)
- [cqrs-es](https://docs.rs/cqrs-es)
- [Event Sourcing — Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
