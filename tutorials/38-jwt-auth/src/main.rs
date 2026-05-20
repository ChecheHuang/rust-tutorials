use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

// ── State ───────────────────────────────────────────────────
#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<Vec<UserRow>>>,
    encoding: EncodingKey,
    decoding: DecodingKey,
}

#[derive(Clone, Debug)]
struct UserRow {
    id: u64,
    email: String,
    password_hash: String, // argon2 PHC string
}

// ── JWT claims ──────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: u64,          // user id
    email: String,
    exp: usize,        // unix timestamp
}

// ── DTO ─────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
struct Register {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct TokenResp {
    access_token: String,
    token_type: &'static str,
}

// ── 密碼雜湊 ────────────────────────────────────────────────
fn hash_password(pw: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(pw.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .to_string();
    Ok(hash)
}

fn verify_password(pw: &str, phc: &str) -> bool {
    let parsed = match PasswordHash::new(phc) {
        Ok(p) => p,
        Err(_) => return false,
    };
    Argon2::default().verify_password(pw.as_bytes(), &parsed).is_ok()
}

// ── handlers ────────────────────────────────────────────────
async fn register(
    State(s): State<AppState>,
    Json(req): Json<Register>,
) -> Result<Json<TokenResp>, StatusCode> {
    let hash = hash_password(&req.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut users = s.users.lock().await;
    if users.iter().any(|u| u.email == req.email) {
        return Err(StatusCode::CONFLICT);
    }
    let id = users.len() as u64 + 1;
    users.push(UserRow {
        id,
        email: req.email.clone(),
        password_hash: hash,
    });
    drop(users);
    Ok(Json(issue_token(&s, id, &req.email)))
}

async fn login(
    State(s): State<AppState>,
    Json(req): Json<Register>,
) -> Result<Json<TokenResp>, StatusCode> {
    let users = s.users.lock().await;
    let user = users
        .iter()
        .find(|u| u.email == req.email)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();
    drop(users);

    if !verify_password(&req.password, &user.password_hash) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(issue_token(&s, user.id, &user.email)))
}

fn issue_token(s: &AppState, id: u64, email: &str) -> TokenResp {
    let claims = Claims {
        sub: id,
        email: email.into(),
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
    };
    let token = encode(&Header::default(), &claims, &s.encoding).unwrap();
    TokenResp { access_token: token, token_type: "Bearer" }
}

#[derive(Clone, Debug)]
struct AuthUser(u64, String);

async fn me(req: Request) -> impl IntoResponse {
    let user = req.extensions().get::<AuthUser>().cloned().unwrap();
    Json(serde_json::json!({ "id": user.0, "email": user.1 }))
}

// ── auth middleware ─────────────────────────────────────────
async fn require_auth(
    State(s): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let header_val = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = header_val
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = decode::<Claims>(token, &s.decoding, &Validation::default())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut()
        .insert(AuthUser(data.claims.sub, data.claims.email));
    Ok(next.run(req).await)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 真實環境用 env / secret manager，不要硬編
    let secret = b"super-secret-do-not-leak";
    let state = AppState {
        users: Arc::new(Mutex::new(vec![])),
        encoding: EncodingKey::from_secret(secret),
        decoding: DecodingKey::from_secret(secret),
    };

    let protected = Router::new()
        .route("/me", get(me))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .merge(protected)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("listening on 3000");
    println!("\n試試：");
    println!("  curl -X POST -H 'content-type: application/json' \\");
    println!("       -d '{{\"email\":\"a@x.com\",\"password\":\"p4ssw0rd\"}}' \\");
    println!("       localhost:3000/register");
    println!("  # 拿到 token 後：");
    println!("  curl -H 'authorization: Bearer <token>' localhost:3000/me\n");
    axum::serve(listener, app).await.unwrap();
}
