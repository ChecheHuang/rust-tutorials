# 故事版：披薩店的加料單與出餐模式 🍕

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「披薩店依照加料單做披薩、試做和正式出餐用不同做法」的故事，幫你「感覺」出 features、`#[cfg(...)]`、optional dependency、profiles（dev / release、`opt-level`、`lto`、`strip`、`panic`）、`build.rs` 與 `env!`、`Cargo.toml` 與 `Cargo.lock`、`[workspace.dependencies]`、workspace lints、常用 cargo 子命令、`cargo-make`。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 加料單

阿傑開了一間披薩店，妹妹小晴放學後來幫忙。

每張訂單上都有一格**加料單**。店裡規定：

- 「**招牌起司**」是**預設**的，客人沒特別說就會加。
- 「**額外配料**」要客人自己勾才加。
- 客人也可以說「**預設的全部不要**」，只做最基本的餅皮。
- 或是「**全部都加**」。

> 這就是 **features**：本章 `Cargo.toml` 寫 `[features] default = ["fancy"]`、`fancy = []`、`extra = []`。`cargo run` 用預設（`fancy`），`cargo run --no-default-features` 關掉預設，`cargo run --features extra` 加上 `extra`，`cargo build --all-features` 全開。

---

## 廚房看單做事

廚房的師傅做披薩時，會**先看加料單**：

- 有勾招牌起司 → 喊「起司加上去了！」
- 沒勾 → 喊「這張不加起司！」
- 有勾額外配料 → 再多喊一句。

**沒勾的配料，師傅連拿都不會拿出來**，那一步根本不存在於這張披薩上。

> 這就是 **`#[cfg(feature = "...")]`**：`main.rs` 裡 `#[cfg(feature = "fancy")] println!("fancy feature is ON");`、`#[cfg(not(feature = "fancy"))]`、`#[cfg(feature = "extra")]`。條件不成立的程式碼**在編譯期就被拿掉**，不會出現在 binary 裡。

---

## 加料只能「加」

有一位客人說：「我勾了『**清淡版**』，所以請把起司**拿掉**。」

阿傑搖頭：「我們的加料單**只能加東西，不能拿東西**。兩個人一起點同一個披薩，一個勾 A、一個勾 B，我們就兩樣都加。要是有一格是『拿掉起司』，另一格是『要起司』，廚房就吵起來了。」

有些昂貴的食材，像松露，平常**不進貨**；只有客人勾了「松露口味」，才順便叫貨。

> 這就是 **features 要 additive**：feature 只能**增加**功能，不能改變或移除既有 API，也不適合做「A 或 B 二選一」。昂貴食材是 **optional dependency**：本章 `serde = { workspace = true, optional = true }`，在 feature 裡寫 `gpu = ["dep:cuda"]` 就是「勾了才把這個依賴拉進來」。

---

## 店在哪裡、今天是不是試做

阿傑的店有好幾家分店，每家的烤箱不太一樣：A 店用電烤箱、B 店用石窯。食譜裡寫著「**如果是石窯，就這樣做**」。

另外，食譜也會看「**今天是試做還是正式出餐**」：試做時會多量一次溫度、多做一些檢查。

> 這就是 **cfg 條件**：`#[cfg(target_os = "windows")]` / `"linux"` / `"macos"` 依平台編譯不同程式碼；`#[cfg(debug_assertions)]` 在 dev build 成立，release 不成立，所以 main 會印 `DEBUG build` 或 `RELEASE build`。

---

## 試做模式 vs 正式出餐模式

平常研究新口味時，阿傑用**試做模式**：隨便烤、快速出爐，改了就能馬上再烤一次，順便在每塊披薩上**插小旗子**寫「這是第幾步做的」，方便找問題。

正式出餐用**出餐模式**：慢慢烤、整條生產線一起調整到最好、把小旗子**拔掉**讓盒子變輕。出問題時也不慢慢收拾，**直接關掉那台烤箱**。

還有一種「**出餐但保留小旗子**」的模式，從出餐模式改一點點就好。

> 這就是 **profiles**：`cargo run` / `cargo test` 用 `[profile.dev]`（`opt-level = 0`、`debug = true`、`incremental = true`）；`--release` 用 `[profile.release]`。`opt-level = 3`、`lto = true`（跨 crate 優化）、`codegen-units = 1`、`strip = true`（拔小旗子）、`panic = "abort"`（直接關烤箱，不 unwind）。自訂 profile 用 `inherits = "release"` 再加 `debug = true`。代價是編譯變慢。

---

## 開店前的準備時間，和印在盒子上的字

每天開店前，有一段**準備時間**：照著醬料配方先熬好醬、在披薩盒上**印好今天的日期和分店名**。這段準備是**在做任何披薩之前**就完成的。

披薩做好後，盒子上已經印著店名和菜單版本，客人打開就看得到，**不用再去問店員**。

> 這就是 **`build.rs`** 與 **`env!`**：crate root 的 `build.rs` 在編譯前執行，用來 codegen、編 C 程式碼、注入資訊，例如 `println!("cargo:rustc-env=GIT_HASH=...")`、`println!("cargo:rerun-if-changed=schema.sql")`。程式裡用 `env!("...")` 在**編譯期**讀進來；本章用 Cargo 內建的 `env!("CARGO_PKG_NAME")`、`env!("CARGO_PKG_VERSION")`。注意 build.rs 每次都要跑，寫太重會拖慢編譯。

---

## 進貨單和確切批號

阿傑的進貨單上寫：「**起司：1 號系列都可以**」。這是一個**範圍**。

但第一次進貨時，倉庫會在另一本**批號簿**上記下「**這次拿的是 1.4.2 批號、包裝上的防偽碼是多少**」。之後誰來照著做，都拿一模一樣的那批。

自己開店的人，會把批號簿**收好一起保管**；可是如果你是**賣醬料給別人**的，就別把批號簿塞給客人，讓客人用他們自己的批號簿。

> 這就是 **`Cargo.toml` vs `Cargo.lock`**：`Cargo.toml` 寫 SemVer 範圍（`"1"` = `>=1.0, <2.0`），`Cargo.lock` 鎖住具體版本與 hash。binary 要 commit `Cargo.lock` 保證可重現；library 通常不 commit。`cargo update` 在範圍內更新，`cargo update -p tokio --precise 1.35` 釘住特定版本。

---

## 連鎖店統一進貨、統一店規

阿傑後來開了好幾家連鎖店。總店發了一張**統一進貨單**：「每家店的起司都用同一個牌子同一系列。」分店只要寫「起司：**照總店**」。

不然 A 店一種起司、B 店另一種起司，倉庫就要存**兩套**。總店也發了一份**統一店規**：「每家店都要洗手、圍裙要乾淨」，分店寫一句「**照總店店規**」就好。

> 這就是 **`[workspace.dependencies]`** 與 **workspace lints**：root 寫 `tokio = { version = "1", features = ["full"] }`，member 寫 `tokio.workspace = true`，避免 dependency tree 裡有多份 tokio。lint 也一樣：root 寫 `[workspace.lints.clippy] pedantic = "warn"`，member 寫 `[lints] workspace = true`。

---

## 盤點工具和打烊清單

小晴幫忙**盤點**：

- 畫一張「每種食材從哪裡來」的**來源樹**，順便找出**同一種食材買了兩個牌子**的地方。
- 查查看有沒有食材被**公告有問題**。
- 看看哪一種食材**佔最多空間**。

每天打烊，還有一張**打烊清單**：「擦桌子 → 檢查瓦斯 → 鎖門」。只要喊一聲「**打烊！**」，就照清單依序做完，在哪一家分店都一樣。

> 這就是 **cargo 子命令** 與 **`cargo-make`**：`cargo tree`、`cargo tree --duplicates`、`cargo audit`（已知 CVE）、`cargo deny check`、`cargo bloat`、`cargo install`。`Makefile.toml` 寫 `[tasks.ci] dependencies = ["fmt-check", "clippy", "test"]`，跑 `cargo make ci` 就依序執行，跨平台；也可以用 `just`。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 加料單 | **`[features]`** |
| 預設的招牌起司 | `default = ["fancy"]` |
| 客人自己勾的額外配料 | `extra`、`--features extra` |
| 預設的全部不要 | `--no-default-features` |
| 師傅看單才拿配料 | **`#[cfg(feature = "...")]`** |
| 加料只能加，不能拿掉 | features 必須 **additive** |
| 勾了才叫貨的松露 | **optional dependency**、`dep:` |
| 電烤箱 vs 石窯 | `#[cfg(target_os = "...")]` |
| 今天是不是試做 | `#[cfg(debug_assertions)]` |
| 試做模式 | **`[profile.dev]`** |
| 正式出餐模式 | **`[profile.release]`** |
| 整條生產線一起調 | `lto`、`codegen-units = 1` |
| 拔掉小旗子 | `strip`、`debug = false` |
| 直接關掉烤箱 | `panic = "abort"` |
| 出餐但保留小旗子 | `inherits = "release"` + `debug = true` |
| 開店前的準備時間 | **`build.rs`** |
| 印在盒子上的店名、版本 | **`env!("CARGO_PKG_NAME")`**、`cargo:rustc-env` |
| 進貨單上的範圍 | `Cargo.toml` 的 SemVer 範圍 |
| 批號簿 | **`Cargo.lock`** |
| 統一進貨單 | **`[workspace.dependencies]`** |
| 統一店規 | `[workspace.lints]` + `[lints] workspace = true` |
| 盤點工具 | `cargo tree`、`cargo audit`、`cargo bloat` |
| 打烊清單 | **`cargo-make`** 的 `Makefile.toml` |

---

## 一句話記住

> **features 是只能「加料」的加料單（`#[cfg(feature)]` 沒勾就不做），profile 是試做與正式出餐兩種做法，`build.rs` 是開店前的準備時間，`Cargo.lock` 是記下確切批號的本子，連鎖店就用 `[workspace.dependencies]` 統一進貨。**
