# 41. 依賴注入

> 範圍：構造函式注入、`Arc<dyn Trait>`、shaku container、什麼時候需要框架

## 「DI 框架」在 Rust 是反 idiomatic 的

Java / .NET 大量 DI 框架的痛點 Rust 沒那麼大：
- Java 無泛型 monomorphization、沒 Rust 的 `cargo check`，runtime 才知道錯
- 巨型 enterprise codebase 才有「不知道誰用誰」的問題
- Rust 的 `Arc<dyn Trait>` + 構造函式 + main 組裝 = 90% 場景夠用

**結論**：先學 manual wiring，框架是備胎。

## manual wiring（推薦）

```rust
let mailer: Arc<dyn Mailer> = Arc::new(StdoutMailer);
let clock: Arc<dyn Clock> = Arc::new(SystemClock);
let use_case = NotifyUseCase::new(mailer, clock);
```

優點：
- 編譯器幫你檢查
- 程式碼直白，沒有「魔法」
- IDE 跳轉、refactor 都 work
- 啟動順序你說了算

缺點（也是優點）：要自己排好建構順序。但這恰好強迫你想清楚依賴。

## 三種注入策略

### 1. 構造函式注入（首選）

```rust
struct UseCase { repo: Arc<dyn Repo> }
impl UseCase {
    fn new(repo: Arc<dyn Repo>) -> Self { Self { repo } }
}
```

### 2. 泛型注入（零成本）

```rust
struct UseCase<R: Repo> { repo: R }
impl<R: Repo> UseCase<R> {
    fn new(repo: R) -> Self { Self { repo } }
}
```

性能極致；但多型場景或回傳 `UseCase<R>` 變麻煩。

### 3. State + extractor（web 場景）

```rust
#[derive(Clone)]
struct AppState { repo: Arc<dyn Repo>, ... }

async fn handler(State(s): State<AppState>) { ... }
```

axum / actix 已內建。

## DI 框架：shaku

```rust
#[derive(Component)]
#[shaku(interface = Mailer)]
struct StdoutMailer;

#[derive(Component)]
#[shaku(interface = Notifier)]
struct NotifierImpl {
    #[shaku(inject)]
    mailer: Arc<dyn Mailer>,
}

module! { App { components = [StdoutMailer, NotifierImpl], providers = [] } }

let app = App::builder().build();
let n: &dyn Notifier = app.resolve_ref();
```

shaku 編譯期解析依賴圖（不像 Spring 在 runtime）；漏組件編譯不過。

## 什麼時候考慮 DI 框架

- **N > 30 個元件、依賴圖深** — manual 已經難維護
- **多套組態**（生產 / 測試 / staging）需要動態替換
- **想要 lifecycle 管理**（init / shutdown 順序）

一般 microservice 通常不到這規模——不需要。

## 跟其他語言常見 DI 框架對照

| 語言 | 框架 | 解析時機 |
|------|------|----------|
| Java | Spring / Guice | runtime（reflection） |
| .NET | MS DI | runtime |
| Rust | shaku | 編譯期 |
| Rust | manual | 編譯期（更直白） |

Rust 沒 runtime reflection，所有 DI 框架本質上都是 codegen / macro。所以「Rust 的 DI 框架」實際上是「自動寫 wiring」的工具。

## Service Locator 反模式

```rust
// ❌ 不要這樣寫
let mailer = GLOBAL_CONTAINER.get::<dyn Mailer>();
```

把依賴隱藏在全域容器 = 沒人知道誰依賴什麼 + 測試災難。**永遠在建構時把依賴傳入**。

## 測試

```rust
struct InMemoryRepo(Vec<Order>);
impl Repo for InMemoryRepo { ... }

#[tokio::test]
async fn test_use_case() {
    let use_case = UseCase::new(Arc::new(InMemoryRepo::default()));
    // ...
}
```

manual wiring 在測試最舒服：在 setup 直接 `UseCase::new(fake_repo)`。

## 常見陷阱

1. **濫用 trait** — 一個只有單一實作的 trait 是冗餘抽象。需要替換才加。
2. **`Arc<Mutex<dyn T>>`** — 通常 `Arc<dyn T>` 就夠（內部如有狀態自己處理 sync），多包 Mutex 增加耦合。
3. **循環依賴** — A 依賴 B、B 依賴 A。Rust 編譯就不過（init 順序）。重構成 `A、B → C`。
4. **拿 `Arc::clone(&x)` 寫成 `x.clone()`** — 雖然行為一樣，前者更清楚是 Arc 而不是內部資料 clone。

## 練習

1. 把第 40 章的 `PayOrderUseCase` 從 `Arc<dyn>` 改成泛型，看程式碼變化。
2. 用 shaku 重寫第 40 章的組裝。
3. 寫一個假 `Clock`，把當前時間 freeze 在某點，測時間敏感的邏輯。
4. 評估：你的 codebase 加 shaku 會帶來淨收益還是淨複雜度？

## 延伸閱讀

- [shaku](https://docs.rs/shaku)
- [Inversion of Control — Martin Fowler](https://martinfowler.com/articles/injection.html)
