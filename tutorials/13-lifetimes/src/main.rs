// ── 簽章中的 lifetime ───────────────────────────────────────
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}

// ── struct 持有 reference 必須標 lifetime ──────────────────
struct Excerpt<'a> {
    part: &'a str,
}

impl<'a> Excerpt<'a> {
    fn announce(&self, msg: &str) -> &str {
        println!("Announcement: {msg}");
        self.part // 回的是 self 的 lifetime，編譯器透過 elision 推導
    }
}

// ── 'static — 整個 program 存活 ─────────────────────────────
fn forever() -> &'static str {
    "I live for the entire program"
}

// ── lifetime elision：簡單情況不必標 ────────────────────────
// fn first_word<'a>(s: &'a str) -> &'a str { ... }
// 編譯器幫你推導：
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

// ── 多個 input lifetime（不能 elide） ───────────────────────
fn pick_a<'a, 'b>(a: &'a str, _b: &'b str) -> &'a str {
    a
}

// ── HRTB (Higher-Ranked Trait Bound) ────────────────────────
fn apply_to_str<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> usize, // 「對任何 lifetime 'a」
{
    println!("len(\"abc\") = {}", f("abc"));
    println!("len(\"hello world\") = {}", f("hello world"));
}

fn main() {
    // ── 基本使用 ─────────────────────────────────────────────
    let s1 = String::from("long string is long");
    {
        let s2 = String::from("short");
        let result = longest(s1.as_str(), s2.as_str());
        println!("longest: {result}");
    }
    // result 在 s2 drop 後不可用——編譯器強制

    // ── struct holding reference ────────────────────────────
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();
    let excerpt = Excerpt {
        part: first_sentence,
    };
    let p = excerpt.announce("hi there");
    println!("excerpt part: {p}");

    // ── 'static ─────────────────────────────────────────────
    println!("{}", forever());

    // ── elision ─────────────────────────────────────────────
    let s = String::from("hello world");
    println!("first word: '{}'", first_word(&s));

    // ── HRTB ────────────────────────────────────────────────
    apply_to_str(|s: &str| s.len());

    // ── lifetime mismatch 範例（編譯失敗） ─────────────────
    // let r;
    // {
    //     let x = 5;
    //     r = &x;     // ERROR: x doesn't live long enough
    // }
    // println!("{r}");

    // ── 多個 input lifetime ──────────────────────────────────
    let a = String::from("aaa");
    let b = String::from("b");
    let picked = pick_a(&a, &b);
    println!("picked: {picked}");
}
