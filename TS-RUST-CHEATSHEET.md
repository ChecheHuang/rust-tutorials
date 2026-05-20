# TypeScript ↔ Rust 速查表

從 TypeScript 開發者視角整理的對照。每節都連到對應章節，有需要請點進去看完整討論。

> 本表只涵蓋 **Part 1–4（01–30）** 的語言核心。Web / 部署 / 桌面開發章節（31–54）的對照價值較低，未涵蓋。

## 心智模型先讀這四點

1. **TS 型別在 runtime 不存在；Rust 型別影響 codegen**。`as` 在兩邊意義不同——TS 是 type assertion（騙編譯器），Rust 是 value conversion（真的截斷 bytes）。
2. **TS 永遠 reference 語意；Rust 預設 move 語意**。`let b = a` 在 TS 是別名、在 Rust 是 ownership 轉移。共享需要明示：`&a`（借用）/ `a.clone()`（深複製）/ `Rc::new(a)`（refcount）。
3. **沒有 GC，靠編譯期 borrow check 防 dangling 與 data race**。代價：要學 ownership / borrowing / lifetime 三件套。回報：runtime 零開銷、無 STW、編譯通過 = thread-safe。
4. **錯誤是值（`Result`）不是 exception（`throw`）**。函式簽章寫出可能 fail；`?` 是 try/catch 的早返糖；忘處理錯誤編譯失敗。

---

## 第 1 區：語法與基本型別

### 變數宣告與基本型別 → [ch02](./tutorials/02-variables-types/)

| TS | Rust |
|---|---|
| `const x = 5` | `let x = 5;`（預設不可變） |
| `let x = 5` | `let mut x = 5;` |
| `number`（統一 float） | `i32` / `u64` / `f64` 等明確 |
| `if (truthy)` | `if cond`（必須是 `bool`） |
| `null` / `undefined` | `Option<T>::None`（只一個） |
| `void`（型別） | `()`（unit type，有一個值） |
| `const X = 5` | `const X: u32 = 5;`（**編譯期常數**） |

### 控制流 → [ch03](./tutorials/03-control-flow/)

| TS | Rust |
|---|---|
| `cond ? a : b` | `if cond { a } else { b }`（expression） |
| `switch (x)` + fallthrough | `match x { ... }` 強制窮盡、無 fallthrough |
| discriminated union narrowing | enum + `match` |
| `for (const x of arr)` | `for x in &v` |
| `for (let i=0; i<n; i++)` | `for i in 0..n` |
| `if (!x) return; const y = x.foo;` | `let Some(y) = opt else { return; };` |
| labeled `break outer;` | `break 'outer;` |

### 函式與閉包 → [ch04](./tutorials/04-functions-closures/)

| TS | Rust |
|---|---|
| `(x) => x + 1` | `|x| x + 1` |
| `function f(): T { return ...; }` | `fn f() -> T { ... }`（尾端 expression 無分號回傳） |
| 預設參數 `f(x = 0)` | 沒有（用 `Option<T>` 或 builder） |
| 變長 `...args` | 沒有（用 slice 或 macro） |
| closure 型別 alias | `Fn` / `FnMut` / `FnOnce` trait |
| 一律 dynamic dispatch | 預設 static dispatch（`impl Fn`） |

---

## 第 2 區：Ownership 與資料

### Ownership → [ch05](./tutorials/05-ownership/)

**這是 TS 沒有的東西**。

| 想做 | TS | Rust |
|---|---|---|
| 兩個變數指同物件 | `const b = a;`（自然） | 不行；要 `&a`（借用）或 `Rc::clone(&a)`（共享 ownership） |
| 深複製 | `structuredClone(a)` | `a.clone()`（必須顯式） |
| 函式不取走資料 | 自然（reference） | `f(&a)`（借用） |
| 函式取走資料 | 沒這概念 | `f(a)`（move，a 之後不可用） |
| 提早釋放 | `a = null`（祈禱 GC） | `drop(a)`（立即） |

### Borrowing → [ch06](./tutorials/06-borrowing/)

**TS 沒有對應**。Rust 引入兩種 reference：

| 形式 | 數量 | 用途 |
|---|---|---|
| `&T` | 多個並存 OK | 唯讀借用 |
| `&mut T` | **同一時刻只一個** | 可變借用 |

核心規則：**一寫多讀互斥**——data race 在編譯期被擋下。

### Strings / Slices → [ch07](./tutorials/07-slices-strings/)

| TS | Rust |
|---|---|
| `string`（UTF-16） | `String`（owned）/ `&str`（借用），UTF-8 |
| `T[]` | `Vec<T>`（owned）/ `&[T]`（借用） |
| `s.length` | `s.len()` → **bytes**；`s.chars().count()` → 字元數 |
| `s[0]` | **不能**索引；用 `s.chars().next()` 或 `s.as_bytes()[0]` |
| `s.slice(0, 3)` | `&s[..3]`（byte 邊界 panic） |

### Struct / Enum → [ch08](./tutorials/08-struct-enum/)

| TS | Rust |
|---|---|
| `interface User { id: number }` | `struct User { id: u32 }` |
| structural typing | nominal typing |
| `{ kind: "x" } \| { kind: "y" }` | `enum Shape { X, Y }` |
| `T \| undefined \| null` | `Option<T>`（一個 None） |
| `name?: string` | `name: Option<String>` |
| decorator 自動生成 | `#[derive(Debug, Clone, ...)]` |

### Collections → [ch09](./tutorials/09-collections/)

| TS | Rust |
|---|---|
| `Array<T>` | `Vec<T>` |
| `Map<K, V>` | `HashMap<K, V>` / `BTreeMap`（排序） |
| `Set<T>` | `HashSet<T>` |
| 沒有 deque | `VecDeque<T>`（O(1) 兩端） |
| `arr[i]` → undefined | `v[i]` panic / `v.get(i)` → `Option` |
| `Map` 保插入順序 | `HashMap` **不保證順序** |
| `m.set(k, (m.get(k) ?? 0) + 1)` | `*m.entry(k).or_insert(0) += 1` |

### 錯誤處理 → [ch10](./tutorials/10-error-handling/) / [ch19](./tutorials/19-advanced-errors/)

| TS | Rust |
|---|---|
| `throw new Error(...)` | `return Err(...)` |
| `try/catch` | `match` / `?` |
| `Promise.reject` | `async fn -> Result<T, E>` |
| 自訂 `class extends Error` | `#[derive(thiserror::Error)] enum` |
| `Error("...", { cause: e })` | `#[from]` + `.context("...")` (anyhow) |
| 簽章不顯示 throw | `-> Result<T, E>` 明寫 |
| 編譯不擋忘 catch | `#[must_use]`，編譯器警告 |

---

## 第 3 區：型別系統進階

### 泛型 → [ch11](./tutorials/11-generics/)

| TS | Rust |
|---|---|
| `<T>` | `<T>` |
| `<T extends Foo>` | `<T: Foo>` |
| `<T extends Foo & Bar>` | `<T: Foo + Bar>` |
| 編譯後擦除 | monomorphization（每個 T 一份特化） |
| dynamic dispatch only | static（預設）+ dynamic（`dyn`） |
| 沒有 const generic | `<const N: usize>` |

### Traits → [ch12](./tutorials/12-traits/)

| TS interface | Rust trait |
|---|---|
| `interface I { f(): T }` | `trait I { fn f(&self) -> T; }` |
| 沒 default 實作 | trait 內可寫 default body |
| structural 自動符合 | 必須顯式 `impl I for Type` |
| 永遠 dynamic | `impl Trait`（static） / `dyn Trait`（dynamic） |
| 為內建型別加方法 = prototype | `impl MyTrait for i32`（孤兒規則） |
| `interface B extends A` | `trait B: A`（supertrait） |

### Lifetimes → [ch13](./tutorials/13-lifetimes/)

**TS 完全沒對應**。GC 永遠保證 reference valid；Rust 編譯期證明 reference 不會 dangle。

- `&'a str`：reference 至少活到 `'a`
- `'static`：活到 program 結束（字串字面值預設這個）
- 多數時候 elision 規則自動推導，不必手寫

### Smart Pointers → [ch14](./tutorials/14-smart-pointers/)

| TS 情境 | Rust |
|---|---|
| 物件是 reference（一定） | `&T` / `Rc<T>` / `Arc<T>`（看需要） |
| 跨 Worker 共享 | `Arc<T>`（同 process，不必序列化） |
| 共享可變（單緒） | `Rc<RefCell<T>>` |
| 共享可變（多緒） | `Arc<Mutex<T>>` / `Arc<RwLock<T>>` |
| WeakRef | `Weak<T>` |
| 隨意共享（無 alloc） | `Cow<T>`（borrow until mutate） |
| 放 heap | `Box::new(x)` |

### Iterators → [ch15](./tutorials/15-iterators/)

| TS | Rust |
|---|---|
| `arr.map(f).filter(p).reduce(...)` | `iter.map(f).filter(p).fold(...)` |
| eager（每步建陣列） | **lazy**（沒 consumer 不跑） |
| `arr.find(p)` / `arr.some(p)` / `arr.every(p)` | `.find()` / `.any()` / `.all()` |
| `arr.flatMap(f)` | `.flat_map(f)` |
| `[...arr1, ...arr2]` | `.chain(other)` |
| 沒原生 zip | `.zip(other)` |
| 並行需要 Worker | `rayon::par_iter()` 一行平行 |

---

## 第 4 區：Macro / 編譯期

### Declarative Macros → [ch16](./tutorials/16-declarative-macros/)

TS 沒對應。Rust `macro_rules!` 操作 token 樹層級，能拿 expression / type / pattern 當參數。

| 場景 | Rust |
|---|---|
| 變長參數 | `$($x:expr),*` |
| 接受 expression | `$x:expr` |
| 接受 type | `$x:ty` |
| 自訂 DSL | `map! { "a" => 1 }` |

### Procedural Macros → [ch17](./tutorials/17-procedural-macros/)

對應 **TS decorator + 編譯器 plugin**——但在 compile time 跑、零 runtime 開銷。

| TS | Rust |
|---|---|
| `@Serializable class Foo` | `#[derive(Serialize)] struct Foo` |
| `@Get('/users') method()` | `#[get("/users")] async fn ...` |
| `class-transformer` runtime reflection | serde 編譯期生成程式碼 |
| Prisma codegen | `sqlx::query!` 編譯期驗 SQL |

### Unsafe / FFI → [ch18](./tutorials/18-unsafe-ffi/)

| 需求 | TS / Node | Rust |
|---|---|---|
| 呼叫 C lib | `ffi-napi` / `koffi` | `extern "C" {}` + `unsafe` |
| 寫 native addon | C++ / Rust + N-API | `pub extern "C" fn` + `crate-type = ["cdylib"]` |
| 寫 Node module | `napi-rs`（Rust → Node） | `napi-rs` |
| 編 WASM | 不可能（要轉 AssemblyScript） | `wasm-bindgen` 直接編 |

---

## 第 5 區：工程化

### Modules / Workspace → [ch20](./tutorials/20-modules-workspace/)

| TS / Node | Rust |
|---|---|
| 檔案 = module | `mod foo;` 顯式宣告才納入 |
| `import { x } from './foo'` | `mod foo; use foo::x;` |
| `export const x` | `pub fn x()` |
| 沒中間可見度 | `pub(crate)` / `pub(super)` / `pub(in path)` |
| pnpm / nx workspace | `[workspace]` in Cargo.toml |
| `"foo": "workspace:*"` | `foo = { path = "../foo" }` |
| catalog 共用版本 | `[workspace.dependencies]` |

### Testing → [ch21](./tutorials/21-testing/)

| TS（jest / vitest） | Rust |
|---|---|
| `test("name", () => ...)` | `#[test] fn name() { ... }` |
| `expect(a).toBe(b)` | `assert_eq!(a, b)` |
| `expect(fn).toThrow()` | `#[should_panic]` |
| 同檔測試 | `#[cfg(test)] mod tests` |
| `__tests__/` | `tests/` 目錄（黑盒，獨立 crate） |
| 沒對應 | doc test（`///` 範例真的當測試跑） |
| jest mock | `mockall` crate |
| jest coverage | `cargo tarpaulin` / `cargo llvm-cov` |
| `fast-check` | `proptest` / `quickcheck` |

### Cargo / Build → [ch22](./tutorials/22-cargo-advanced/)

| Node | Rust |
|---|---|
| `package.json` | `Cargo.toml` |
| 沒對應 | `[features]`（編譯期條件編譯） |
| 沒對應 | `[profile.release]`（多套編譯設定） |
| `prebuild` script | `build.rs` |
| `process.env.X`（runtime） | `env!("X")`（編譯期）/ `std::env::var`（runtime） |
| `npm audit` | `cargo audit` |
| 沒標準 | `cargo deny`（license + CVE + 來源） |
| `npm ls` | `cargo tree` |

### Logging → [ch23](./tutorials/23-tracing-log/)

| Node（pino） | Rust（tracing） |
|---|---|
| `log.info({ x }, 'msg')` | `info!(x = ..., "msg")` |
| `log.child({ ... })` | `span!` / `#[instrument]` |
| AsyncLocalStorage 跨 await | Span 自動跟著 future |
| `LOG_LEVEL=info` | `RUST_LOG=info,my_crate=debug` |
| `pino-pretty` / JSON transport | `.fmt().pretty()` / `.json()` |

### Config → [ch24](./tutorials/24-config/)

| Node | Rust |
|---|---|
| `dotenv` + `zod` | `dotenvy` + `serde` + `figment` / `config` |
| `process.env.PORT` | `Env::prefixed("APP_")` + serde deserialize |
| `z.infer<...>` | struct 就是 schema |
| 自己 redact secret | `secrecy::Secret<T>` 預設 `[REDACTED]` |

### Benchmark → [ch25](./tutorials/25-benchmark/)

| Node | Rust（criterion） |
|---|---|
| `benchmark.js` / `tinybench` | `criterion` |
| 自己看 mean | mean + 95% 信賴區間 + p-value |
| 自己存 baseline | `--save-baseline` / `--baseline` |
| 沒對應（V8 一律 JIT） | **必須 release**（cargo bench 預設） |
| 沒對應 | `black_box(x)` 阻 dead-code elim |

---

## 第 6 區：Async

### async / await → [ch26](./tutorials/26-async-await/)

| TS | Rust |
|---|---|
| `async function` | `async fn` |
| `Promise<T>` | `impl Future<Output = T>` |
| **eager**（建 Promise 就跑） | **lazy**（建 Future 不跑） |
| V8 內建 event loop | 沒內建，要選 tokio |
| `Promise.all([a, b])` | `tokio::join!(a, b)` |
| `Promise.race([a, b])` | `tokio::select! { ... }` |
| `Promise.allSettled` | `JoinSet` / `futures::join_all` |
| Worker（postMessage 序列化） | `tokio::spawn`（同 memory） |
| 沒內建 sleep | `tokio::time::sleep(...)` |
| AbortController | task drop / `JoinHandle::abort` |

### Tokio runtime → [ch27](./tutorials/27-tokio/)

| Node | Tokio |
|---|---|
| libuv（內建單一） | tokio runtime（可建多個） |
| Worker（重） | `spawn_blocking`（輕、共享 memory） |
| `setInterval` | `tokio::time::interval` |
| graceful shutdown 自己接 SIGTERM | `tokio::signal` + `CancellationToken` |

### Send / Sync / Pin → [ch28](./tutorials/28-send-sync-pin/)

**TS 幾乎無對應**（Node 單緒 + Worker 序列化）。

- `Send`：可 move 到別 thread
- `Sync`：`&T` 可跨 thread 共享
- `Pin`：物件不再搬動（async self-referential）
- 編譯期保證 thread safety、無 data race

### Channels → [ch29](./tutorials/29-async-sync-channels/)

| TS / Node | Tokio |
|---|---|
| EventEmitter | `mpsc::channel`（型別化 + bounded） |
| Promise resolve | `oneshot::channel` |
| RxJS Subject | `broadcast::channel` |
| RxJS BehaviorSubject | `watch::channel` |
| `p-limit` | `Semaphore` |
| AbortController + signal | `CancellationToken` |

### Streams → [ch30](./tutorials/30-streams/)

| TS | Rust |
|---|---|
| `AsyncIterable<T>` / `for await ... of` | `Stream<Item = T>` / `while let Some(x) = s.next().await` |
| `async function*` | `async_stream::stream! { yield ...; }` |
| `p-map({ concurrency: N })` | `.buffer_unordered(N)` |
| RxJS `bufferTime` | `.chunks_timeout(n, d)` |
| RxJS `throttleTime` | `.throttle(d)` |

---

## 速查心法

- 寫 `let b = a` 之前，問自己：**我要 alias、要 deep copy、還是要 move**？
- 寫函式參數，問自己：**我要不要取走所有權**？不要就 `&T`（唯讀）/ `&mut T`（可改）。
- 看到 `?` 後面接函式，腦中翻成「`try { ... } catch (e) { throw e; }`」。
- 看到 `Option<T>` / `Result<T, E>`，腦中翻成「`T | undefined` / `T | Error`」，但要 `match` / `if let` / `?` 才能拆。
- 看到 `Vec<Box<dyn Trait>>`，腦中翻成「TS `Trait[]`」（dynamic dispatch、runtime polymorphism）。
- 編譯失敗訊息**慢慢讀**——Rust 編譯器很細，多半告訴你怎麼修。
- 卡 lifetime 時，先試**不標**或**改用 owned 型別**（`String` 取代 `&str`），多半解決一半問題。
- 卡 borrow check 時，先想「**有沒有辦法把這變數的 mutate 集中在一個 scope 內**」。
- 卡 async Send 時，先想「**有什麼東西跨了 await 點**」——多半是 `Rc` 或 `std::Mutex` guard。

---

## 不在這份速查表內的章節

- **Part 5（31–39）Web**：axum / actix-web / sqlx / SeaORM / JWT / WebSocket
- **Part 6（40–45）分散式**：Clean Architecture / DI / gRPC / MQ / CQRS
- **Part 7（46–50）部署**：Docker / CI/CD / K8s / Prometheus / Profiling
- **Part 8（51–54）桌面**：Tauri / HID USB / 跨平台打包

這些章節對應的是「**框架特性**」而非「**語言觀念**」，TS 對照價值低；學到時直接讀章節內容即可。
