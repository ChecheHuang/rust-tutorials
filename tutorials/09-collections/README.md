# 09. 集合型別

> 範圍：Vec、HashMap、HashSet、BTreeMap、VecDeque、容量與配置

## 選擇指引

| 需求 | 用 |
|------|---|
| 線性 list、有序、push/pop 在尾端 | `Vec<T>` |
| key → value、最快查詢、不在乎順序 | `HashMap<K, V>` |
| 去重集合、最快 contains | `HashSet<T>` |
| 依 key 排序的 map | `BTreeMap<K, V>` |
| 依值排序的 set | `BTreeSet<T>` |
| 雙端 push/pop（queue / deque） | `VecDeque<T>` |
| 優先佇列（priority queue） | `BinaryHeap<T>` |
| 鏈結串列（**少用**） | `LinkedList<T>` |

## `Vec<T>`

連續記憶體的 growable array。底層 (ptr, len, cap)：

```rust
let mut v: Vec<i32> = Vec::new();
v.push(1);

let v2 = vec![10, 20, 30];           // macro
let v3 = vec![0; 100];                // 100 個 0
```

### 索引 vs `get`

```rust
let x = v[0];               // panic if out of bounds
let y = v.get(99);          // 回 Option<&T>，安全
```

公開 API / 不確定 index 是否有效時 prefer `.get()`。

### 容量管理

- `Vec::with_capacity(n)` — 預先 alloc，避免 push 時反覆 realloc
- `v.capacity()` — 目前 alloc 容量
- `v.reserve(n)` — 確保至少多 `n` 個額外容量
- `v.shrink_to_fit()` — 縮回 len

push 觸發 realloc 時，所有 `&T` 借用會失效——這就是為何 `for x in &v { v.push(...) }` 編譯失敗。

### 常用 method

```rust
v.push(x); v.pop();              // 尾端
v.insert(i, x); v.remove(i);     // 中間（O(n) 因為要 shift）
v.swap_remove(i);                // 與最後一個交換並 pop（O(1)）
v.sort();                         // 需 T: Ord
v.sort_by_key(|x| x.field);
v.contains(&x);                  // O(n)
v.iter() / v.iter_mut() / v.into_iter();
v.extend(other_iter);
```

## `HashMap<K, V>`

```rust
use std::collections::HashMap;

let mut scores: HashMap<String, u32> = HashMap::new();
scores.insert("alice".into(), 95);

if let Some(s) = scores.get("alice") { ... }
```

key 必須 `Hash + Eq`，預設使用 `SipHash`（HashDoS 安全）。對性能敏感的 internal map，可換 `ahash` / `fxhash`。

### `entry` API — idiomatic 插入/更新

```rust
// 「沒有就 0，然後 +1」
*scores.entry("carol".into()).or_insert(0) += 1;
```

省去 `get_mut` + `insert` 兩次 hash 查詢，是 Rust 計數的標準寫法。

### 常用 method

```rust
map.insert(k, v);            // 回 Option<V>（舊值）
map.get(&k); map.get_mut(&k);
map.remove(&k);
map.contains_key(&k);
map.entry(k).or_insert(default);
map.entry(k).or_insert_with(|| heavy());
map.entry(k).and_modify(|v| *v += 1).or_insert(0);
for (k, v) in &map { ... }
for k in map.keys() { ... }
for v in map.values() { ... }
```

順序：HashMap **不保證** 順序（每次 iterate 都可能不同，但同一個 program run 內穩定）。

## `HashSet<T>`

`HashMap<T, ()>` 的薄包裝。當你只在乎「有沒有」時用。

```rust
let mut tags: HashSet<&str> = HashSet::new();
tags.insert("rust");

let a: HashSet<i32> = [1, 2, 3].into_iter().collect();
let b: HashSet<i32> = [2, 3, 4].into_iter().collect();
a.intersection(&b);   // ∩
a.union(&b);          // ∪
a.difference(&b);     // a − b
```

## `BTreeMap` / `BTreeSet`

跟 HashMap / HashSet 同 API，但**依 key 排序**（B-tree 實作）。

| 何時用 BTreeMap | 何時用 HashMap |
|-----------------|----------------|
| 需要排序 iterate | 不在乎順序 |
| 需要範圍查詢（`range(..)`） | 純 lookup |
| key 較少 / 較大、cache 友善 | key 多、單次查詢快 |
| 不需要 `Hash` 實作 | key 必須 `Hash` |

```rust
use std::collections::BTreeMap;
let mut m: BTreeMap<i32, &str> = BTreeMap::new();
m.insert(3, "c"); m.insert(1, "a"); m.insert(2, "b");
for (k, v) in &m { /* 1,2,3 順序 */ }
for (k, v) in m.range(2..) { /* 2,3 */ }
```

## `VecDeque`

ring buffer 實作的雙端 queue：

```rust
let mut q: VecDeque<i32> = VecDeque::new();
q.push_back(1);
q.push_front(0);
q.pop_front();    // O(1) 兩端 push/pop
```

實作 FIFO queue prefer `VecDeque` 而非 `Vec::remove(0)`（後者 O(n)）。

## 從 iterator 建構：`collect`

```rust
let v: Vec<i32> = (1..=5).collect();
let m: HashMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();
let s: HashSet<i32> = [1, 2, 2, 3].into_iter().collect();
```

`collect` 是 generic over `FromIterator`，最強大的轉換 method。turbofish 可用：

```rust
let v = (1..=5).collect::<Vec<_>>();
```

## 對照 TypeScript

集合層面 TS 跟 Rust 都熟悉。差別在 Rust 多了 sorted map（`BTreeMap`）、雙端 queue、明顯的容量管理、以及 `entry` API。

### 對照表

| 需求 | TypeScript / JS | Rust |
|---|---|---|
| 線性 list | `Array` / `T[]` | `Vec<T>` |
| key→value | `Map<K, V>` / 物件 | `HashMap<K, V>` |
| 集合 | `Set<T>` | `HashSet<T>` |
| 排序 map | （沒有，要排陣列） | `BTreeMap<K, V>` |
| 排序 set | （沒有） | `BTreeSet<T>` |
| Queue / Deque | `Array.shift()`（**O(n)**） | `VecDeque<T>`（兩端 O(1)） |
| Priority queue | （沒有，要 lib） | `BinaryHeap<T>` |
| 鏈結串列 | （沒有） | `LinkedList<T>`（**少用**） |
| 索引越界 | `arr[100]` → `undefined` | `v[100]` **panic**；安全用 `v.get(100)` → `Option<&T>` |
| 插入後改 | 隨意（不影響 iteration 是另一回事） | 借用規則擋住「borrow + push」 |
| 預配置容量 | `new Array(n)`（sparse） | `Vec::with_capacity(n)` |
| Map 順序 | `Map` 保留**插入順序** | `HashMap` **無序**；要有序用 `BTreeMap` |
| 物件當 dict | `obj[key]` | 不行（用 `HashMap`） |
| 計數慣用法 | `m.set(k, (m.get(k) ?? 0) + 1)` | `*m.entry(k).or_insert(0) += 1` |
| Map / filter | `arr.map().filter()` | `iter.map().filter().collect::<Vec<_>>()`（第 15 章） |

### 程式碼對照

```ts
// TS
const v: number[] = [];
v.push(1);
const arr = [10, 20, 30];
const x = arr[1];                    // 20
const y = arr[99];                   // undefined（不會 throw）

const m = new Map<string, number>();
m.set("alice", 95);
const score = m.get("alice");        // 95 | undefined

// 計數
const count = new Map<string, number>();
for (const word of words) {
  count.set(word, (count.get(word) ?? 0) + 1);
}
```

```rust
// Rust
let mut v: Vec<i32> = Vec::new();
v.push(1);
let arr = vec![10, 20, 30];
let x = arr[1];                       // 20
// let y = arr[99];                   // panic!
let y = arr.get(99);                  // None

use std::collections::HashMap;
let mut m: HashMap<String, u32> = HashMap::new();
m.insert("alice".into(), 95);
let score = m.get("alice");           // Option<&u32>

// 計數 — entry API（一次 hash 完成）
let mut count: HashMap<&str, u32> = HashMap::new();
for word in words {
    *count.entry(word).or_insert(0) += 1;
}
```

### 心智模型差異

1. **索引語義不同**。JS `arr[100]` 給 `undefined`、`obj[key]` 給 `undefined`——「**找不到不會死**」。Rust `v[100]` panic、`v.get(100)` 回 `Option<&T>`——要嘛你保證 in-bounds，要嘛接 None。直覺從 JS 帶來的話，所有對外輸入記得用 `.get()`。
2. **`HashMap` 不保證順序**。JS `Map` 跟物件都**保留插入順序**（ES2015 起）；Rust `HashMap` 每次 iterate 順序可能不同。要可重現順序用 `BTreeMap`（依 key 排序）或 `indexmap` crate（保留插入順序）。
3. **`entry` API 是 Rust 慣用法**。JS 寫 `m.get(k) ?? 0` 然後 `m.set(k, v + 1)`——查兩次 hash。Rust `*m.entry(k).or_insert(0) += 1` 一次完成查+插+改，看起來奇怪但極常用。
4. **借用規則拘束 iteration**。JS 寫 `for (const x of arr) arr.push(x)` 不會編譯失敗（runtime 可能無限迴圈但不擋）。Rust 編譯期擋下，因為 `for x in &v` 拿 `&v` 借用、`push` 要 `&mut`——衝突。
5. **容量管理會影響效能**。JS 引擎內部 array 容量是黑盒；Rust `Vec::with_capacity(n)` 預先 alloc 是 idiomatic 做法。Loop 內 push 不預配置可能反覆 realloc（n 次 push 是 O(n log n)）。
6. **double-ended queue 是不同型別**。JS `arr.shift()` 雖然能用但 O(n)（要 shift 所有元素）；Rust 設計 `VecDeque` 給雙端使用（環形 buffer），兩端 O(1)。
7. **沒有「物件當 dict」習慣**。JS 寫 `const cache = {}; cache[key] = ...` 很常見；Rust 沒這個——一律 `HashMap`，省了 prototype 污染、null prototype 那些坑。

## 常見陷阱

1. **`v[i]` panic** — 對外輸入用 `.get(i)`。
2. **iterate 時 push** — 借用 vs 修改衝突；先 collect 要改的 index，再做。
3. **`HashMap::iter` 順序不固定** — 測試別依賴順序，用 `BTreeMap` 或 sort。
4. **預設 hasher 不快**：`HashMap<String, _>` 的 SipHash 對攻擊安全但慢。internal 高頻 map 換 `ahash` / `fxhash`。
5. **`LinkedList` 幾乎沒用** — 沒有 cache locality，95% 場景 `VecDeque` 都更好。
6. **`HashMap<&str, _>` vs `HashMap<String, _>`** — `&str` key 不能持久（lifetime 綁），存到 struct 多半要 `String`。

## 練習

1. 用 `HashMap` 計算字串中每個 char 出現次數，輸出最多的前 3 個（提示：`entry().or_insert(0)` + `sort_by_key`）。
2. 寫 `fn most_common<T: Hash + Eq>(items: &[T]) -> Option<&T>`。
3. 用 `BTreeMap` 實作一個 leaderboard：插入 (score, name)，能取出 top N。
4. 比較 `Vec::remove(0)` 與 `VecDeque::pop_front()` 在 100 萬筆 pop 時的時間（提示：第 25 章 criterion）。

## 延伸閱讀

- [`std::collections` 文件](https://doc.rust-lang.org/std/collections/)
- [The Rust Book — Ch 8 Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html)
