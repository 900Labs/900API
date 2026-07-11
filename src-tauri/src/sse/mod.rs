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
    pub error: Option<String>,
    pub event_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedSseEvent {
    event_type: String,
    data: String,
    id: Option<String>,
    retry: Option<u64>,
}

#[derive(Default)]
struct SseDecoder {
    buffer: Vec<u8>,
    event_type: String,
    data_lines: Vec<String>,
    event_id: Option<String>,
    retry: Option<u64>,
}

impl SseDecoder {
    fn push(&mut self, chunk: &[u8]) -> Vec<DecodedSseEvent> {
        self.buffer.extend_from_slice(chunk);
        self.drain_lines(false)
    }

    fn finish(&mut self) -> Vec<DecodedSseEvent> {
        let events = self.drain_lines(true);
        self.buffer.clear();
        self.reset_event();
        events
    }

    fn drain_lines(&mut self, finish: bool) -> Vec<DecodedSseEvent> {
        let mut events = Vec::new();
        while let Some(index) = self
            .buffer
            .iter()
            .position(|byte| *byte == b'\n' || *byte == b'\r')
        {
            if !finish && self.buffer[index] == b'\r' && index + 1 == self.buffer.len() {
                break;
            }

            let delimiter_len =
                if self.buffer[index] == b'\r' && self.buffer.get(index + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
            let line = String::from_utf8_lossy(&self.buffer[..index]).into_owned();
            self.buffer.drain(..index + delimiter_len);
            if let Some(event) = self.process_line(&line) {
                events.push(event);
            }
        }

        events
    }

    fn process_line(&mut self, line: &str) -> Option<DecodedSseEvent> {
        if line.is_empty() {
            return self.dispatch_event();
        }
        if line.starts_with(':') {
            return None;
        }

        let (field, value) = line
            .split_once(':')
            .map(|(field, value)| (field, value.strip_prefix(' ').unwrap_or(value)))
            .unwrap_or((line, ""));
        match field {
            "data" => self.data_lines.push(value.to_string()),
            "event" => self.event_type = value.to_string(),
            "id" if !value.contains('\0') => self.event_id = Some(value.to_string()),
            "retry" if value.chars().all(|character| character.is_ascii_digit()) => {
                self.retry = value.parse().ok();
            }
            _ => {}
        }
        None
    }

    fn dispatch_event(&mut self) -> Option<DecodedSseEvent> {
        if self.data_lines.is_empty() {
            self.reset_event();
            return None;
        }

        let event = DecodedSseEvent {
            event_type: if self.event_type.is_empty() {
                "message".to_string()
            } else {
                std::mem::take(&mut self.event_type)
            },
            data: self.data_lines.join("\n"),
            id: self.event_id.take(),
            retry: self.retry.take(),
        };
        self.data_lines.clear();
        Some(event)
    }

    fn reset_event(&mut self) {
        self.event_type.clear();
        self.data_lines.clear();
        self.event_id = None;
        self.retry = None;
    }
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
    let _ = app.emit(
        &format!("sse-{}-state", id),
        SseConnectionState {
            id: id.clone(),
            url: url.clone(),
            status: "connecting".to_string(),
            error: None,
            event_count: 0,
        },
    );

    // Build request
    let mut header_map = HeaderMap::new();
    header_map.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("text/event-stream"),
    );
    header_map.insert(
        reqwest::header::CACHE_CONTROL,
        reqwest::header::HeaderValue::from_static("no-cache"),
    );

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
            let _ = app.emit(
                &format!("sse-{}-state", id),
                SseConnectionState {
                    id: id.clone(),
                    url: url.clone(),
                    status: "error".to_string(),
                    error: Some(e.to_string()),
                    event_count: 0,
                },
            );
            return Err(SseError::Sse(e.to_string()));
        }
    };

    if !response.status().is_success() {
        let message = format!("SSE endpoint returned HTTP {}", response.status());
        let _ = app.emit(
            &format!("sse-{}-state", id),
            SseConnectionState {
                id: id.clone(),
                url: url.clone(),
                status: "error".to_string(),
                error: Some(message.clone()),
                event_count: 0,
            },
        );
        return Err(SseError::Sse(message));
    }

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
                    error: None,
                    event_count: 0,
                },
                cancel: cancel_tx,
            },
        );
    }

    // Emit connected state
    let _ = app.emit(
        &format!("sse-{}-state", id),
        SseConnectionState {
            id: id.clone(),
            url: url.clone(),
            status: "connected".to_string(),
            error: None,
            event_count: 0,
        },
    );

    let manager_clone = manager.clone();
    let id_clone = id.clone();
    let app_clone = app.clone();

    // Spawn task to read SSE stream
    tokio::spawn(async move {
        let stream = response.bytes_stream();
        use futures_util::StreamExt;
        let mut stream = stream;

        let mut decoder = SseDecoder::default();
        let mut event_count: u64 = 0;

        let mut cancelled = false;
        let mut stream_error = None;

        tokio::select! {
            _ = async {
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            for decoded in decoder.push(&chunk) {
                                emit_decoded_event(
                                    &app_clone,
                                    &manager_clone,
                                    &id_clone,
                                    &mut event_count,
                                    decoded,
                                );
                            }
                        }
                        Err(e) => {
                            log::error!("SSE stream error: {}", e);
                            stream_error = Some(e.to_string());
                            break;
                        }
                    }
                }
            } => {},
            _ = cancel_rx => {
                cancelled = true;
            }
        }

        if !cancelled && stream_error.is_none() {
            for decoded in decoder.finish() {
                emit_decoded_event(
                    &app_clone,
                    &manager_clone,
                    &id_clone,
                    &mut event_count,
                    decoded,
                );
            }
        }

        // Update state to disconnected
        let status = if stream_error.is_some() {
            "error"
        } else {
            "disconnected"
        };
        let mut connections = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(conn) = connections.get_mut(&id_clone) {
            conn.state.status = status.to_string();
            conn.state.error = stream_error;
            let state_event = format!("sse-{}-state", id_clone);
            let _ = app_clone.emit(&state_event, &conn.state.clone());
        }
        connections.remove(&id_clone);
    });

    Ok(())
}

fn emit_decoded_event(
    app: &AppHandle,
    manager: &SseManager,
    connection_id: &str,
    event_count: &mut u64,
    decoded: DecodedSseEvent,
) {
    let event = SseEvent {
        id: decoded
            .id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        event_type: decoded.event_type,
        data: decoded.data,
        retry: decoded.retry,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };
    *event_count += 1;
    let _ = app.emit(&format!("sse-{}-event", connection_id), &event);

    let mut connections = manager.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(connection) = connections.get_mut(connection_id) {
        connection.state.event_count = *event_count;
    }
}

pub fn disconnect_sse(manager: &SseManager, id: &str) -> Result<(), SseError> {
    let mut connections = manager.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(conn) = connections.remove(id) {
        let _ = conn.cancel.send(());
    }
    Ok(())
}

pub fn get_sse_state(manager: &SseManager, id: &str) -> Result<SseConnectionState, SseError> {
    let connections = manager.lock().unwrap_or_else(|e| e.into_inner());
    let conn = connections
        .get(id)
        .ok_or_else(|| SseError::NotFound(id.to_string()))?;
    Ok(conn.state.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_handles_fields_values_crlf_and_boundaries_split_across_chunks() {
        let mut decoder = SseDecoder::default();
        let chunks: &[&[u8]] = &[
            b"eve",
            b"nt: update\r\nid: 7\r",
            b"\ndata: first ",
            b"half\r\ndata: second\r\n\r",
            b"\n",
        ];
        let mut events = Vec::new();
        for chunk in chunks {
            events.extend(decoder.push(chunk));
        }

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "update");
        assert_eq!(events[0].id.as_deref(), Some("7"));
        assert_eq!(events[0].data, "first half\nsecond");
    }

    #[test]
    fn decoder_emits_multiple_events_with_lf_and_cr_boundaries() {
        let mut decoder = SseDecoder::default();
        let mut events = decoder.push(b"data: one\n\ndata: two\r\rnext:");
        events.extend(decoder.finish());

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "one");
        assert_eq!(events[1].data, "two");
    }

    #[test]
    fn decoder_preserves_utf8_for_terminated_final_event() {
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(b"data: caf\xc3").is_empty());
        let events = decoder.push(b"\xa9\n\n");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "caf\u{e9}");
        assert!(decoder.finish().is_empty());
    }

    #[test]
    fn decoder_emits_final_event_terminated_by_trailing_cr_blank_line() {
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(b"data: final\r\r").is_empty());

        let events = decoder.finish();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "final");
    }

    #[test]
    fn decoder_discards_unterminated_final_event() {
        let mut decoder = SseDecoder::default();
        assert!(decoder.push(b"data: incomplete\n").is_empty());
        assert!(decoder.finish().is_empty());

        assert!(decoder.push(b"data: also incomplete").is_empty());
        assert!(decoder.finish().is_empty());
    }
}
