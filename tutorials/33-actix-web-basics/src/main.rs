use actix_web::{
    delete, get, post,
    web::{self, Data, Json, Path, Query},
    App, HttpResponse, HttpServer, Responder, ResponseError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::info;

// ── 共享狀態 ────────────────────────────────────────────────
struct AppState {
    users: Mutex<HashMap<u64, User>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct ListParams {
    #[serde(default)]
    limit: Option<usize>,
}

// ── 自訂錯誤 ────────────────────────────────────────────────
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("not found")]
    NotFound,
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::NotFound => actix_web::http::StatusCode::NOT_FOUND,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": self.to_string(),
        }))
    }
}

// ── handlers（用 attribute macro 註冊路由） ────────────────
#[get("/")]
async fn root() -> impl Responder {
    "Hello, actix-web!"
}

#[get("/users")]
async fn list_users(s: Data<AppState>, q: Query<ListParams>) -> impl Responder {
    let users = s.users.lock().unwrap();
    let mut all: Vec<User> = users.values().cloned().collect();
    all.sort_by_key(|u| u.id);
    if let Some(limit) = q.limit {
        all.truncate(limit);
    }
    Json(all)
}

#[get("/users/{id}")]
async fn get_user(id: Path<u64>, s: Data<AppState>) -> Result<Json<User>, AppError> {
    let users = s.users.lock().unwrap();
    users
        .get(&id.into_inner())
        .cloned()
        .map(Json)
        .ok_or(AppError::NotFound)
}

#[post("/users")]
async fn create_user(s: Data<AppState>, payload: Json<CreateUser>) -> impl Responder {
    let mut users = s.users.lock().unwrap();
    let id = users.keys().max().copied().unwrap_or(0) + 1;
    let user = User {
        id,
        name: payload.name.clone(),
        email: payload.email.clone(),
    };
    users.insert(id, user.clone());
    info!(id, "user created");
    HttpResponse::Created().json(user)
}

#[delete("/users/{id}")]
async fn delete_user(id: Path<u64>, s: Data<AppState>) -> impl Responder {
    let mut users = s.users.lock().unwrap();
    if users.remove(&id.into_inner()).is_some() {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let state = Data::new(AppState {
        users: Mutex::new(HashMap::from([(
            1,
            User {
                id: 1,
                name: "alice".into(),
                email: "a@example.com".into(),
            },
        )])),
    });

    info!("actix-web listening on http://0.0.0.0:3000");
    println!("\n試試（同 axum 章的 endpoint）：");
    println!("  curl localhost:3000/users\n");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(tracing_actix_web::TracingLogger::default())
            .service(root)
            .service(list_users)
            .service(get_user)
            .service(create_user)
            .service(delete_user)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
