# 35. sqlx + Postgres

> 範圍：connection pool、query macro、transaction、migrations、compile-time checked SQL

## sqlx 在 Rust ORM 生態的定位

| 工具 | 風格 |
|------|------|
| **sqlx** | SQL-first，類似 Go 的 `database/sql` + 編譯期檢查 |
| **SeaORM**（第 36 章） | Active Record + Query Builder |
| **Diesel** | Schema-first，極致型別安全但學習曲線陡 |

**選擇原則**：SQL 寫得順、要編譯期檢查的選 sqlx；要 ORM 抽象的選 SeaORM。新手通常先學 sqlx。

## 環境

```bash
docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=pw postgres:16
export DATABASE_URL=postgres://postgres:pw@localhost/postgres
cargo install sqlx-cli --no-default-features --features postgres,rustls
```

## Connection Pool

```rust
let pool = PgPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&db_url).await?;
```

⚠️ **整個 app 共用一個 pool**，handler 拿 `&pool`。不要每 request 建新 pool。

## 兩種 query 方式

### 1. runtime query（`query_as::<_, T>`）

```rust
sqlx::query_as::<_, User>("SELECT ... WHERE id = $1")
    .bind(id)
    .fetch_one(&pool).await?;
```

優點：不用 DATABASE_URL 也能編譯。  
缺點：SQL 錯誤要執行才知道。

### 2. compile-time query（`query!` / `query_as!`）

```rust
sqlx::query_as!(User, "SELECT id, name FROM users WHERE id = $1", id)
    .fetch_one(&pool).await?;
```

優點：編譯期連 DB 檢查 SQL + 型別。  
缺點：build 時需要 DATABASE_URL 或 `cargo sqlx prepare` 出來的 `.sqlx/` 離線快取。

**實務建議**：CI 用離線快取（`SQLX_OFFLINE=true`），開發機可以線上 check。

## fetch 系列

| 方法 | 用途 |
|------|------|
| `fetch_one` | 一定要剛好 1 列；0 列回 `Error::RowNotFound` |
| `fetch_optional` | 0 或 1 列，回 `Option<T>` |
| `fetch_all` | 取全部到 `Vec` |
| `fetch(...)` | 回 `Stream`，逐筆處理(大查詢) |
| `execute` | 不取資料（INSERT / UPDATE / DELETE） |

## Transaction

```rust
let mut tx = pool.begin().await?;
sqlx::query("...").execute(&mut *tx).await?;
sqlx::query("...").execute(&mut *tx).await?;
tx.commit().await?;     // 漏 commit + drop = ROLLBACK
```

`&mut *tx` 的關鍵：`Transaction` deref 到 connection 但需要 mutable borrow。

## Migrations

```bash
sqlx migrate add create_users
# 寫進 migrations/20260520_*.sql
sqlx migrate run
```

Rust 內也可以：
```rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

啟動時自動跑 migration——適合 monolith；微服務多會獨立 deploy migration job 而非由 app 跑。

## 型別映射（Postgres → Rust）

| Postgres | Rust |
|----------|------|
| `text` / `varchar` | `String` |
| `int4` | `i32` |
| `int8` | `i64` |
| `bool` | `bool` |
| `uuid` | `uuid::Uuid` |
| `timestamptz` | `chrono::DateTime<Utc>` |
| `jsonb` | `serde_json::Value` 或 `Json<T>` |
| `text[]` | `Vec<String>` |
| `null` | `Option<T>` |

## 常見陷阱

1. **`fetch_one` 沒結果會 error** — 0 列場景要用 `fetch_optional`。
2. **`bind` 順序對應 `$1 $2 ...`** — 弄錯不會編譯失敗（runtime 版），會 runtime 噴。
3. **連線洩漏** — `Transaction` 沒 commit 也沒 rollback 卡 connection；確保走完。
4. **SQL injection** — 永遠用 `bind`，**不要** `format!` 把使用者輸入塞 SQL。
5. **`Decimal` 用 `rust_decimal`** — 別用 `f64` 表金錢。
6. **Compile-time check 卡 CI** — 沒設 `SQLX_OFFLINE` 又沒 DB 就跑 `cargo sqlx prepare` 進 commit。
7. **`Postgres` Notify/Listen** — sqlx 沒原生 stream API 包好；要 `PgListener`。
8. **`enum` 對到 Postgres enum** — 加 `#[sqlx(type_name = "...")]` 屬性，且 `derive(sqlx::Type)`。

## 練習

1. 寫 users + posts（一對多）schema，做 `JOIN` query 對到 nested struct。
2. 把上面範例改成 `query!` 版本，體驗編譯期錯誤。
3. 用 `tx` 寫一個轉帳：A -100, B +100，故意讓中間 panic，驗證 rollback。
4. 用 `PgListener` 訂閱 `NOTIFY/LISTEN`。

## 延伸閱讀

- [sqlx](https://docs.rs/sqlx)
- [sqlx-cli](https://crates.io/crates/sqlx-cli)
