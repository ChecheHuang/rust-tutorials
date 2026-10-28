# 故事版：圖書館的尋句小隊 🔍

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「圖書館要在幾萬本書裡，找出每一句含有某個詞的句子」的故事，幫你「感覺」出 `mini-rg` 這整個專案是怎麼運作的——任務卡怎麼看、誰負責走書架、誰負責翻書、誰負責唸出結果、大家怎麼知道可以收工，以及怎麼確認它真的沒壞。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 一張任務卡

林老師是圖書館的館長。有一天，小朋友交來一張**任務卡**：

> 「請幫我找出所有寫著 **fn main** 的句子，從**這一區**開始找。」

任務卡上還有好幾個可以勾的小格子：

- □ 大小寫不用分（Hello 和 hello 都算）
- □ 照字面找，不要把符號當成特殊暗號
- □ 連「員工專用」的隱藏書架也要找
- □ 只找某一類書（例如只找書名結尾是 `.rs` 的）
- □ 要派幾位小書僮
- □ 不要用彩色筆
- □ 只要告訴我**哪幾本書**有，不用抄句子

如果小朋友沒寫從哪一區開始，林老師就從**自己站的這一區**開始。

> 這就是 **`src/args.rs`**：`#[derive(Parser)] struct Args` 用 **clap derive** 定義參數：`pattern`、`paths`（`#[arg(default_value = ".")]`）、`-i` / `--ignore-case`、`-F` / `--fixed-strings`、`--hidden`、`-g` / `--glob`（可多次）、`-j` / `--threads`、`--no-color`、`-l` / `--files-with-matches`。每個欄位上的 `///` 註解會變成 `--help` 的說明。`main.rs` 一開頭就是 `Args::parse()`。

---

## 先把「要找什麼」做成一把放大鏡

林老師沒有讓每個小書僮自己去理解任務卡。她先把「要找的詞」做成一把**特製放大鏡**：

- 勾了「照字面找」，她會先把詞裡的特殊符號**包起來**，讓放大鏡只認字面。
- 勾了「大小寫不分」，放大鏡就**不管大小寫**。

萬一小朋友寫的暗號根本是亂碼，放大鏡做不出來，林老師**當場就把任務卡退回去**，說清楚哪裡寫錯，而不是讓小書僮們白跑一趟。

做好的放大鏡會**複製很多把**，每個小書僮手上都有一把一模一樣的。

> 這就是 **`SearchOpts::from_args`**（`src/search.rs`）：`fixed_strings` 時先 `regex::escape(&args.pattern)`，再用 `RegexBuilder::new(&pat).case_insensitive(args.ignore_case).build()?`。regex 編譯失敗會透過 `?` 一路傳回 `main`，由 `anyhow::Result` 印出錯誤。`SearchOpts` 是 `#[derive(Clone)]`，每個 worker 拿一份 `opts.clone()`。

---

## 一台推車，最多放一千本

林老師找了三種角色：

- **帶路員**（只有一位）：負責走遍書架，把要翻的書一本一本放上**推車**。
- **小書僮**（好幾位）：從推車上拿書來翻。
- **唸稿員**（只有一位）：負責把找到的句子唸給小朋友聽。

推車**最多只能放一千本書**。如果小書僮翻得慢，推車滿了，帶路員就得**停下來等**，不會把書堆到滿地都是。

唸稿員那邊也有一台一樣大小的推車，小書僮把「找到了什麼」寫成紙條放上去。

> 這就是 **`main.rs` 的整體編排**：用 `crossbeam_channel::bounded::<PathBuf>(1024)` 做「書的推車」（`job_tx` / `job_rx`），再用 `bounded::<printer::Output>(1024)` 做「紙條推車」（`out_tx` / `out_rx`）。**bounded** 代表滿了 `send` 會阻塞，形成自然的背壓。走書架、翻書、輸出三件事解耦，各做各的。

---

## 小書僮有幾位？

任務卡上如果寫了「派 4 位」，就派 4 位；沒寫的話，林老師看看今天圖書館有幾張桌子，**有幾張桌子就派幾位**。就算寫了「0 位」，她也至少派 1 位。

每位小書僮都拿著同一台推車的把手（大家**共用一台推車**，誰空手誰就拿下一本），還有一張通往唸稿員推車的通行證。

> 這就是 **worker threads**：`let n = args.threads.unwrap_or_else(num_cpus::get).max(1);`，然後 `thread::spawn` n 次。每個 worker 拿到 `job_rx.clone()` 和 `out_tx.clone()`——crossbeam 的 receiver 可以 clone，多個 worker 一起搶同一個佇列。迴圈是 `while let Ok(path) = job_rx.recv()`。

---

## 帶路員走書架，跳過「請勿翻閱」

帶路員從任務卡上寫的那幾區開始，**一排一排走**。

- 書架上貼著「**請勿翻閱**」清單的（例如倉庫、草稿區），他就跳過。
- 「**員工專用**」的隱藏書架，除非任務卡勾了，不然也跳過。
- 任務卡寫了「只找 `.rs` 書」，他就只拿那一類；寫了「`!target/*`」就把那區排除。
- 走到的如果是**一整個書架**而不是一本書，他就繼續往裡面走，只把**真正的書**放上推車。

> 這就是 **`src/walker.rs`**：用 `ignore` crate 的 `WalkBuilder::new(first)`，其他路徑用 `builder.add(p)` 加入；`.hidden(!args.hidden).git_ignore(true).git_exclude(true)` 讓它尊重 `.gitignore`。`--glob` 透過 `OverrideBuilder` 轉成 overrides。走訪結果 `walker.flatten()` 後只把 `p.is_file()` 的路徑交給 `sink` 閉包，而 `main` 傳進去的閉包就是 `job_tx.send(p)`。

---

## 翻書：先瞄一眼，是不是圖畫書

小書僮拿到一本書，第一件事是**先翻開最前面幾頁瞄一下**。

如果裡面出現了**一種文字書絕對不會有的空白記號**，那這本就是圖畫書或樂譜，不是文字書，直接放回去，**也不用跟唸稿員說**。

如果是文字書，就**一行一行**往下讀，用放大鏡照每一行：

- 照到了，就記下「**第幾行、從第幾個字到第幾個字**」，連同這一行的內容寫在紙條上。
- 有些字印得模糊、看不太清楚，小書僮不會因此卡住，而是**照著能看懂的樣子抄下來**，看不懂的字畫一個問號。
- 如果任務卡勾了「只要書名」，照到**第一行**就可以停了，不用再讀下去。

這本書如果打不開（例如被鎖住了），小書僮就寫一張「**這本打不開**」的紙條。

> 這就是 **`search::search_path`**：`BufReader::with_capacity(64 * 1024, f)`，先 `fill_buf()` 檢查前 8192 bytes 是否含 `0`（NUL），是就回 `Output::skipped(path)`。接著 `read_until(b'\n', &mut buf)` 逐行讀**原始 bytes**，用 `regex::bytes::Regex` 的 `find(line)` 比對，避免非 UTF-8 檔案直接失敗；顯示時才 `String::from_utf8_lossy`。命中存成 `Match { line_no, start, end, content }`；`files_with_matches` 時 `break`。錯誤時 worker 送出 `Output::error(path, e.to_string())`。

---

## 三種紙條

唸稿員的推車上只會出現三種紙條：

1. 「**這本書有找到**」：附上書名和一串句子。
2. 「**這本是圖畫書，跳過了**」。
3. 「**這本打不開**」：附上原因。

> 這就是 **`src/printer.rs` 的 `enum Output`**：`File { path, matches, files_only }`、`Skipped(PathBuf)`、`Error { path, msg }`，各有一個建構函式 `Output::file` / `Output::skipped` / `Output::error`。用 enum 讓每種結果都有明確的形狀，`match` 時一個都不會漏。

---

## 唸稿員只有一支麥克風

唸稿員手上只有**一支麥克風**。因為只有他在唸，兩本書的句子**不會互相插嘴、混在一起**。

他唸的方式很固定：

- 先用**紫色**唸書名。
- 每一句前面用**綠色**唸行號。
- 句子裡被放大鏡照到的那幾個字，用**紅色、加粗**唸出來，其他照常。
- 唸完一本，空一行。

「圖畫書」的紙條他直接丟掉不唸；「打不開」的紙條，他會走到**旁邊另一支專門報錯的麥克風**去講，不跟正常結果混在一起。

還有一件事：如果小朋友不是站在台下聽，而是**拿錄音機錄下來**要給別人轉錄，唸稿員就**不用顏色**，免得錄音裡出現一堆奇怪的「換顏色」雜音。任務卡勾了「不要彩色筆」也一樣。

> 這就是 **`Printer`**：`out: Mutex<StandardStream>`，透過 `Arc<Printer>` 交給專屬的印 thread（`for out in out_rx { printer_clone.print(&out) }`）。`Printer::new` 在 `no_color || !atty::is(atty::Stream::Stdout)` 時用 `ColorChoice::Never`——**輸出被導到管線或檔案時自動關顏色**。`print` 用 `termcolor` 的 `set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))` 等上色，match 片段用 `split_at` 切成 `pre / mid / rest`。錯誤走 `eprintln!` 到 **stderr**。

---

## 小朋友聽到一半就走了

有時候小朋友只想聽**前十句**，聽完就走掉了。

唸稿員對著空蕩蕩的台下繼續講，麥克風沒人聽——他**不會因此昏倒**，只是安靜地把剩下的紙條處理完。

> 這就是 **broken pipe 不 panic**：`mini-rg foo | head` 時 stdout 會被提早關閉。`Printer::print` 裡所有的 `write!` / `writeln!` 都是 `let _ = ...`，忽略寫入錯誤，而不是用會 panic 的 `println!`。

---

## 大家怎麼知道可以收工？

收工的順序很重要：

1. 林老師先把**自己那張**通往唸稿員推車的通行證**撕掉**——她自己不會放紙條，只有小書僮們會。
2. 帶路員走完所有書架，**放開書推車的把手**，宣布「不會再有新書了」。
3. 小書僮們把推車上剩下的書翻完，發現推車空了、**而且再也不會有人放書**，就各自下班，順手撕掉自己的通行證。
4. 等**最後一張**通行證被撕掉，唸稿員就知道「不會再有紙條了」，唸完最後一張也下班。
5. 林老師等所有人都下班，才關燈鎖門。

如果第 1 步林老師忘了撕自己的通行證，唸稿員就會**一直等**她那張永遠不會來的紙條，大家都回不了家。

> 這就是 **用 drop 關閉 channel**：`drop(out_tx)` 放在 spawn 完 worker 之後，因為只要還有任何一個 `Sender` 活著，`out_rx` 的迭代就不會結束。`walker::walk(...)` 結束後 `drop(job_tx)`，worker 的 `job_rx.recv()` 回傳 `Err` 跳出迴圈，worker 結束時它的 `out_tx` 也被 drop。最後 `for w in workers { w.join() }`、`printer_thread.join()`，`main` 回傳 `Ok(())`。

---

## 驗收：在空房間裡排練

林老師每次改了做法，都會先到一間**空房間**排練：在裡面擺幾本特別準備的書，再請尋句小隊照任務卡找一遍，看結果對不對。

- 放一本寫著「foo bar」的書，找「foo」→ 唸出來的要有「foo bar」。
- 放一本「Hello World」，勾「大小寫不分」找「hello」→ 要找得到。
- 放 `a.rs` 和 `a.txt`，只找 `.rs` 書 → 只能聽到 `a.rs`，**不能**聽到 `a.txt`。
- 放兩本書，勾「只要書名」找「foo」→ 只能聽到有 foo 的那一本。

排練完，空房間**自動清乾淨**，不會留下垃圾。

> 這就是 **`tests/integration.rs`**：`tempfile::tempdir()` 建臨時目錄（離開 scope 自動刪除），`assert_cmd::Command::cargo_bin("mini-rg")` 真的去跑編好的 binary，再用 `predicates` 的 `predicate::str::contains(...)` 和 `.not()` 檢查 stdout。四個測試：`finds_match_in_file`、`case_insensitive`、`glob_filter`、`files_with_matches`。

---

## 正式開館前，把書僮訓練到最快

正式開館版本，林老師會花比較多時間**集訓**：把所有小書僮的動作**一起看過、一起優化**，訓練手冊裡用不到的註解也撕掉，讓大家身上帶的東西更輕。訓練要花比較久，但之後每次找句子都更快。

> 這就是 **`Cargo.toml` 的 `[profile.release]`**：`lto = "thin"`（跨 crate 優化）、`codegen-units = 1`（整體一起優化，編譯較慢但執行較快）、`strip = "debuginfo"`（拿掉除錯資訊縮小 binary）。`cargo install --path .` 就能裝成全域指令。

---

## 回到現實：故事 ↔ 程式碼對照表

| 故事裡的東西 | 真正的程式概念 | 在哪個檔案 |
|------|------|------|
| 任務卡和小格子 | **clap derive** 的 `Args` | `src/args.rs` |
| 沒寫就從這一區開始 | `#[arg(default_value = ".")]` | `src/args.rs` |
| 特製放大鏡 | `SearchOpts` + `regex::bytes::Regex` | `src/search.rs` |
| 照字面找就把符號包起來 | `regex::escape` | `src/search.rs` |
| 暗號亂碼當場退件 | `build()?` → `anyhow::Result` 傳回 `main` | `src/search.rs`、`src/main.rs` |
| 一台最多一千本的推車 | `crossbeam_channel::bounded(1024)` | `src/main.rs` |
| 有幾張桌子派幾位 | `args.threads.unwrap_or_else(num_cpus::get).max(1)` | `src/main.rs` |
| 大家共用一台推車 | 多個 worker `job_rx.clone()` 搶同一佇列 | `src/main.rs` |
| 帶路員走書架 | `ignore::WalkBuilder` | `src/walker.rs` |
| 「請勿翻閱」清單 | `.git_ignore(true)` / `.git_exclude(true)` | `src/walker.rs` |
| 員工專用書架 | `.hidden(!args.hidden)` | `src/walker.rs` |
| 只找 `.rs` 書 | `OverrideBuilder` + `--glob` | `src/walker.rs` |
| 瞄一眼是不是圖畫書 | 前 8192 bytes 含 NUL → `Output::skipped` | `src/search.rs` |
| 一行一行讀 | `read_until(b'\n', ...)` | `src/search.rs` |
| 模糊的字畫問號 | `String::from_utf8_lossy` | `src/search.rs` |
| 第幾行、第幾字到第幾字 | `Match { line_no, start, end, content }` | `src/printer.rs` |
| 三種紙條 | `enum Output { File, Skipped, Error }` | `src/printer.rs` |
| 唯一的麥克風 | `Mutex<StandardStream>` + 專屬印 thread | `src/printer.rs`、`src/main.rs` |
| 紫色書名、綠色行號、紅色關鍵字 | `termcolor` 的 `ColorSpec` | `src/printer.rs` |
| 被錄音就不用顏色 | `!atty::is(Stdout)` → `ColorChoice::Never` | `src/printer.rs` |
| 報錯的另一支麥克風 | `eprintln!` 到 stderr | `src/printer.rs` |
| 聽到一半走掉也不昏倒 | `let _ = writeln!(...)`，broken pipe 不 panic | `src/printer.rs` |
| 撕通行證、放開把手 | `drop(out_tx)`、`drop(job_tx)` 關閉 channel | `src/main.rs` |
| 等大家下班才鎖門 | `join()` 所有 thread | `src/main.rs` |
| 空房間排練 | `assert_cmd` + `tempfile` 整合測試 | `tests/integration.rs` |
| 正式開館前集訓 | `[profile.release]`：`lto`、`codegen-units`、`strip` | `Cargo.toml` |

---

## 一句話記住

> **任務卡先做成一把放大鏡，帶路員走書架、跳過請勿翻閱的區域，把書放上有限大小的推車；小書僮們搶著翻、跳過圖畫書，唸稿員用唯一的麥克風上色唸出來——大家撕掉通行證，就知道可以一起收工。**
