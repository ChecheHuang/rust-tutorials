use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_with::{serde_as, DisplayFromStr};
use validator::Validate;

// ── 基本 derive ──────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    #[serde(default)]                 // 沒這欄位就給預設
    active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
}

// ── rename / rename_all ──────────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResp {
    request_id: String,
    is_success: bool,
}

// ── tag — 處理 enum ─────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event {
    Login { user: String },
    Logout { user: String, reason: String },
    Heartbeat,
}

// ── flatten — 攤平 nested ───────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
struct Page<T> {
    page: u32,
    size: u32,
    #[serde(flatten)]
    data: T,
}

// ── serde_with：字串轉數字 ───────────────────────────────────
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct Money {
    #[serde_as(as = "DisplayFromStr")]
    amount: f64,
    currency: String,
}

// ── validator ────────────────────────────────────────────────
#[derive(Debug, Deserialize, Validate)]
struct SignUp {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 基本 ser/de
    let u = User {
        id: 1,
        name: "alice".into(),
        email: "a@x.com".into(),
        active: true,
        avatar: None,
    };
    let s = serde_json::to_string_pretty(&u)?;
    println!("=== User (avatar 被 skip) ===\n{s}");

    let raw = r#"{"id":2,"name":"bob","email":"b@x.com"}"#;
    let u2: User = serde_json::from_str(raw)?;
    println!("\n=== 反序列化 active 預設值 ===\n{u2:?}");

    // 2. camelCase
    let r = ApiResp {
        request_id: "abc".into(),
        is_success: true,
    };
    println!("\n=== camelCase ===\n{}", serde_json::to_string(&r)?);

    // 3. enum tag
    for e in [
        Event::Login { user: "alice".into() },
        Event::Logout { user: "bob".into(), reason: "timeout".into() },
        Event::Heartbeat,
    ] {
        println!("=== Event ===\n{}", serde_json::to_string(&e)?);
    }

    // 4. flatten
    let page = Page {
        page: 1,
        size: 10,
        data: vec![u],
    };
    println!("\n=== flatten ===\n{}", serde_json::to_string_pretty(&page)?);

    // 5. serde_with
    let money_json = r#"{"amount":"123.45","currency":"USD"}"#;
    let m: Money = serde_json::from_str(money_json)?;
    println!("\n=== serde_with: amount 字串→f64 ===\n{m:?}");

    // 6. validator
    let bad = SignUp {
        email: "not-email".into(),
        password: "short".into(),
        age: 5,
    };
    match bad.validate() {
        Ok(_) => println!("valid"),
        Err(e) => println!("\n=== validator 錯誤 ===\n{e}"),
    }

    // 7. dynamic Value
    let v: Value = json!({
        "name": "carol",
        "tags": ["rust", "json"],
        "meta": { "ts": Utc::now().to_rfc3339() }
    });
    println!("\n=== Value 動態存取 ===\nname = {}", v["name"]);
    println!("first tag = {}", v["tags"][0]);

    // 8. 部分解析（用 Value 當 inbox 再解到具體 struct）
    let partial: Value = serde_json::from_str(r#"{"a":1,"b":"x","extra":null}"#)?;
    let a = partial.get("a").and_then(|v| v.as_i64()).unwrap_or(0);
    println!("a (任意 schema 摘) = {a}");

    let _now: DateTime<Utc> = Utc::now();
    Ok(())
}
