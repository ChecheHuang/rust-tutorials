# 36. SeaORM

> 範圍：Entity / ActiveModel、CRUD、relation、migration、與 sqlx 取捨

## 何時用 SeaORM 而不是 sqlx

| 場景 | 選擇 |
|------|------|
| 需要 ORM 抽象、跨多種 DB（Postgres / MySQL / SQLite） | SeaORM |
| 物件導向風格、不想直接寫 SQL | SeaORM |
| 想要 compile-time check 的 SQL | sqlx |
| 一次性 / 報表複雜 query | sqlx |
| 微服務裡讀寫操作模式固定 | 兩者皆可，看團隊偏好 |

實務常見：**SeaORM 處理 CRUD、複雜查詢 fallback 到 raw SQL**。SeaORM 也允許 raw query。

## 核心概念

```
Entity      ──── 對應一張 table，包含 schema metadata
Model       ──── plain struct，select 出來的型別
ActiveModel ──── 含 Set/Unchanged 標記，做 insert/update
Relation    ──── 跟其他 Entity 的關聯（一對多、多對多）
```

## ActiveValue：精準 update

```rust
let mut a = alice.into_active_model();
a.name = Set("Alice (updated)".into());
// email 維持 Unchanged → UPDATE 不會帶 email
a.update(&db).await?;
```

跟 ORM 常見的「整個 model 都 UPDATE」不同，SeaORM 只 update 你 `Set` 的欄位。**對並發場景很重要**。

## Migration

SeaORM 自己有 migration framework：

```bash
sea-orm-cli migrate init       # 建 migration crate
sea-orm-cli migrate generate create_users
sea-orm-cli migrate up
```

Migration 用 Rust 寫（`Migration` trait + `MigrationName`），不是 SQL string。可程式化生成複雜 migration，但對只想寫 SQL 的人會覺得啰嗦。

## Entity 生成

實務不會手寫 Entity，用：

```bash
sea-orm-cli generate entity \
    -u postgres://u:p@localhost/db \
    -o src/entities
```

讀目前 DB schema 反向生成 Entity code。

## 關聯

```rust
// 一對多：一個 User 多個 Post
#[derive(DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)] pub id: i32,
    pub user_id: i32,
    pub title: String,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
```

查詢：

```rust
let posts = post::Entity::find()
    .filter(post::Column::UserId.eq(user_id))
    .all(&db).await?;

// 連 User 一起拉
let with_user: Vec<(post::Model, Option<user::Model>)> =
    post::Entity::find()
        .find_also_related(user::Entity)
        .all(&db).await?;
```

## QueryBuilder

```rust
use sea_orm::{ColumnTrait, QueryFilter};

User::find()
    .filter(user::Column::Email.like("%@example.com"))
    .filter(user::Column::Name.is_not_null())
    .order_by_desc(user::Column::Id)
    .paginate(&db, 20)
    .fetch_page(0).await?;
```

## Transaction

```rust
let txn = db.begin().await?;
some_entity.insert(&txn).await?;
other_entity.update(&txn).await?;
txn.commit().await?;
```

跟 sqlx 模式相同，但 ActiveModel API 都接受 `&txn`。

## 與 sqlx 共用

SeaORM 底層就是 sqlx；可以直接拿底層 pool：

```rust
let inner: &sqlx::PgPool = db.get_postgres_connection_pool();
```

複雜場景（CTE、window function、explain）就用 sqlx 寫。

## 常見陷阱

1. **`Set` vs `Unchanged` 混淆** — 沒 `Set` 的欄位 update 時不會寫入，這是 feature 不是 bug。
2. **`into_active_model`** — Model 變 ActiveModel 時所有欄位是 `Unchanged`，要改的欄位才 `Set`。
3. **Migration crate 獨立** — 通常會分 `migration/` 子 crate，不能跟主 crate 混。
4. **效能** — Active Model 比 sqlx 直接 raw 慢一點（多一層）；hot path 想極致就寫 raw。
5. **Type code generation 跟 DB 版本綁** — schema 變了要 regen entity，CI 要有檢查。

## 練習

1. 加 `Post` entity，做 user → posts 一對多。
2. 把上面的範例改用 Postgres 跑（同樣 SeaORM 程式碼，只改 connection string）。
3. 寫一個用 `find_with_related` 的查詢（n+1 vs JOIN 比較）。
4. 用 SeaORM migration 寫一個加欄位的 migration，跑 up/down。

## 延伸閱讀

- [SeaORM Book](https://www.sea-ql.org/SeaORM/docs/)
- [sea-orm-cli](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/)
