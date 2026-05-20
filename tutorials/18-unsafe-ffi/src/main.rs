use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// ── extern "C" 宣告：呼叫 C 函式 ────────────────────────────
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn abs(x: i32) -> i32;
}

// ── extern "C" 定義：給 C 端呼叫的 Rust 函式 ────────────────
// 加 #[no_mangle] 避免 name mangling，讓 linker 找得到符號名
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    // ── raw pointer ─────────────────────────────────────────
    let mut x = 5;
    let r_const: *const i32 = &x;
    let r_mut: *mut i32 = &mut x;

    // 建立 raw pointer 安全；deref 才需要 unsafe
    unsafe {
        println!("*r_const = {}", *r_const);
        *r_mut = 10;
        println!("after write: x = {x}");
    }

    // ── split_at_mut：拿兩個 mutable slice ──────────────────
    // 安全 API 內部用 unsafe（編譯期無法證明兩段不重疊）
    let mut v = vec![1, 2, 3, 4, 5];
    let (left, right) = v.split_at_mut(2);
    left[0] = 100;
    right[0] = 200;
    println!("after split: {v:?}");

    // ── unsafe fn ───────────────────────────────────────────
    unsafe {
        dangerous();
    }

    // ── FFI 呼叫 libc ───────────────────────────────────────
    let cs = CString::new("hello world").expect("no null bytes");
    let len = unsafe { strlen(cs.as_ptr()) };
    println!("C strlen = {len}");

    let a = unsafe { abs(-42) };
    println!("C abs(-42) = {a}");

    // ── 反向：把 C string 轉 Rust ───────────────────────────
    // 模擬 C 端拿到 *const c_char
    let raw: *const c_char = cs.as_ptr();
    let back: &CStr = unsafe { CStr::from_ptr(raw) };
    let s = back.to_str().unwrap();
    println!("back to Rust: '{s}'");

    // ── 自己定義的 extern "C" fn 可被同 binary 呼叫 ─────────
    let sum = rust_add(3, 4);
    println!("rust_add via ABI = {sum}");

    // ── 不變式：建構 unsafe wrapper ─────────────────────────
    let safe = SafeIndex::new(vec![10, 20, 30]);
    println!("safe.get(1) = {}", safe.get(1));
}

unsafe fn dangerous() {
    println!("dangerous() called");
}

// ── 範例：用 unsafe 內部、提供 safe API ─────────────────────
struct SafeIndex {
    data: Vec<i32>,
}

impl SafeIndex {
    fn new(data: Vec<i32>) -> Self {
        Self { data }
    }

    // 公開 API 是 safe 的：自己 check bound
    fn get(&self, i: usize) -> i32 {
        assert!(i < self.data.len(), "out of bounds");
        // 已驗證 → 用 unsafe 跳過再次 check
        unsafe { *self.data.get_unchecked(i) }
    }
}
