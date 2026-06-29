use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode},
    response::Response,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::sync::RwLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MockError {
    #[error("Mock server error: {0}")]
    Server(String),
    #[error("Port already in use: {0}")]
    PortInUse(u16),
    #[error("Server not running: {0}")]
    NotRunning(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRoute {
    pub id: String,
    pub method: String,
    pub path: String, // e.g. "/api/users/:id"
    pub status: u16,
    pub headers: Vec<crate::models::KeyValue>,
    pub body: String,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockServerConfig {
    pub port: u16,
    pub routes: Vec<MockRoute>,
    #[serde(default)]
    pub bind_host: Option<String>,
    #[serde(default)]
    pub cors_permissive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockServerState {
    pub port: u16,
    pub running: bool,
    pub request_count: u64,
    pub bind_host: String,
    pub cors_permissive: bool,
}

pub struct MockServer {
    pub config: MockServerConfig,
    pub request_count: Arc<RwLock<u64>>,
    pub shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

pub type MockManager = Arc<RwLock<HashMap<u16, MockServer>>>;

pub fn create_mock_manager() -> MockManager {
    Arc::new(RwLock::new(HashMap::new()))
}

#[derive(Clone)]
struct AppState {
    routes: Arc<RwLock<Vec<MockRoute>>>,
    request_count: Arc<RwLock<u64>>,
}

pub async fn start_mock_server(
    manager: MockManager,
    config: MockServerConfig,
) -> Result<(), MockError> {
    let port = config.port;
    let bind_ip = normalize_bind_host(config.bind_host.as_deref())?;
    let bind_host = bind_ip.to_string();

    // Check if already running
    {
        let servers = manager.read().unwrap_or_else(|e| e.into_inner());
        if servers.contains_key(&port) {
            return Err(MockError::PortInUse(port));
        }
    }

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let routes = Arc::new(RwLock::new(config.routes.clone()));
    let request_count = Arc::new(RwLock::new(0u64));

    let state = AppState {
        routes: routes.clone(),
        request_count: request_count.clone(),
    };

    // Build router with catch-all route
    let mut app = Router::new().fallback(move |req: Request| {
        let state = state.clone();
        async move { handle_request_inner(state, req).await }
    });

    if config.cors_permissive {
        app = app.layer(tower_http::cors::CorsLayer::permissive());
    }

    let addr = SocketAddr::new(bind_ip, port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| MockError::Server(e.to_string()))?;

    // Store server
    {
        let mut servers = manager.write().unwrap_or_else(|e| e.into_inner());
        servers.insert(
            port,
            MockServer {
                config: MockServerConfig {
                    bind_host: Some(bind_host),
                    ..config.clone()
                },
                request_count: request_count.clone(),
                shutdown: Some(shutdown_tx),
            },
        );
    }

    let manager_clone = manager.clone();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();

        // Clean up
        let mut servers = manager_clone.write().unwrap_or_else(|e| e.into_inner());
        servers.remove(&port);
    });

    Ok(())
}

pub fn stop_mock_server(manager: &MockManager, port: u16) -> Result<(), MockError> {
    let mut servers = manager.write().unwrap_or_else(|e| e.into_inner());
    if let Some(mut server) = servers.remove(&port) {
        if let Some(shutdown) = server.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
    Ok(())
}

pub fn get_mock_server_state(
    manager: &MockManager,
    port: u16,
) -> Result<MockServerState, MockError> {
    let servers = manager.read().unwrap_or_else(|e| e.into_inner());
    let server = servers.get(&port).ok_or(MockError::NotRunning(port))?;
    let request_count = *server
        .request_count
        .read()
        .unwrap_or_else(|e| e.into_inner());
    let bind_host = server
        .config
        .bind_host
        .clone()
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let cors_permissive = server.config.cors_permissive;

    Ok(MockServerState {
        port,
        running: true,
        request_count,
        bind_host,
        cors_permissive,
    })
}

pub fn list_mock_servers(manager: &MockManager) -> Vec<u16> {
    let servers = manager.read().unwrap_or_else(|e| e.into_inner());
    servers.keys().cloned().collect()
}

async fn handle_request_inner(state: AppState, request: Request) -> Response {
    // Increment request count
    {
        let mut count = state
            .request_count
            .write()
            .unwrap_or_else(|e| e.into_inner());
        *count += 1;
    }

    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let route = {
        let routes = state.routes.read().unwrap_or_else(|e| e.into_inner());
        routes
            .iter()
            .find(|route| {
                let route_method = match route.method.to_uppercase().as_str() {
                    "GET" => Some(Method::GET),
                    "POST" => Some(Method::POST),
                    "PUT" => Some(Method::PUT),
                    "PATCH" => Some(Method::PATCH),
                    "DELETE" => Some(Method::DELETE),
                    "HEAD" => Some(Method::HEAD),
                    "OPTIONS" => Some(Method::OPTIONS),
                    "*" | "ANY" => None,
                    _ => None,
                };

                let method_matches = route_method.is_none() || route_method == Some(method.clone());
                let path_matches = match_path(&route.path, &path);

                method_matches && path_matches
            })
            .cloned()
    };

    let route = match route {
        Some(r) => r,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!(
                    "{{\"error\":\"No mock route found for {} {}\"}}",
                    method, path
                )))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap()
                });
        }
    };

    // Apply delay
    if route.delay_ms > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(route.delay_ms)).await;
    }

    // Build response
    let status = StatusCode::from_u16(route.status).unwrap_or(StatusCode::OK);
    let mut response = Response::builder().status(status);

    for h in &route.headers {
        if h.enabled && !h.key.is_empty() {
            if let Ok(name) = axum::http::HeaderName::from_bytes(h.key.as_bytes()) {
                if let Ok(value) = HeaderValue::from_str(&h.value) {
                    response = response.header(name, value);
                }
            }
        }
    }

    // Ensure Content-Type if not set
    let has_ct = route
        .headers
        .iter()
        .any(|h| h.enabled && h.key.eq_ignore_ascii_case("content-type"));
    if !has_ct {
        response = response.header("Content-Type", "application/json");
    }

    response.body(Body::from(route.body)).unwrap_or_else(|_| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    })
}

fn match_path(pattern: &str, actual: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let actual_parts: Vec<&str> = actual.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_parts.len() != actual_parts.len() {
        // Check for wildcard at end
        if !pattern_parts.is_empty() && pattern_parts[pattern_parts.len() - 1] == "*" {
            return actual_parts.len() >= pattern_parts.len() - 1
                && pattern_parts[..pattern_parts.len() - 1]
                    .iter()
                    .zip(actual_parts.iter())
                    .all(|(p, a)| p.starts_with(':') || p == a);
        }
        return false;
    }

    pattern_parts
        .iter()
        .zip(actual_parts.iter())
        .all(|(p, a)| p.starts_with(':') || p == a || *p == "*")
}

fn normalize_bind_host(host: Option<&str>) -> Result<IpAddr, MockError> {
    let host = host.unwrap_or("127.0.0.1").trim();
    let host = if host.is_empty() || host.eq_ignore_ascii_case("localhost") {
        "127.0.0.1"
    } else {
        host
    };

    host.parse::<IpAddr>().map_err(|_| {
        MockError::Server(format!(
            "Invalid bind host '{}'. Use an IP address, localhost, or 0.0.0.0 for LAN access",
            host
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_path_exact() {
        assert!(match_path("/api/users", "/api/users"));
    }

    #[test]
    fn test_match_path_params() {
        assert!(match_path("/api/users/:id", "/api/users/123"));
        assert!(match_path("/api/users/:id/posts", "/api/users/123/posts"));
    }

    #[test]
    fn test_match_path_no_match() {
        assert!(!match_path("/api/users", "/api/posts"));
        assert!(!match_path("/api/users/:id", "/api/users/123/posts"));
    }

    #[test]
    fn test_match_path_wildcard() {
        assert!(match_path("/api/*", "/api/anything"));
        assert!(match_path("/api/*", "/api/foo/bar"));
    }

    #[test]
    fn test_match_path_double_wildcard() {
        assert!(match_path("/api/users/:id", "/api/users/abc-123"));
    }

    #[test]
    fn test_default_bind_host_is_loopback() {
        let ip = normalize_bind_host(None).unwrap();
        assert_eq!(ip.to_string(), "127.0.0.1");
    }

    #[test]
    fn test_localhost_normalizes_to_loopback() {
        let ip = normalize_bind_host(Some("localhost")).unwrap();
        assert_eq!(ip.to_string(), "127.0.0.1");
    }

    #[test]
    fn test_lan_bind_host_is_explicit() {
        let ip = normalize_bind_host(Some("0.0.0.0")).unwrap();
        assert_eq!(ip.to_string(), "0.0.0.0");
    }

    #[test]
    fn test_invalid_bind_host_is_rejected() {
        assert!(normalize_bind_host(Some("example.com")).is_err());
    }
}
