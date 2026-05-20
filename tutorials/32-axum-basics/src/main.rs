use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;

// ── 共享狀態 ────────────────────────────────────────────────
#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<HashMap<u64, User>>>,
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

// ── handlers ────────────────────────────────────────────────
async fn root() -> &'static str {
    "Hello, axum!"
}

async fn list_users(
    State(s): State<AppState>,
    Query(p): Query<ListParams>,
) -> Json<Vec<User>> {
    let users = s.users.lock().unwrap();
    let mut all: Vec<User> = users.values().cloned().collect();
    all.sort_by_key(|u| u.id);
    if let Some(limit) = p.limit {
        all.truncate(limit);
    }
    Json(all)
}

async fn get_user(
    Path(id): Path<u64>,
    State(s): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    let users = s.users.lock().unwrap();
    users
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_user(
    State(s): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let mut users = s.users.lock().unwrap();
    let id = users.keys().max().copied().unwrap_or(0) + 1;
    let user = User {
        id,
        name: payload.name,
        email: payload.email,
    };
    users.insert(id, user.clone());
    info!(id, "user created");
    (StatusCode::CREATED, Json(user))
}

async fn delete_user(
    Path(id): Path<u64>,
    State(s): State<AppState>,
) -> StatusCode {
    let mut users = s.users.lock().unwrap();
    if users.remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        users: Arc::new(Mutex::new(HashMap::from([(
            1,
            User {
                id: 1,
                name: "alice".into(),
                email: "a@example.com".into(),
            },
        )]))),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).delete(delete_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    info!("axum listening on http://0.0.0.0:3000");
    println!("\n試試：");
    println!("  curl localhost:3000/");
    println!("  curl localhost:3000/users");
    println!("  curl localhost:3000/users/1");
    println!("  curl -X POST localhost:3000/users -H 'content-type: application/json' \\");
    println!("       -d '{{\"name\":\"bob\",\"email\":\"b@example.com\"}}'");
    println!("  curl -X DELETE localhost:3000/users/2\n");

    axum::serve(listener, app).await.unwrap();
}
