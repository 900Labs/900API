use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Debug, Error)]
pub enum SseError {
    #[error("SSE error: {0}")]
    Sse(String),
    #[error("Connection not found: {0}")]
    NotFound(String),
    #[error("Connection already exists: {0}")]
    AlreadyExists(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    pub id: String,
    pub event_type: String, // "message" or custom event name
    pub data: String,
    pub retry: Option<u64>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConnectionState {
    pub id: String,
    pub url: String,
    pub status: String, // "connecting" | "connected" | "disconnected" | "error"
    pub event_count: u64,
}

pub struct SseConnection {
    pub state: SseConnectionState,
    pub cancel: oneshot::Sender<()>,
}

pub type SseManager = Arc<Mutex<std::collections::HashMap<String, SseConnection>>>;

pub fn create_sse_manager() -> SseManager {
    Arc::new(Mutex::new(std::collections::HashMap::new()))
}

pub async fn connect_sse(
    app: AppHandle,
    manager: SseManager,
    id: String,
    url: String,
    headers: Vec<crate::models::KeyValue>,
) -> Result<(), SseError> {
    // Check if already connected
    {
        let connections = manager.lock().unwrap_or_else(|e| e.into_inner());
        if connections.contains_key(&id) {
            return Err(SseError::AlreadyExists(id));
        }
    }

    // Emit connecting state
    let _ = app.emit(&format!("sse-{}-state", id), SseConnectionState {
        id: id.clone(),
        url: url.clone(),
        status: "connecting".to_string(),
        event_count: 0,
    });

    // Build request
    let mut header_map = HeaderMap::new();
    header_map.insert("Accept", "text/event-stream".parse().unwrap());
    header_map.insert("Cache-Control", "no-cache".parse().unwrap());

    for h in &headers {
        if h.enabled && !h.key.is_empty() {
            if let (Ok(name), Ok(value)) = (
                reqwest::header::HeaderName::from_bytes(h.key.as_bytes()),
                reqwest::header::HeaderValue::from_str(&h.value),
            ) {
                header_map.append(name, value);
            }
        }
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(|e| SseError::Sse(e.to_string()))?;

    let response = match client.get(&url).headers(header_map).send().await {
        Ok(r) => r,
        Err(e) => {
            let _ = app.emit(&format!("sse-{}-state", id), SseConnectionState {
                id: id.clone(),
                url: url.clone(),
                status: "error".to_string(),
                event_count: 0,
            });
            return Err(SseError::Sse(e.to_string()));
        }
    };

    // Create cancellation channel
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    // Store connection
    {
        let mut connections = manager.lock().unwrap_or_else(|e| e.into_inner());
        connections.insert(
            id.clone(),
            SseConnection {
                state: SseConnectionState {
                    id: id.clone(),
                    url: url.clone(),
                    status: "connected".to_string(),
                    event_count: 0,
                },
                cancel: cancel_tx,
            },
        );
    }

    // Emit connected state
    let _ = app.emit(&format!("sse-{}-state", id), SseConnectionState {
        id: id.clone(),
        url: url.clone(),
        status: "connected".to_string(),
        event_count: 0,
    });

    let manager_clone = manager.clone();
    let id_clone = id.clone();
    let app_clone = app.clone();

    // Spawn task to read SSE stream
    tokio::spawn(async move {
        let stream = response.bytes_stream();
        use futures_util::StreamExt;
        let mut stream = stream;

        // Buffer for accumulating SSE data
        let mut event_type = String::new();
        let mut data_lines: Vec<String> = Vec::new();
        let mut event_id: Option<String> = None;
        let mut retry: Option<u64> = None;
        let mut event_count: u64 = 0;

        let mut _cancelled = false;

        tokio::select! {
            _ = async {
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            let text = String::from_utf8_lossy(&chunk);
                            for line in text.lines() {
                                if line.is_empty() {
                                    // Empty line = event boundary
                                    if !data_lines.is_empty() {
                                        let data = data_lines.join("\n");
                                        let et = if event_type.is_empty() {
                                            "message".to_string()
                                        } else {
                                            event_type.clone()
                                        };

                                        let sse_event = SseEvent {
                                            id: event_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                                            event_type: et,
                                            data,
                                            retry,
                                            timestamp: chrono::Utc::now().timestamp_millis() as u64,
                                        };

                                        event_count += 1;

                                        let event_name = format!("sse-{}-event", id_clone);
                                        let _ = app_clone.emit(&event_name, &sse_event);

                                        let mut connections = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
                                        if let Some(conn) = connections.get_mut(&id_clone) {
                                            conn.state.event_count = event_count;
                                        }

                                        // Reset for next event
                                        event_type.clear();
                                        data_lines.clear();
                                        event_id = None;
                                        retry = None;
                                    }
                                } else if let Some(rest) = line.strip_prefix("data:") {
                                    data_lines.push(rest.trim().to_string());
                                } else if let Some(rest) = line.strip_prefix("event:") {
                                    event_type = rest.trim().to_string();
                                } else if let Some(rest) = line.strip_prefix("id:") {
                                    event_id = Some(rest.trim().to_string());
                                } else if let Some(rest) = line.strip_prefix("retry:") {
                                    if let Ok(ms) = rest.trim().parse::<u64>() {
                                        retry = Some(ms);
                                    }
                                }
                                // Comments (lines starting with :) are ignored
                            }
                        }
                        Err(e) => {
                            log::error!("SSE stream error: {}", e);
                            break;
                        }
                    }
                }
            } => {},
            _ = cancel_rx => {
                _cancelled = true;
            }
        }

        // Update state to disconnected
        let status = "disconnected";
        let mut connections = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(conn) = connections.get_mut(&id_clone) {
            conn.state.status = status.to_string();
            let state_event = format!("sse-{}-state", id_clone);
            let _ = app_clone.emit(&state_event, &conn.state.clone());
        }
        connections.remove(&id_clone);
    });

    Ok(())
}

pub fn disconnect_sse(
    manager: &SseManager,
    id: &str,
) -> Result<(), SseError> {
    let mut connections = manager.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(conn) = connections.remove(id) {
        let _ = conn.cancel.send(());
    }
    Ok(())
}

pub fn get_sse_state(
    manager: &SseManager,
    id: &str,
) -> Result<SseConnectionState, SseError> {
    let connections = manager.lock().unwrap_or_else(|e| e.into_inner());
    let conn = connections
        .get(id)
        .ok_or_else(|| SseError::NotFound(id.to_string()))?;
    Ok(conn.state.clone())
}
