use std::fs;
use std::num::ParseIntError;

// ── 自訂錯誤型別（手寫版；第 19 章用 thiserror 自動化） ─────
#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(ParseIntError),
    NotFound(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "io error: {e}"),
            AppError::Parse(e) => write!(f, "parse error: {e}"),
            AppError::NotFound(s) => write!(f, "not found: {s}"),
        }
    }
}

impl std::error::Error for AppError {}

// 讓 `?` 自動把 io::Error 轉成 AppError
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

fn main() {
    // ── Option 基礎 ──────────────────────────────────────────
    let v = vec![1, 2, 3];
    match v.first() {
        Some(x) => println!("first = {x}"),
        None => println!("empty"),
    }
    let doubled_first = v.first().map(|x| x * 2);
    println!("doubled_first = {doubled_first:?}");

    // ── Result 基礎 ──────────────────────────────────────────
    let parsed: Result<i32, _> = "42".parse();
    match parsed {
        Ok(n) => println!("parsed = {n}"),
        Err(e) => println!("err = {e}"),
    }

    // ── ? 運算子 ─────────────────────────────────────────────
    match parse_and_double("21") {
        Ok(x) => println!("21*2 = {x}"),
        Err(e) => println!("error: {e}"),
    }
    match parse_and_double("oops") {
        Ok(_) => unreachable!(),
        Err(e) => println!("expected error: {e}"),
    }

    // ── ? 跨多步 + From 自動轉錯 ─────────────────────────────
    match load_count("nonexistent.txt") {
        Ok(n) => println!("count = {n}"),
        Err(e) => println!("load err: {e}"),
    }

    // ── Option 處理方式 ──────────────────────────────────────
    let bad: Result<i32, _> = "not a num".parse();
    let safe = bad.unwrap_or(0);
    println!("unwrap_or → {safe}");

    let maybe: Option<i32> = None;
    let as_result: Result<i32, &str> = maybe.ok_or("missing");
    println!("ok_or → {as_result:?}");

    // ── ? 也適用 Option ──────────────────────────────────────
    println!("first_digit(\"abc12\") = {:?}", first_digit("abc12"));
    println!("first_digit(\"abc\") = {:?}", first_digit("abc"));

    // ── panic vs 錯誤 ────────────────────────────────────────
    // panic! 用於「不可能發生」、「程式 bug」
    // Result 用於「會發生且可恢復」（IO、parse、network）
    let _ = std::panic::catch_unwind(|| {
        // panic!("bug!"); // 註解掉
    });
}

fn parse_and_double(s: &str) -> Result<i32, AppError> {
    let n: i32 = s.parse()?; // ParseIntError → AppError 透過 From
    Ok(n * 2)
}

fn load_count(path: &str) -> Result<u32, AppError> {
    let s = fs::read_to_string(path)?; // io::Error → AppError
    let n: u32 = s.trim().parse()?; // ParseIntError → AppError
    Ok(n)
}

fn first_digit(s: &str) -> Option<u32> {
    let c = s.chars().find(|c| c.is_ascii_digit())?;
    c.to_digit(10)
}
