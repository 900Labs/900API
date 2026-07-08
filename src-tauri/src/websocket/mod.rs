use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use thiserror::Error;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Error)]
pub enum WsError {
    #[error("WebSocket error: {0}")]
    Ws(String),
    #[error("Connection not found: {0}")]
    NotFound(String),
    #[error("Connection already exists: {0}")]
    AlreadyExists(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub id: String,
    pub direction: String, // "sent" | "received"
    pub content: String,
    pub message_type: String, // "text" | "binary" | "ping" | "pong" | "close"
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConnectionState {
    pub id: String,
    pub url: String,
    pub status: String, // "connecting" | "connected" | "disconnected" | "error"
    pub messages: Vec<WsMessage>,
}

pub struct WsConnection {
    pub state: WsConnectionState,
    pub tx: tokio::sync::mpsc::UnboundedSender<String>,
}

pub type WsManager = Arc<Mutex<std::collections::HashMap<String, WsConnection>>>;

pub fn create_ws_manager() -> WsManager {
    Arc::new(Mutex::new(std::collections::HashMap::new()))
}

pub async fn connect_websocket(
    app: AppHandle,
    manager: WsManager,
    id: String,
    url: String,
) -> Result<(), WsError> {
    // Check if already connected
    {
        let connections = manager.lock().unwrap_or_else(|e| e.into_inner());
        if connections.contains_key(&id) {
            return Err(WsError::AlreadyExists(id));
        }
    }

    // Emit connecting state
    let _ = app.emit(
        &format!("ws-{}-state", id),
        WsConnectionState {
            id: id.clone(),
            url: url.clone(),
            status: "connecting".to_string(),
            messages: vec![],
        },
    );

    // Connect
    let (ws_stream, _) = match connect_async(&url).await {
        Ok(s) => s,
        Err(e) => {
            let _ = app.emit(
                &format!("ws-{}-state", id),
                WsConnectionState {
                    id: id.clone(),
                    url: url.clone(),
                    status: "error".to_string(),
                    messages: vec![],
                },
            );
            return Err(WsError::Ws(e.to_string()));
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Create channel for sending messages
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Store connection
    {
        let mut connections = manager.lock().unwrap_or_else(|e| e.into_inner());
        connections.insert(
            id.clone(),
            WsConnection {
                state: WsConnectionState {
                    id: id.clone(),
                    url: url.clone(),
                    status: "connected".to_string(),
                    messages: vec![],
                },
                tx,
            },
        );
    }

    // Emit connected state
    let _ = app.emit(
        &format!("ws-{}-state", id),
        WsConnectionState {
            id: id.clone(),
            url: url.clone(),
            status: "connected".to_string(),
            messages: vec![],
        },
    );

    let manager_clone = manager.clone();
    let id_clone = id.clone();
    let app_clone = app.clone();

    // Spawn task to handle outgoing messages
    let id_for_write = id.clone();
    let manager_for_write = manager.clone();
    let app_for_write = app.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(Message::Text(msg.clone())).await.is_err() {
                break;
            }

            // Record sent message
            let ws_msg = WsMessage {
                id: uuid::Uuid::new_v4().to_string(),
                direction: "sent".to_string(),
                content: msg,
                message_type: "text".to_string(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            };

            let event = format!("ws-{}-message", id_for_write);
            let _ = app_for_write.emit(&event, &ws_msg);

            let mut connections = manager_for_write.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(conn) = connections.get_mut(&id_for_write) {
                conn.state.messages.push(ws_msg);
            }
        }
    });

    // Spawn task to handle incoming messages
    tokio::spawn(async move {
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(msg) => {
                    let (content, message_type) = match &msg {
                        Message::Text(t) => (t.to_string(), "text"),
                        Message::Binary(b) => (format!("[binary: {} bytes]", b.len()), "binary"),
                        Message::Ping(_) => ("[ping]".to_string(), "ping"),
                        Message::Pong(_) => ("[pong]".to_string(), "pong"),
                        Message::Close(_) => ("[closed]".to_string(), "close"),
                        _ => continue,
                    };

                    let ws_msg = WsMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        direction: "received".to_string(),
                        content,
                        message_type: message_type.to_string(),
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    };

                    let event = format!("ws-{}-message", id_clone);
                    let _ = app_clone.emit(&event, &ws_msg);

                    let mut connections = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(conn) = connections.get_mut(&id_clone) {
                        if msg.is_close() {
                            conn.state.status = "disconnected".to_string();
                            let state_event = format!("ws-{}-state", id_clone);
                            let _ = app_clone.emit(&state_event, &conn.state.clone());
                        }
                        conn.state.messages.push(ws_msg);
                    }
                }
                Err(e) => {
                    let mut connections = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(conn) = connections.get_mut(&id_clone) {
                        conn.state.status = "error".to_string();
                        let state_event = format!("ws-{}-state", id_clone);
                        let _ = app_clone.emit(&state_event, &conn.state.clone());
                    }
                    log::error!("WebSocket read error: {}", e);
                    break;
                }
            }
        }

        // Clean up on disconnect
        let mut connections = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
        connections.remove(&id_clone);
    });

    Ok(())
}

pub fn send_websocket_message(
    manager: &WsManager,
    id: &str,
    message: String,
) -> Result<(), WsError> {
    let connections = manager.lock().unwrap_or_else(|e| e.into_inner());
    let conn = connections
        .get(id)
        .ok_or_else(|| WsError::NotFound(id.to_string()))?;
    conn.tx
        .send(message)
        .map_err(|e| WsError::Ws(e.to_string()))
}

pub fn disconnect_websocket(manager: &WsManager, id: &str) -> Result<(), WsError> {
    let mut connections = manager.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(conn) = connections.remove(id) {
        // Dropping the tx will close the send task
        drop(conn.tx);
    }
    Ok(())
}

pub fn get_websocket_state(manager: &WsManager, id: &str) -> Result<WsConnectionState, WsError> {
    let connections = manager.lock().unwrap_or_else(|e| e.into_inner());
    let conn = connections
        .get(id)
        .ok_or_else(|| WsError::NotFound(id.to_string()))?;
    Ok(conn.state.clone())
}
