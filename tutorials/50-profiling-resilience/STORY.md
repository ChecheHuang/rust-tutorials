# 故事版：麵包店的監視器和打電話守則 🍞

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「社區麵包店找出哪裡慢、又要跟不太可靠的麵粉行打交道」的故事，幫你「感覺」出 tokio-console、flamegraph、timeout、retry + backoff + jitter、circuit breaker、rate limit、bulkhead 和 graceful degradation。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 店裡裝了一台監視器

老陳的麵包店有好幾個學徒，大家都很忙，可是麵包就是出得慢。

老陳在店裡裝了一台**監視器**，畫面上每個學徒頭上都有一個小標籤：「正在揉麵」「在等烤箱」「在等冰箱鑰匙」。看了一下午，他發現：阿豆、小麥、阿福三個人都在**等同一把冰箱鑰匙**，而拿著鑰匙的人跑去接電話了。

> 這就是 **tokio-console**：在程式裡 `console_subscriber::init()`，再用 `tokio-console` 連上去，就能看到每個 async task 在忙什麼、等了多久、被哪個資源（mutex、channel）卡住。它有額外負擔，通常只在開發環境開。

---

## 一張越寬越燒時間的蛋糕圖

另一個問題：烤箱一直很燙，可是不知道是哪一道步驟在吃時間。

老陳把一整天的工作畫成一塊**層層疊起的蛋糕**：最底層是「做麵包」，上面一層分成「揉麵」「發酵」「烤」，再上面又細分……**每一層越寬，代表花的時間越多**。

他一眼看到最上層有一塊**又寬又在頂端**的：「手工一顆一顆數芝麻」。改用量匙之後，整條產線快了一大截。

> 這就是 **flamegraph**：`cargo flamegraph --bin myapp` 產生互動式 SVG。**寬 = CPU 時間占比、高 = 呼叫堆疊深度**，找又寬又靠頂的就是 hot function。`pprof-rs` 可以把取樣器嵌進程式裡。注意 flamegraph 跨 `await` 的堆疊會斷掉，async 問題要搭配 tokio-console 看。

---

## 打電話最多等一下

麵粉行的老闆娘接電話有時很快，有時一直不接。

有一次阿豆打過去，對方一直沒接，他就**拿著話筒站了一整個早上**，烤箱旁沒人顧，整家店都停了。

老陳定下第一條守則：「**打電話最多等一下下，沒接就掛掉，先去做別的事。**」

> 這就是 **timeout**：範例的 `with_timeout(fut, ms)` 用 `tokio::time::timeout(Duration::from_millis(ms), fut)` 包住任何 future，超時就回 `Err("timeout 100ms")`。demo 裡要睡 300ms 的工作，被 100ms 的 timeout 截斷。**沒有 timeout 的 IO 等於放火**，像 reqwest 預設就沒有 timeout。

---

## 沒接通？等一下再打，而且越等越久

麵粉行的電話常常前兩次佔線，第三次才通。

老陳教阿豆：「沒接通不要馬上狂打。**第一次等一下下，第二次等兩倍，第三次再兩倍**，最多等到一個上限就好。」

可是社區裡還有五家店也在打給麵粉行。如果大家都「剛好等 1 分鐘再打」，五支電話會**同一秒**一起響，又全部佔線。所以老陳又說：「每次等的時間，再**隨便多加一點點**，大家就錯開了。」

還有一條：「問『麵粉還有沒有』可以重打；但『**付訂金**』不可以亂重打，不然會付兩次！」

> 這就是 **retry + exponential backoff + jitter**：範例的 `retry(op, max)` 從 `backoff_ms = 50` 開始，每次失敗加上 `gen_range(0..backoff_ms / 2)` 的隨機 jitter 再 `sleep`，然後 `backoff_ms = (backoff_ms * 2).min(2000)`。`flaky_call` 前兩次失敗、第三次成功。jitter 是為了避免 **thundering herd**。只重試 **idempotent** 的操作；5xx 和 timeout 重試，4xx 不要（429 例外）。

---

## 電話上貼「暫停」貼紙

有一天麵粉行整個停電。阿豆一直打、一直失敗。

老陳說：「**連續失敗 3 次**，就在電話上貼『暫停』貼紙。貼紙還在的時候，誰都**不准打**，直接當作失敗，去用備用麵粉。」

過了一陣子，貼紙到期，老陳把它換成一張黃色的「**試打中**」貼紙：只派**一個人**去試打一通，其他人還是不准碰電話。通了，撕掉貼紙、失敗次數歸零；又沒通，就再貼一張新的「暫停」。

這樣麵粉行恢復電力的時候，才不會被一大堆電話同時轟炸。

> 這就是 **circuit breaker**：範例的 `Breaker::new(3, Duration::from_millis(500))`，狀態是 `BreakerState` 三種：`Closed { failures }` 正常通過並計失敗；連續失敗到 `threshold` 就變 `Open { until }`，冷卻期間 `call()` 直接回 `Err("circuit open")`、根本不呼叫後端；冷卻到了，第一個請求把狀態改成 **`HalfOpen`** 當探測，探測期間其他請求一樣被拒。探測成功 → `Closed`，失敗 → 再 `Open`。demo 先連打 5 次（3 次真的失敗、2 次被擋），再示範一次探測失敗、一次探測成功。

---

## 一秒只能打五通

麵粉行老闆娘說：「你們店一秒鐘最多打 5 通，再多我就不接了。」

老陳在電話旁放了一個**代幣罐**：一開始裝著 5 枚代幣，之後每隔一小段時間自動補一枚。打電話前要先拿一枚，**罐子空了就排隊等**，等到有代幣才打。

阿豆一口氣要打 12 通：前 5 通馬上打完，後面的就一通一通、**平均每 0.2 秒**一通慢慢打。

> 這就是 **rate limit（token bucket）**：`rate_limited_demo()` 用 `governor` 的 `Quota::per_second(NonZeroU32::new(5).unwrap())` 建 `RateLimiter::direct(quota)`，每次 `lim.until_ready().await` 拿到 token 才繼續，log 的 `elapsed_ms` 可以看到節奏。server 端限制別人、client 端限制自己，兩端都會用到。

---

## 不同工作用不同的烤箱

以前店裡只有一座大烤箱。生日蛋糕訂單一多，烤箱被蛋糕佔滿，**早餐吐司就全部出不來**。

老陳把烤箱分成兩座：一座專門烤蛋糕，一座專門烤吐司。蛋糕再怎麼塞爆，吐司照樣準時出爐。

> 這就是 **bulkhead pattern**：把資源分桶，例如讀和寫各用一個 `Semaphore`（`read_sem.acquire().await`）。一個 endpoint 慢爆，不會拖垮全部。

---

## 沒有奶油？先賣陽春麵包

有一天奶油供應商出事。老陳沒有拉下鐵門，而是在門口貼：「**今日奶油麵包暫停，陽春吐司照常供應。**」

客人雖然少了選擇，但還是買得到早餐。

> 這就是 **graceful degradation**：外部依賴掛了就回退到簡單版，例如推薦系統掛了回熱門列表、cache 掛了直接查 DB。寫程式時就要想：這個失敗該 **fail-fast** 還是 **fallback**？

---

## 把守則一層一層套上去

老陳把上面這些守則印成**一張護貝卡**，貼在每支電話旁：先看代幣、再看有沒有暫停貼紙、打出去最多等一下、沒通就照規矩重打。

不管打給麵粉行、奶油行還是送貨司機，**都套同一張卡**。

而店門口的大看板會亮燈提醒：「暫停貼紙被貼上了！」「今天重打的比例超過一成！」——代表供應商那邊有事。

> 這就是 **tower 的 layer 組合**：`ServiceBuilder::new().layer(TimeoutLayer::new(...)).layer(RetryLayer::new(...)).layer(RateLimitLayer::new(...))` 可以套在任何 tower service 上（reqwest、hyper、tonic）。搭配第 49 章的觀測，對 **circuit open**、**retry rate**、**P99 latency** 設 alert。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 監視器上的學徒標籤 | **tokio-console**（`console_subscriber::init()`） |
| 大家等同一把冰箱鑰匙 | task 被 **mutex** 卡住 |
| 層層疊起的蛋糕圖 | **flamegraph**（寬 = CPU 時間） |
| 又寬又在頂端的那塊 | **hot function** |
| 打電話最多等一下 | **timeout**：`with_timeout` / `tokio::time::timeout` |
| 等一下再打，越等越久 | **exponential backoff**：`backoff_ms * 2`，上限 2000 |
| 隨便多加一點點 | **jitter**，避免 thundering herd |
| 付訂金不能亂重打 | 只 retry **idempotent** 操作 |
| 「暫停」貼紙 | **circuit breaker**：`Breaker`、`BreakerState::Open` |
| 連續失敗 3 次 | `threshold` / `fail_count` |
| 「試打中」貼紙，只派一個人打 | **`BreakerState::HalfOpen`**：只放一個探測請求 |
| 代幣罐 | **token bucket**：`governor::RateLimiter` |
| 拿到代幣才打 | `lim.until_ready().await` |
| 蛋糕和吐司分開烤 | **bulkhead**（分開的 `Semaphore`） |
| 先賣陽春吐司 | **graceful degradation** / fallback |
| 貼在電話旁的護貝卡 | **tower `ServiceBuilder` + layer** |

---

## 一句話記住

> **監視器看誰在等、蛋糕圖看誰最燒時間；打電話一定設時限，沒通就越等越久、還要錯開再打，連續失敗就貼暫停貼紙，一秒只拿五枚代幣——外面出事時，店裡照樣開門。**
