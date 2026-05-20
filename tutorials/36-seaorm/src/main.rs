// SeaORM 示範。用 SQLite in-memory，跑起來不用準備外部 DB。

use anyhow::Result;
use sea_orm::{
    sea_query::{ColumnDef, Table},
    ActiveModelTrait, ActiveValue, ConnectionTrait, Database, DbBackend, EntityTrait,
    IntoActiveModel, QueryOrder, Schema, Set,
};

// ── Entity ──────────────────────────────────────────────────
// 實務上會用 `sea-orm-cli generate entity` 自動生成。
// 這裡手寫展示結構。
mod user {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "users")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(unique)]
        pub email: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

use user::{ActiveModel as UserAM, Entity as User, Model as UserModel};

async fn create_schema(db: &sea_orm::DatabaseConnection) -> Result<()> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let mut stmt = schema.create_table_from_entity(User);
    // 對 SQLite 補 if not exists
    stmt.if_not_exists();
    db.execute(backend.build(&stmt)).await?;
    let _ = Table::create(); // 引入避免警告
    let _ = ColumnDef::new(""); // 同上
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // SQLite 記憶體
    let db = Database::connect("sqlite::memory:").await?;
    create_schema(&db).await?;

    // INSERT
    let alice = UserAM {
        name: Set("alice".to_string()),
        email: Set("a@x.com".to_string()),
        ..Default::default()
    };
    let alice: UserModel = alice.insert(&db).await?;
    println!("inserted: {alice:?}");

    let bob = UserAM {
        name: Set("bob".to_string()),
        email: Set("b@x.com".to_string()),
        ..Default::default()
    };
    bob.insert(&db).await?;

    // SELECT
    let all: Vec<UserModel> = User::find()
        .order_by_asc(user::Column::Id)
        .all(&db)
        .await?;
    for u in &all {
        println!("- #{} {} <{}>", u.id, u.name, u.email);
    }

    // UPDATE
    let mut a = alice.into_active_model();
    a.name = Set("Alice (updated)".into());
    let updated = a.update(&db).await?;
    println!("updated: {updated:?}");

    // DELETE
    let res = User::delete_by_id(updated.id).exec(&db).await?;
    println!("deleted rows: {}", res.rows_affected);

    // backend 用法（避免警告）
    let _ = DbBackend::Sqlite;
    let _ = ActiveValue::set(0_i32);
    Ok(())
}
