# 25. 效能基準測試

> 範圍：criterion、micro-benchmark、避免被編譯器優化掉、與 profiling 銜接

## 為什麼不用 `Instant`

```rust
let start = Instant::now();
do_work();
println!("{:?}", start.elapsed());
```

問題：
- 沒迭代 → 雜訊大（系統 jitter、CPU 頻率變動）
- 沒 warmup → 第一次跑包含 cold cache
- 沒統計 → 看不出顯著性
- 沒對照 → 不知道 baseline

**`criterion`** 處理掉這些：自動 warm-up、多次 sample、計算 mean/median/std dev、HTML report、跑 regression detection。

## 安裝

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "sum_methods"   # 對應 benches/sum_methods.rs
harness = false         # 關掉 default test harness，用 criterion 自己的
```

## 撰寫 benchmark

```rust
// benches/sum_methods.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sums(c: &mut Criterion) {
    let v: Vec<i32> = (1..=10_000).collect();

    c.bench_function("sum_iter", |b| {
        b.iter(|| v.iter().sum::<i32>());
    });
}

criterion_group!(benches, bench_sums);
criterion_main!(benches);
```

跑：

```bash
cargo bench
```

報告：`target/criterion/report/index.html`

## `black_box` — 阻止編譯器作弊

```rust
b.iter(|| sum_loop(black_box(&v)));
```

LLVM 看到「結果沒被用到」會把整個 benchmark **優化掉**——測出 0ns。`black_box` 騙編譯器「這個值我之後會用」，強迫真的執行。

兩個 black_box 點：
- input：`black_box(&v)` 假裝 v 是 runtime 來的
- output：return value 也可 `black_box`，避免被 dead-code elimination 掉

實務上 input black_box 通常已夠。

## 比較多個 implementation

```rust
let mut group = c.benchmark_group("sum");
for size in [100, 1_000, 10_000, 100_000] {
    let v: Vec<i32> = (1..=size).collect();
    group.bench_with_input(BenchmarkId::new("loop", size), &v, |b, v| {
        b.iter(|| sum_loop(black_box(v)));
    });
    group.bench_with_input(BenchmarkId::new("iter", size), &v, |b, v| {
        b.iter(|| sum_iter(black_box(v)));
    });
}
group.finish();
```

`criterion` 自動產生比較圖表（HTML）。輸入大小變動時可看到不同實作的 scaling。

## 統計輸出解讀

```
sum_iter                time:   [4.21 µs 4.23 µs 4.26 µs]
                        change: [-1.5% +0.2% +1.8%] (p = 0.78)
                        No change in performance detected.
```

- **time**：[lower bound, median, upper bound]（95% 信賴區間）
- **change**：跟上次的差異
- **p < 0.05**：統計顯著
- 「**No change**」/「**Performance has improved**」是 criterion 的判斷

## debug vs release

```bash
cargo bench                  # 預設 bench profile = release
cargo bench --no-default-features
```

**永遠在 release 跑 benchmark**——debug 的測試結果可能跟 release 差 100×。

## micro-bench 的限制

micro-benchmark 適合測**單一函式**。但要小心：

1. **Cache 偏差**：循環測同個資料、cache hit 率不真實
2. **Branch predictor**：固定 input 讓分支預測完美，real-world 不會
3. **Inline 過度**：被測函式可能被 LLVM inline 到 bench harness，量不到真實 call 成本
4. **未 model 真實 contention**：在 single-threaded 跑出 1ns 不代表 production thread pool 一樣快

對 end-to-end performance：用 production-like load test（如 `wrk` 跑 HTTP server）。

## 與 profiling 銜接

criterion 告訴你「**哪段慢**」，profiler 告訴你「**為何慢**」。常用組合：

| 工具 | 用途 |
|------|------|
| `cargo flamegraph` | 火焰圖（CPU 哪行燒最久） |
| `samply` | 跨平台輕量 profiler |
| `perf`（Linux）/ `Instruments`（macOS）/ `VTune` | OS-level profiler |
| `tokio-console` | async runtime profiling |
| `cargo bloat` | binary size 分析 |

第 50 章詳述 profiling。

## 對照 TypeScript

Node 端常見 `benchmark.js` / `tinybench` / `vitest bench`。Rust `criterion` 在統計嚴謹度上強很多——多次 sample、警暖、信賴區間、HTML 報告、跟上次 baseline 比較。

### 對照表

| 需求 | Node / TypeScript | Rust |
|---|---|---|
| Bench framework | benchmark.js / tinybench / vitest bench | `criterion` |
| Setup | npm install + 寫 .bench.ts | `[dev-dependencies] criterion` + `[[bench]]` |
| 多次 sample | 多半要自己設 | 自動，預設 100 sample |
| 統計輸出 | mean / ops/sec | mean + 信賴區間 + p-value |
| 與上次 baseline 比 | 自己存（沒標準工具） | `--save-baseline` / `--baseline` |
| HTML 報告 | 沒標準 | 內建 |
| 阻止 dead-code elim | 大多沒這層問題（JIT） | `black_box(...)` |
| 多版本對比 | 自己包 | `benchmark_group` + `bench_with_input` |
| 跟 profiler 接 | clinic.js / 0x | `cargo flamegraph` / `samply` / `perf` |
| 編譯模式 | TS / V8 一律 JIT | **必須 release**（`cargo bench` 預設） |
| Micro vs macro | 自己決定 | 同 |

### 程式碼對照

```ts
// TS — tinybench
import { Bench } from 'tinybench';

const bench = new Bench({ time: 1000 });

bench
  .add('sum reduce', () => {
    const v = Array.from({ length: 10000 }, (_, i) => i);
    return v.reduce((a, b) => a + b, 0);
  })
  .add('sum loop', () => {
    const v = Array.from({ length: 10000 }, (_, i) => i);
    let s = 0;
    for (const x of v) s += x;
    return s;
  });

await bench.run();
console.table(bench.table());
```

```rust
// Rust — criterion
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sums(c: &mut Criterion) {
    let v: Vec<i32> = (1..=10_000).collect();

    c.bench_function("sum_iter", |b| {
        b.iter(|| v.iter().sum::<i32>())
    });

    c.bench_function("sum_loop", |b| {
        b.iter(|| {
            let mut s = 0;
            for &x in black_box(&v) { s += x; }
            s
        })
    });
}

criterion_group!(benches, bench_sums);
criterion_main!(benches);
```

跑：`cargo bench`，HTML 報告在 `target/criterion/report/index.html`。

### 心智模型差異

1. **`black_box` 是 Rust 特有的關鍵字**。LLVM 看到 benchmark 的回傳值沒用就 dead-code eliminate，整段測試被砍掉變 0ns。`black_box(x)` 騙編譯器「這個值我之後要用」強迫真的執行。Node JIT 較少這問題（V8 不會這麼狠的優化）。
2. **編譯模式不可忽略**。Node 跑 bench 沒「debug / release」這回事；Rust **必須 `cargo bench`（背後是 release profile）**，debug build 結果比 release 慢 10–100×，完全不能信。
3. **統計嚴謹度高**。criterion 預設 100 sample、warm-up、計算 95% 信賴區間、判斷統計顯著性（p < 0.05）——「**這次比上次快 2%」是雜訊還是真的，框架直接告訴你**。Node 工具多半只給 mean + std dev，要自己判斷。
4. **`--save-baseline` 是 regression test 利器**。改 code 前 `cargo bench -- --save-baseline before`，改完 `cargo bench -- --baseline before`，directly 告訴你變快多少。CI 內鎖性能不退步常用。
5. **跟 profiler 接最後一哩**。criterion 告訴你「**哪段慢**」，flamegraph 告訴你「**為何慢**」。Node 對應 clinic.js / 0x。Rust 比 Node 多一層：可以跟 OS-level profiler（`perf`、`samply`）直接整合，因為 Rust 編出來是真正的 native binary。
6. **micro-bench 限制**。同 Node：cache 偏差、branch predictor 太準、inline 過度——這些跨語言都一樣。Rust 因為跟 hardware 距離近，更需要 macro-level benchmark（`wrk` 打真實 HTTP server）配合 micro-bench。
7. **`cargo flamegraph`**。Linux/macOS 直接 `cargo install flamegraph` 然後 `cargo flamegraph --bench foo` 出火焰圖——Node 對應 0x 但 Rust 整合度更高。

## 常見陷阱

1. **debug build 跑 bench** — 結果完全不能信。
2. **沒 black_box** — 測出 0ns，覺得自己很神。
3. **同個資料反覆跑**：cache 太熱，不真實。可在每 iter 內動態 generate 或 `b.iter_with_setup`。
4. **沒比 baseline** — 改完不知道有沒有變快。`cargo bench -- --save-baseline before`、`--baseline before`。
5. **跑時系統有別的 workload** — 浪費 1 小時得到不可信數據；先關背景程式。

## 練習

1. 為 `Vec::insert(0, x)` 與 `VecDeque::push_front(x)` 寫對照 bench（10K push）。
2. 比 `String::push_str` 與 `format!("{a}{b}")` 在 100 次連接時的差距。
3. 用 `bench_with_input` 看 binary search 在 size 100 / 10K / 1M / 100M 的 scaling。
4. `cargo install flamegraph`，對本章 bench 跑 `cargo flamegraph --bench sum_methods`。

## 延伸閱讀

- [criterion.rs 文件](https://bheisler.github.io/criterion.rs/book/)
- [The Benchmarking Game (Rust)](https://benchmarksgame-team.pages.debian.net/benchmarksgame/fastest/rust.html)
