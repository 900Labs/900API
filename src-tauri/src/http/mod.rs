use crate::models::{BodyType, EnvironmentVariable, HttpMethod, KeyValue, RequestConfig, ResponseData};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use thiserror::Error;

pub mod variables;

static SHARED_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    SHARED_CLIENT.get_or_init(|| {
        Client::builder()
            .danger_accept_invalid_certs(false)
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(20)
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(30))
            .tcp_nodelay(true)
            .build()
            .expect("failed to build shared HTTP client")
    })
}

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP client error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
}

pub async fn send_request(config: &RequestConfig) -> Result<ResponseData, HttpError> {
    let client = get_client();

    // Build URL with query params
    let mut url = url::Url::parse(&config.url)
        .map_err(|e| HttpError::InvalidUrl(format!("{}: {}", e, config.url)))?;

    for param in &config.params {
        if param.enabled && !param.key.is_empty() {
            url.query_pairs_mut().append_pair(&param.key, &param.value);
        }
    }

    let method = match config.method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::PATCH => reqwest::Method::PATCH,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::HEAD => reqwest::Method::HEAD,
        HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
    };

    let mut request = client.request(method, url.as_str());

    // Apply headers
    let mut header_map = reqwest::header::HeaderMap::new();
    for header in &config.headers {
        if header.enabled && !header.key.is_empty() {
            let name = reqwest::header::HeaderName::from_bytes(header.key.as_bytes())
                .map_err(|e| HttpError::RequestFailed(format!("Invalid header name '{}': {}", header.key, e)))?;
            let value = reqwest::header::HeaderValue::from_str(&header.value)
                .map_err(|e| HttpError::RequestFailed(format!("Invalid header value for '{}': {}", header.key, e)))?;
            header_map.append(name, value);
        }
    }
    request = request.headers(header_map);

    // Apply auth
    let method_str = format!("{:?}", config.method);
    let body_bytes = config.body.as_bytes();
    request = crate::auth::apply_auth(request, &config.auth, url.as_str(), &method_str, body_bytes);

    // Apply body
    request = match &config.body_type {
        BodyType::None => request,
        BodyType::Json => {
            request.header("Content-Type", "application/json").body(config.body.clone())
        }
        BodyType::Raw => request.body(config.body.clone()),
        BodyType::FormData => {
            let mut form = reqwest::multipart::Form::new();
            // Parse body as key=value lines or JSON array
            if let Ok(entries) = serde_json::from_str::<Vec<KeyValue>>(&config.body) {
                for entry in entries {
                    form = form.text(entry.key, entry.value);
                }
            }
            request.multipart(form)
        }
        BodyType::XWwwFormUrlencoded => {
            let mut form_data = Vec::new();
            if let Ok(entries) = serde_json::from_str::<Vec<KeyValue>>(&config.body) {
                for entry in entries {
                    form_data.push((entry.key, entry.value));
                }
            }
            request.form(&form_data)
        }
        BodyType::Binary => request.body(config.body.clone()),
    };

    let start = Instant::now();
    let response = request.send().await?;
    let elapsed = start.elapsed();

    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("").to_string();

    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response.text().await.unwrap_or_default();
    let size_bytes = body.len();

    Ok(ResponseData {
        status,
        status_text,
        headers,
        body,
        time_ms: elapsed.as_millis() as u64,
        size_bytes,
    })
}

pub async fn send_graphql(
    url: &str,
    query: &str,
    variables: &str,
    operation_name: Option<&str>,
    headers: &[KeyValue],
    auth: &crate::models::AuthConfig,
    env_vars: &[EnvironmentVariable],
) -> Result<ResponseData, HttpError> {
    let resolved_url = variables::resolve_variables(url, env_vars);

    let client = get_client();

    // Parse variables JSON
    let variables_json: serde_json::Value = if variables.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(variables)
            .map_err(|e| HttpError::RequestFailed(format!("Invalid variables JSON: {}", e)))?
    };

    let mut payload = serde_json::json!({
        "query": query,
        "variables": variables_json,
    });

    if let Some(op) = operation_name {
        if !op.is_empty() {
            payload["operationName"] = serde_json::json!(op);
        }
    }

    let mut request = client
        .post(&resolved_url)
        .header("Content-Type", "application/json")
        .json(&payload);

    // Apply headers
    let mut header_map = reqwest::header::HeaderMap::new();
    let resolved_headers = variables::resolve_key_values(headers, env_vars);
    for header in &resolved_headers {
        if header.enabled && !header.key.is_empty() {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(header.key.as_bytes()) {
                if let Ok(value) = reqwest::header::HeaderValue::from_str(&header.value) {
                    header_map.append(name, value);
                }
            }
        }
    }
    request = request.headers(header_map);

    // Apply auth
    let payload_str = serde_json::to_string(&payload).unwrap_or_default();
    request = crate::auth::apply_auth(request, auth, &resolved_url, "POST", payload_str.as_bytes());

    let start = Instant::now();
    let response = request.send().await?;
    let elapsed = start.elapsed();

    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("").to_string();

    let resp_headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response.text().await.unwrap_or_default();
    let size_bytes = body.len();

    Ok(ResponseData {
        status,
        status_text,
        headers: resp_headers,
        body,
        time_ms: elapsed.as_millis() as u64,
        size_bytes,
    })
}
