# 故事版：運動會的賽跑計時員 ⏱️

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「學校運動會幫選手計時」的故事，幫你「感覺」出為什麼不用 `Instant` 直接量、`criterion` 怎麼熱身和多次取樣、`black_box` 防止偷懶、怎麼看信賴區間和 p 值、為什麼一定要 release、micro-benchmark 的限制，以及 profiler 跟它怎麼搭配。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 按一次碼錶，到底準不準？

運動會前一週，體育老師請小宇當**計時員**。要比的是三位選手：**阿圈**、**阿流**、**阿疊**，看誰跑一百公尺最快。

小宇拿起碼錶，等阿圈跑一次，按下停止：**13.8 秒**。

可是那一下剛好吹來一陣大風，阿圈的鞋帶又鬆了，而且他根本還沒熱身。小宇自己按錶的手指也慢了一點點。

老師搖搖頭：「只跑一次，什麼都不準。」

> 這就是 **為什麼不用 `Instant`**：`Instant::now()` 量一次 `elapsed()`，沒迭代 → 雜訊大；沒 warmup → 第一次包含 cold cache；沒統計 → 看不出差異是真的還是運氣；沒對照 → 不知道 baseline。

---

## 請來專業的計時團隊

老師請來一個**專業計時團隊**。他們做事很有規矩：

1. 選手先**熱身**幾圈，不算成績。
2. 正式開始後，**讓選手跑很多很多次**。
3. 把所有成績寫下來，算出平均、中間值、忽快忽慢的程度。
4. 最後畫成一張**彩色海報**貼在公布欄。

要請這個團隊，得先在運動會的**報名表**上寫清楚：「這場比賽叫 `sum_methods`，**不要用學校原本的計時方式**，交給專業團隊。」

> 這就是 **`criterion`**：自動 warm-up、多次 sample、算 mean / median / std dev、產生 HTML 報告。在 `Cargo.toml` 的 `[dev-dependencies]` 加 `criterion`，再用 `[[bench]] name = "sum_methods"` 配 `harness = false` 關掉預設的測試 harness。程式放在 `benches/sum_methods.rs`，用 `cargo bench` 跑，報告在 `target/criterion/report/index.html`。

---

## 三位選手上場

三位選手跑的是同一件事：**把一萬個數字加起來**。只是方法不一樣：

- **阿圈**：一個一個數，手上拿一張紙記總數。
- **阿流**：讓數字排成一條河，河流到盡頭自然就是總和。
- **阿疊**：每拿到一個就疊到手上那疊上面，最後看疊了多少。

計時團隊替每位選手開一條**跑道**，跑道上寫著選手名字，然後喊：「跑！再跑！再跑！」

> 這就是 **`bench_function` + `b.iter`**：`bench_sums` 裡用 `c.bench_function("sum_loop", |b| b.iter(...))` 替 `sum_loop`、`sum_iter`、`sum_fold` 各開一條跑道。`b.iter` 裡的東西會被重複跑很多次。最後用 `criterion_group!(benches, bench_sums)` 和 `criterion_main!(benches)` 把它們登記起來。

---

## 偷懶的選手

第一輪成績出來，大家嚇一跳：阿流的成績是 **0 秒**！

小宇跑去問阿流。阿流小聲說：「我發現**根本沒有人在看我最後算出來的總數**，所以我……站在起點就直接說『跑完了』。」

原來有一位**超聰明的教練**一直在幫選手省力。只要他發現「這個答案沒人要」，就叫選手乾脆別跑；甚至他發現「每次的數字都一樣」，就先偷偷算好答案背起來。

計時團隊想到辦法：拿一塊**黑布**把題目蓋住，跟教練說：「這些數字是**比賽當下才會公布的**，你猜不到，而且答案**等一下一定會有人來看**。」

教練沒辦法偷吃步，阿流只好老老實實跑完。

> 這就是 **`black_box`**：LLVM（聰明教練）看到「結果沒被用到」或「輸入是固定的」，可能把整段運算**優化掉**，量出 0ns。`black_box(&v)` 假裝 `v` 是 runtime 才來的，強迫真的執行。程式裡寫成 `b.iter(|| sum_loop(black_box(&v)))`。回傳值也可以 `black_box`，但實務上包 input 通常就夠。

---

## 不同距離分組比

老師又問：「短跑快的人，長跑也快嗎？」

於是計時團隊把比賽分成好幾組：**100 公尺、1000 公尺、一萬公尺、十萬公尺**。每一組都讓阿圈和阿流各跑，最後畫一張折線圖，看誰在距離變長時落後。

> 這就是 **`benchmark_group` + `bench_with_input`**：`c.benchmark_group("sum")` 開一組，對不同 `size` 用 `BenchmarkId::new("loop", size)` 各量一次，最後 `group.finish()`。`criterion` 會自動畫出比較圖，看出不同實作的 scaling。

---

## 看懂成績單

成績單上寫著：

> **阿流　時間：[4.21　4.23　4.26]　跟上次比：[-1.5%　+0.2%　+1.8%]　（p = 0.78）　沒有變化**

小宇看不懂，老師解釋：

- 中間的 **4.23** 是最常見的成績；左右兩個數字是「**我們很有把握，真正的實力落在這中間**」。
- 「跟上次比」是跟上週練習的成績比。
- **p 值**像是「這個差異是運氣的機率」。p 很大，代表可能只是運氣，所以團隊說「**沒有變化**」。p 要小於 0.05，才算真的變快或變慢。

> 這就是 **統計輸出解讀**：`time: [lower, median, upper]` 是 95% 信賴區間；`change` 是跟上次 baseline 的差異；`p < 0.05` 才是統計顯著。`criterion` 會自己判斷「No change in performance detected」或「Performance has improved」。

---

## 穿拖鞋跑不算數

有一次，阿疊穿著**拖鞋**來練習，成績慢了一大截。

老師說：「正式比賽穿的是**釘鞋**。拖鞋的成績拿來比，一點意義都沒有，可能差一百倍！」

> 這就是 **debug vs release**：debug 編譯的成績可能跟 release 差 100×。`cargo bench` 預設就用 bench profile（release 等級的最佳化），**永遠在 release 跑 benchmark**。

---

## 操場跑得快，不代表馬拉松跑得快

阿流在學校操場上成績超漂亮。可是老師提醒：

- 操場每天都一樣，阿流**早就背熟每個彎道**；真正的馬拉松路線每次都不同。
- 操場上只有他一個人；真正比賽有一大群人**擠在一起搶跑道**。

「操場成績很好，只能說**這一段**你很快。想知道整場比賽，要去跑一場真的。」

> 這就是 **micro-benchmark 的限制**：cache 太熱（重複同一份資料）、branch predictor 太準（固定 input）、被測函式被 inline 進 harness、沒模擬真實的 contention。end-to-end 效能要用接近 production 的 load test（例如用 `wrk` 打 HTTP server）。

---

## 碼錶說「誰慢」，慢動作錄影說「為什麼慢」

阿疊雖然穿上釘鞋，還是比較慢。計時團隊只能告訴他「你慢了 0.3 秒」，卻說不出原因。

教練拿出**慢動作攝影機**，一格一格看：原來阿疊每跨一步，手都會多擺一下。

> 這就是 **跟 profiling 銜接**：`criterion` 告訴你「**哪段慢**」，profiler 告訴你「**為什麼慢**」。常用 `cargo flamegraph`（火焰圖）、`samply`、`perf`、`tokio-console`、`cargo bloat`。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 按一次碼錶 | `Instant::now()` + `elapsed()`（雜訊大） |
| 專業計時團隊 | **`criterion`** |
| 報名表上寫「交給專業團隊」 | `[[bench]] name = "sum_methods"`、`harness = false` |
| 熱身不算成績 | **warm-up** |
| 跑很多很多次 | **多次 sample** |
| 阿圈、阿流、阿疊 | `sum_loop`、`sum_iter`、`sum_fold` |
| 每位選手一條跑道 | `c.bench_function(...)` + `b.iter(...)` |
| 登記比賽 | `criterion_group!` / `criterion_main!` |
| 超聰明的偷懶教練 | **LLVM 最佳化**（dead-code elimination） |
| 蓋住題目的黑布 | **`black_box`** |
| 不同距離分組 | `benchmark_group` + `bench_with_input` + `BenchmarkId` |
| 「真正實力落在這中間」 | **95% 信賴區間** `[lower, median, upper]` |
| 是運氣的機率 | **p 值**（< 0.05 才顯著） |
| 拖鞋 vs 釘鞋 | **debug vs release** |
| 操場 vs 馬拉松 | **micro-benchmark vs load test** |
| 彩色海報 | `target/criterion/report/index.html` |
| 慢動作攝影機 | **profiler**（`cargo flamegraph`、`samply`） |

---

## 一句話記住

> **量速度要請專業計時團隊（`criterion`）：先熱身、跑很多次、看信賴區間；用黑布（`black_box`）蓋住題目防止教練偷吃步，一定穿釘鞋（release）跑；碼錶告訴你誰慢，慢動作錄影（profiler）才告訴你為什麼慢。**
