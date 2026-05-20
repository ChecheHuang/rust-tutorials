use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

fn main() {
    // ── Vec<T> ───────────────────────────────────────────────
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    let v2 = vec![10, 20, 30]; // macro
    println!("v={v:?}  v2={v2:?}");

    println!("v[0] = {}", v[0]);
    println!("v.get(99) = {:?}", v.get(99)); // 安全（None）

    for x in &v {
        print!("{x} ");
    }
    println!();
    for x in v.iter().rev() {
        print!("{x} ");
    }
    println!();

    // 容量管理
    let v3: Vec<i32> = Vec::with_capacity(1000);
    println!(
        "v3: len={} cap={} (預先 alloc 避免反覆 realloc)",
        v3.len(),
        v3.capacity()
    );

    // pop / insert / remove
    let mut v4 = vec![1, 2, 3, 4];
    v4.pop(); // [1,2,3]
    v4.insert(0, 0); // [0,1,2,3]
    v4.remove(2); // [0,1,3]
    println!("after edits: {v4:?}");

    // ── HashMap<K, V> ────────────────────────────────────────
    let mut scores: HashMap<String, u32> = HashMap::new();
    scores.insert("alice".into(), 95);
    scores.insert("bob".into(), 80);

    if let Some(score) = scores.get("alice") {
        println!("alice = {score}");
    }

    // entry API：插入或更新（idiomatic）
    *scores.entry("carol".into()).or_insert(0) += 1;
    *scores.entry("carol".into()).or_insert(0) += 1;
    println!("carol after 2 inc = {}", scores["carol"]);

    for (name, score) in &scores {
        println!("  {name}: {score}");
    }

    // ── HashSet<T> ───────────────────────────────────────────
    let mut tags: HashSet<&str> = HashSet::new();
    tags.insert("rust");
    tags.insert("systems");
    tags.insert("rust"); // 重複插入無效
    println!("tags = {tags:?} (len={})", tags.len());
    println!("contains rust? {}", tags.contains("rust"));

    // 集合運算
    let a: HashSet<i32> = [1, 2, 3].into_iter().collect();
    let b: HashSet<i32> = [2, 3, 4].into_iter().collect();
    let inter: HashSet<&i32> = a.intersection(&b).collect();
    let union: HashSet<&i32> = a.union(&b).collect();
    let diff: HashSet<&i32> = a.difference(&b).collect();
    println!("a ∩ b = {inter:?}");
    println!("a ∪ b = {union:?}");
    println!("a − b = {diff:?}");

    // ── BTreeMap：依 key 排序 ────────────────────────────────
    let mut btm: BTreeMap<i32, &str> = BTreeMap::new();
    btm.insert(3, "three");
    btm.insert(1, "one");
    btm.insert(2, "two");
    for (k, v) in &btm {
        println!("  {k} → {v}"); // 1, 2, 3
    }

    // ── VecDeque：雙端 queue ─────────────────────────────────
    let mut q: VecDeque<i32> = VecDeque::new();
    q.push_back(1);
    q.push_back(2);
    q.push_front(0);
    println!("q = {q:?}");
    println!("pop_front = {:?}", q.pop_front());

    // ── 從 iterator 建構（collect） ──────────────────────────
    let from_iter: Vec<i32> = (1..=5).collect();
    let map_from_pairs: HashMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();
    println!("from_iter = {from_iter:?}");
    println!("map_from_pairs = {map_from_pairs:?}");
}
