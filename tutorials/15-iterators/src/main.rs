use std::collections::HashMap;

fn main() {
    let nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // ── 三種 iterator：iter / iter_mut / into_iter ─────────
    for x in nums.iter() {
        print!("{x} ");
    } // &i32
    println!();

    let mut v = vec![1, 2, 3];
    for x in v.iter_mut() {
        *x *= 10;
    } // &mut i32
    println!("after iter_mut: {v:?}");

    let v2 = vec![1, 2, 3];
    let _consumed: Vec<i32> = v2.into_iter().map(|x| x + 100).collect(); // T

    // ── adapter chain（lazy） ───────────────────────────────
    let sum_of_squares: i32 = nums.iter().filter(|&&x| x % 2 == 0).map(|&x| x * x).sum();
    println!("sum of even squares: {sum_of_squares}"); // 220

    // ── collect 成各種 collection ────────────────────────────
    let doubled: Vec<i32> = nums.iter().map(|&x| x * 2).collect();
    let str_nums: Vec<String> = nums.iter().map(|x| x.to_string()).collect();
    let hash: HashMap<i32, i32> = nums.iter().map(|&x| (x, x * x)).collect();
    println!("doubled: {doubled:?}");
    println!("first 3 strs: {:?}", &str_nums[..3]);
    println!("hash[5] = {:?}", hash.get(&5));

    // ── 常用 adapter ─────────────────────────────────────────
    let total: i32 = nums.iter().sum();
    let product: i64 = nums.iter().map(|&x| x as i64).product();
    let max = nums.iter().max();
    let min = nums.iter().min();
    let count = nums.iter().count();
    println!("sum={total} product={product} max={max:?} min={min:?} count={count}");

    // take / skip / step_by
    let taken: Vec<_> = (1..).take(5).collect(); // 無限 → 取前 5
    let skipped: Vec<_> = nums.iter().skip(3).take(3).collect();
    let stepped: Vec<_> = (0..20).step_by(3).collect();
    println!("take 5: {taken:?}");
    println!("skip 3 take 3: {skipped:?}");
    println!("step_by 3: {stepped:?}");

    // chain / zip / enumerate
    let a = [1, 2, 3];
    let b = [10, 20, 30];
    let chained: Vec<i32> = a.iter().chain(b.iter()).copied().collect();
    let zipped: Vec<(i32, i32)> = a.iter().zip(b.iter()).map(|(&x, &y)| (x, y)).collect();
    let enumerated: Vec<(usize, i32)> = a.iter().copied().enumerate().collect();
    println!("chain: {chained:?}");
    println!("zip: {zipped:?}");
    println!("enum: {enumerated:?}");

    // find / any / all / position
    let first_even = nums.iter().find(|&&x| x % 2 == 0);
    println!("first even: {first_even:?}");
    let has_seven = nums.iter().any(|&x| x == 7);
    let all_positive = nums.iter().all(|&x| x > 0);
    let pos_of_5 = nums.iter().position(|&x| x == 5);
    println!("has 7: {has_seven}, all positive: {all_positive}, pos of 5: {pos_of_5:?}");

    // fold = manual reduce
    let sum_fold = nums.iter().fold(0, |acc, &x| acc + x);
    println!("fold sum = {sum_fold}");

    let csv = nums
        .iter()
        .fold(String::new(), |mut s, x| {
            if !s.is_empty() {
                s.push(',');
            }
            s.push_str(&x.to_string());
            s
        });
    println!("csv: {csv}");

    // ── flat_map ─────────────────────────────────────────────
    let words = vec!["hello world", "rust is fun"];
    let all_words: Vec<&str> = words.iter().flat_map(|s| s.split(' ')).collect();
    println!("flat_map: {all_words:?}");

    // ── 自訂 Iterator ────────────────────────────────────────
    let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("fibs: {fibs:?}");
}

// 實作自己的 Iterator
struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Self { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let cur = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(cur) // 無限 iterator
    }
}
