// ── 內嵌 module（單檔示範） ─────────────────────────────────
mod auth {
    pub fn login(user: &str) -> String {
        format!("welcome, {user}")
    }

    // 子 module
    pub mod password {
        pub fn hash(p: &str) -> String {
            // 假裝是 hash
            format!("<hash:{p}>")
        }

        // 內部 helper，不 export
        #[allow(dead_code)]
        fn salt() -> &'static str {
            "static-salt"
        }
    }

    // pub(crate)：只對 crate 內可見
    pub(crate) struct Session {
        pub user: String,
    }
}

// ── 用 use 縮短路徑 ─────────────────────────────────────────
use auth::password::hash;
use auth::{Session, login};

// ── re-export ───────────────────────────────────────────────
pub use auth::login as public_login;

mod math {
    // 函式可省 visibility 標註 → 預設 private（module 內可見）
    fn private_helper(x: i32) -> i32 {
        x + 1
    }

    pub fn double(x: i32) -> i32 {
        private_helper(x) * 2
    }
}

fn main() {
    println!("{}", login("alice"));
    println!("{}", auth::password::hash("p@ss"));
    println!("{}", hash("via use")); // 短路徑
    let s = Session {
        user: "bob".into(),
    };
    println!("session user: {}", s.user);

    println!("math::double(5) = {}", math::double(5));
    // math::private_helper(5);  // ERROR: private

    println!("re-export: {}", public_login("carol"));
}
