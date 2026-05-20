// 本章重點是 CI/CD pipeline 設定（見 README 與 .github/workflows）。
// 這個 binary 只是給 CI 有東西可以跑。

fn main() {
    println!("ch47 binary — see README for CI/CD pipeline examples");
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_works() {
        assert_eq!(1 + 1, 2);
    }
}
