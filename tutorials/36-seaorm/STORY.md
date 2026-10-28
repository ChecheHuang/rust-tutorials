# 故事版：童軍團的團員名冊 ⛺

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「童軍團用一本名冊管理團員」的故事，幫你「感覺」出 SeaORM 的 Entity、Model、ActiveModel、`Set` / `Unchanged` / `NotSet`、CRUD（`insert` / `find` / `update` / `delete_by_id`）、Relation、migration，以及它和 sqlx 的關係是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 名冊的範本

青松童軍團的**團長阿正**，設計了一份**團員名冊範本**。範本上規定每位團員有三格：

- **編號**：最重要的一格，每人都不一樣，**由名冊自動往下編**。
- **名字**。
- **信箱**：**不能跟別人重複**。

範本最上面還寫著：「這本名冊叫做 **users**。」

> 這就是 **Entity**：`mod user` 裡的 `#[derive(DeriveEntityModel)] #[sea_orm(table_name = "users")] pub struct Model { ... }`。`#[sea_orm(primary_key)]` 標記 `id`，`#[sea_orm(unique)]` 標記 `email`。derive 會一口氣幫你生出 `Entity`、`Model`、`ActiveModel`、`Column` 這幾個型別。

### 從舊名冊抄範本

隔壁團已經用了好幾年名冊，阿正懶得自己畫，就**照著那本舊名冊反過來抄出範本**。

> 實務上很少手寫 Entity，而是用 `sea-orm-cli generate entity -u <DB 網址> -o src/entities`，讀現有資料庫的 schema 自動產生。範例為了展示結構才手寫。

---

## 照範本畫出空白名冊

今天是夏令營第一天，阿正在營地的**白板**上，照著範本畫出空白名冊的格子。他先看一下：「白板上**已經畫過了嗎？**沒有才畫。」

白板很方便，營隊結束一擦就沒了，**不用另外準備一本真的簿子**。

> 這就是 **`create_schema`**：`Schema::new(backend).create_table_from_entity(User)` 從 Entity 產生 `CREATE TABLE`，`stmt.if_not_exists()` 補上「沒有才建」，再用 `db.execute(backend.build(&stmt))` 執行。`Database::connect("sqlite::memory:")` 就是白板——SQLite 記憶體資料庫，程式結束就消失，不用準備外部 DB。

---

## 入團申請單上的貼紙

小隊長**小蓉**帶新團員**阿凱**來報名。她拿出一張**入團申請單**，申請單的每一格旁邊都可以貼一種**貼紙**：

- 🟢「**要寫入**」：這格的內容請寫進名冊。
- ⚪「**不動**」：這格維持原樣。
- ⬜「**留白**」：這格我不填，讓名冊自己處理。

小蓉在「名字：阿凱」、「信箱：k@x.com」旁邊貼了 🟢，**編號那格留白** ⬜——因為編號由名冊自動編。

阿正收下申請單，寫進名冊，然後把一張**正式團員卡**還給阿凱，上面已經印好了編號 **1**。

> 這就是 **ActiveModel 與 `insert`**：`UserAM { name: Set("alice".to_string()), email: Set(...), ..Default::default() }`。`Set(...)` 是 🟢，`..Default::default()` 讓其他欄位（`id`）變成 `NotSet` ⬜，由資料庫自動編號。`alice.insert(&db).await?` 回傳的是 `UserModel`——裡面已經有資料庫給的 `id`。

---

## 翻名冊點名

晚上集合，阿正要點名。他請名冊「**全部列出來，照編號從小排到大**」，然後一張一張念出來：「1 號阿凱、2 號小咪……」

拿到手上的是一張張**普通的團員卡**，只是拿來看的，上面**沒有任何貼紙**。

> 這就是 **`find` 與 Model**：`User::find().order_by_asc(user::Column::Id).all(&db).await?` 回傳 `Vec<UserModel>`。`Model` 是單純的 struct，代表「從資料庫讀出來的一列」。想加條件可以 `.filter(user::Column::Email.like("%@example.com"))`，要分頁用 `.paginate(&db, 20)`。

---

## 只改名字那一格

阿凱想把名字改成「阿凱（隊長）」。小蓉拿出他的團員卡，**轉成一張申請單**。

轉好的申請單上，**每一格都先貼好「不動」⚪**。小蓉只把「名字」那格換成 🟢「要寫入」，填上新名字。

阿正改名冊時**只動名字那一格**，信箱那一格碰都不碰。剛好這時另一位隊長正在幫阿凱改信箱，兩個人改的格子不一樣，**誰也不會蓋掉誰的修改**。

> 這就是 **`into_active_model` + `Set` + `update`**：`let mut a = alice.into_active_model();` 轉出來的每個欄位都是 `Unchanged`；`a.name = Set("Alice (updated)".into());` 只把 name 變成 `Set`；`a.update(&db).await?` 產生的 `UPDATE` **只會帶 name**。沒 `Set` 的欄位不會寫入，這是 feature 不是 bug，在並發場景特別重要。

---

## 退團

營隊結束，阿凱要搬家了。阿正說：「把**編號 1** 的那一行劃掉。」劃完後他看一下：「**劃掉了 1 行**，沒錯。」

> 這就是 **`delete_by_id`**：`User::delete_by_id(updated.id).exec(&db).await?` 回傳 `DeleteResult`，`res.rows_affected` 告訴你刪了幾列。

---

## 小隊和團員的關係

青松團分成好幾個**小隊**。每個小隊有很多團員，但每個團員**只屬於一個小隊**。阿正在團員名冊多加一格「屬於哪個小隊」，並在範本上寫明：「這一格對到**小隊名冊**。」

之後想問「老鷹小隊有哪些人？」或「每位團員和他的小隊一起列出來」，名冊就知道要去哪一本找。

> 這就是 **Relation**：一對多關係在 Entity 的 `enum Relation` 和 `impl Related<...> for Entity` 裡宣告。查詢可以用 `.filter(post::Column::UserId.eq(user_id))`，或 `find_also_related(user::Entity)` 一次拿到 `(post::Model, Option<user::Model>)`。範例的 `Relation` 是空的（`pub enum Relation {}`），因為只有一張表。

---

## 改格子的工程紀錄

明年團長想在名冊上加一格「生日」。他不是直接拿筆亂畫，而是寫一張**工程卡**：「第 N 號工程：加上生日格」，並且寫好**怎麼加**，也寫好**萬一要復原怎麼拿掉**。

這些工程卡是用團長自己的**正式寫法**寫的，不是隨便的便條紙。

> 這就是 **SeaORM migration**：`sea-orm-cli migrate init` 建立獨立的 `migration/` crate，`migrate generate create_users` 產生新的一份，`migrate up` 執行。Migration 用 Rust 寫（實作 `MigrationTrait`，有 `up` / `down`），不是 SQL 字串。

---

## 名冊底下還是同一組館員

名冊雖然好用，但其實阿正每次寫名冊，**背後都是請同一組館員幫他跑腿**（就是上一章圖書館的那組人）。

遇到特別複雜的問題，例如「依年齡分組再算平均、還要排名」，名冊的格子不夠用，阿正就**直接寫一張館員專用的單子**交給他們。

以後營隊從白板搬到**正式的大簿子**，名冊寫法完全不用改，只要換一個存放地址。

> 這就是 **SeaORM 與 sqlx 的關係**：SeaORM 底層就是 sqlx（範例的 feature 是 `sqlx-sqlite`）。CRUD 用 SeaORM，CTE、window function 等複雜查詢可以改用 raw SQL 或拿底層 sqlx pool。跨 DB（SQLite / Postgres / MySQL）只要換 `Database::connect(...)` 的連線字串。Transaction 也類似：`let txn = db.begin().await?;` 之後把 `&txn` 傳給 `insert` / `update`，最後 `txn.commit()`。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 名冊範本 | **Entity**（`DeriveEntityModel`） |
| 範本名稱 users | `#[sea_orm(table_name = "users")]` |
| 自動往下編的編號 | `#[sea_orm(primary_key)]` |
| 信箱不能重複 | `#[sea_orm(unique)]` |
| 照舊名冊抄範本 | `sea-orm-cli generate entity` |
| 白板 | `Database::connect("sqlite::memory:")` |
| 照範本畫空白名冊 | `create_table_from_entity` + `if_not_exists()` |
| 普通的團員卡 | **Model**（`UserModel`） |
| 有貼紙的申請單 | **ActiveModel**（`UserAM`） |
| 🟢 要寫入 | `Set(...)` |
| ⚪ 不動 | `Unchanged` |
| ⬜ 留白 | `NotSet`（`..Default::default()`） |
| 寫進名冊、拿回正式團員卡 | `insert(&db)` → `Model` |
| 照編號排好全部列出 | `User::find().order_by_asc(...).all(&db)` |
| 團員卡轉申請單 | `into_active_model()`（全部 `Unchanged`） |
| 只改名字那格 | `Set` name + `update(&db)` |
| 劃掉 1 行 | `delete_by_id(...).exec(&db)`、`rows_affected` |
| 小隊 ↔ 團員 | **Relation**、`find_also_related` |
| 工程卡 | SeaORM **migration**（Rust 寫，`up` / `down`） |
| 背後跑腿的館員 | 底層是 **sqlx** |

---

## 一句話記住

> **SeaORM 就是一本有範本的名冊：Entity 是範本、Model 是拿來看的卡、ActiveModel 是貼了貼紙的申請單——只有貼「要寫入」（`Set`）的格子才會真的被改。**
