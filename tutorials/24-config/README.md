# 24. 設定管理

> 範圍：config / figment crate、環境變數 + 檔案 + CLI 合併、secret 處理

## 設計目標

一個服務的設定有幾個層次的來源：

```
低優先（被覆寫）          高優先（覆寫一切）
─────────────────────────────────────────────
default       <  config file  <  env var  <  CLI flag
（程式內嵌）   （toml/yaml）    （prod 配置）  （臨時調整）
```

任何成熟的設定方案都該支援這個層級合併。

## 三個選擇

| Crate | 風格 |
|-------|------|
| **手刻 + serde + toml** | 完全控制，最少 dep；簡單場景夠用 |
| [`config`](https://crates.io/crates/config) | 多來源 merge，老牌 |
| [`figment`](https://crates.io/crates/figment) | 來自 Rocket，profile 概念優秀，現代風格 |

## 手刻方案（本章 main.rs）

```rust
#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
    #[serde(default)]
    features: FeatureFlags,
}
```

關鍵 serde 屬性：

| Attribute | 用途 |
|-----------|------|
| `#[serde(default)]` | 缺欄位時用 `Default::default()` |
| `#[serde(default = "func")]` | 用指定函式回的值當 default |
| `#[serde(rename = "name")]` | 序列化時用別名 |
| `#[serde(skip)]` | 不序列化 |
| `#[serde(flatten)]` | 攤平 nested struct |

讀檔 + parse：

```rust
let s = std::fs::read_to_string("config.toml")?;
let cfg: Config = toml::from_str(&s)?;
```

env override 手寫 if/let 串。簡單但繁瑣。

## `figment` 推薦

```toml
[dependencies]
figment = { version = "0.10", features = ["toml", "env", "yaml"] }
```

```rust
use figment::{Figment, providers::{Format, Toml, Env, Serialized}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct Config {
    host: String,
    port: u16,
    db_url: String,
}

let cfg: Config = Figment::from(Serialized::defaults(Config {
        host: "127.0.0.1".into(),
        port: 8080,
        db_url: "postgres://localhost/dev".into(),
    }))
    .merge(Toml::file("app.toml"))
    .merge(Env::prefixed("APP_"))     // APP_PORT=9000 → port = 9000
    .extract()?;
```

特性：
- 來源無限疊加，**後面 override 前面**
- profile 支援（`Toml::file("app.toml").nested()` 讓不同 [section] 是不同 profile）
- 錯誤訊息把 conflict 指到具體欄位

## 環境變數慣例

- 用 prefix 避免污染：`APP_*`、`MY_SERVICE_*`
- nested 用 `_` 或 `__` 分層：`APP_SERVER_PORT` → `server.port`
- bool 接 `"true"`/`"false"`（或 `1`/`0`，由 lib 規範）

## Secret 處理

**絕對不要**把 password、token 寫進 commit 過的 config file。

### 方法 1：env var

最常見。設定檔放結構，密碼從 env 注入：

```toml
# app.toml（commit）
[database]
url = "${DATABASE_URL}"   # 或留空，由 env 補
```

```bash
DATABASE_URL=postgres://user:pass@host/db cargo run
```

### 方法 2：secret file（不 commit）

`.env` + `dotenvy` crate：

```bash
# .env（在 .gitignore）
DATABASE_URL=...
JWT_SECRET=...
```

```rust
dotenvy::dotenv().ok();
let url = env::var("DATABASE_URL")?;
```

注意：`.env` 是開發方便，**production 不該用**——用 K8s Secret / Vault / SSM。

### 方法 3：Secrets manager

production：AWS Secrets Manager、HashiCorp Vault、GCP Secret Manager。Rust SDK 都有。

### `secrecy` crate — 防誤 log

```rust
use secrecy::{Secret, ExposeSecret};

#[derive(Deserialize)]
struct Config {
    api_key: Secret<String>,
}

let key = cfg.api_key.expose_secret();   // 顯式
println!("{:?}", cfg);                    // → api_key: [REDACTED]
```

`Secret<T>` 的 `Debug` 自動 redact，避免不小心 log 出來。

## Profile：dev / staging / prod

```toml
# app.toml
[default]
log_level = "info"

[default.database]
pool_size = 10

[dev]
log_level = "debug"

[dev.database]
url = "postgres://localhost/dev"

[prod]
log_level = "warn"

[prod.database]
url = "postgres://prod-host/main"
pool_size = 50
```

```rust
let profile = env::var("APP_PROFILE").unwrap_or_else(|_| "dev".into());
let cfg: Config = Figment::from(Toml::file("app.toml").nested())
    .select(&profile)
    .merge(Env::prefixed("APP_"))
    .extract()?;
```

`APP_PROFILE=prod cargo run`。

## 驗證

deserialize 後額外檢查：

```rust
fn validate(cfg: &Config) -> Result<(), String> {
    if cfg.server.port == 0 { return Err("port can't be 0".into()); }
    if cfg.database.pool_size == 0 { return Err("pool_size > 0".into()); }
    Ok(())
}
```

或用 [`validator`](https://crates.io/crates/validator) crate derive 風格。

## 對照 TypeScript

Node 端常見組合：`dotenv` 載 `.env`、`zod` / `joi` 驗證 schema、自己手刻 default 合併。Rust 用 `figment` / `config` 把這幾步合一，加上 serde derive 自動把 TOML/YAML 映射到 struct。

### 對照表

| 需求 | Node / TypeScript | Rust |
|---|---|---|
| 載 `.env` | `dotenv` / `dotenvx` | `dotenvy` |
| 多來源合併 | 自己寫 / config crate | `figment` / `config` |
| 設定檔格式 | JSON / YAML / .env | TOML（預設）/ YAML / JSON |
| 驗證 schema | zod / joi / class-validator | serde + `validator` crate |
| 從 struct 推 schema | Zod 推 TypeScript | serde derive 用 struct 當 schema |
| 環境變數 prefix | 自己約定 | `Env::prefixed("APP_")` |
| Profile（dev/prod） | `NODE_ENV` + 多檔 | figment profile 概念 |
| Secret 別 log | 自己 redact | `secrecy::Secret<T>` 預設 redact |
| Secret 來源 | env var / Vault / SSM | env var / Vault / SSM（rust SDK） |
| Hot reload | `chokidar` watch + reload | `notify` crate + `watch::Sender` |

### 程式碼對照

```ts
// TS — dotenv + zod
import 'dotenv/config';
import { z } from 'zod';

const ConfigSchema = z.object({
  server: z.object({
    host: z.string().default('127.0.0.1'),
    port: z.coerce.number().default(8080),
  }),
  database: z.object({
    url: z.string(),
  }),
});

type Config = z.infer<typeof ConfigSchema>;

const cfg = ConfigSchema.parse({
  server: { port: process.env.APP_SERVER_PORT },
  database: { url: process.env.DATABASE_URL },
});
```

```rust
// Rust — figment + serde
use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Env, Serialized}};

#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_port")]
    port: u16,
}
fn default_host() -> String { "127.0.0.1".into() }
fn default_port() -> u16 { 8080 }

#[derive(Deserialize)]
struct DatabaseConfig { url: String }

let cfg: Config = Figment::new()
    .merge(Toml::file("app.toml"))
    .merge(Env::prefixed("APP_").split("_"))   // APP_SERVER_PORT → server.port
    .extract()?;
```

Secret：

```ts
// TS — 自己處理
const dbPassword = process.env.DB_PASSWORD;
// 自己注意別 log
```

```rust
// Rust — secrecy crate
use secrecy::{Secret, ExposeSecret};

#[derive(Deserialize)]
struct Config {
    db_password: Secret<String>,
}

let cfg: Config = ...;
let pw = cfg.db_password.expose_secret();
println!("{:?}", cfg);   // → db_password: [REDACTED]
```

### 心智模型差異

1. **schema 從 struct 來**。Zod 寫一份 schema 然後 `z.infer` 推 TS 型別；Rust 直接寫 struct，serde derive 自動處理「**JSON/TOML/YAML → struct**」。一份來源、編譯期型別檢查。
2. **缺欄位編譯失敗**。TS 拿 `process.env.X` 永遠是 `string | undefined`，要自己 narrowing。Rust deserialize 缺欄位**直接 error**（除非 `#[serde(default)]`）——壞設定提前死。
3. **multi-source merge 是 first-class**。Node 拼 `dotenv` + JSON + CLI 多半自己寫；Rust figment 一行串：`Figment::new().merge(...).merge(...).merge(...)`。最後 merge 的優先。
4. **TOML 是 Rust 預設**。Node 多用 JSON / YAML；Rust 預設 TOML（`Cargo.toml` 也是 TOML），社群偏好。差別不大，習慣後 TOML 對註解友善。
5. **`Secret<T>` 防誤 log**。對應 TS 沒有專屬型別——隨便 `console.log(cfg)` 就洩漏。Rust `Secret<String>` 的 `Debug` 預設印 `[REDACTED]`，要拿值必須 `expose_secret()`，明確「我知道在做什麼」。
6. **`.env` 在 production 是反 pattern**。Node 生態歷史上 dotenv 進 production 很常見，但實務上 K8s Secret / Vault / SSM 更好。Rust 慣例：`.env` 只是 dev 方便，production 從 env var 注入。
7. **deny_unknown_fields**。production 服務建議加 `#[serde(deny_unknown_fields)]`——key 拼錯不會被當成 silent default，直接報錯。對應 TS Zod 的 `.strict()`。
8. **hot reload 不是必需**。Node 用 `chokidar` watch `.env` 改動 reload 是某些 dev workflow；Rust 用 `notify` + `tokio::sync::watch::Sender` 一樣可做，但 production 慣例是「**重啟服務換設定**」（K8s rolling restart）。

## 常見陷阱

1. **env var 沒帶 prefix** — `PORT=...` 跟系統其他工具撞，必加自己的 prefix。
2. **secret 落到 log** — `Debug` derive 加在含密碼的 struct，一不小心整個結構被 log。用 `secrecy` 或手寫 `Debug`。
3. **`#[serde(deny_unknown_fields)]`** — production 建議加，防 typo（key 拼錯不會被發現）。
4. **TOML 用 dotted key 易讀** — `server.port = 8080` 等同 `[server]\nport = 8080`，但混用要小心。
5. **CLI override 跟 env 衝突** — 建議 CLI > env > file > default 固定順序，文件寫清楚。

## 練習

1. 把本章手刻方案改成用 `figment`，比較程式碼複雜度。
2. 加 `[dev]` 與 `[prod]` profile，用 `APP_PROFILE` 切換。
3. 用 `secrecy::Secret<String>` 包 `db_password`，確認 `println!("{cfg:?}")` 不會洩漏。
4. 把 default 改成「找 `app.toml`，找不到就用內嵌 default」。

## 延伸閱讀

- [figment 文件](https://docs.rs/figment/)
- [config crate](https://docs.rs/config/)
- [12-Factor App: III. Config](https://12factor.net/config)
