# 21. 測試

> 範圍：#[test]、整合測試、doc test、mockall、cargo nextest

## Rust 測試的三種等級

| 層級 | 位置 | 看到 |
|------|------|------|
| 單元測試 | 同檔案 `#[cfg(test)] mod tests` | private items（白盒） |
| 整合測試 | `tests/*.rs` | 只能看 `pub` API（黑盒） |
| doc test | `///` 註解中的 ` ``` ``` ` 區塊 | 跟整合測試一樣 |

**全部一行指令**：

```bash
cargo test
```

## 單元測試

慣例：放在被測 module 同檔案末端：

```rust
pub fn factorial(n: u64) -> u64 { (1..=n).product() }

#[cfg(test)]
mod tests {
    use super::*;          // 進口父 module 的 items

    #[test]
    fn factorial_zero() {
        assert_eq!(factorial(0), 1);
    }
}
```

- `#[cfg(test)]` 表示只在 `cargo test` 時編譯
- `mod tests` 是慣例名稱
- `use super::*` 讓 test 看到外層所有 items

### 常用斷言

```rust
assert!(cond);
assert!(cond, "msg with {}", value);
assert_eq!(a, b);
assert_ne!(a, b);
```

訊息會包含失敗時的左右值，debug 友善。要更豐富的斷言用 [`pretty_assertions`](https://crates.io/crates/pretty_assertions)（彩色 diff）。

### `#[should_panic]`

測試該 panic 的情況：

```rust
#[test]
#[should_panic(expected = "overflow")]
fn overflow_panics() {
    // ...
}
```

`expected = "..."` 比對 panic 訊息（substring match）。

### 回 Result 的測試

```rust
#[test]
fn parses() -> Result<(), Box<dyn std::error::Error>> {
    let n: i32 = "42".parse()?;
    assert_eq!(n, 42);
    Ok(())
}
```

避免一連串 `unwrap()`。

### `#[ignore]`

慢的測試、需要外部資源的測試，標記後**預設不跑**：

```rust
#[test]
#[ignore = "needs DB"]
fn db_integration() { ... }
```

跑：`cargo test -- --ignored`。

## 整合測試

`tests/` 目錄下每個 `.rs` 檔是一個獨立 crate：

```
my_crate/
├── src/lib.rs
├── tests/
│   ├── api.rs
│   └── flow.rs
└── ...
```

```rust
// tests/api.rs
use my_crate::Client;

#[test]
fn end_to_end() {
    let c = Client::new();
    assert_eq!(c.ping(), "pong");
}
```

只能用 `pub` API。模擬真實使用者視角，是黑盒測試。

`tests/common/mod.rs` 放共用 helper（檔案名 `mod.rs` 才不會被當成獨立 test crate）。

## Doc Test

doc comment 內的 code block 會被當測試跑：

```rust
/// Returns the factorial of n.
///
/// # Examples
///
/// ```
/// use my_crate::factorial;
/// assert_eq!(factorial(5), 120);
/// ```
pub fn factorial(n: u64) -> u64 { ... }
```

跑 `cargo test --doc`。**Rust 生態超愛 doc test**——既當文件又當測試，保證範例不過時。

選項：
- `` ```ignore `` — 不跑
- `` ```no_run `` — 編譯但不執行
- `` ```should_panic ``
- `` ```compile_fail `` — 確認**該失敗編譯**（測試 trait bound）

> 注意：doc test 只對 **library crate** 有效，本章是 binary 所以 doc test 不跑。

## 測試 private items

單元測試（同檔案）能看 private。整合測試只能看 public：

```rust
// src/lib.rs
fn private_helper() -> i32 { 42 }     // 不 pub

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_private() {
        assert_eq!(private_helper(), 42);   // OK
    }
}

// tests/api.rs
use my_crate::private_helper;            // ERROR: private
```

## Mocking — `mockall`

```toml
[dev-dependencies]
mockall = "0.12"
```

```rust
use mockall::automock;

#[automock]
trait Db {
    fn get_user(&self, id: u64) -> Option<String>;
}

fn greet(db: &dyn Db, id: u64) -> String {
    db.get_user(id).unwrap_or_else(|| "anon".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mocked() {
        let mut mock = MockDb::new();
        mock.expect_get_user()
            .with(mockall::predicate::eq(1))
            .returning(|_| Some("alice".into()));

        assert_eq!(greet(&mock, 1), "alice");
    }
}
```

精準控制 mock 行為（次數、參數、回傳值）。

## `cargo nextest` — 更快的 test runner

```bash
cargo install cargo-nextest
cargo nextest run
```

- 用 process-per-test 並行，比 `cargo test` 快
- 漂亮的 progress UI
- 更好的失敗訊息聚合
- CI 友善

新專案直接用 `cargo nextest`，舊習慣才用 `cargo test`。

## 測試組織建議

- **單元測試**：放函式行為（純運算、邊界）
- **整合測試**：放 user-facing flow（API 端到端）
- **doc test**：放 minimal usage example，保證文件可跑
- **`#[ignore]`**：跨 process / 跨網路的 expensive 測試，CI 上才跑

## Property-based testing — `proptest` / `quickcheck`

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn reverse_twice(v in any::<Vec<i32>>()) {
        let r: Vec<i32> = v.iter().rev().rev().copied().collect();
        prop_assert_eq!(v, r);
    }
}
```

讓框架自動產生 input、找最小化的反例。對 parser、編譯器、資料結構特別有用。

## 對照 TypeScript

TS 測試是「**選一個 framework**」（jest / vitest / mocha / ava），跟 source code 分檔（`__tests__/` 或 `.test.ts`）。Rust 測試是「**語言內建**」——`#[test]` 屬性、`cargo test` 一個指令、單元測試直接放在 source 檔尾端。

### 對照表

| 需求 | TypeScript | Rust |
|---|---|---|
| Framework | jest / vitest / mocha | **內建**（`#[test]`） |
| 設定檔 | `jest.config.js` / `vitest.config.ts` | 無；Cargo.toml 就夠 |
| 標記測試 | `test("name", () => ...)` / `it(...)` | `#[test] fn name() { ... }` |
| Assert | `expect(a).toBe(b)` | `assert_eq!(a, b)` |
| Setup / teardown | `beforeEach` / `afterEach` | 沒對應；用 fixture 函式 |
| 跑單一 test | `vitest run -t name` | `cargo test name` |
| 單元測試位置 | `foo.test.ts` 旁邊 | 同檔案末尾 `#[cfg(test)] mod tests` |
| 整合測試 | `__tests__/` 或 `*.test.ts` | `tests/` 目錄（每檔獨立 crate） |
| 文件內範例會跑 | TSDoc 只當註解 | `///` 內的 ```` ``` ```` block **真的當 test 跑** |
| 測 private | 同檔測試可看 | 同檔測試可看（`use super::*`） |
| Mock | jest mock / vi.mock | `mockall` crate |
| 跑 ignored | `xit(...)` / `it.skip` | `#[ignore]` + `cargo test -- --ignored` |
| 平行執行 | jest 預設並行 | 預設並行（`cargo test -- --test-threads=1` 改） |
| 快測試 runner | jest 預設 | `cargo nextest run`（比 `cargo test` 快） |
| 期望 throw | `expect(fn).toThrow()` | `#[should_panic(expected = "...")]` |
| Coverage | `--coverage` | `cargo tarpaulin` / `cargo llvm-cov` |
| Property-based | `fast-check` | `proptest` / `quickcheck` |

### 程式碼對照

```ts
// TS — vitest
import { describe, it, expect } from 'vitest';
import { factorial } from './math';

describe('factorial', () => {
  it('returns 1 for 0', () => {
    expect(factorial(0)).toBe(1);
  });
  it('handles 5', () => {
    expect(factorial(5)).toBe(120);
  });
  it('throws for negative', () => {
    expect(() => factorial(-1)).toThrow();
  });
});
```

```rust
// Rust — 內建 test
pub fn factorial(n: i64) -> i64 {
    if n < 0 { panic!("negative"); }
    (1..=n).product()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_1_for_0() {
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn handles_5() {
        assert_eq!(factorial(5), 120);
    }

    #[test]
    #[should_panic(expected = "negative")]
    fn panics_for_negative() {
        factorial(-1);
    }
}
```

Doc test（**TS 沒有對應**）：

```rust
/// 計算 n 階乘
///
/// # Examples
///
/// ```
/// use my_crate::factorial;
/// assert_eq!(factorial(5), 120);
/// ```
pub fn factorial(n: i64) -> i64 { ... }
```

`cargo test --doc` 真的會跑——保證文件範例不過時。

Mock：

```ts
// TS — vi.mock
import { vi } from 'vitest';
const db = { getUser: vi.fn().mockResolvedValue({ name: 'alice' }) };
```

```rust
// Rust — mockall
use mockall::automock;

#[automock]
trait Db {
    fn get_user(&self, id: u64) -> Option<String>;
}

let mut mock = MockDb::new();
mock.expect_get_user()
    .returning(|_| Some("alice".into()));
```

### 心智模型差異

1. **沒有 describe / it 樹狀分組**。Rust 測試一律 `#[test] fn ...`，靠**函式名** + module 結構分類。沒有 `beforeEach` —— setup 寫成普通 helper 函式。一開始覺得樸素，但少了「巢狀 describe 的 magic」實際比較好讀。
2. **單元測試混在 source 檔末尾是慣例**。TS 寫 `foo.ts` + `foo.test.ts`；Rust 在 `foo.rs` 末尾寫 `#[cfg(test)] mod tests { ... }`。好處：能看到 private 函式、貼近被測 code、`#[cfg(test)]` 編譯時被忽略不影響 binary。
3. **整合測試是「黑盒」**。Rust `tests/*.rs` 每檔是獨立 crate，只能用 `pub` API——強迫你想「對外介面是什麼」。TS 沒這層強制，整合測試常變相導入 internal。
4. **Doc test 是 Rust 文化的支柱**。文件範例 = 測試 = 編譯保證——TS 完全沒對應（typedoc 只是 render）。寫 lib 時 `#[doc] examples` 是 first-class 工具，保證範例不過時。
5. **panic test 與 should-throw test 等同**。`#[should_panic(expected = "msg")]` ≈ `expect(fn).toThrow('msg')`。但 Rust 慣例：盡量回 `Result` 而非 panic，所以 `should_panic` 用得比 TS toThrow 少。
6. **Mock 文化弱**。Rust 社群 prefer **trait + 真實實作 + fake impl** 多於 mock framework；mockall 有用但不是 idiomatic 預設。對應 TS jest 的「mock 一切」文化，Rust 偏向「**設計時就讓 trait 可注入**」。
7. **CI 友善**。`cargo test` 預設輸出可讀、return code 正確、可平行——不需要額外配置就是 CI-ready。`cargo nextest` 更現代，process-per-test 隔離。
8. **Property-based testing 容易**。`proptest` 讓你寫「任意 input 都滿足某性質」的測試，框架幫你找最小化反例。TS `fast-check` 同類但 Rust 因為型別系統強，定義 input domain 更精準。

## 常見陷阱

1. **`cargo test` 跑得慢** — 每個 integration test 都是獨立 binary，編譯時間累積。考慮把多測合一檔。
2. **測試裡跑網路 / DB** — 加 `#[ignore]` 別讓 default `cargo test` 卡。
3. **`unwrap()` 滿天飛** — test 用 `?` + `Result` 反而清楚。
4. **沒 `assert_eq!` 訊息** — 失敗時看左右值不知道哪邊錯，看 `pretty_assertions::assert_eq!`。
5. **不會 mock async**：mockall 對 async trait 需要 `mockall::mock!` 顯式定義或用 `async-trait`。

## 練習

1. 為第 10 章的 `parse_and_double` 寫 4 個測試（成功、parse 失敗、邊界、negative）。
2. 把單元測試 + 整合測試 + doc test 三種都寫一次，跑 `cargo test --verbose` 看編譯/執行區別。
3. 用 `mockall` mock 一個 `trait HttpClient`，寫測試驗證 retry 邏輯被叫 3 次。
4. 把專案的 `cargo test` 改成 `cargo nextest run`，比較時間。

## 延伸閱讀

- [The Rust Book — Ch 11 Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [`mockall` 文件](https://docs.rs/mockall/)
- [`cargo-nextest`](https://nexte.st/)
- [`proptest` book](https://proptest-rs.github.io/proptest/intro.html)
