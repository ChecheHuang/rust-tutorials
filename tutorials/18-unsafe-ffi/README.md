# 18. unsafe 與 FFI

> 範圍：unsafe 邊界、raw pointer、extern "C"、cbindgen 入門

## `unsafe` 不是「關掉安全」

很多人誤解 `unsafe` 是「告訴編譯器不用檢查」。**錯**。`unsafe` 真正的意思是：

> 「**我（程式設計師）對接下來這幾件事的安全性負責**——因為編譯器沒辦法替我證明。」

借用檢查、型別系統**仍然作用**。`unsafe` 只解鎖 5 件平常不能做的事：

1. **deref raw pointer**（`*const T` / `*mut T`）
2. **呼叫 `unsafe fn`**（含 FFI）
3. **存取 / 修改 mutable static**
4. **實作 `unsafe trait`**（如 `Send`、`Sync`）
5. **存取 union field**

## Raw Pointer

```rust
let mut x = 5;
let p1: *const i32 = &x;
let p2: *mut i32 = &mut x;
```

跟 `&T` / `&mut T` 的差別：
- 可以 null
- 不會自動 deref
- 不受借用規則保護
- 允許別名（aliasing）

**建立** raw pointer **不需要** unsafe；**deref** 才需要：

```rust
unsafe { println!("{}", *p1); }
```

來源：
- `&x as *const T` / `&mut x as *mut T`
- `Box::into_raw(b)` / `Vec::into_raw_parts(v)`
- 從 C FFI 收到

## 何時用 unsafe

**絕大多數應用程式不該寫 unsafe**。應用層幾乎所有需求都有 safe 替代。

該寫的情境：
1. **FFI** — 跟 C library 互通
2. **底層原始碼** — kernel、driver、embedded
3. **效能 hot path** — 已 benchmark 確認且能證明正確
4. **資料結構作者** — 寫 lock-free queue、自訂 allocator
5. **包裝既有 safe API** — 像 `Vec::split_at_mut`，內部用 unsafe 但 export safe

## 設計模式：safe wrapper

```rust
struct SafeIndex { data: Vec<i32> }

impl SafeIndex {
    fn get(&self, i: usize) -> i32 {
        assert!(i < self.data.len());
        unsafe { *self.data.get_unchecked(i) }
    }
}
```

- 公開 API（`get`）是 safe
- 內部用 unsafe 達成 / 加速
- 對 unsafe 的**前提條件**已用 `assert!` 確認

這是標準函式庫到處用的模式。`Vec::push` 內部 `unsafe { ptr::write(...) }`，但呼叫者寫的是純 safe code。

## FFI — Foreign Function Interface

### 呼叫 C 函式

```rust
extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

let cs = CString::new("hello").unwrap();
let n = unsafe { strlen(cs.as_ptr()) };
```

- `extern "C" { ... }` 宣告外部 ABI 的 fn
- 呼叫一律 unsafe（編譯器無法驗證 C 端的 contract）
- 字串要轉成 `CString`（內含結尾 null）

### 定義給 C 呼叫的 Rust 函式

```rust
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}
```

- `extern "C"` 用 C 呼叫慣例（calling convention）
- `#[no_mangle]` 關掉 Rust 的 name mangling，讓 linker 找得到符號 `rust_add`
- 編譯成 `cdylib` / `staticlib` 後可從 C 端 link

`Cargo.toml`：

```toml
[lib]
crate-type = ["cdylib", "staticlib"]
```

### C ↔ Rust 型別對照

| C | Rust |
|---|------|
| `int` | `c_int`（通常 `i32`） |
| `unsigned int` | `c_uint`（通常 `u32`） |
| `size_t` | `usize` |
| `char` | `c_char`（通常 `i8`） |
| `char*` | `*mut c_char` 或 `CString` |
| `const char*` | `*const c_char` 或 `&CStr` |
| `void*` | `*mut c_void` |
| `struct Foo` | `#[repr(C)] struct Foo` |

`#[repr(C)]` 是關鍵：強制 struct field 用 C ABI 排列。

### 處理 string

```rust
// Rust → C：用 CString
let cs = CString::new("hello").unwrap();
call_c(cs.as_ptr());

// C → Rust：用 CStr
let s = unsafe { CStr::from_ptr(c_str_ptr) };
let rust_str = s.to_str().unwrap();   // 失敗 = 非 UTF-8
```

`CString` owned、`CStr` 借用。注意 `CString::new` 不允許內含 null byte。

## cbindgen — 自動產 C header

```bash
cargo install cbindgen
cbindgen --crate my_lib --output my_lib.h
```

把 Rust 的 `pub extern "C" fn` 變成對應的 C header，省手動同步的痛苦。

## bindgen — 從 C header 產 Rust binding

```bash
cargo install bindgen-cli
bindgen wrapper.h -o bindings.rs
```

把 C header 翻譯成 Rust 的 `extern "C" { ... }` 宣告。寫 wrapper crate 包 C library 的標準工具。

## unsafe 的安全 contract

寫 `unsafe fn` 一定要寫 doc comment 說明 caller 必須保證什麼：

```rust
/// # Safety
///
/// `ptr` 必須是 valid、aligned、指向初始化過的 `T`，
/// 且在這個 reference 存活期間不能被別處修改或釋放。
unsafe fn read_value<T>(ptr: *const T) -> T {
    ptr.read()
}
```

慣例：`/// # Safety` 是社群標準 section。

## 對照 TypeScript

TS / Node 跟 native code 互通是透過 N-API / NAPI-RS / WebAssembly / FFI 模組。**TS 本身完全沒有「unsafe」概念**——所有 JS 程式都在 V8 沙箱內，無法直接戳記憶體。

### 對照表

| 需求 | TypeScript / Node | Rust |
|---|---|---|
| 呼叫 C library | `ffi-napi` / `koffi` / native addon | `extern "C" {}` |
| 寫 native addon | C/C++/Rust + N-API | 直接 `pub extern "C" fn` |
| 寫 lib 給其他語言用 | TS 不行（要 Rust/C++/Go） | `crate-type = ["cdylib"]` |
| Raw pointer | 無對應 | `*const T` / `*mut T` |
| 跳出沙箱 | 不可能 | `unsafe { ... }` |
| Buffer / 共享記憶體 | `ArrayBuffer` / `SharedArrayBuffer` | `&[u8]` / `Vec<u8>` / raw pointer |
| 字串給 C | 自己加 `\0` + `Buffer` | `CString::new("...")` |
| 從 C 收字串 | C addon → JS string | `CStr::from_ptr(...)` |
| WASM | TS 編 WASM 沒辦法直接 | Rust 直接編 WASM 是主流 |
| 偵測 UB | 沒這層問題 | `cargo +nightly miri` |
| ABI 穩定性 | N-API 抽象掉 | Rust ABI 不穩定，FFI 強制 `extern "C"` |

### 程式碼對照

從 Node 呼叫 native lib：

```ts
// TS — 透過 koffi
import koffi from 'koffi';
const lib = koffi.load('libc.so.6');
const strlen = lib.func('strlen', 'size_t', ['str']);
console.log(strlen('hello'));   // 5
```

```rust
// Rust — extern "C" 直接呼叫
use std::ffi::{c_char, CString};

extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

let cs = CString::new("hello").unwrap();
let n = unsafe { strlen(cs.as_ptr()) };
println!("{n}");   // 5
```

寫 lib 給 Node 用：

```ts
// 純 TS 寫不出來，必須用 Rust / C++ / Go
```

```rust
// Rust — 一個函式 export 給 C / Node ABI
// Cargo.toml: crate-type = ["cdylib"]
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}
// 編譯後 .so / .dll / .dylib 可被 Node ffi load
```

或更現代：用 `napi-rs` 直接寫 Node native module：

```rust
use napi_derive::napi;

#[napi]
fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}
// 編譯後直接 import 'my-module' 用，型別 .d.ts 自動產
```

### 心智模型差異

1. **`unsafe` 不是「關閉檢查」**。常見誤解：以為 `unsafe` 就是 C。錯。`unsafe` 是「**我（程式員）保證接下來這 5 件事的前提條件**」——borrow check、型別系統仍然作用。
2. **大多數 Rust 應用程式不寫 unsafe**。應用層幾乎所有需求都有 safe 替代。寫 unsafe 的場景：FFI、底層原始碼、效能 hot path、寫資料結構作者、包裝既有 safe API。TS 開發者轉 Rust 早期幾乎不會碰。
3. **safe wrapper pattern**：寫 `unsafe` 程式碼但 **export safe API**。`Vec::push` 內部 `unsafe { ptr::write(...) }`，但你呼叫的是 safe code。標準函式庫到處用這模式。
4. **FFI 是必要時的橋**。Rust 比 TS 在這層強很多——TS 永遠透過 N-API、無法直接呼叫 C；Rust `extern "C"` 跟 C 是 binary 級互通。同個道理：寫給其他語言（Python / Node / Ruby）用的高效 native lib，Rust 是主流選擇之一。
5. **`#[repr(C)]` 不是預設**。Rust struct 預設由編譯器決定 field 排列（field reorder、optimize size）；要跟 C 互通必須 `#[repr(C)]` 強制 C ABI 佈局。TS 沒這問題（永遠走 JSON serialization）。
6. **CString / CStr 的存在感**。C 字串是 NUL-terminated，Rust string 不是。`CString::new("foo")` 加結尾 `\0`、`CStr::from_ptr(p)` 從 C 收。TS 用 Buffer + `null` byte 處理，但通常被 lib 包裝掉。
7. **Miri 是 unsafe 的好朋友**。`cargo +nightly miri test` 用 MIR-level interpreter 跑你的測試，抓 unsafe code 的 UB（dangling、未初始化、alignment）。TS 沒對應的工具——但 TS 也沒這層問題。
8. **WASM 是 Rust 的甜蜜點**。`wasm-pack` + `wasm-bindgen` 讓 Rust 編成 WASM 給 web / Node 用，型別映射、JS interop、async 都有 idiomatic 解。TS 編成 WASM 不存在（要轉成 AssemblyScript 之類）。

## 常見陷阱

1. **`*const T` 也要遵守對齊** — 解 raw pointer 到不對齊位址是 UB（即使讀 byte）。
2. **dangling pointer** — `let p = &x as *const i32; { let x = 5; } println!("{}", unsafe { *p });` UB。
3. **alias rule** — 即使是 raw pointer，若同時存在 `&mut T` 與 raw pointer 互動，仍可能 UB（Stacked Borrows / Tree Borrows）。
4. **`std::mem::transmute`** — 危險度頂端，幾乎永遠有更安全的替代。
5. **panic 跨 FFI 邊界** — Rust panic unwind 到 C 是 UB；FFI 邊界要 `catch_unwind`。
6. **`CString::new` panic 的 null byte** — 用 `?` 或 `unwrap_or_default` 處理。

## Miri — 偵測 UB 的 interpreter

```bash
cargo +nightly miri test
```

`miri` 是 Rust 的 MIR-level interpreter，能在測試時抓出 unsafe code 的 UB（dangling pointer、未初始化讀、alignment 違反）。寫任何 unsafe 都該 miri 跑過。

## 練習

1. 用 `extern "C"` 呼叫 libc 的 `getpid()`，印出當前 process id。
2. 寫一個 Rust `pub extern "C" fn fib(n: u32) -> u64`，compile 成 `cdylib`，從 Python `ctypes` 呼叫看看。
3. 改寫 `SafeIndex::get` 加入 `get_checked` 與 `get_unchecked_fast`（後者真的不檢查，註明 unsafe contract）。
4. 跑 `cargo +nightly miri run`（先 `rustup +nightly component add miri`）看本章 main.rs 是否乾淨。

## 延伸閱讀

- [Rustonomicon — Unsafe Rust](https://doc.rust-lang.org/nomicon/)
- [The Rustonomicon — FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [cbindgen](https://github.com/eqrion/cbindgen) / [bindgen](https://github.com/rust-lang/rust-bindgen)
