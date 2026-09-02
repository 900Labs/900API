use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

const GRPC_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_GRPC_RESPONSE_BYTES: usize = 50 * 1024 * 1024;
const GRPC_STATUS_UNKNOWN: i32 = 2;

#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("gRPC error: {0}")]
    Http(String),
    #[error("Invalid hex input: {0}")]
    InvalidHex(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcResponse {
    pub status: u16,
    pub grpc_status: i32,
    pub grpc_message: String,
    pub body_hex: String,
    pub body_size: usize,
    pub time_ms: u64,
    pub headers: HashMap<String, String>,
    pub trailers: HashMap<String, String>,
}

pub async fn send_grpc_unary(
    address: &str,
    service_method: &str,
    body_hex: &str,
    headers: &[crate::models::KeyValue],
    use_tls: bool,
) -> Result<GrpcResponse, GrpcError> {
    let protobuf_bytes = decode_hex(body_hex)?;

    // Build gRPC framed message: [compressed(1)] [length(4 BE)] [message]
    let mut framed = Vec::with_capacity(5 + protobuf_bytes.len());
    framed.push(0u8); // not compressed
    framed.extend_from_slice(&(protobuf_bytes.len() as u32).to_be_bytes());
    framed.extend_from_slice(&protobuf_bytes);

    // Build URL
    let scheme = if use_tls { "https" } else { "http" };
    let path = if service_method.starts_with('/') {
        service_method.to_string()
    } else {
        format!("/{}", service_method)
    };
    let url = format!("{}://{}{}", scheme, address, path);

    // Build headers
    let mut header_map = HeaderMap::new();
    header_map.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/grpc"),
    );
    header_map.insert(
        reqwest::header::TE,
        reqwest::header::HeaderValue::from_static("trailers"),
    );

    for h in headers {
        if h.enabled && !h.key.is_empty() {
            if let (Ok(name), Ok(value)) = (
                reqwest::header::HeaderName::from_bytes(h.key.as_bytes()),
                reqwest::header::HeaderValue::from_str(&h.value),
            ) {
                header_map.append(name, value);
            }
        }
    }

    // Build client with HTTP/2 prior knowledge for plaintext, or normal for TLS
    let client_builder = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .timeout(GRPC_TIMEOUT);

    let client = if use_tls {
        client_builder
            .build()
            .map_err(|e| GrpcError::Http(e.to_string()))?
    } else {
        client_builder
            .http2_prior_knowledge()
            .build()
            .map_err(|e| GrpcError::Http(e.to_string()))?
    };

    let start = Instant::now();

    let response = client
        .post(&url)
        .headers(header_map)
        .body(framed)
        .send()
        .await
        .map_err(|e| GrpcError::Http(e.to_string()))?;

    let elapsed = start.elapsed();
    let status = response.status().as_u16();

    // Collect headers
    let resp_headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let content_length = response.content_length();
    if let Some(length) = content_length {
        if length as usize > MAX_GRPC_RESPONSE_BYTES {
            return Err(GrpcError::Http(format!(
                "gRPC response body too large: {length} bytes (limit {MAX_GRPC_RESPONSE_BYTES})"
            )));
        }
    }

    // Get response body with a hard cap
    let body_bytes = read_body_capped(response, content_length).await?;

    // Parse gRPC framing: [compressed(1)] [length(4 BE)] [message]
    // An empty body is the norm for unary error responses, where the real
    // status lives in the HTTP/2 trailers (not exposed by reqwest), so it
    // must never be reported as a successful call.
    let (grpc_status, grpc_message, message_bytes) = if body_bytes.len() >= 5 {
        let len = u32::from_be_bytes([body_bytes[1], body_bytes[2], body_bytes[3], body_bytes[4]])
            as usize;

        if body_bytes.len() >= 5 + len {
            (0, String::new(), body_bytes[5..5 + len].to_vec())
        } else {
            (0, String::new(), body_bytes.to_vec())
        }
    } else if status == 200 {
        (
            GRPC_STATUS_UNKNOWN,
            "gRPC status unavailable: response body empty or malformed (trailers not readable)"
                .to_string(),
            body_bytes.to_vec(),
        )
    } else {
        (
            grpc_status_for_http_status(status),
            format!("HTTP {status}: gRPC call failed without a grpc-status"),
            body_bytes.to_vec(),
        )
    };

    Ok(GrpcResponse {
        status,
        grpc_status,
        grpc_message,
        body_hex: encode_hex(&message_bytes),
        body_size: message_bytes.len(),
        time_ms: elapsed.as_millis() as u64,
        headers: resp_headers,
        trailers: HashMap::new(),
    })
}

async fn read_body_capped(
    response: reqwest::Response,
    content_length: Option<u64>,
) -> Result<Vec<u8>, GrpcError> {
    let mut body = Vec::new();
    let mut stream = response;
    while let Some(chunk) = stream
        .chunk()
        .await
        .map_err(|e| GrpcError::Http(e.to_string()))?
    {
        if body.len() + chunk.len() > MAX_GRPC_RESPONSE_BYTES {
            return Err(GrpcError::Http(format!(
                "gRPC response body too large: exceeds {MAX_GRPC_RESPONSE_BYTES} byte limit"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    let _ = content_length;
    Ok(body)
}

fn grpc_status_for_http_status(status: u16) -> i32 {
    match status {
        400 => 13,                   // INTERNAL
        401 => 16,                   // UNAUTHENTICATED
        403 => 7,                    // PERMISSION_DENIED
        404 => 12,                   // UNIMPLEMENTED
        429 | 502 | 503 | 504 => 14, // UNAVAILABLE
        _ => GRPC_STATUS_UNKNOWN,
    }
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, GrpcError> {
    let hex = hex.trim().replace([' ', '\n', '\r'], "");
    if hex.is_empty() {
        return Ok(Vec::new());
    }
    hex::decode(&hex).map_err(|e| GrpcError::InvalidHex(e.to_string()))
}

fn encode_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex_empty() {
        assert!(decode_hex("").unwrap().is_empty());
    }

    #[test]
    fn test_decode_hex_valid() {
        let result = decode_hex("48656c6c6f").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_with_spaces() {
        let result = decode_hex("48 65 6c 6c 6f").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_invalid() {
        assert!(decode_hex("xyz").is_err());
    }

    #[test]
    fn test_encode_hex() {
        assert_eq!(encode_hex(b"Hello"), "48656c6c6f");
    }

    #[test]
    fn test_roundtrip() {
        let original = "deadbeef";
        let bytes = decode_hex(original).unwrap();
        let encoded = encode_hex(&bytes);
        assert_eq!(encoded, original);
    }
}
