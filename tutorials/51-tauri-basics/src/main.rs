// Tauri v2 範例。實際跑要：
//   1. npm create tauri-app  生出完整骨架
//   2. 或用 cargo create-tauri-app
//
// 這個檔案只展示 backend command 結構。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GreetResponse {
    message: String,
    length: usize,
}

#[tauri::command]
fn greet(name: &str) -> GreetResponse {
    let message = format!("Hello, {name}!");
    let length = name.chars().count();
    GreetResponse { message, length }
}

#[tauri::command]
async fn do_async_work(seconds: u64) -> Result<String, String> {
    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
    Ok(format!("waited {seconds}s"))
}

#[tauri::command]
fn read_secret(state: tauri::State<AppConfig>) -> String {
    state.welcome_message.clone()
}

struct AppConfig {
    welcome_message: String,
}

fn main() {
    tauri::Builder::default()
        .manage(AppConfig {
            welcome_message: "Welcome to Tauri!".into(),
        })
        .invoke_handler(tauri::generate_handler![greet, do_async_work, read_secret])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
