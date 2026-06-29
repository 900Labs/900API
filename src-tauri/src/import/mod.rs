use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    pub item: Vec<PostmanItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanItem {
    pub name: String,
    #[serde(default)]
    pub request: Option<PostmanRequest>,
    #[serde(default)]
    pub item: Vec<PostmanItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanRequest {
    pub method: String,
    pub url: PostmanUrl,
    #[serde(default)]
    pub header: Vec<PostmanHeader>,
    #[serde(default)]
    pub body: Option<PostmanBody>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanUrl {
    #[serde(default)]
    pub raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanHeader {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanBody {
    pub mode: String,
    #[serde(default)]
    pub raw: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub bearer: Option<Vec<PostmanKeyValue>>,
    #[serde(default)]
    pub basic: Option<Vec<PostmanKeyValue>>,
    #[serde(default)]
    pub apikey: Option<Vec<PostmanKeyValue>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanKeyValue {
    pub key: String,
    pub value: String,
}

pub struct ImportedRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,
    pub params: String,
    pub body_type: String,
    pub body: String,
    pub auth_type: String,
    pub auth_config: String,
}

pub fn import_postman_collection(path: &Path) -> Result<(String, Option<String>, Vec<ImportedRequest>), ImportError> {
    let content = std::fs::read_to_string(path)?;
    let collection: PostmanCollection = serde_json::from_str(&content)?;

    let name = collection.info.name;
    let description = collection.info.description;

    let mut requests = Vec::new();
    for item in &collection.item {
        collect_items(item, &mut requests);
    }

    Ok((name, description, requests))
}

fn collect_items(item: &PostmanItem, requests: &mut Vec<ImportedRequest>) {
    // If this item has a request, it's a leaf node
    if let Some(ref req) = item.request {
        let headers_json = serde_json::to_string(
            &req.header.iter().map(|h| {
                serde_json::json!({
                    "key": h.key,
                    "value": h.value,
                    "enabled": !h.disabled,
                })
            }).collect::<Vec<_>>(),
        ).unwrap_or_else(|_| "[]".to_string());

        let (body_type, body) = req.body.as_ref().map(|b| {
            let bt = match b.mode.as_str() {
                "raw" => "raw",
                "urlencoded" => "x_www_form_urlencoded",
                "formdata" => "form_data",
                _ => "none",
            };
            (bt.to_string(), b.raw.clone().unwrap_or_default())
        }).unwrap_or(("none".to_string(), String::new()));

        let (auth_type, auth_config) = req.auth.as_ref().map(|a| {
            let at = match a.auth_type.as_str() {
                "bearer" => "bearer",
                "basic" => "basic",
                "apikey" => "api_key",
                _ => "none",
            };
            let config = match a.auth_type.as_str() {
                "bearer" => {
                    let token = a.bearer.as_ref()
                        .and_then(|v| v.iter().find(|kv| kv.key == "token"))
                        .map(|kv| kv.value.clone())
                        .unwrap_or_default();
                    serde_json::json!({"auth_type": "bearer", "token": token, "username": "", "password": "", "api_key": "", "api_key_name": "", "api_key_in": "header"})
                }
                "basic" => {
                    let username = a.basic.as_ref()
                        .and_then(|v| v.iter().find(|kv| kv.key == "username"))
                        .map(|kv| kv.value.clone())
                        .unwrap_or_default();
                    let password = a.basic.as_ref()
                        .and_then(|v| v.iter().find(|kv| kv.key == "password"))
                        .map(|kv| kv.value.clone())
                        .unwrap_or_default();
                    serde_json::json!({"auth_type": "basic", "token": "", "username": username, "password": password, "api_key": "", "api_key_name": "", "api_key_in": "header"})
                }
                "apikey" => {
                    let key = a.apikey.as_ref()
                        .and_then(|v| v.iter().find(|kv| kv.key == "key"))
                        .map(|kv| kv.value.clone())
                        .unwrap_or_default();
                    let value = a.apikey.as_ref()
                        .and_then(|v| v.iter().find(|kv| kv.key == "value"))
                        .map(|kv| kv.value.clone())
                        .unwrap_or_default();
                    let in_header = a.apikey.as_ref()
                        .and_then(|v| v.iter().find(|kv| kv.key == "in"))
                        .map(|kv| kv.value.clone())
                        .unwrap_or_else(|| "header".to_string());
                    serde_json::json!({"auth_type": "api_key", "token": "", "username": "", "password": "", "api_key": value, "api_key_name": key, "api_key_in": in_header})
                }
                _ => serde_json::json!({"auth_type": "none", "token": "", "username": "", "password": "", "api_key": "", "api_key_name": "", "api_key_in": "header"}),
            };
            (at.to_string(), config.to_string())
        }).unwrap_or(("none".to_string(), r#"{"auth_type":"none"}"#.to_string()));

        requests.push(ImportedRequest {
            name: item.name.clone(),
            method: req.method.clone(),
            url: req.url.raw.clone(),
            headers: headers_json,
            params: "[]".to_string(),
            body_type,
            body,
            auth_type,
            auth_config,
        });
    }

    // Recurse into sub-items (folders)
    for sub_item in &item.item {
        collect_items(sub_item, requests);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_postman_collection() {
        let json = r#"{
            "info": {
                "name": "Test Collection",
                "description": "A test collection"
            },
            "item": [
                {
                    "name": "Get Users",
                    "request": {
                        "method": "GET",
                        "url": {"raw": "https://api.example.com/users"},
                        "header": [
                            {"key": "Authorization", "value": "Bearer token", "disabled": false}
                        ]
                    }
                },
                {
                    "name": "Create User",
                    "request": {
                        "method": "POST",
                        "url": {"raw": "https://api.example.com/users"},
                        "body": {
                            "mode": "raw",
                            "raw": "{\"name\":\"John\"}"
                        }
                    }
                }
            ]
        }"#;

        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-postman-test-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, json).unwrap();

        let (name, desc, requests) = import_postman_collection(&path).unwrap();
        assert_eq!(name, "Test Collection");
        assert_eq!(desc, Some("A test collection".to_string()));
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].name, "Get Users");
        assert_eq!(requests[0].method, "GET");
        assert_eq!(requests[1].name, "Create User");
        assert_eq!(requests[1].method, "POST");
        assert_eq!(requests[1].body_type, "raw");
        assert_eq!(requests[1].body, "{\"name\":\"John\"}");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_import_postman_with_folders() {
        let json = r#"{
            "info": {"name": "Folder Collection"},
            "item": [
                {
                    "name": "Folder 1",
                    "item": [
                        {
                            "name": "Request in Folder",
                            "request": {
                                "method": "GET",
                                "url": {"raw": "https://example.com/test"}
                            }
                        }
                    ]
                }
            ]
        }"#;

        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-postman-folder-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, json).unwrap();

        let (name, _, requests) = import_postman_collection(&path).unwrap();
        assert_eq!(name, "Folder Collection");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].name, "Request in Folder");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_import_postman_bearer_auth() {
        let json = r#"{
            "info": {"name": "Auth Collection"},
            "item": [
                {
                    "name": "Protected Request",
                    "request": {
                        "method": "GET",
                        "url": {"raw": "https://api.example.com/protected"},
                        "auth": {
                            "type": "bearer",
                            "bearer": [{"key": "token", "value": "abc123"}]
                        }
                    }
                }
            ]
        }"#;

        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-postman-auth-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, json).unwrap();

        let (_, _, requests) = import_postman_collection(&path).unwrap();
        assert_eq!(requests[0].auth_type, "bearer");
        assert!(requests[0].auth_config.contains("abc123"));

        let _ = std::fs::remove_file(&path);
    }
}
