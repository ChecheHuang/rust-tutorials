# 故事版：校外教學的準備清單 🎒

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「準備校外教學的書包，清單一層一層被改寫」的故事，幫你「感覺」出設定的優先順序（default < file < env < CLI）、用 serde 定義設定結構、`#[serde(default)]`、環境變數 override 與 prefix、`figment` 的 merge、profile、secret 處理（`.env`、`secrecy`）、設定驗證。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 四層清單，後面的蓋過前面的

小芽明天要去校外教學。決定書包裡要帶什麼，有四個來源：

1. **媽媽的固定清單**：每次出門都這樣帶，貼在冰箱上。
2. **學校的通知單**：這次的地點、集合時間。
3. **當天早上看天氣**：下雨就要多帶傘。
4. **出門前爸爸臨時說**：「今天改帶這個！」

如果幾個來源說的不一樣，就聽**越後面的**：爸爸臨時說的蓋過天氣，天氣蓋過通知單，通知單蓋過固定清單。

> 這就是 **設定的層級合併**：`default < config file < env var < CLI flag`。預設寫在程式裡，設定檔是 toml / yaml，正式環境用環境變數，臨時調整用 CLI 參數。順序要固定，並寫在文件裡。

---

## 清單長什麼樣子

媽媽的清單分成幾大格：「**出發**」那一格寫地點和集合點；「**午餐**」那一格寫便當和要帶幾雙筷子；「**特別活動**」那一格寫要不要參加新的體驗、要不要試玩測試版的遊戲。

媽媽把這張固定清單**直接寫在冰箱門上**，所以就算找不到其他通知，至少還有它。

> 這就是 **用 serde 定義設定結構**：`struct Config { server: ServerConfig, database: DatabaseConfig, features: FeatureFlags }`，加上 `#[derive(Debug, Deserialize)]`。`load_config()` 從內嵌的 `const DEFAULT_TOML` 開始，用 `toml::from_str(DEFAULT_TOML)?` 讀成 `Config`。

---

## 沒寫的就照慣例

媽媽的清單上，「午餐」那格**沒寫要帶幾雙筷子**。小芽問：「那要帶幾雙？」

媽媽說：「沒寫就照我們家的**慣例**，帶 10 雙，全班分著用。」

「特別活動」那一整格**根本沒寫**。那就全部當作「**不參加**」。

> 這就是 **`#[serde(default)]`**：`DatabaseConfig` 的 `#[serde(default = "default_pool_size")] pool_size` 缺欄位時呼叫 `default_pool_size()` 得到 `10`；`Config` 的 `#[serde(default)] features` 整段缺少時用 `FeatureFlags::default()`，`new_ui` 和 `beta` 都是 `false`。

---

## 當天早上的便利貼

出門前，家裡冰箱上會貼幾張**便利貼**：

- 「**小芽_出發_集合點：9000 號門**」→ 那就改成 9000 號門。
- 「**小芽_特別活動_測試版：要**」→ 那就參加測試版。

便利貼**一定要寫「小芽_」開頭**。因為家裡還有哥哥，如果便利貼只寫「集合點」，誰也不知道是寫給誰的，哥哥可能也跑去 9000 號門了。

還有一件事：便利貼上寫的「9000」要**真的是數字**。寫成「九千左右」的話，媽媽會說：「這看不懂，重寫。」

> 這就是 **環境變數 override**：`env::var("APP_SERVER_PORT")` 存在就 `cfg.server.port = p.parse()?;`（不是數字就回錯誤）；`env::var("APP_FEATURES_BETA").ok().as_deref() == Some("true")` 才把 `beta` 打開；`DATABASE_URL` 覆寫 `database.url`。自己的變數要加 **prefix**（`APP_`），nested 用 `_` 或 `__` 分層，免得跟系統其他工具撞名。

---

## 疊透明片

小芽覺得每張便利貼都要媽媽一句一句確認，好慢。

老師教了一個方法：把每一份清單都畫在**透明片**上，依序**疊起來**：最底下是固定清單、上面是學校通知單、再上面是寫著「小芽_」的便利貼。從上往下看，**上面那層有寫的就蓋過下面**，沒寫的地方就看得到底下那層。疊好之後，一次**整理成最後的清單**。

> 這就是 **`figment`**：`Figment::from(Serialized::defaults(...)).merge(Toml::file("app.toml")).merge(Env::prefixed("APP_")).extract()?`，**後面 merge 的蓋過前面**；`extract()` 時才變成 `Config`，出錯會指出是哪個欄位。比本章手刻的 `if let Ok(...)` 一串清爽很多。另一個選擇是老牌的 `config` crate。

---

## 不同的場合，不同的清單

學校的通知單其實有好幾頁：「**平常上課**」一頁、「**校外教學**」一頁、「**畢業旅行**」一頁。每一頁都先寫共同的部分，再寫那一頁特別的。

出門前只要說「**今天是校外教學**」，就翻到那一頁看；沒說的話，就當作平常上課。

> 這就是 **profile**：`app.toml` 裡寫 `[default]`、`[dev]`、`[prod]`，用 `Toml::file("app.toml").nested()` 讀，再 `.select(&profile)` 選一頁；`profile` 從 `env::var("APP_PROFILE").unwrap_or_else(|_| "dev".into())` 來。`APP_PROFILE=prod cargo run` 就是翻到正式那一頁。

---

## 保險箱密碼不寫在清單上

這次校外教學要帶一張**門票**，門票上有一組密碼。小芽差點把密碼**直接寫在通知單上**——可是通知單要貼在班上給大家看！

媽媽說：

- 密碼要**當天早上才告訴你**，不要寫在會被大家看到的地方。
- 在家練習的時候，可以寫在**自己抽屜裡的小紙條**上，但那張紙條**絕對不能帶去學校**；真的出發那天，要從老師那邊的**保險箱**拿。
- 要是有人問「你書包裡有什麼？」，小芽要回答「**有一張門票，密碼保密**」，不能把密碼念出來。只有要進場時，才拿出來給驗票員看。

> 這就是 **secret 處理**：密碼不要寫進會 commit 的設定檔，用環境變數注入（例如 `DATABASE_URL=... cargo run`）。開發時用 `.env` + `dotenvy::dotenv().ok()`，`.env` 放進 `.gitignore`，**production 不用 `.env`**，改用 K8s Secret、Vault、AWS Secrets Manager。`secrecy` 的 `SecretString`（0.10 版，舊版叫 `Secret<String>`）在 `Debug` 時印 `[REDACTED]`，要用時顯式呼叫 `expose_secret()`。

---

## 出門前再檢查一次

清單整理好之後，媽媽會**最後再看一次**：

- 「集合點寫『0 號門』？沒有 0 號門啊，重寫。」
- 「筷子帶 0 雙？那大家怎麼吃飯，重寫。」
- 「這張便利貼寫『集合典』？是不是寫錯字了？我們清單上沒有這一項。」

有問題就**在出門前**說出來，總比到了半路才發現好。

> 這就是 **驗證**：deserialize 之後再寫 `validate(&cfg)`，例如 `port == 0`、`pool_size == 0` 就回錯誤；也可以用 `validator` crate。`#[serde(deny_unknown_fields)]` 讓拼錯的 key 直接報錯，而不是被默默忽略。本章 `main` 回傳 `Result`，`load_config()?` 失敗就在啟動時停下來。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 四層來源，後面的蓋過前面 | **default < file < env < CLI** |
| 清單的幾大格 | `Config` / `ServerConfig` / `DatabaseConfig` / `FeatureFlags` |
| 寫在冰箱門上的固定清單 | 內嵌的 `DEFAULT_TOML` + `toml::from_str` |
| 沒寫就帶 10 雙筷子 | `#[serde(default = "default_pool_size")]` |
| 整格沒寫就不參加 | `#[serde(default)]` + `Default` |
| 當天的便利貼 | **環境變數 override**（`env::var`） |
| 便利貼要寫「小芽_」開頭 | **prefix** `APP_` |
| 「九千左右」看不懂 | `p.parse()?` 失敗回錯誤 |
| 疊透明片 | **`figment`** 的 `.merge(...)` |
| 整理成最後清單 | `.extract()` |
| 通知單的不同頁 | **profile**（`[dev]`、`[prod]`、`.select()`） |
| 當天才說的密碼 | secret 用 env 注入 |
| 抽屜裡的小紙條 | `.env` + `dotenvy`（不 commit、不上 prod） |
| 老師的保險箱 | Vault / K8s Secret / Secrets Manager |
| 「有門票，密碼保密」 | **`secrecy::Secret`** 的 `[REDACTED]` |
| 進場才拿出來 | `expose_secret()` |
| 出門前再檢查 | **`validate`**、`deny_unknown_fields` |

---

## 一句話記住

> **設定就像校外教學的清單：冰箱上的固定清單（default）、學校通知單（file）、當天便利貼（env，記得加 `APP_` 開頭）、爸爸臨時交代（CLI）一層疊一層，後面的蓋過前面；密碼不寫在清單上，問起來只說「保密」（`secrecy`），出門前再檢查一次（validate）。**
