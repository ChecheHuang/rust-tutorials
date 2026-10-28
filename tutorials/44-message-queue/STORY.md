# 故事版：小鎮郵局的分揀中心 📮

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「小鎮郵局分揀中心」的故事，幫你「感覺」出訊息佇列、解耦和削峰、NATS 的 subject 與萬用字元、request/reply、JetStream 與 Redis Streams、consumer group、ack、at-least-once 與冪等、DLQ、outbox，還有 Kafka 的 partition 是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 以前：每封信都要親手送到

小鎮的麵包店老闆**阿德**，每天要通知很多人：「新口味出爐了！」「訂單做好了！」

以前他都**自己跑**去每一家敲門。要是對方不在家，他就站在門口一直等；過年訂單一多，他整天都在跑腿，麵包都烤焦了。

後來鎮上開了一間**郵局分揀中心**。阿德只要把信**丟進郵筒**，就可以回去烤麵包。誰來送、什麼時候送、送給幾個人，他都不用管。

過年那天信件爆量，郵局就先把信**堆在倉庫**，郵差一批一批慢慢送，誰都不會被擠垮。

> 這就是 **訊息佇列（MQ）**：**解耦**——producer（阿德）不需要知道誰會處理；**削峰填谷**——瞬間流量先寫進 queue 慢慢消化；還有失敗重試、一則事件多個訂閱者（fan-out）。

---

## 信封上的分區標籤

郵局的每一封信都要貼一張**分區標籤**，用點點隔開，由大到小：

- `活動.會員.新加入`
- `活動.訂單.已付款`

郵差可以只訂「`活動.會員.＊`」（會員底下**某一個**小類），也可以訂「`活動.>`」（**活動開頭的全部都要**）。

> 這就是 **NATS 的 subject 與萬用字元**：`client.publish(format!("events.kind{i}"), body.into())` 發佈到某個 subject；`client.subscribe("events.>")` 用 `>` 收下底下所有層；`*` 只配一層。`nats_demo.rs` 把 `Event { kind, payload }` 用 `serde_json::to_vec` 打包，收到後 `serde_json::from_slice(&msg.payload)` 打開。

### 廣場上的喊話

不過，郵局最基本的服務其實像在**廣場上喊話**：當下有在聽的郵差就聽到，**不在場的就錯過了**，郵局不會留底。

> 這就是 **core NATS pub/sub**：不持久、沒有 ack。要留存、可確認、可重播，要用 **JetStream**（`async_nats::jetstream::new(client)` 建 stream，consumer 處理完 `m.ack()`）。

---

## 附回郵信封的明信片

有一天阿德想問郵局：「我的信送到了嗎？」

他寄出一張明信片，**裡面附了一個回郵信封**，信封上寫著「請回寄到這裡」。郵局的服務員收到後，就把答案「**送到了！**」塞進那個回郵信封寄回來。阿德在家等著，很快就收到回音。

> 這就是 **request/reply**：`client.request("svc.ping", "hello?".into())` 寄出去並等回覆；服務端 `subscribe("svc.ping")`，收到後看 `msg.reply`（回郵信封的地址），`publish(reply, "pong".into())` 回寄。NATS 內建這種 RPC 模式。

---

## 會留底的包裹登記簿

除了喊話，郵局還有一本**包裹登記簿**。每件包裹進來，就**往下登記一行**，自動編一個號碼，不會擦掉。

郵局原本就有一塊小白板（第 42 章那塊），乾脆直接在那邊開一本登記簿，**不用另外蓋倉庫**。

> 這就是 **Redis Streams**：`redis_stream.rs` 的 producer 用 `p.xadd(STREAM, "*", &[("job", format!("work-{i}"))])` 往 `tasks` 追加 5 筆，`"*"` 是自動編號。優點是用既有 Redis、免新元件；缺點是規模上限不如 Kafka，DLQ 要自己做。

---

## 郵差小隊：一件包裹只派給一個人

郵局成立了一支**郵差小隊**，叫「`workers`」。阿凱是隊員 `w-1`。

規矩是：**同一件包裹，小隊裡只會派給一個人**，不會兩個人搶著送同一件。

阿凱每次跟櫃台說：「**給我新的**，一次最多 2 件；如果現在沒有，我在這等 2 秒。」

> 這就是 **consumer group**：`ensure_group()` 用 `XGROUP CREATE tasks workers $ MKSTREAM` 建小隊；`StreamReadOptions::default().group(GROUP, CONSUMER).count(2).block(2000)` 是「我是 workers 的 w-1、一次 2 筆、最多等 2000 毫秒」；`xread_options(&[STREAM], &[">"], &opts)` 的 `">"` 就是「給我還沒派給任何人的新包裹」。

---

## 送到要簽收

阿凱把包裹送到之後，一定要回郵局**蓋簽收章**。

如果阿凱半路跌倒、包裹沒送到、也沒回來簽收，郵局等了一陣子就會說：「這件還沒簽收，**再派一次**。」

所以有時候，同一件包裹可能**被送兩次**——第一次其實送到了，只是阿凱忘了回來簽。

> 這就是 **ack 與 at-least-once**：`conn.xack(STREAM, GROUP, &[entry.id])` 是簽收。沒 ack 的訊息會被重新派送 → **不會丟，但可能重複**。三種語意：at-most-once（可能丟）、at-least-once（不丟可能重）、exactly-once（極難，需要 outbox + 去重）。實務多採 **at-least-once + 冪等**。

### 收件人的小本子

麵包店的客人小花很聰明。她有一本小本子，記著「**收過的包裹編號**」。同一個編號再送來一次，她就說：「這個我收過了，謝謝。」不會付兩次錢。

> 這就是 **冪等（idempotency）**：consumer 用 message id 去重，重複收到也不會產生兩次副作用。沒做冪等，是 at-least-once 最常見的陷阱。

---

## 怎麼送都送不到的包裹

有一件包裹的地址寫錯了，怎麼送都送不到。阿凱一直被派去送同一件，後面的包裹全卡住了。

局長王伯伯說：「**失敗三次的，搬到『問題包裹區』**，另外處理，別擋到其他人。」

> 這就是 **DLQ（dead letter queue）**：多次處理失敗的「毒訊息」移到另一個 queue（例如 `tasks:dlq`），避免卡住整個 consumer。

---

## 先記在帳本，再交給郵局

有一天，阿德**在帳本上記了「訂單成立」**，正要去寄通知信，結果跌了一跤，信沒寄出去。客人一直等不到通知。

另一天反過來：信寄出去了，帳本卻忘了記。

後來阿德學會：**記帳的同時**，在同一本帳本的最後一頁「待寄清單」也寫一行。之後有個小幫手專門看那一頁，一行一行拿去郵筒寄，寄完打勾。就算跌倒，待寄清單也還在。

> 這就是 **outbox pattern**：在同一個 DB transaction 裡寫業務表 + outbox 表，commit 後由背景 worker 讀 outbox → publish → 標記已發。DB 是 source of truth，訊息可以重試。

---

## 大城市的分區輸送帶

隔壁的大城市郵局一天要處理上億件。他們把包裹分成好幾條**輸送帶**：**同一個地址的包裹，永遠走同一條帶**，所以同一家人收到的順序不會亂；但不同輸送帶之間，誰先誰後就不一定了。

每個郵差在自己的輸送帶上夾一個**書籤**，記著「我送到第幾件」。

還有一種郵局，信件先送到**分信台**，分信台依照規則決定放進哪一格信箱：照地址、照類別、或是全部信箱都放一份。

> 這就是 **Kafka 和 RabbitMQ**：Kafka 的 topic 分成 **partition**，同 **key** 進同一個 partition，順序只在 partition 內保證，**offset** 是書籤；Rust 用 `rdkafka`（包 librdkafka C 函式庫）。RabbitMQ 是 publisher → **exchange** → routing key → queue，exchange 有 direct / topic / fanout / headers；Rust 用 `lapin`。

---

## 局長的寄件守則

1. **別把鋼琴塞進信封。** 大東西放倉庫，信裡只寫「提貨單號碼」。
2. **廣場喊話不等於掛號信。** 要確定送到，就要簽收。
3. **別以為不同輸送帶會照順序到。**
4. **改信封格式要讓舊郵差也看得懂。** 新增欄位就好，別亂改舊的。

> 這就是 **常見陷阱**：大 payload 用 S3 + 訊息只帶 URL；把 at-most-once 當持久；假設跨 partition 有順序；schema 演進要 backward-compatible（protobuf / Avro / JSON Schema）。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 阿德把信丟進郵筒 | **producer** publish |
| 郵局分揀中心 | **訊息佇列（MQ）** |
| 過年先堆倉庫慢慢送 | **削峰填谷** |
| 分區標籤 `活動.會員.新加入` | **NATS subject** |
| 「`活動.>`」全部都要 | **萬用字元** `events.>`、`*` |
| 廣場上喊話 | **core NATS pub/sub**（不持久） |
| 附回郵信封的明信片 | **request/reply**：`client.request` + `msg.reply` |
| 會留底的包裹登記簿 | **Redis Streams** `xadd` / JetStream |
| 郵差小隊 workers、隊員 w-1 | **consumer group** `XGROUP CREATE` |
| 「給我新的，一次 2 件，等 2 秒」 | `xread_options(..., &[">"], opts)`、`count(2)`、`block(2000)` |
| 簽收章 | **ack**：`xack` |
| 沒簽收就再派一次 | **at-least-once** 重送 |
| 小花的收件編號本 | **冪等 consumer**（用 id 去重） |
| 問題包裹區 | **DLQ** |
| 帳本最後一頁的待寄清單 | **outbox pattern** |
| 同地址走同一條輸送帶 | **Kafka partition + key** |
| 郵差的書籤 | **offset** |
| 分信台 | **RabbitMQ exchange** |
| 信裡只寫提貨單號碼 | 大 payload 放 **S3**，訊息帶 URL |

---

## 一句話記住

> **訊息佇列是小鎮郵局：寄件人丟進郵筒就能回去工作，郵差小隊一件包裹只派一人、送到要簽收（ack），沒簽收就再送一次——所以收件人要記得「這個編號我收過了」（冪等），送不到的就搬去問題包裹區（DLQ）。**
