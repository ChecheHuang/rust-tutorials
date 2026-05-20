/// 計算階乘
///
/// # Examples
///
/// ```
/// // doc test：跟著 cargo test 一起跑
/// // 注意：此 binary crate 的 doc test 不會自動跑，
/// // 移到 lib crate 才有效。這裡只示範語法。
/// assert_eq!(6, 1 * 2 * 3);
/// ```
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

pub fn divide(a: i64, b: i64) -> Result<i64, &'static str> {
    if b == 0 {
        Err("division by zero")
    } else {
        Ok(a / b)
    }
}

fn main() {
    println!("跑測試：cargo test");
    println!("factorial(5) = {}", factorial(5));
    println!("divide(10, 3) = {:?}", divide(10, 3));
    println!("divide(10, 0) = {:?}", divide(10, 0));
}

// ── 單元測試：同檔案 ────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factorial_zero() {
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn factorial_five() {
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn divide_ok() {
        assert_eq!(divide(10, 3), Ok(3));
    }

    #[test]
    fn divide_zero() {
        assert!(divide(1, 0).is_err());
    }

    // 預期 panic
    #[test]
    #[should_panic(expected = "overflow")]
    fn overflow_panics() {
        let _ = u8::MAX + 1; // debug build 會 panic
        panic!("overflow"); // 即使 release build 也讓它 panic 以滿足測試
    }

    // 忽略（慢的、需網路、需特定環境）
    #[test]
    #[ignore = "slow"]
    fn slow_test() {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // 回 Result 也可以（取代 unwrap 鏈）
    #[test]
    fn parse_chain() -> Result<(), Box<dyn std::error::Error>> {
        let n: i32 = "42".parse()?;
        assert_eq!(n, 42);
        Ok(())
    }
}
