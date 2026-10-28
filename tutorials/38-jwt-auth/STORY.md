# 故事版：遊樂園的蓋章手環 🎢

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「去遊樂園辦年卡、戴手環玩設施」的故事，幫你「感覺」出 argon2 密碼雜湊與 salt、JWT 的三段結構、claims 與 `exp`、`encode` / `decode` 與 `Validation`、`Authorization: Bearer`、auth middleware、把使用者放進 extensions、`route_layer`，以及 JWT 為什麼很難撤銷。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 三個櫃台

彩虹遊樂園有三個地方：

1. **辦卡櫃台**：第一次來的人在這裡辦年卡。
2. **入園櫃台**：已經有年卡的人在這裡報上暗號，領今天的手環。
3. **設施區**：雲霄飛車、旋轉木馬……要**戴手環**才能玩。

> 這就是 **整體流程**：`POST /register` → 雜湊密碼存起來、簽發 JWT；`POST /login` → 找使用者、驗密碼、簽發 JWT；`GET /me` → middleware 驗 token、把使用者資訊交給 handler。

---

## 櫃台不記你的暗號原文

小樂辦年卡時，櫃台姊姊請他設一個**暗號**：「p4ssw0rd」。

可是姊姊**不把暗號抄在本子上**。她先從抽屜拿出一小包**這張卡專屬的獨門香料**，跟暗號一起丟進一台**又慢又吃力的攪拌機**，攪出一團誰也還原不回去的**泥巴印**。本子上只記：「這包香料長什麼樣、攪拌機用幾檔、攪出來的泥巴印」。

就算小偷偷走本子，也**看不出暗號是什麼**。而且因為每個人的香料都不同，兩個人暗號一樣，泥巴印也不一樣。攪拌機故意做得很慢，小偷想一個一個猜，要猜到天荒地老。

如果有人用同一個信箱又來辦卡，姊姊會說：「**這個信箱已經辦過了**。」

> 這就是 **argon2 密碼雜湊**：`hash_password` 裡 `SaltString::generate(&mut OsRng)` 產生隨機 salt（獨門香料），`Argon2::default().hash_password(pw.as_bytes(), &salt)` 得到 PHC 字串，**裡面自帶 salt 與參數**，存進 `UserRow.password_hash`。不要用 MD5 / SHA256 存密碼——它們太快，暴力猜很容易。重複信箱回 `StatusCode::CONFLICT`（409）。

---

## 報暗號領手環

隔天小樂來入園櫃台：「我是 a@x.com，暗號是 p4ssw0rd。」

姊姊翻出他那一頁，用**同一包香料、同一檔攪拌機**把他剛說的暗號再攪一次，看看泥巴印**是不是一模一樣**。一樣，就發手環。

如果信箱找不到、或泥巴印不一樣，姊姊都只說同一句：「**資料不對喔**。」不會告訴你是信箱錯還是暗號錯，免得壞人慢慢試。

> 這就是 **`verify_password`**：`PasswordHash::new(phc)` 解析出 salt 和參數，`Argon2::default().verify_password(pw.as_bytes(), &parsed).is_ok()` 比對。`login` 裡找不到使用者和密碼錯誤都回 `StatusCode::UNAUTHORIZED`（401）。

---

## 手環上的三段

手環分成**三段**，中間用小點隔開：

1. **第一段**：「這是什麼手環、用哪一種印章蓋的」。
2. **第二段**：「戴的人是 **1 號、a@x.com**，**下午 3 點前有效**」。
3. **第三段**：一個**特殊印章**。這顆印章只有遊樂園**辦公室裡鎖著的那一顆**蓋得出來，而且是根據前兩段的內容蓋的——前面改了一個字，印章就對不上。

> 這就是 **JWT 結構**：`header.payload.signature`。`issue_token` 建立 `Claims { sub: id, email, exp }`，`exp` 是一小時後的 unix timestamp；`encode(&Header::default(), &claims, &s.encoding)` 用 `EncodingKey::from_secret(secret)` 簽章。`Header::default()` 是 **HS256**：簽和驗用同一把 secret，適合單體服務；多服務可以用 RS256 / ES256（私鑰簽、公鑰驗）。

---

## 手環是透明的

小樂把手環舉起來對著太陽看——咦，第二段**誰都看得到**！上面清清楚楚寫著他的編號和信箱。

姊姊說：「對呀，手環只是**防偽**，不是**保密**。所以我們絕對不會把你的暗號、零用錢卡號印在上面。」

> 這就是 **JWT payload 沒有加密**：它只是 base64 編碼，任何人都能解開來看。簽章保證「沒被改過」，不保證「看不到」。密碼、卡號等敏感資料絕對不能放進 claims。

---

## 設施入口查手環

小樂跑到雲霄飛車入口。管理員伯伯伸手：「**手環給我看一下**。」他會依序檢查：

1. 有沒有戴手環？沒有 → 不能玩。
2. 手環是不是寫著「手環：」這種格式開頭？不是 → 不能玩。
3. 印章對不對得上？有沒有**過期**？有人把「用哪種印章」改成「**不用印章**」想矇混？→ 全部擋下。

都沒問題，才讓小樂進去。

> 這就是 **auth middleware `require_auth`**：從 `header::AUTHORIZATION` 拿值，`.strip_prefix("Bearer ")` 取出 token，`decode::<Claims>(token, &s.decoding, &Validation::default())` 驗證簽章和 `exp`。任何一步失敗都回 401。`Validation::default()` 預設會驗 `exp`、也會拒絕 `alg: none`；別自己關掉這些檢查。不同機器時間差幾秒，可以設 `validation.leeway`。

---

## 排隊單上蓋個「已確認」

伯伯查完手環，在小樂的**排隊單**上蓋一個章：「**已確認：1 號，a@x.com**」。

雲霄飛車的駕駛員不用再查手環，只要看排隊單上的章，就知道坐上來的是誰。

> 這就是 **把使用者放進 extensions**：`req.extensions_mut().insert(AuthUser(data.claims.sub, data.claims.email))`，再 `next.run(req).await`。handler `me` 用 `req.extensions().get::<AuthUser>()` 取出來。更 idiomatic 的寫法是讓 `AuthUser` 實作 `FromRequestParts`，handler 直接寫 `async fn me(user: AuthUser)`；actix 對應的是 `FromRequest`。

---

## 只有設施區要查手環

辦卡櫃台和入園櫃台**當然不用手環**——不然沒手環的人永遠領不到手環！只有設施區的入口才站著管理員伯伯。

> 這就是 **`route_layer`**：`protected` 這個 `Router` 只有 `/me`，用 `.route_layer(middleware::from_fn_with_state(state.clone(), require_auth))` 套上 auth；再用 `.merge(protected)` 跟 `/register`、`/login` 合在一起。`from_fn_with_state` 讓 middleware 拿得到 `AppState` 裡的 `decoding` key。

---

## 手環發出去就收不回來

有一天，小樂的手環被別人撿走了。他跑去問：「可以讓那個手環**馬上失效**嗎？」

姊姊有點為難：「管理員只看**手環本身**對不對，不會打電話回辦公室問。所以手環發出去，**在過期前都算數**。」

遊樂園想了幾個辦法：

- 手環**只給 15 分鐘**，到期再拿另一張**長效的換手環券**來換新的。
- 在入口貼一張「**作廢手環號碼**」清單，管理員多看一眼。
- 最極端：**換一顆新印章**，所有舊手環一次全部作廢。

> 這就是 **JWT 撤銷困難**：驗證是離線的，伺服器不查資料庫。常見對策：**短 `exp` + refresh token**、用 `jti` 做**黑名單**（存 Redis 到過期為止）、**rotate secret**（所有人登出）。需要「立刻踢人」就改用 session。

---

## 印章要鎖好

最後，園長特別交代：印章要夠**複雜**（不能是一顆簡單的橡皮擦章），要**鎖在保險箱**，不能放在櫃台上；也不要跟客人的暗號本子放在一起，免得一次全被偷。

> 這就是 **secret 管理**：HS256 的 secret 至少 32 byte、隨機產生，從 env 或 secret manager 注入。範例的 `b"super-secret-do-not-leak"` 硬編在程式裡**只是示範**。JWT secret 和密碼資料要分開保管。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust 概念 |
|------|------|
| 辦卡櫃台 / 入園櫃台 / 設施區 | `/register` / `/login` / `/me` |
| 暗號 | 使用者密碼 |
| 每張卡專屬的香料 | **salt**（`SaltString::generate(&mut OsRng)`） |
| 又慢又吃力的攪拌機 | **argon2**（`Argon2::default()`） |
| 本子上的泥巴印 | PHC 字串 `password_hash` |
| 同一個信箱又來辦 | 409 `CONFLICT` |
| 再攪一次比對 | `verify_password` |
| 「資料不對喔」 | 401 `UNAUTHORIZED` |
| 三段手環 | **JWT** `header.payload.signature` |
| 第二段寫的資料 | `Claims { sub, email, exp }` |
| 下午 3 點前有效 | `exp` |
| 辦公室裡的印章 | secret → `EncodingKey` / `DecodingKey` |
| 蓋章 | `encode(&Header::default(), ...)`（HS256） |
| 手環是透明的 | payload 只是 **base64**，沒加密 |
| 管理員查手環 | `require_auth` middleware |
| 「手環：」開頭 | `Authorization: Bearer <token>` |
| 查印章、查過期 | `decode` + `Validation::default()` |
| 排隊單上的「已確認」章 | `req.extensions_mut().insert(AuthUser(...))` |
| 駕駛員看章 | `req.extensions().get::<AuthUser>()` |
| 只有設施區有管理員 | `route_layer` + `merge` |
| 短效手環 + 換手環券 | access token + **refresh token** |
| 作廢手環清單 | `jti` **黑名單** |
| 換一顆新印章 | **rotate secret** |

---

## 一句話記住

> **密碼要加香料慢慢攪（argon2 + salt），只存泥巴印；登入後發一條蓋了印章的透明手環（JWT），設施入口只查印章和期限——所以別在手環上寫祕密，也別讓手環活太久。**
