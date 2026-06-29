use crate::models::{AuthConfig, AuthType};
use reqwest::header::{HeaderMap, HeaderValue};
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

pub fn apply_auth(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    url: &str,
    method: &str,
    body: &[u8],
) -> reqwest::RequestBuilder {
    match auth.auth_type {
        AuthType::None => request,
        AuthType::Basic => {
            if !auth.username.is_empty() {
                request.basic_auth(&auth.username, Some(&auth.password))
            } else {
                request
            }
        }
        AuthType::Bearer => {
            if !auth.token.is_empty() {
                request.bearer_auth(&auth.token)
            } else {
                request
            }
        }
        AuthType::ApiKey => {
            if !auth.api_key.is_empty() && !auth.api_key_name.is_empty() {
                if auth.api_key_in == "query" {
                    request.query(&[(auth.api_key_name.as_str(), auth.api_key.as_str())])
                } else {
                    request.header(&auth.api_key_name, &auth.api_key)
                }
            } else {
                request
            }
        }
        AuthType::OAuth2 => {
            if !auth.oauth2_access_token.is_empty() {
                let token_type = if auth.oauth2_token_type.is_empty() {
                    "Bearer"
                } else {
                    &auth.oauth2_token_type
                };
                let header_value = format!("{} {}", token_type, auth.oauth2_access_token);
                if let Ok(value) = HeaderValue::from_str(&header_value) {
                    request.header("Authorization", value)
                } else {
                    request
                }
            } else {
                request
            }
        }
        AuthType::OAuth1 => {
            apply_oauth1(request, auth, url, method, body)
        }
        AuthType::AwsSigV4 => {
            apply_aws_sig_v4(request, auth, url, method, body)
        }
        AuthType::Hawk => {
            apply_hawk(request, auth, url, method)
        }
    }
}

fn apply_oauth1(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    _url: &str,
    _method: &str,
    _body: &[u8],
) -> reqwest::RequestBuilder {
    if auth.oauth1_consumer_key.is_empty() {
        return request;
    }

    let timestamp = chrono::Utc::now().timestamp().to_string();
    let nonce = uuid::Uuid::new_v4().to_string();

    let mut params: Vec<(String, String)> = vec![
        ("oauth_consumer_key".to_string(), auth.oauth1_consumer_key.clone()),
        ("oauth_signature_method".to_string(), "HMAC-SHA256".to_string()),
        ("oauth_timestamp".to_string(), timestamp),
        ("oauth_nonce".to_string(), nonce),
        ("oauth_version".to_string(), "1.0".to_string()),
    ];

    if !auth.oauth1_token.is_empty() {
        params.push(("oauth_token".to_string(), auth.oauth1_token.clone()));
    }

    // Build signature base string
    let normalized_params: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let base_string = format!(
        "{}&{}&{}",
        _method.to_uppercase(),
        percent_encode(_url),
        percent_encode(&normalized_params)
    );

    // Sign with HMAC-SHA256
    let signing_key = format!(
        "{}&{}",
        percent_encode(&auth.oauth1_consumer_secret),
        percent_encode(&auth.oauth1_token_secret)
    );

    let mut mac = HmacSha256::new_from_slice(signing_key.as_bytes())
        .unwrap_or_else(|_| HmacSha256::new_from_slice(&[0u8; 1]).unwrap());
    mac.update(base_string.as_bytes());
    let signature = mac.finalize().into_bytes();
    let signature_b64 = general_purpose::STANDARD.encode(signature);

    params.push(("oauth_signature".to_string(), signature_b64));

    // Build Authorization header
    let auth_header: String = params
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, percent_encode(v)))
        .collect::<Vec<_>>()
        .join(", ");

    let header_value = format!("OAuth {}", auth_header);
    if let Ok(value) = HeaderValue::from_str(&header_value) {
        request.header("Authorization", value)
    } else {
        request
    }
}

fn apply_aws_sig_v4(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    url: &str,
    method: &str,
    body: &[u8],
) -> reqwest::RequestBuilder {
    if auth.aws_access_key_id.is_empty() || auth.aws_secret_access_key.is_empty() {
        return request;
    }

    let region = if auth.aws_region.is_empty() { "us-east-1" } else { &auth.aws_region };
    let service = if auth.aws_service.is_empty() { "execute-api" } else { &auth.aws_service };

    let now = chrono::Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();

    // Parse URL
    let parsed_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return request,
    };

    let host = parsed_url.host_str().unwrap_or("");
    let path = parsed_url.path();
    let query = parsed_url.query().unwrap_or("");

    // Payload hash
    let payload_hash = hex::encode(Sha256::digest(body));

    // Canonical request
    let canonical_request = format!(
        "{}\n{}\n{}\nhost:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n\nhost;x-amz-content-sha256;x-amz-date\n{}",
        method.to_uppercase(),
        path,
        query,
        host,
        payload_hash,
        amz_date,
        payload_hash
    );

    let canonical_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

    // String to sign
    let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, region, service);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        canonical_hash
    );

    // Signing key
    let k_date = hmac_sign(format!("AWS4{}", auth.aws_secret_access_key).as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sign(&k_date, region.as_bytes());
    let k_service = hmac_sign(&k_region, service.as_bytes());
    let k_signing = hmac_sign(&k_service, b"aws4_request");

    let signature = hex::encode(hmac_sign(&k_signing, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={}",
        auth.aws_access_key_id,
        credential_scope,
        signature
    );

    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&authorization) {
        headers.insert("Authorization", v);
    }
    if let Ok(v) = HeaderValue::from_str(&amz_date) {
        headers.insert("x-amz-date", v);
    }
    if let Ok(v) = HeaderValue::from_str(&payload_hash) {
        headers.insert("x-amz-content-sha256", v);
    }
    if let Ok(v) = HeaderValue::from_str(host) {
        headers.insert("host", v);
    }

    request.headers(headers)
}

fn apply_hawk(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    url: &str,
    method: &str,
) -> reqwest::RequestBuilder {
    if auth.hawk_id.is_empty() || auth.hawk_key.is_empty() {
        return request;
    }

    let parsed_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return request,
    };

    let host = parsed_url.host_str().unwrap_or("");
    let port = parsed_url.port().unwrap_or(if parsed_url.scheme() == "https" { 443 } else { 80 });
    let path = parsed_url.path();

    let timestamp = chrono::Utc::now().timestamp();
    let nonce = uuid::Uuid::new_v4().to_string();

    let algorithm = if auth.hawk_algorithm.is_empty() {
        "sha256"
    } else {
        &auth.hawk_algorithm
    };

    // Build normalized string
    let normalized = format!(
        "hawk.1.header\n{}\n{}\n{}\n{}\n{}\n{}\n\n",
        timestamp,
        nonce,
        method.to_uppercase(),
        path,
        host.to_lowercase(),
        port
    );

    let mut mac = HmacSha256::new_from_slice(auth.hawk_key.as_bytes())
        .unwrap_or_else(|_| HmacSha256::new_from_slice(&[0u8; 1]).unwrap());
    mac.update(normalized.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    let auth_header = format!(
        "Hawk id=\"{}\", mac=\"{}\", ts=\"{}\", nonce=\"{}\", algorithm=\"HMAC-{}\"",
        auth.hawk_id,
        signature,
        timestamp,
        nonce,
        algorithm.to_uppercase()
    );

    if let Ok(value) = HeaderValue::from_str(&auth_header) {
        request.header("Authorization", value)
    } else {
        request
    }
}

fn hmac_sign(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key)
        .unwrap_or_else(|_| HmacSha256::new_from_slice(&[0u8; 1]).unwrap());
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn percent_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~' => result.push(c),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode_basic() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
    }

    #[test]
    fn test_percent_encode_special() {
        assert_eq!(percent_encode("a+b=c"), "a%2Bb%3Dc");
    }

    #[test]
    fn test_percent_encode_no_encode() {
        assert_eq!(percent_encode("abc123-._~"), "abc123-._~");
    }

    #[test]
    fn test_oauth2_bearer_token() {
        let auth = AuthConfig {
            auth_type: AuthType::OAuth2,
            oauth2_access_token: "test-token".to_string(),
            ..Default::default()
        };
        // Just verify it doesn't panic
        let client = reqwest::Client::new();
        let _ = apply_auth(
            client.get("https://example.com"),
            &auth,
            "https://example.com",
            "GET",
            &[],
        );
    }

    #[test]
    fn test_aws_sig_v4_signing() {
        let auth = AuthConfig {
            auth_type: AuthType::AwsSigV4,
            aws_access_key_id: "AKIDEXAMPLE".to_string(),
            aws_secret_access_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".to_string(),
            aws_region: "us-east-1".to_string(),
            aws_service: "service".to_string(),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let _ = apply_auth(
            client.get("https://example.com/test"),
            &auth,
            "https://example.com/test",
            "GET",
            b"",
        );
    }

    #[test]
    fn test_hawk_auth() {
        let auth = AuthConfig {
            auth_type: AuthType::Hawk,
            hawk_id: "test-id".to_string(),
            hawk_key: "test-key".to_string(),
            hawk_algorithm: "sha256".to_string(),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let _ = apply_auth(
            client.get("https://example.com/api"),
            &auth,
            "https://example.com/api",
            "GET",
            &[],
        );
    }
}
