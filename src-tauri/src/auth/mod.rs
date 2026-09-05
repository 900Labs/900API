use crate::models::{AuthConfig, AuthType};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;
type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid auth URL: {0}")]
    Url(String),
    #[error("Auth error: {0}")]
    Configuration(String),
}

pub fn apply_auth(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    url: &str,
    method: &str,
    body: &[u8],
) -> Result<reqwest::RequestBuilder, AuthError> {
    match auth.auth_type {
        AuthType::None => Ok(request),
        AuthType::Basic => {
            if auth.username.is_empty() {
                Err(AuthError::Configuration(
                    "Basic auth is enabled but the username is empty".to_string(),
                ))
            } else {
                Ok(request.basic_auth(&auth.username, Some(&auth.password)))
            }
        }
        AuthType::Bearer => {
            if auth.token.is_empty() {
                Err(AuthError::Configuration(
                    "Bearer auth is enabled but the token is empty".to_string(),
                ))
            } else {
                Ok(request.bearer_auth(&auth.token))
            }
        }
        AuthType::ApiKey => {
            if auth.api_key.is_empty() || auth.api_key_name.is_empty() {
                return Err(AuthError::Configuration(
                    "API key auth is enabled but the key or key name is empty".to_string(),
                ));
            }
            if auth.api_key_in == "query" {
                Ok(request.query(&[(auth.api_key_name.as_str(), auth.api_key.as_str())]))
            } else {
                HeaderValue::from_str(&auth.api_key).map_err(|error| {
                    AuthError::Configuration(format!("Invalid API key header value: {error}"))
                })?;
                Ok(request.header(&auth.api_key_name, &auth.api_key))
            }
        }
        AuthType::OAuth2 => {
            if auth.oauth2_access_token.is_empty() {
                return Err(AuthError::Configuration(
                    "OAuth 2.0 auth is enabled but the access token is empty".to_string(),
                ));
            }
            let token_type = if auth.oauth2_token_type.is_empty() {
                "Bearer"
            } else {
                &auth.oauth2_token_type
            };
            let header_value = format!("{} {}", token_type, auth.oauth2_access_token);
            let value = HeaderValue::from_str(&header_value).map_err(|error| {
                AuthError::Configuration(format!("Invalid Authorization header value: {error}"))
            })?;
            Ok(request.header("Authorization", value))
        }
        AuthType::OAuth1 => apply_oauth1(request, auth, url, method),
        AuthType::AwsSigV4 => apply_aws_sig_v4(request, auth, url, method, body),
        AuthType::Hawk => apply_hawk(request, auth, url, method),
    }
}

fn apply_oauth1(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    url: &str,
    method: &str,
) -> Result<reqwest::RequestBuilder, AuthError> {
    if auth.oauth1_consumer_key.is_empty() {
        return Err(AuthError::Configuration(
            "OAuth 1.0 auth is enabled but the consumer key is empty".to_string(),
        ));
    }

    let timestamp = chrono::Utc::now().timestamp().to_string();
    let nonce = uuid::Uuid::new_v4().to_string();

    let mut params: Vec<(String, String)> = vec![
        (
            "oauth_consumer_key".to_string(),
            auth.oauth1_consumer_key.clone(),
        ),
        (
            "oauth_signature_method".to_string(),
            "HMAC-SHA1".to_string(),
        ),
        ("oauth_timestamp".to_string(), timestamp),
        ("oauth_nonce".to_string(), nonce),
        ("oauth_version".to_string(), "1.0".to_string()),
    ];

    if !auth.oauth1_token.is_empty() {
        params.push(("oauth_token".to_string(), auth.oauth1_token.clone()));
    }

    let parsed_url =
        url::Url::parse(url).map_err(|error| AuthError::Url(format!("{url}: {error}")))?;
    for (key, value) in parsed_url.query_pairs() {
        params.push((key.to_string(), value.to_string()));
    }

    let signature = oauth1_signature(
        method,
        url,
        &params,
        &auth.oauth1_consumer_secret,
        &auth.oauth1_token_secret,
    )?;

    params.push(("oauth_signature".to_string(), signature));

    let auth_header: String = params
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join(", ");

    let header_value = format!("OAuth {auth_header}");
    let value = HeaderValue::from_str(&header_value).map_err(|error| {
        AuthError::Configuration(format!("Invalid OAuth 1.0 header value: {error}"))
    })?;
    Ok(request.header("Authorization", value))
}

fn oauth1_signature(
    method: &str,
    url: &str,
    params: &[(String, String)],
    consumer_secret: &str,
    token_secret: &str,
) -> Result<String, AuthError> {
    let mut encoded_pairs: Vec<(String, String)> = params
        .iter()
        .map(|(k, v)| (percent_encode(k), percent_encode(v)))
        .collect();
    encoded_pairs.sort();

    let normalized_params: String = encoded_pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");

    let base_url = base_oauth1_url(url)?;
    let base_string = format!(
        "{}&{}&{}",
        percent_encode(&method.to_uppercase()),
        percent_encode(&base_url),
        percent_encode(&normalized_params)
    );

    let signing_key = format!(
        "{}&{}",
        percent_encode(consumer_secret),
        percent_encode(token_secret)
    );

    Ok(
        general_purpose::STANDARD
            .encode(hmac_sha1(signing_key.as_bytes(), base_string.as_bytes())?),
    )
}

fn apply_aws_sig_v4(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    url: &str,
    method: &str,
    body: &[u8],
) -> Result<reqwest::RequestBuilder, AuthError> {
    if auth.aws_access_key_id.is_empty() || auth.aws_secret_access_key.is_empty() {
        return Err(AuthError::Configuration(
            "AWS SigV4 auth is enabled but the access key or secret key is empty".to_string(),
        ));
    }

    let region = if auth.aws_region.is_empty() {
        "us-east-1"
    } else {
        &auth.aws_region
    };
    let service = if auth.aws_service.is_empty() {
        "execute-api"
    } else {
        &auth.aws_service
    };

    let now = chrono::Utc::now();

    let (headers, _authorization) = sign_aws_sig_v4(auth, url, method, body, region, service, now)?;

    Ok(request.headers(headers))
}

fn sign_aws_sig_v4(
    auth: &AuthConfig,
    url: &str,
    method: &str,
    body: &[u8],
    region: &str,
    service: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(HeaderMap, String), AuthError> {
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();

    let parsed_url =
        url::Url::parse(url).map_err(|error| AuthError::Url(format!("{url}: {error}")))?;

    let host = parsed_url.host_str().unwrap_or("");
    if host.is_empty() {
        return Err(AuthError::Url(format!("URL has no host component: {url}")));
    }
    let canonical_path = canonical_aws_path(parsed_url.path());
    let canonical_query = canonical_aws_query(parsed_url.query().unwrap_or(""));

    let payload_hash = hex::encode(Sha256::digest(body));

    let canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        host.to_lowercase(),
        payload_hash,
        amz_date
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method.to_uppercase(),
        canonical_path,
        canonical_query,
        canonical_headers,
        signed_headers,
        payload_hash
    );

    let canonical_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

    let credential_scope = format!("{date_stamp}/{region}/{service}/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date, credential_scope, canonical_hash
    );

    let k_date = hmac_sha256(
        format!("AWS4{}", auth.aws_secret_access_key).as_bytes(),
        date_stamp.as_bytes(),
    )?;
    let k_region = hmac_sha256(&k_date, region.as_bytes())?;
    let k_service = hmac_sha256(&k_region, service.as_bytes())?;
    let k_signing = hmac_sha256(&k_service, b"aws4_request")?;

    let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes())?);

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        auth.aws_access_key_id, credential_scope, signed_headers, signature
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&authorization).map_err(|error| {
            AuthError::Configuration(format!("Invalid Authorization header: {error}"))
        })?,
    );
    headers.insert(
        "x-amz-date",
        HeaderValue::from_str(&amz_date).map_err(|error| {
            AuthError::Configuration(format!("Invalid x-amz-date header: {error}"))
        })?,
    );
    headers.insert(
        "x-amz-content-sha256",
        HeaderValue::from_str(&payload_hash).map_err(|error| {
            AuthError::Configuration(format!("Invalid x-amz-content-sha256 header: {error}"))
        })?,
    );
    headers.insert(
        "host",
        HeaderValue::from_str(host)
            .map_err(|error| AuthError::Configuration(format!("Invalid host header: {error}")))?,
    );

    Ok((headers, authorization))
}

fn apply_hawk(
    request: reqwest::RequestBuilder,
    auth: &AuthConfig,
    url: &str,
    method: &str,
) -> Result<reqwest::RequestBuilder, AuthError> {
    if auth.hawk_id.is_empty() || auth.hawk_key.is_empty() {
        return Err(AuthError::Configuration(
            "Hawk auth is enabled but the id or key is empty".to_string(),
        ));
    }

    let parsed_url =
        url::Url::parse(url).map_err(|error| AuthError::Url(format!("{url}: {error}")))?;

    let host = parsed_url.host_str().unwrap_or("");
    let port = parsed_url
        .port()
        .unwrap_or(if parsed_url.scheme() == "https" {
            443
        } else {
            80
        });
    let path = parsed_url.path();

    let timestamp = chrono::Utc::now().timestamp();
    let nonce = uuid::Uuid::new_v4().to_string();

    let algorithm = if auth.hawk_algorithm.is_empty() {
        "sha256"
    } else {
        &auth.hawk_algorithm
    };

    let normalized = format!(
        "hawk.1.header\n{}\n{}\n{}\n{}\n{}\n{}\n\n",
        timestamp,
        nonce,
        method.to_uppercase(),
        path,
        host.to_lowercase(),
        port
    );

    let signature = general_purpose::STANDARD.encode(hmac_sha256(
        auth.hawk_key.as_bytes(),
        normalized.as_bytes(),
    )?);

    let auth_header = format!(
        "Hawk id=\"{}\", mac=\"{}\", ts=\"{}\", nonce=\"{}\", algorithm=\"HMAC-{}\"",
        auth.hawk_id,
        signature,
        timestamp,
        nonce,
        algorithm.to_uppercase()
    );

    let value = HeaderValue::from_str(&auth_header)
        .map_err(|error| AuthError::Configuration(format!("Invalid Hawk header value: {error}")))?;
    Ok(request.header("Authorization", value))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, AuthError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|error| AuthError::Configuration(format!("Invalid HMAC key: {error}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn hmac_sha1(key: &[u8], data: &[u8]) -> Result<Vec<u8>, AuthError> {
    let mut mac = HmacSha1::new_from_slice(key)
        .map_err(|error| AuthError::Configuration(format!("Invalid HMAC key: {error}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn base_oauth1_url(url: &str) -> Result<String, AuthError> {
    let parsed = url::Url::parse(url).map_err(|error| AuthError::Url(format!("{url}: {error}")))?;
    let host = parsed.host_str().unwrap_or("");
    let port = match (parsed.port(), parsed.scheme()) {
        (Some(port), _) => format!(":{port}"),
        (None, "https") => String::new(),
        (None, "http") => String::new(),
        (None, _) => String::new(),
    };
    Ok(format!(
        "{}://{}{}{}",
        parsed.scheme(),
        host,
        port,
        parsed.path()
    ))
}

fn canonical_aws_path(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    path.split('/')
        .map(|segment| aws_uri_encode(&percent_decode(segment), true))
        .collect::<Vec<_>>()
        .join("/")
}

fn canonical_aws_query(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    let mut pairs: Vec<(String, String)> = query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((key, value)) => (
                aws_uri_encode(&percent_decode(key), true),
                aws_uri_encode(&percent_decode(value), true),
            ),
            None => (aws_uri_encode(&percent_decode(pair), true), String::new()),
        })
        .collect();
    pairs.sort();
    pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn aws_uri_encode(value: &str, encode_slash: bool) -> String {
    let mut result = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char)
            }
            b'/' if !encode_slash => result.push('/'),
            _ => result.push_str(&format!("%{byte:02X}")),
        }
    }
    result
}

fn percent_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~' => result.push(c),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{byte:02X}"));
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
        let client = reqwest::Client::new();
        let result = apply_auth(
            client.get("https://example.com"),
            &auth,
            "https://example.com",
            "GET",
            &[],
        );
        assert!(result.is_ok());
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
        let result = apply_auth(
            client.get("https://example.com/test"),
            &auth,
            "https://example.com/test",
            "GET",
            b"",
        );
        assert!(result.is_ok());
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
        let result = apply_auth(
            client.get("https://example.com/api"),
            &auth,
            "https://example.com/api",
            "GET",
            &[],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_aws_sig_v4_golden_value() {
        let auth = AuthConfig {
            auth_type: AuthType::AwsSigV4,
            aws_access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            aws_secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            ..Default::default()
        };
        let now = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2013, 5, 24, 0, 0, 0).unwrap();
        let (headers, authorization) = sign_aws_sig_v4(
            &auth,
            "https://examplebucket.s3.amazonaws.com/test.txt",
            "GET",
            b"",
            "us-east-1",
            "s3",
            now,
        )
        .unwrap();

        assert_eq!(
            headers.get("x-amz-content-sha256").unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            authorization,
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=df548e2ce037944d03f3e68682813b093763996d597cf890ca3d9037fd231eb4"
        );
    }

    #[test]
    fn test_aws_sig_v4_canonicalizes_query_and_path() {
        assert_eq!(
            canonical_aws_query("b=2&a=1&key=a%20b"),
            "a=1&b=2&key=a%20b"
        );
        assert_eq!(canonical_aws_query("flag&z=1"), "flag=&z=1");
        assert_eq!(canonical_aws_path("/a%20b/c+d"), "/a%20b/c%2Bd");
        assert_eq!(canonical_aws_path(""), "/");
    }

    #[test]
    fn test_hmac_sha1_known_vector() {
        let signature = hmac_sha1(b"key", b"The quick brown fox jumps over the lazy dog").unwrap();
        assert_eq!(
            general_purpose::STANDARD.encode(signature),
            "3nybhbi3iqa8ino29wqQcBydtNk="
        );
    }

    #[test]
    fn test_oauth1_signature_matches_known_value() {
        let params = vec![
            ("oauth_consumer_key".to_string(), "xyz".to_string()),
            (
                "oauth_signature_method".to_string(),
                "HMAC-SHA1".to_string(),
            ),
            ("z_param".to_string(), "value".to_string()),
            ("a_param".to_string(), "first".to_string()),
        ];
        let signature = oauth1_signature(
            "GET",
            "https://api.example.com/resource?a_param=first&z_param=value",
            &params,
            "consumer-secret",
            "token-secret",
        )
        .unwrap();
        assert_eq!(signature, "YJkgDA/zabV0ANr6nTlOHZCYeK0=");
    }

    #[test]
    fn test_oauth1_signature_is_order_independent() {
        let set_a = vec![
            ("oauth_consumer_key".to_string(), "xyz".to_string()),
            ("a_param".to_string(), "first".to_string()),
            ("z_param".to_string(), "value".to_string()),
        ];
        let set_b = vec![
            ("z_param".to_string(), "value".to_string()),
            ("oauth_consumer_key".to_string(), "xyz".to_string()),
            ("a_param".to_string(), "first".to_string()),
        ];
        let signature_a = oauth1_signature(
            "GET",
            "https://api.example.com/resource",
            &set_a,
            "consumer-secret",
            "token-secret",
        )
        .unwrap();
        let signature_b = oauth1_signature(
            "GET",
            "https://api.example.com/resource",
            &set_b,
            "consumer-secret",
            "token-secret",
        )
        .unwrap();
        assert_eq!(signature_a, signature_b);
    }

    #[test]
    fn test_invalid_url_is_an_error() {
        let auth = AuthConfig {
            auth_type: AuthType::AwsSigV4,
            aws_access_key_id: "AKIDEXAMPLE".to_string(),
            aws_secret_access_key: "secret".to_string(),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let result = apply_auth(
            client.get("https://ok.example.com"),
            &auth,
            "not a url",
            "GET",
            b"",
        );
        assert!(result.is_err());

        let hawk = AuthConfig {
            auth_type: AuthType::Hawk,
            hawk_id: "id".to_string(),
            hawk_key: "key".to_string(),
            ..Default::default()
        };
        let result = apply_auth(
            client.get("https://ok.example.com"),
            &hawk,
            "not a url",
            "GET",
            &[],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_credentials_is_an_error() {
        let client = reqwest::Client::new();

        let basic = AuthConfig {
            auth_type: AuthType::Basic,
            ..Default::default()
        };
        assert!(apply_auth(
            client.get("https://example.com"),
            &basic,
            "https://example.com",
            "GET",
            &[]
        )
        .is_err());

        let bearer = AuthConfig {
            auth_type: AuthType::Bearer,
            ..Default::default()
        };
        assert!(apply_auth(
            client.get("https://example.com"),
            &bearer,
            "https://example.com",
            "GET",
            &[]
        )
        .is_err());

        let oauth1 = AuthConfig {
            auth_type: AuthType::OAuth1,
            ..Default::default()
        };
        assert!(apply_auth(
            client.get("https://example.com"),
            &oauth1,
            "https://example.com",
            "GET",
            &[]
        )
        .is_err());
    }
}
