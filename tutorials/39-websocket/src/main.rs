use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    // broadcast：所有連線都收得到（聊天室）
    tx: broadcast::Sender<ChatMsg>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChatMsg {
    user: String,
    text: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 完成 WS 握手後跑 handle_socket
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // 訂閱廣播
    let mut rx = state.tx.subscribe();

    // task A：把廣播訊息往 client 寫
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap_or_default();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // task B：把 client 訊息廣播
    let tx = state.tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(t) => {
                    match serde_json::from_str::<ChatMsg>(&t) {
                        Ok(m) => { let _ = tx.send(m); }
                        Err(e) => warn!(?e, "bad message"),
                    }
                }
                Message::Close(_) => break,
                Message::Ping(p) => {
                    // axum 自動回 pong；這裡示意
                    info!(?p, "got ping");
                }
                _ => {}
            }
        }
    });

    // 任一邊掛了就一起收
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    info!("websocket disconnected");
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<title>ws demo</title>
<input id="u" placeholder="name"><input id="m" placeholder="msg"><button onclick="send()">send</button>
<pre id="log"></pre>
<script>
const ws = new WebSocket("ws://" + location.host + "/ws");
ws.onmessage = e => log.textContent += e.data + "\n";
function send() {
  ws.send(JSON.stringify({user: u.value, text: m.value}));
  m.value = "";
}
</script>"#,
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let (tx, _) = broadcast::channel::<ChatMsg>(100);
    let state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("open http://localhost:3000 in two tabs");
    axum::serve(listener, app).await.unwrap();
}
