use criterion::{black_box, criterion_group, criterion_main, Criterion};

// 為了 bench 能獨立編譯，把目標函式複製在這（簡化教學）。
// 實務上 prefer 把目標函式放 lib crate 由 bench import。
fn sum_loop(v: &[i32]) -> i32 {
    let mut s = 0;
    for &x in v {
        s += x;
    }
    s
}

fn sum_iter(v: &[i32]) -> i32 {
    v.iter().sum()
}

fn sum_fold(v: &[i32]) -> i32 {
    v.iter().fold(0, |acc, &x| acc + x)
}

fn bench_sums(c: &mut Criterion) {
    let v: Vec<i32> = (1..=10_000).collect();

    c.bench_function("sum_loop", |b| {
        b.iter(|| sum_loop(black_box(&v)));
    });

    c.bench_function("sum_iter", |b| {
        b.iter(|| sum_iter(black_box(&v)));
    });

    c.bench_function("sum_fold", |b| {
        b.iter(|| sum_fold(black_box(&v)));
    });
}

criterion_group!(benches, bench_sums);
criterion_main!(benches);
