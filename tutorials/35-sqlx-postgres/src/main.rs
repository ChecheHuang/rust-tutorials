// 範例為示意 — 真要跑要啟動 Postgres。
// docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=pw postgres:16
// 然後 DATABASE_URL=postgres://postgres:pw@localhost/postgres cargo run

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct User {
    id: Uuid,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
}

async fn create_schema(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id          UUID PRIMARY KEY,
            name        TEXT NOT NULL,
            email       TEXT NOT NULL UNIQUE,
            created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_user(pool: &PgPool, name: &str, email: &str) -> Result<User> {
    let id = Uuid::new_v4();
    // query_as! 是 compile-time check 版本（需要 DATABASE_URL）
    // 這裡用 runtime 版本，普及度較高
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, name, email)
        VALUES ($1, $2, $3)
        RETURNING id, name, email, created_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(email)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

async fn get_user(pool: &PgPool, id: Uuid) -> Result<Option<User>> {
    let u = sqlx::query_as::<_, User>(
        "SELECT id, name, email, created_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(u)
}

async fn list_users(pool: &PgPool, limit: i64) -> Result<Vec<User>> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email, created_at FROM users ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(users)
}

async fn transfer_demo(pool: &PgPool, from: Uuid, to: Uuid) -> Result<()> {
    // 用 transaction：兩個 update 必須一起成功
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE users SET name = name || '-from' WHERE id = $1")
        .bind(from)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users SET name = name || '-to' WHERE id = $1")
        .bind(to)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("沒設 DATABASE_URL；下面的程式碼僅作為範例展示，不會實際跑。");
        eprintln!("docker run --rm -p 5432:5432 -e POSTGRES_PASSWORD=pw postgres:16");
        eprintln!("export DATABASE_URL=postgres://postgres:pw@localhost/postgres");
        return String::new();
    });

    if db_url.is_empty() {
        return Ok(());
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await?;

    create_schema(&pool).await?;
    let alice = insert_user(&pool, "alice", &format!("a-{}@x.com", Uuid::new_v4())).await?;
    let bob = insert_user(&pool, "bob", &format!("b-{}@x.com", Uuid::new_v4())).await?;
    println!("inserted: {alice:?}\n          {bob:?}");

    let got = get_user(&pool, alice.id).await?;
    println!("fetched: {got:?}");

    transfer_demo(&pool, alice.id, bob.id).await?;
    let all = list_users(&pool, 10).await?;
    for u in &all {
        println!("- {} <{}>", u.name, u.email);
    }
    Ok(())
}
