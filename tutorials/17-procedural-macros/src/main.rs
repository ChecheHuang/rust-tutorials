use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── derive macro：serde 為 struct 生成 (de)serialize 邏輯 ──
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    #[serde(default)]
    bio: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
}

// ── derive macro：thiserror 自動生成 Error impl ─────────────
#[derive(Debug, Error)]
enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error at line {line}: {msg}")]
    Parse { line: u32, msg: String },

    #[error("not found: {0}")]
    NotFound(String),
}

fn main() {
    // ── derive 產生的 serialize ────────────────────────────
    let u = User {
        id: 1,
        name: "alice".into(),
        bio: "rustacean".into(),
        email: Some("a@example.com".into()),
    };
    let json = serde_json::to_string_pretty(&u).unwrap();
    println!("serialized:\n{json}");

    // 反序列化
    let raw = r#"{"id": 2, "name": "bob"}"#;
    let bob: User = serde_json::from_str(raw).unwrap();
    println!("deserialized: {bob:?}");

    // ── thiserror 產生的 Error ──────────────────────────────
    let err = AppError::Parse {
        line: 42,
        msg: "unexpected token".into(),
    };
    println!("error: {err}");
    println!("debug: {err:?}");

    // ── 透過 Error trait 取 source ──────────────────────────
    use std::error::Error;
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let wrapped: AppError = io.into(); // #[from] 讓這變可能
    println!("source: {:?}", wrapped.source());

    // ── 內建 derive 也是 proc macro：Debug、Clone、PartialEq…
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Coord(i32, i32);
    let c = Coord(3, 4);
    let c2 = c.clone();
    assert_eq!(c, c2);
    println!("Coord: {c:?}, equal = {}", c == c2);
}
