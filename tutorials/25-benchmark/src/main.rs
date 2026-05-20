// 被 benchmark 的目標函式（同時當作 lib 用）
pub fn sum_loop(v: &[i32]) -> i32 {
    let mut s = 0;
    for &x in v {
        s += x;
    }
    s
}

pub fn sum_iter(v: &[i32]) -> i32 {
    v.iter().sum()
}

pub fn sum_fold(v: &[i32]) -> i32 {
    v.iter().fold(0, |acc, &x| acc + x)
}

fn main() {
    let v: Vec<i32> = (1..=1000).collect();
    println!("sum_loop  = {}", sum_loop(&v));
    println!("sum_iter  = {}", sum_iter(&v));
    println!("sum_fold  = {}", sum_fold(&v));

    println!("\n跑 benchmark：");
    println!("  cargo bench");
    println!("\n結果報告：target/criterion/report/index.html");
}
