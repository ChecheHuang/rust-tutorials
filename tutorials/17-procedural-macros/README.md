# 17. 過程宏（Procedural Macros）

> 範圍：derive / attribute / function-like、syn + quote + proc-macro2、何時用何時不用

## 跟 declarative macro 的差別

| | `macro_rules!`（宣告式） | proc macro（過程式） |
|---|---|---|
| 寫法 | pattern matching | 寫 Rust 程式碼處理 token |
| 能做的事 | 文法 substitution | 任何編譯期計算 |
| 複雜度 | 簡單 | 高（要懂 syn / quote） |
| 工具 | 純語法 | `syn` AST、`quote` 生成、`proc-macro2` |
| Crate 設定 | 一般 crate | 必須 `[lib] proc-macro = true` |
| 例子 | `vec!`、`println!` | `#[derive(...)]`、`#[tokio::main]`、`tracing::instrument` |

## 三種 proc macro

### 1. Derive macro

```rust
#[derive(Serialize, Deserialize)]
struct User { id: u64, name: String }
```

`Serialize` / `Deserialize` 都是 derive macro，看到 struct 後**產生** `impl Serialize for User` 與 `impl Deserialize for User`。

最常見的 derive：
- `serde`：`Serialize`、`Deserialize`
- `thiserror`：`Error`
- `clap`：`Parser`、`Subcommand`
- `prost`：`Message`（protobuf）

### 2. Attribute macro

```rust
#[tokio::main]
async fn main() { ... }

#[tracing::instrument]
fn process(x: i32) -> i32 { ... }
```

接收一個 item（fn / struct / impl block），可以**修改或包裝**它。`#[tokio::main]` 把 async fn 包成同步 runtime entry。

### 3. Function-like macro

```rust
sqlx::query!("SELECT * FROM users WHERE id = $1", id);
html! { <div>{name}</div> };
```

語法跟 `macro_rules!` 一樣 (`name!(...)`) 但能做更複雜的處理。`sqlx::query!` 會**在編譯期連 DB 驗證 SQL**——這是 declarative macro 做不到的。

## 寫一個 proc macro 的結構

proc macro 必須在**獨立的 crate** 內，crate type 是 `proc-macro`：

```
my_derive/
├── Cargo.toml      # [lib] proc-macro = true
├── src/lib.rs      # 用 syn、quote 寫的 macro
└── ...

my_app/
├── Cargo.toml      # 依賴 my_derive
└── src/main.rs     # 用 #[derive(MyMacro)]
```

最小 derive macro 範例（**示意**，本 chapter Cargo 未啟用）：

```rust
// my_derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Hello)]
pub fn hello(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        impl #name {
            fn hello() {
                println!("Hello from {}!", stringify!(#name));
            }
        }
    };

    TokenStream::from(expanded)
}
```

呼叫端：

```rust
#[derive(Hello)]
struct Foo;

fn main() {
    Foo::hello();   // 印 "Hello from Foo!"
}
```

## 三大工具

| Crate | 用途 |
|-------|------|
| `proc-macro2` | 對 `proc-macro` 的 wrapper，可在非 proc-macro crate 內測試 |
| `syn` | 把 `TokenStream` 解析成 Rust AST |
| `quote` | 用類似 `macro_rules!` 的語法寫 AST 模板 |

工作流：

```
input TokenStream
      ↓ syn::parse
   Rust AST
      ↓ 你的邏輯（檢查、變換）
  quote! { ... }
      ↓
output TokenStream
```

## 本章 main.rs 在做什麼

main.rs 不寫 proc macro，而是**使用**現成的：

```rust
#[derive(Debug, Serialize, Deserialize)]
struct User { ... }

#[derive(Debug, Error)]
enum AppError { ... }
```

- `serde` 的 derive 看 struct field 生成 `Serialize::serialize` 與 `Deserialize::deserialize` 實作
- `thiserror` 的 derive 看 enum variant 生成 `Display` + `std::error::Error::source` 實作
- `#[from]`、`#[serde(default)]` 是 derive 提供的 **helper attribute**

跑 `cargo expand` 可看到展開後的程式碼。

## 為什麼 proc macro 在獨立 crate

proc macro 在**編譯目標 crate 之前**就被執行（編譯器要呼叫它做 codegen）。所以 proc macro crate 必須先編譯，且 crate type 是 `proc-macro`——產生編譯器能載入的 `.so` / `.dll`。

副作用：**proc macro crate 不能跟一般 lib 同時 export 普通函式**。常見模式：
- `my_derive`：純 proc macro
- `my_derive_lib`（或 `my_crate`）：reexport `my_derive`，並提供 runtime trait

`serde` 就是 `serde` + `serde_derive` 這樣分。

## 何時自己寫

- 標準函式庫沒提供某個 derive，但你的 codebase 重複寫 N 次某種 impl
- 想為內部 DSL 做編譯期驗證（像 sqlx）
- 寫 framework，希望使用者用宣告式風格

**不要為 1 個 struct 寫 proc macro**——成本太高。先看是否有現成 crate（多半有）。

## 進階：proc-macro-workshop

[dtolnay/proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop) 是學 proc macro 的事實標準教材，有 5 個漸進專案：
1. `derive(Builder)` — builder pattern
2. `derive(CustomDebug)` — 自訂 Debug
3. `seq!(...)` — 重複 token 序列
4. `sorted!` — 編譯期排序檢查
5. `bitfield!` — bit-level struct

做完這 5 個基本能寫 95% 的實用 proc macro。本章只做入門 user 視角；真要寫 proc macro 請走 workshop。

## 對照 TypeScript

Proc macro **最接近 TS 的 decorator + compiler transformer**，但能力大很多。TS decorator 是 runtime（裝飾 class / method）；proc macro 是**編譯期**真的改 / 生成程式碼。`#[derive(Serialize)]` ≈ 「編譯時為你寫 serialize 程式碼」，TS 對應的是 `class-transformer` 或 `tsoa` 之類 lib 用 reflection metadata。

### 對照表

| 需求 | TypeScript | Rust |
|---|---|---|
| 為 class 加行為 | decorator `@Logged class Foo` | attribute macro `#[logged] fn foo()` |
| 為 class 自動生成方法 | decorator + reflection | `#[derive(Builder)]` |
| 編譯期驗證 | type checker | proc macro（任意邏輯） |
| 生成新 type | `as const` / utility types | proc macro 生成 struct / impl |
| 重新打字 | template literal types | proc macro |
| 範例：序列化 | `class-transformer` decorator | `#[derive(Serialize)]`（serde） |
| 範例：路由 | `@Get('/users') method()` | `#[get("/users")] async fn ...`（actix） |
| 範例：DB query | template string + ORM | `sqlx::query!("SELECT ...")` 編譯期連 DB 驗證 |
| 執行時機 | runtime（class 建立時） | 編譯期 |
| 操作對象 | class / method 本身 | TokenStream → 任意 TokenStream |
| Runtime 開銷 | decorator 包裝 | **零**（已展開成普通程式碼） |

### 程式碼對照

序列化：

```ts
// TS — class-transformer
import { Type, plainToInstance } from 'class-transformer';

class User {
  id!: number;
  name!: string;
  @Type(() => Date)
  createdAt!: Date;
}

const u = plainToInstance(User, jsonData);   // runtime 反射
```

```rust
// Rust — serde derive
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

let u: User = serde_json::from_str(&json_data)?;   // 編譯期生成的程式碼跑
```

差別：TS 在 runtime 讀 metadata 把 plain object 轉 class instance；Rust 在編譯期把 `#[derive(Deserialize)]` 展開成具體 `impl Deserialize for User`，runtime **零反射**、零開銷。

Web routing：

```ts
// TS — NestJS / class decorators
@Controller('users')
class UsersController {
  @Get(':id')
  findOne(@Param('id') id: string) { return this.svc.find(id); }
}
```

```rust
// Rust — actix-web attribute macros
#[get("/users/{id}")]
async fn find_one(path: web::Path<String>) -> impl Responder {
    // ...
}
```

### 心智模型差異

1. **「decorator at compile time」**。TS decorator runtime 包裝（呼叫時跑邏輯）；Rust proc macro 在 cargo 編譯時跑 Rust code 生成程式碼。所以 Rust binary 內**沒有 macro 痕跡**，runtime 看到的是展開後的純 Rust——這是 zero-cost abstraction 的核心機制。
2. **`syn` + `quote` 是 TS 沒有的工具鏈**。要寫 proc macro 必學：`syn` 把 TokenStream 解成 AST、`quote!` 像模板字串寫出 AST。對應 TS 不存在的層級——TS 沒有編譯器 plugin 框架（除了 `ttypescript` 之類 hack）。
3. **獨立 crate 是強制要求**。Rust proc macro **必須**在 `proc-macro = true` 的 crate 內，因為要先編譯出來給後面用。所以你看 serde / thiserror 都是 `serde` + `serde_derive` 兩個 crate。TS decorator 寫在同檔案就行。
4. **`#[derive]` 是 Rust 生態的「重力」**。看到 `#[derive(Debug, Clone, Serialize, Deserialize, Default)]` 是日常——一行解 5 個 impl。TS 對應通常是 utility type 或 mixin，沒這麼集中。
5. **編譯期驗證能跨系統**。`sqlx::query!("SELECT name FROM users WHERE id = $1", id)` **在編譯時**連你的 PostgreSQL 驗證 SQL 語法、欄位、型別——TS 同類功能（Prisma 之類）多半依賴 codegen 腳本而非 macro。
6. **錯誤訊息位置會跳**。Macro 展開後的錯誤指到展開後的程式碼位置——span 沒處理好的 proc macro 會讓使用者看到「在我沒寫過的程式碼裡有錯」。寫 proc macro 要花力氣在 span（`syn` 的 `Spanned` trait）。
7. **使用者視角 vs 作者視角**。多數 Rust 程式員是 proc macro **使用者**（用 derive、attribute），不寫；少數 lib 作者寫。要寫請走 dtolnay/proc-macro-workshop 練習。

## 常見陷阱

1. **proc macro 慢編譯** — 每個 macro 展開都是執行 Rust code，使用量大時編譯時間明顯。
2. **錯誤訊息位置** — span 沒處理好的話 error 指到 macro 內部，使用者霧水。
3. **derive helper attribute 不是公開語法** — `#[serde(...)]` 只有 derive 知道意思，外部看不到。
4. **IDE 支援不一** — rust-analyzer 對自訂 proc macro 的展開不見得能精準分析，可能 false-positive lint。
5. **`syn` 的 `Item` enum 巨大** — 學 syn 別硬背，邊查邊寫。

## 練習

1. 跑 `cargo install cargo-expand`，對本 main.rs 跑 `cargo expand`，看 `#[derive(Serialize)]` 展開後長什麼樣。
2. 用 `clap` derive 寫一個 CLI（subcommand + flag + env），對比手動 builder API。
3. 看 [serde derive source](https://github.com/serde-rs/serde/blob/master/serde_derive/src/lib.rs) 第一個 function 怎麼用 syn parse input。
4. 做 proc-macro-workshop 的 `builder` 練習。

## 延伸閱讀

- [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop)
- [The Little Book of Rust Macros — proc macros](https://veykril.github.io/tlborm/proc-macros.html)
- [syn 文件](https://docs.rs/syn/)、[quote 文件](https://docs.rs/quote/)
