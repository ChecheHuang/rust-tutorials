# 34. JSON 與 serde

> 範圍：Serialize / Deserialize derive、欄位屬性、enum 標籤、serde_with、validator、動態 Value

## serde 模型

```
Rust 型別 ──Serialize──► serde 模型 ──serializer──► JSON/YAML/TOML/...
Rust 型別 ◄─Deserialize─ serde 模型 ◄─deserializer──
```

**重點**：`Serialize` / `Deserialize` 跟格式無關。同一個 struct 可同時對 JSON、TOML、bincode、MsgPack。

## 常用屬性

| 屬性 | 用途 |
|------|------|
| `#[serde(rename = "foo")]` | 改欄位名 |
| `#[serde(rename_all = "camelCase")]` | struct 級重命名 |
| `#[serde(default)]` | 反序列化缺欄位給預設 |
| `#[serde(skip)]` | 兩邊都跳 |
| `#[serde(skip_serializing_if = "Option::is_none")]` | 條件性 skip |
| `#[serde(flatten)]` | 攤平 nested struct |
| `#[serde(tag = "type")]` | enum 內部標籤 |
| `#[serde(transparent)]` | newtype 透傳 |
| `#[serde(with = "module")]` | 自訂 ser/de |

## enum 三種標籤策略

```rust
// 1. externally tagged（預設）
{"Login":{"user":"a"}}

// 2. internally tagged：#[serde(tag = "type")]
{"type":"login","user":"a"}

// 3. adjacent：#[serde(tag = "t", content = "c")]
{"t":"login","c":{"user":"a"}}
```

實務 API 多用 internally tagged（最像 OpenAPI discriminator）。

## serde_with — 解 awkward JSON

跟外部 API 對接常遇到：
- 數字以字串傳：`"123.45"` 不是 `123.45`
- 時間以毫秒整數：`1672531200000`
- 空字串代表 None

`serde_with` 提供 helper：

```rust
#[serde_as]
struct X {
    #[serde_as(as = "DisplayFromStr")]
    n: f64,
    #[serde_as(as = "chrono::serde::ts_milliseconds")]
    t: DateTime<Utc>,
}
```

## validator — derive 驗證

```rust
#[derive(Deserialize, Validate)]
struct SignUp {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

req.validate()?;   // 失敗回多錯誤
```

實務：在 axum/actix handler 收 `Json<T>` 後立刻 `.validate()?`，把錯誤訊息打回 400。

## 動態 `Value`

不知道 schema、或只要其中幾欄：

```rust
let v: Value = serde_json::from_str(s)?;
let id = v["data"]["id"].as_u64().unwrap_or(0);
```

⚠️ **不要全寫 `Value`** — 失去型別安全。混合策略：頂層用 struct、未知欄位 catch 進 `extra: HashMap<String, Value>`。

## 自訂 ser/de

兩種方式：
1. **`#[serde(with = "module")]`**：寫 `fn serialize`、`fn deserialize`。
2. **實作 trait**：手寫 `impl Serialize / Deserialize`。

寫 ser/de 是高階用法，先看 `serde_with` 有沒有現成方案。

## 常見陷阱

1. **`u64` 大於 `2^53` 會在 JS 端精度丟失** — 序列化成字串：`#[serde_as(as = "DisplayFromStr")]`。
2. **`Option<T>` 反序列化** — 缺欄位 = `None`；要區分 `null` 跟「沒這欄」用 `Option<Option<T>>` + `serde_with::rust::double_option`。
3. **enum default** — `#[serde(default)]` 對 enum 沒意義，要自己給 `Default`。
4. **`#[serde(flatten)]` + `#[serde(deny_unknown_fields)]` 衝突** — flatten 後無法 deny。
5. **`f64::NAN / INFINITY`** — JSON 不支援，serde_json 會報錯。
6. **大數字解析** — `u128` / `i128` 預設不支援 JSON，要 feature。

## 練習

1. 設計一個 enum 表示 webhook event（push、pr_opened、issue_commented），用 internally tagged 策略。
2. 寫一個 `Money` 結構，金額用 i64 表分（不浮點），對外 JSON 是 "12.34" 字串。
3. 用 `validator` 寫一組信用卡 / 手機格式檢查。
4. 比較 `serde_json::to_string` 和 `to_vec`、`to_writer` 的場景差異。

## 延伸閱讀

- [serde docs](https://serde.rs/)
- [serde_with](https://docs.rs/serde_with)
- [validator](https://docs.rs/validator)
