# 故事版：大廚房和外送便當盒 🍱

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「在大廚房做菜、裝進便當盒送出去」的故事，幫你「感覺」出 Docker image、多階段 build、cargo-chef 的依賴快取、distroless 與 scratch、靜態連結（musl）、nonroot、`.dockerignore`、healthcheck、docker-compose 和 image 安全是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 便當送到哪裡都一樣好吃

阿寶家開了一間便當店。以前客人說：「我想在**公司**吃你們的滷肉飯。」阿寶只好把整套鍋子、瓦斯爐搬過去現煮——結果公司的瓦斯爐火力不一樣，滷肉煮出來味道都怪怪的。

後來媽媽說：「我們在**自己的廚房**煮好，裝進一個**密封便當盒**送過去。不管送到哪裡，打開來都是一樣的味道。」

> 這就是 **Docker image 和 container**：把程式和它需要的執行環境**打包成一個 image**（便當盒），到哪台機器跑（打開吃）都一樣。本章的程式是一個最小的 axum server：`/` 回 `"Hello from inside Docker!"`，`/healthz` 回 `"ok"`。

---

## 第一次裝便當：把整間廚房塞進去

阿寶第一次裝便當，太緊張了：他把**菜、鍋子、砧板、菜刀、瓦斯爐、整袋麵粉**全部塞進一個超大的盒子。

客人打開嚇一跳：「我只要一盒滷肉飯，你怎麼送來一整間廚房？」盒子重到外送員搬不動。

> 這就是 **單階段 build 的問題**：直接用 `rust:1.83` 當最終 image，編譯器、原始碼、`target/` 全部留在裡面，image 肥到 **~1.4 GB**。

---

## 在廚房煮，只把菜裝進盒

媽媽教他分兩步：

1. **在大廚房煮菜**：這裡有所有工具，鍋碗瓢盆隨便用。
2. **只把煮好的菜**，夾進一個**乾淨的小便當盒**。

鍋子、菜刀都留在廚房。便當盒裡只有菜。

> 這就是 **多階段 build（multi-stage）**：本章 `Dockerfile` 的 `FROM rust:1.83-slim AS chef` / `builder` 是大廚房，`cargo build --release --bin ch46` 煮菜；`FROM gcr.io/distroless/cc-debian12 AS runtime` 是小便當盒，用 `COPY --from=builder /app/target/release/ch46 /app/ch46` 只夾出做好的那一個 binary。

---

## 先熬好的高湯

阿寶家的滷肉飯，最花時間的是**熬高湯**，要熬好幾個小時。可是高湯的配方幾乎**不會變**，常改的只有最後灑的蔥花和調味。

以前阿寶每改一次蔥花，就從頭熬一次高湯，客人等到天黑。

媽媽說：「先把**高湯的配方**抄在一張小卡上。只要小卡沒變，就直接用**冰箱裡昨天熬好的**；只有配方真的改了，才重熬。」

而且一定要**先**熬高湯、**後**放蔥花。如果一開始就把所有食材倒下去，蔥花一改，整鍋高湯都得重來。

> 這就是 **cargo-chef**：`planner` stage 跑 `cargo chef prepare --recipe-path recipe.json --bin ch46`，把依賴整理成 `recipe.json`（配方小卡）；`builder` stage 先 `COPY --from=planner /app/recipe.json` 再 `cargo chef cook --release`，**只編依賴**，這層可以被 Docker 快取；最後才 `COPY . .` 編你的程式碼。只有 `Cargo.toml` / `Cargo.lock` 變了才重編依賴。常見陷阱就是把 `COPY . .` 放在快取層之前。

---

## 什麼都沒附的便當盒

便當盒有好幾種：

- **大餐盒**：附餐具、附醬料、還有一把小刀。很方便，但很重，而且刀子可能被拿去做壞事。
- **極簡便當盒**：**沒有刀、沒有瓦斯爐**，只附了一小包「**基本醬油**」，因為滷肉飯一定要搭醬油才能吃。
- **一張保鮮膜**：什麼都沒有，連醬油都沒有。菜**必須自己已經是完整的一道**，連醬油都先拌進去才行。

> 這就是 **runtime base image 的選擇**：`debian:slim`（~80 MB，有完整環境好 debug）、`gcr.io/distroless/cc-debian12`（~20 MB，**沒有 shell、沒有 apt**，攻擊面小，但預裝 glibc 這包「醬油」）、`scratch`（0 MB，什麼都沒有）。本章用 distroless/cc，image 約 **40 MB**。

---

## 把醬油先拌進菜裡

阿寶想挑戰最輕的「**保鮮膜便當**」。他得用另一種煮法，把醬油**一起煮進菜裡**，這樣就不需要盒子附醬油了。

不過這種醬油跟平常那種**味道有點不同**：有些菜要換配方才好吃，炒得也比較慢一點，要另外換一把快鍋。

> 這就是 **靜態連結（musl）+ scratch**：`rustup target add x86_64-unknown-linux-musl`，`cargo build --release --target x86_64-unknown-linux-musl`，再 `FROM scratch` + `COPY` binary，約 **10 MB**。musl 和 glibc 有差：預設 allocator 較慢（可換 jemalloc / mimalloc）、DNS 解析方式不同、OpenSSL 相關 crate 常改用純 Rust 的 rustls。`scratch` 也沒有 CA 證書和 tzdata，要自己 COPY 進去。

---

## 外送員不拿萬用鑰匙

便當送到客人公司後，外送員要進大樓放便當。公司給他的是一張**訪客證**，只能進到茶水間，不是一把**能開所有門的萬用鑰匙**。

就算外送員被壞人騙了，壞人也只能進茶水間。

> 這就是 **`USER nonroot`**：distroless 內建 `nonroot` 使用者（uid 65532）。容器內絕對不該用 root 跑。另外 `EXPOSE 8080` 標示服務的埠，程式裡讀 `PORT` 環境變數，預設 8080，`SocketAddr::from(([0, 0, 0, 0], port))` 監聽所有介面。

---

## 不要把日記和鑰匙帶進廚房

有一次阿寶把**整個書包**倒在廚房桌上一起「準備」，裡面有他的**日記本**、家裡的**鑰匙**，還有一堆垃圾。差一點全被裝進便當盒裡送給客人。

媽媽在廚房門口貼了一張紙：「**這些東西不准帶進來**：日記、垃圾、鑰匙。」

> 這就是 **`.dockerignore`**：本章排除 `target/`、`.git/`、`.idea/`、`.vscode/` 等。實務上也要排除 `.env`；**不要把 secret 寫死在 image**，用環境變數、掛載或 secret manager。

---

## 「便當還好嗎？」

客人公司的總務每隔 30 秒就想確認一下：「那盒便當還好好的嗎？」

如果是大餐盒，總務可以拿盒子附的小刀戳一下看看。可是極簡便當盒**沒有附刀子**！

阿寶想到一個辦法：在便當蓋上做一個**小窗戶**，總務只要從窗戶看一眼，便當就自己回答「**我很好（ok）**」。或者乾脆交給大樓的管理員（下一章會講的大管家）負責巡邏。

> 這就是 **healthcheck**：Dockerfile 的 `HEALTHCHECK ... CMD wget --spider ...` 在 distroless 裡**不能用**，因為沒有 `wget` / `curl`。解法是讓 binary 自帶 `--healthcheck` flag，或交給 K8s 的 liveness probe 去打 `/healthz`。

---

## 野餐籃：便當加飲料一起送

公司要辦野餐，訂了**滷肉飯便當**，還要一桶**冬瓜茶**。阿寶準備了一個**野餐籃**清單：

- 便當：放這裡，貼標籤「需要冬瓜茶先到」。
- 冬瓜茶：用現成的那家牌子。

外送員照清單，一次把整個野餐籃送到。

> 這就是 **docker-compose**：`services` 底下 `app`（`build: .`、`ports: ["8080:8080"]`、`environment: DATABASE_URL`）和 `db`（`image: postgres:16`），`depends_on: [db]` 表示先啟動 db。一個指令就把整組服務帶起來。

---

## 食品安全檢查和封條

便當店越做越大，媽媽訂了四條規矩：

1. **每批便當盒都送檢驗**，看有沒有發霉的材料。
2. **便當盒認明批號**，不能說「隨便哪批都好」。
3. **出貨前貼封條**，客人看到封條完整，就知道沒被人掉包。
4. **在廚房用跟便當盒一樣的醬油**，不然煮的時候好吃，送到才發現味道不對。

> 這就是 **image 安全與一致性**：用 `trivy image` 或 `docker scout` 掃 CVE；`FROM rust:1.83-slim@sha256:...` **pin digest**；用 cosign + sigstore **簽署 image**；builder 和 runtime 的 glibc 版本要一致（都用 Debian 12 系列），否則「build 得起來卻跑不動」。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust / Docker 概念 |
|------|------|
| 密封便當盒 | **Docker image** |
| 打開便當吃 | **container** 執行 |
| 把整間廚房塞進盒子 | 單階段 build，~1.4 GB |
| 在廚房煮、只把菜裝盒 | **多階段 build**：`COPY --from=builder` |
| 高湯配方小卡 | **`recipe.json`**（`cargo chef prepare`） |
| 冰箱裡昨天熬好的高湯 | **`cargo chef cook`** 的依賴快取層 |
| 先熬高湯、後放蔥花 | 先 build 依賴，最後才 `COPY . .` |
| 沒刀、只附醬油的便當盒 | **`distroless/cc-debian12`**（無 shell、有 glibc） |
| 一張保鮮膜 | **`scratch`** |
| 醬油先拌進菜裡 | **靜態連結**：`x86_64-unknown-linux-musl` |
| 不一樣的醬油味 | **musl vs glibc** 差異 |
| 訪客證，不是萬用鑰匙 | **`USER nonroot`** |
| 廚房門口的禁帶清單 | **`.dockerignore`** |
| 便當蓋上的小窗戶 | 程式自己的 **`/healthz`** / `--healthcheck` |
| 野餐籃清單 | **docker-compose**（`depends_on`） |
| 食品檢驗 | **CVE 掃描**：`trivy` / `docker scout` |
| 便當盒批號 | **pin base image digest** |
| 封條 | **cosign 簽署** |

---

## 一句話記住

> **在大廚房煮好、只把菜夾進小便當盒（多階段 build），高湯配方沒變就用冰箱裡的（cargo-chef 快取），便當盒越極簡越安全（distroless / scratch），外送員只拿訪客證（nonroot）——送到哪裡打開都是同一個味道。**
