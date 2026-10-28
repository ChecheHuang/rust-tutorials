# 故事版：夜市雞排攤的計數器和小黑板 📊

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「夜市雞排攤想知道生意好不好、哪裡卡住」的故事，幫你「感覺」出觀測三大支柱、counter / gauge / histogram、Prometheus 拉取 `/metrics`、命名與單位、label 基數爆炸、tracing span 與 trace id、OTLP collector、取樣，以及 Grafana 把三者串起來。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 生意很好，可是阿明說不出「多好」

阿明在夜市開了一家雞排攤，每天晚上都排滿人。

有一天里長問他：「你一晚賣幾塊？客人平均等多久？哪一塊炸太久？」阿明抓抓頭：「呃……很多？很快？」

阿明決定在攤子上放三樣東西：

- 一台**按壓計數器**：記「賣了幾塊」。
- 一本**流水帳**：記「發生了什麼事」，例如「八點十分，油鍋太燙，關小火」。
- 每張點餐單上的**追蹤章**：記「這一份雞排從點餐、裹粉、下鍋、裝袋，一路走了多久」。

> 這就是 **觀測的三大支柱**：**metrics**（數字：多少、多快）、**logs**（事件：發生了什麼）、**traces**（一個請求一路的旅程）。分別常用 Prometheus、Loki、Tempo / Jaeger 收。

---

## 三種不同的記法

阿明的妹妹小芳負責記數字。她發現數字有三種不一樣的脾氣：

- **賣出幾塊**：只會往上加，按一下 +1，晚上收攤前絕不會變少。
- **現在排隊幾個人**：一下 8 個、一下 3 個，**會上下跳**，看的是「此刻」。
- **每塊雞排炸多久**：她準備了一排小盒子，分別寫著「1 分鐘內」「3 分鐘內」「5 分鐘內」。每炸好一塊，就丟一顆彈珠進對應的盒子。收攤時一看，就知道**大部分**炸多久、**最慢的**又有多慢。

> 這就是 **metrics 的三種型別**：**counter** 只增不減（`counter!("http_requests_total", "route" => "/").increment(1)`，寫在 `root()` 和 `slow()` 裡）；**gauge** 上下浮動（`gauge!(...).set(n)`）；**histogram** 記分布（`histogram!("http_request_duration_seconds", ...).record(elapsed)`）。那排小盒子就是 **bucket**。`metrics` crate 只是「記的方式」，真正收數字的後端可以換，就像 `tracing` 之於 log。

---

## 門口那個拿碼表的人

小芳懶得在每個步驟都記時間。她站到**攤子門口**，每個客人一點餐就按下碼表，雞排交到客人手上再按停，然後把秒數丟進彈珠盒。

不管客人點的是原味還是辣味，**都要經過門口**，所以一個人就能幫全部的單計時。

> 這就是 **`timing` middleware**：`axum::middleware::from_fn(timing)` 包住整個 `Router`。它在 `next.run(req).await` 前後用 `Instant::now()` / `start.elapsed()` 量時間，再帶 `method`、`path` 兩個標籤記進 histogram。業務函式完全不用改。

---

## 牆上的小黑板，檢查員自己來抄

阿明在攤子旁邊掛了一塊**小黑板**，上面隨時寫著最新數字：

```
原味雞排賣出：132 塊
炸好時間 1 分鐘內：90 顆彈珠 ……
```

夜市的**檢查員**每隔 15 秒走過來，**自己抄**黑板上的數字帶回辦公室畫成圖。阿明不用跑去辦公室報告，也不用在黑板上鎖——鎖了檢查員就抄不到了，所以黑板只掛在**夜市裡面**，外面的路人看不到。

> 這就是 **Prometheus exporter + scrape**：`PrometheusBuilder::new().install_recorder()` 拿到一個 handle，`/metrics` 路由呼叫 `recorder.render()` 把所有數字印成 Prometheus 文字格式。Prometheus 是**主動來拉（pull）**的。`/metrics` 通常不加認證，而是用網路層隔離。K8s 上用 `ServiceMonitor` 讓它自動找到你。

---

## 寫清楚單位，還有別寫每個人的名字

小芳一開始在黑板寫「炸的時間：3」。阿明問：「3 秒？3 分鐘？」從此她規定：**每個數字後面都要寫單位**，例如「炸雞時間_秒」「賣出_總數」。

後來她想更仔細：「每位客人各自買了幾塊」。於是黑板上開始出現「王小明：1」「陳大華：2」「林阿姨：1」……一個晚上兩千位客人，黑板**寫爆了**，檢查員抄到崩潰。

阿明說：「分類只能用**少少幾種**的東西，例如『原味 / 辣味』『內用 / 外帶』。人名不行，人太多了。」

> 這就是 **命名約定** 和 **label cardinality**：名字要含單位（`http_request_duration_seconds`、`_bytes`、`_total`）。每一種 label 組合就是一條 series，把 `user_id` 放進 label 會讓 series 數爆炸。經驗法則：**label 可能的值超過 100 種就要警惕**。

---

## 一張單子走過好幾站

辣味雞排要經過**點餐台 → 裹粉台 → 油鍋 → 裝袋台**四站。

小芳在每張點餐單上蓋一個**追蹤章**，上面有一串號碼「#A7」。每一站接手時，都在單子背面寫「我這站從幾點幾分做到幾點幾分」，然後**把同一張單子**交給下一站。

收攤後把單子翻過來，就看得出：原來這一份慢，是卡在**油鍋那站**。

> 這就是 **tracing 的 span 與 trace**：每一站是一個 **span**，整張單子是一條 **trace**，「#A7」就是 **trace id**。`#[instrument]` 標在 `root()`、`slow()` 上會自動開 span，`info!("hit /slow")` 是掛在 span 底下的事件。跨服務時，trace id 放在 HTTP header `traceparent: 00-<trace-id>-<span-id>-01` 裡一路傳下去。

---

## 夜市的中央收發室

一開始，每個攤位都自己把單子送去辦公室、自己送流水帳去檔案室，路上塞成一團。

夜市於是設了一間**中央收發室**：大家只要把東西丟進收發室，收發室會**分類**（數字送辦公室、單子送檔案櫃）、**塗掉客人電話號碼**，再整批送出。收發室忙不過來時，攤位不會在門口傻等，而是先**一疊一疊**放著，滿了就丟掉一些，生意照做。

> 這就是 **OTLP collector**：app 透過 `tracing-opentelemetry` 的 layer 把 span 用 OTLP 送給 `otel-collector`，由它統一做 sampling、遮罩敏感資料、轉發到 Tempo / Prometheus / Loki。app 端用 `install_batch` **批次送出**，collector 慢時不拖累 app。業務程式碼一行都不用改。

---

## 不是每張單子都要留

一晚兩千張單，全部收進檔案櫃太佔空間。

- **辦法一**：點餐時就抽籤，**每 10 張留 1 張**。簡單，但萬一出包的那張剛好沒抽到就看不到了。
- **辦法二**：等單子**走完全程**再決定：有出包、或特別慢的**一定留**，其他的才抽籤。

> 這就是 **sampling**：`Sampler::TraceIdRatioBased(0.1)` 是 10% 的 **head sampling**（一開始就決定）；**tail sampling** 在 collector 看完整條 trace 才決定，可以保證錯誤和慢請求 100% 留下。

---

## 彈珠盒要照你在乎的時間切

阿明跟客人保證「**3 分鐘內**一定拿到」。可是小芳的彈珠盒只有「1 分鐘內」「10 分鐘內」兩格，根本看不出 3 分鐘這條線有沒有守住。

她把盒子重新分成「2 分、2.5 分、3 分、3.5 分、4 分」，**在承諾附近切得特別細**。

> 這就是 **histogram bucket 要對齊 SLO**：用 `PrometheusBuilder::new().set_buckets(&[...])` 自訂。承諾 `P95 < 200ms`，就要在 50–500ms 區段放密一點，否則 P95 / P99 會算不準。

---

## 一面大看板串起三本帳

最後，夜市辦公室裝了一面**大看板**：

先看數字圖，發現「八點十分炸雞時間突然變長」→ 點下去，看到那段時間的**追蹤單**，發現都卡在油鍋 → 再點，跳到**流水帳**那一行：「油鍋太燙，關小火」。

三本帳靠著**同一串追蹤號碼**連在一起。

> 這就是 **Grafana 串接**：Prometheus（metrics）、Tempo（traces）、Loki（logs）在同一個 UI 裡互相跳轉。log 裡帶著 `trace_id`，就能從 log 直接跳到對應的 trace。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 計數器、流水帳、追蹤章 | 三大支柱：**metrics / logs / traces** |
| 只會往上加的「賣出幾塊」 | **counter**：`counter!(...).increment(1)` |
| 會上下跳的「排隊人數」 | **gauge**：`gauge!(...).set(n)` |
| 彈珠盒 | **histogram** + **bucket**：`histogram!(...).record(t)` |
| 門口拿碼表的人 | `timing` middleware（`axum::middleware::from_fn`） |
| 牆上的小黑板 | `/metrics` 路由 + `recorder.render()` |
| 檢查員自己來抄 | Prometheus **scrape（pull）** |
| 數字後面寫單位 | 命名約定 `_seconds` / `_bytes` / `_total` |
| 寫每個客人的名字，黑板爆了 | **高 cardinality label** |
| 每一站寫在單子背面 | **span**（`#[instrument]`） |
| 整張單子、號碼「#A7」 | **trace** / **trace id** |
| 單子交給下一站 | `traceparent` header 跨服務傳遞 |
| 中央收發室 | **OTLP collector** |
| 一疊一疊送 | `install_batch` 批次匯出 |
| 每 10 張留 1 張 | **head sampling**（`TraceIdRatioBased(0.1)`） |
| 出包的一定留 | **tail sampling** |
| 在 3 分鐘附近切細的盒子 | `set_buckets` 對齊 **SLO** |
| 辦公室大看板 | **Grafana** 串起 Prometheus / Tempo / Loki |

---

## 一句話記住

> **計數器記「多少」、彈珠盒記「多快」、追蹤章記「卡在哪」；黑板掛著讓檢查員自己來抄，分類只用少少幾種，單子交給收發室批次送——三本帳靠同一串號碼，在大看板上連成一條線。**
