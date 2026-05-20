# 38. JWT 認證

> 範圍：argon2 密碼雜湊、jsonwebtoken 簽發/驗證、auth middleware、extractor pattern

## 流程總覽

```
register → 雜湊密碼存 DB → 簽發 JWT
login    → 取 user → verify 密碼 → 簽發 JWT
protected route → middleware 驗 token → 注入 user → handler 取
```

## 密碼雜湊：argon2

**不要用** MD5、SHA1、SHA256 等通用 hash 存密碼，**也不要用** bcrypt（已被 argon2 取代）：

```rust
let salt = SaltString::generate(&mut OsRng);
let phc = Argon2::default()
    .hash_password(pw.as_bytes(), &salt)?
    .to_string();
// PHC 字串自帶 salt 與參數
```

驗證：

```rust
let parsed = PasswordHash::new(phc)?;
Argon2::default().verify_password(pw.as_bytes(), &parsed).is_ok()
```

實務參數調整：memory cost、iterations。argon2 預設值對伺服器來說可能太低，可調 `Params::new(memory_kib, time_cost, parallelism, output_len)`。

## JWT 結構

```
header . payload . signature
{alg, typ} . {sub, exp, ...} . HMAC(...)
```

選 algorithm：
- **HS256**：對稱（伺服器一個 secret）— 適合單體
- **RS256 / ES256**：非對稱（私鑰簽、公鑰驗）— 適合多服務（一個 issuer、N 個 verifier）

## Claims 必備欄位

| 欄位 | 含義 |
|------|------|
| `sub` | subject — 通常 user id |
| `exp` | expiration — unix timestamp，過期就失效 |
| `iat` | issued-at |
| `iss` | issuer |
| `aud` | audience |
| `jti` | token id（用於 blacklist） |

驗證：`jsonwebtoken::Validation` 預設驗 exp，可以再加 `iss` / `aud`。

## Token 放哪裡

| 方式 | 優點 | 缺點 |
|------|------|------|
| `Authorization: Bearer <token>` header | 簡單、跨 origin | JS 看得到（XSS 風險） |
| HttpOnly Cookie | JS 拿不到 | 要處理 CSRF |
| 兩段式（access + refresh） | access 短命（5–15 分鐘）+ refresh 長命 | 較複雜 |

實務：API 用 Bearer header、瀏覽器 app 用 HttpOnly + SameSite=Strict cookie。

## 範例的設計

- `POST /register` → 雜湊密碼存 in-memory、回 JWT
- `POST /login` → verify 密碼、回 JWT
- `GET /me` → 中介層驗 JWT、注入 `AuthUser` 進 extensions、handler 取

## axum 中的兩種風格

### 1. middleware + Extensions（本範例）

```rust
req.extensions_mut().insert(AuthUser(...));
// handler：
async fn me(req: Request) -> ... {
    let u = req.extensions().get::<AuthUser>().unwrap();
}
```

### 2. 自訂 extractor（更 idiomatic）

```rust
struct AuthUser { id: u64, email: String }

impl<S> FromRequestParts<S> for AuthUser
where S: Send + Sync + DecodingKeyProvider {
    async fn from_request_parts(parts, state) -> Result<Self, StatusCode> {
        let token = parts.headers.get(...)
        // ... decode ...
        Ok(AuthUser { ... })
    }
}

async fn me(user: AuthUser) -> Json<...> { ... }
```

Handler 簽名直接寫 `AuthUser`——無 token 自動 401。

## actix-web 對照

```rust
struct AuthUser(u64);

impl FromRequest for AuthUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future { ... }
}

async fn me(user: AuthUser) -> impl Responder { ... }
```

概念一致，trait 名不同。

## 撤銷與 refresh

JWT **本質上撤銷困難**（驗證離線）。常見做法：
1. **短 expiry + refresh token**：access 15 min，refresh 7 day。
2. **黑名單**：把 `jti` 存 Redis 直到 exp 過期。
3. **rotate secret**：登出所有人（極端）。

純無狀態 JWT 不適合「立即踢人」需求；要的話用 session（Redis 存 session id）。

## 常見陷阱

1. **secret 太短** — HS256 至少 256-bit（32 byte）。隨機產生、env 注入。
2. **alg: none** — 古典攻擊；`Validation` 預設拒絕，別自己 `Validation::insecure()`。
3. **exp 沒驗** — 用 `Validation::default()` 預設驗，自訂時別漏。
4. **clock skew** — 不同機器時間差幾秒會把剛簽的 token 判 invalid；可以 `validation.leeway = 30`。
5. **password 跟 JWT secret 混在同個變數** — 分開來，避免一個洩漏全失守。
6. **把 sensitive info 放 payload** — JWT payload **未加密**，只是 base64。密碼、卡號絕對不能放。

## 練習

1. 加 refresh token endpoint：access 短命、refresh 長命；refresh 用過丟掉（rotation）。
2. 把 in-memory user 換成 sqlx + Postgres。
3. 加 role-based access：claims 多一個 `roles: Vec<String>`，middleware 檢查。
4. 寫一個自訂 `AuthUser` extractor，比較跟 middleware + Extensions 寫法。

## 延伸閱讀

- [jsonwebtoken](https://docs.rs/jsonwebtoken)
- [argon2](https://docs.rs/argon2)
- [OWASP — JWT cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html)
