use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::State;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<ServerState>>,
    next_client_id: Arc<AtomicU64>,
}

struct ServerState {
    rooms: HashMap<String, HashMap<String, u64>>,
    waiting_users: Vec<WaitingUser>,
    clients: HashMap<u64, ClientInfo>,
}

struct WaitingUser {
    client_id: u64,
    user_id: String,
}

struct ClientInfo {
    tx: UnboundedSender<Message>,
    user_id: Option<String>,
    room_id: Option<String>,
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        inner: Arc::new(Mutex::new(ServerState {
            rooms: HashMap::new(),
            waiting_users: Vec::new(),
            clients: HashMap::new(),
        })),
        next_client_id: Arc::new(AtomicU64::new(1)),
    };

    let app = Router::new()
        .route("/", any(root_handler))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3101")
        .await
        .expect("failed to bind on 3101");

    println!("Rust WebSocketServer 运行:3101");
    axum::serve(listener, app)
        .await
        .expect("server stopped unexpectedly");
}

async fn root_handler(
    ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
    State(state): State<AppState>,
) -> Response {
    if let Ok(upgrade) = ws {
        upgrade
            .on_upgrade(move |socket| handle_socket(socket, state))
            .into_response()
    } else {
        (StatusCode::OK, "Rust WSS 服务运行中").into_response()
    }
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let client_id = state.next_client_id.fetch_add(1, Ordering::Relaxed);
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    {
        let mut guard = state.inner.lock().await;
        guard.clients.insert(
            client_id,
            ClientInfo {
                tx: tx.clone(),
                user_id: None,
                room_id: None,
            },
        );
    }

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                if let Some(msg_type) = parsed.get("type").and_then(Value::as_str) {
                    println!("Received: {msg_type}");
                    match msg_type {
                        "join" => handle_join(client_id, &parsed, &state).await,
                        "random_match" => handle_random_match(client_id, &parsed, &state).await,
                        "leave" => handle_leave_message(client_id, &parsed, &state).await,
                        "offer" | "answer" | "ice_candidate" | "screen_share_start"
                        | "screen_share_stop" => {
                            forward_to_room(client_id, text.to_string(), &state).await
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    handle_disconnect(client_id, &state).await;
    send_task.abort();
}

async fn handle_join(client_id: u64, msg: &Value, state: &AppState) {
    let room_id = match msg.get("roomId").and_then(Value::as_str) {
        Some(v) => v.to_string(),
        None => return,
    };
    let user_id = match msg.get("userId").and_then(Value::as_str) {
        Some(v) => v.to_string(),
        None => return,
    };

    let (notify_other, send_to_self) = {
        let mut guard = state.inner.lock().await;

        if let Some(client) = guard.clients.get_mut(&client_id) {
            client.user_id = Some(user_id.clone());
            client.room_id = Some(room_id.clone());
        } else {
            return;
        }

        let room = guard.rooms.entry(room_id.clone()).or_insert_with(HashMap::new);
        room.insert(user_id.clone(), client_id);

        println!(
            "User {} joined room {}, users in room: {}",
            user_id,
            room_id,
            room.len()
        );

        if room.len() == 2 {
            let other_client_id = room
                .iter()
                .find_map(|(uid, cid)| if uid != &user_id { Some(*cid) } else { None });

            (
                other_client_id.map(|cid| {
                    (
                        cid,
                        json!({
                            "type": "peer_joined",
                            "userId": user_id,
                        })
                        .to_string(),
                    )
                }),
                Some(json!({ "type": "ready" }).to_string()),
            )
        } else {
            (None, Some(json!({ "type": "waiting" }).to_string()))
        }
    };

    if let Some((other_id, payload)) = notify_other {
        send_to_client(state, other_id, payload).await;
    }
    if let Some(payload) = send_to_self {
        send_to_client(state, client_id, payload).await;
    }
}

async fn handle_random_match(client_id: u64, msg: &Value, state: &AppState) {
    let user_id = match msg.get("userId").and_then(Value::as_str) {
        Some(v) => v.to_string(),
        None => return,
    };

    enum MatchResult {
        Wait,
        Matched {
            room_id: String,
            other_client_id: u64,
        },
    }

    let result = {
        let mut guard = state.inner.lock().await;

        if let Some(client) = guard.clients.get_mut(&client_id) {
            client.user_id = Some(user_id.clone());
        } else {
            return;
        }

        let mut matched: Option<(u64, String)> = None;
        while let Some(other) = guard.waiting_users.pop() {
            if guard.clients.contains_key(&other.client_id) {
                matched = Some((other.client_id, other.user_id));
                break;
            }
        }

        if let Some((other_client_id, other_user_id)) = matched {
            let room_id = format!(
                "match_{}",
                state.next_client_id.fetch_add(1, Ordering::Relaxed)
            );

            let mut room = HashMap::new();
            room.insert(user_id.clone(), client_id);
            room.insert(other_user_id.clone(), other_client_id);
            guard.rooms.insert(room_id.clone(), room);

            if let Some(client) = guard.clients.get_mut(&client_id) {
                client.room_id = Some(room_id.clone());
            }
            if let Some(other_client) = guard.clients.get_mut(&other_client_id) {
                other_client.room_id = Some(room_id.clone());
            }

            MatchResult::Matched {
                room_id,
                other_client_id,
            }
        } else {
            guard.waiting_users.push(WaitingUser {
                client_id,
                user_id: user_id.clone(),
            });
            MatchResult::Wait
        }
    };

    match result {
        MatchResult::Wait => {
            send_to_client(
                state,
                client_id,
                json!({ "type": "waiting_for_match" }).to_string(),
            )
            .await;
        }
        MatchResult::Matched {
            room_id,
            other_client_id,
        } => {
            send_to_client(
                state,
                client_id,
                json!({
                    "type": "match_found",
                    "roomId": room_id,
                    "isInitiator": true
                })
                .to_string(),
            )
            .await;
            send_to_client(
                state,
                other_client_id,
                json!({
                    "type": "match_found",
                    "roomId": room_id,
                    "isInitiator": false
                })
                .to_string(),
            )
            .await;
        }
    }
}

async fn handle_leave_message(client_id: u64, msg: &Value, state: &AppState) {
    let room_id = msg
        .get("roomId")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let user_id = msg
        .get("userId")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    if let (Some(rid), Some(uid)) = (room_id, user_id) {
        handle_leave_internal(client_id, rid, uid, state).await;
    }
}

async fn handle_leave_internal(client_id: u64, room_id: String, user_id: String, state: &AppState) {
    let notify_targets = {
        let mut guard = state.inner.lock().await;
        let Some(room) = guard.rooms.get_mut(&room_id) else {
            return;
        };
        room.remove(&user_id);
        let room_is_empty = room.is_empty();
        let targets = if room_is_empty {
            Vec::new()
        } else {
            room.values().copied().collect::<Vec<_>>()
        };
        if room_is_empty {
            guard.rooms.remove(&room_id);
        }
        if let Some(client) = guard.clients.get_mut(&client_id) {
            if client.room_id.as_deref() == Some(room_id.as_str()) {
                client.room_id = None;
            }
        }
        targets
    };

    let payload = json!({
        "type": "peer_left",
        "userId": user_id
    })
    .to_string();

    for target in notify_targets {
        send_to_client(state, target, payload.clone()).await;
    }
}

async fn forward_to_room(client_id: u64, raw_msg: String, state: &AppState) {
    let target_clients = {
        let guard = state.inner.lock().await;
        let Some(client) = guard.clients.get(&client_id) else {
            return;
        };
        let Some(room_id) = &client.room_id else {
            return;
        };
        let Some(room) = guard.rooms.get(room_id) else {
            return;
        };

        room.values()
            .copied()
            .filter(|other_id| *other_id != client_id)
            .collect::<Vec<_>>()
    };

    for target in target_clients {
        send_to_client(state, target, raw_msg.clone()).await;
    }
}

async fn handle_disconnect(client_id: u64, state: &AppState) {
    let (room_id, user_id) = {
        let mut guard = state.inner.lock().await;
        guard.waiting_users.retain(|u| u.client_id != client_id);

        if let Some(client) = guard.clients.get(&client_id) {
            (client.room_id.clone(), client.user_id.clone())
        } else {
            (None, None)
        }
    };

    if let (Some(room_id), Some(user_id)) = (room_id, user_id) {
        handle_leave_internal(client_id, room_id, user_id, state).await;
    }

    let mut guard = state.inner.lock().await;
    guard.clients.remove(&client_id);
}

async fn send_to_client(state: &AppState, client_id: u64, payload: String) {
    let sender = {
        let guard = state.inner.lock().await;
        guard.clients.get(&client_id).map(|client| client.tx.clone())
    };

    if let Some(sender) = sender {
        let _ = sender.send(Message::Text(payload.into()));
    }
}
