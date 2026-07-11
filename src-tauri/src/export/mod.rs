use crate::models::{Collection, KeyValue, ResponseExample, SavedRequest};
use api900_core::format::{
    parse_collection, to_pretty_json, CollectionFile, KeyValue as PortableKeyValue, RequestItem,
    ResponseExample as PortableResponseExample, COLLECTION_SCHEMA,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Collection format error: {0}")]
    Format(#[from] api900_core::format::FormatError),
}

#[allow(dead_code)]
pub fn export_collection(
    collection: &Collection,
    requests: &[SavedRequest],
    path: &Path,
) -> Result<(), ExportError> {
    export_collection_with_examples(collection, requests, &HashMap::new(), path)
}

pub fn export_collection_with_examples(
    collection: &Collection,
    requests: &[SavedRequest],
    response_examples: &HashMap<String, Vec<ResponseExample>>,
    path: &Path,
) -> Result<(), ExportError> {
    let exported = collection_to_file(collection, requests, response_examples)?;
    let json = to_pretty_json(&exported)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn collection_to_file(
    collection: &Collection,
    requests: &[SavedRequest],
    response_examples: &HashMap<String, Vec<ResponseExample>>,
) -> Result<CollectionFile, ExportError> {
    Ok(CollectionFile {
        schema: COLLECTION_SCHEMA.to_string(),
        id: Some(collection.id.clone()),
        name: collection.name.clone(),
        description: collection.description.clone(),
        parent_id: collection.parent_id.clone(),
        sort_order: collection.sort_order,
        exported_at: None,
        requests: requests
            .iter()
            .map(|r| {
                Ok(RequestItem {
                    id: Some(r.id.clone()),
                    name: r.name.clone(),
                    method: r.method.clone(),
                    url: r.url.clone(),
                    headers: parse_portable_key_values(&r.headers)?,
                    params: parse_portable_key_values(&r.params)?,
                    body_type: r.body_type.clone(),
                    body: r.body.clone(),
                    auth_type: r.auth_type.clone(),
                    auth_config: parse_json_object(&r.auth_config)?,
                    pre_request_script: r.pre_request_script.clone(),
                    test_script: r.test_script.clone(),
                    settings: parse_json_object(&r.settings)?,
                    sort_order: r.sort_order,
                    response_examples: response_examples
                        .get(&r.id)
                        .map(|examples| {
                            examples
                                .iter()
                                .map(|example| {
                                    Ok(PortableResponseExample {
                                        name: example.name.clone(),
                                        status: example.status,
                                        status_text: example.status_text.clone(),
                                        headers: parse_json_object(&example.headers)?,
                                        body: example.body.clone(),
                                        time_ms: example.time_ms,
                                        size_bytes: example.size_bytes,
                                    })
                                })
                                .collect::<Result<Vec<_>, ExportError>>()
                        })
                        .transpose()?
                        .unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>, ExportError>>()?,
    })
}

pub fn import_collection(path: &Path) -> Result<CollectionFile, ExportError> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_collection(&content)?)
}

fn parse_portable_key_values(input: &str) -> Result<Vec<PortableKeyValue>, ExportError> {
    Ok(serde_json::from_str(input)?)
}

fn parse_json_object(input: &str) -> Result<Value, ExportError> {
    if input.trim().is_empty() {
        Ok(serde_json::json!({}))
    } else {
        Ok(serde_json::from_str(input)?)
    }
}

pub fn export_openapi_collection(
    collection: &Collection,
    requests: &[SavedRequest],
    path: &Path,
) -> Result<(), ExportError> {
    let server_url = collection_server_url(requests);
    let mut paths = Map::new();
    let mut security_schemes = Map::new();

    for request in requests {
        let method = request.method.to_ascii_lowercase();
        if !matches!(
            method.as_str(),
            "get" | "post" | "put" | "patch" | "delete" | "head" | "options"
        ) {
            continue;
        }

        let (path_key, query_from_url) = request_path_and_query(&request.url);
        let path_item = paths
            .entry(path_key)
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(path_object) = path_item.as_object_mut() else {
            continue;
        };

        let mut operation = Map::new();
        operation.insert("summary".to_string(), Value::String(request.name.clone()));
        operation.insert(
            "operationId".to_string(),
            Value::String(operation_id(&request.name, &request.method)),
        );

        let parameters = export_parameters(request, query_from_url);
        if !parameters.is_empty() {
            operation.insert("parameters".to_string(), Value::Array(parameters));
        }

        if let Some(request_body) = export_request_body(request) {
            operation.insert("requestBody".to_string(), request_body);
        }

        if let Some((scheme_name, security)) = export_security(request, &mut security_schemes) {
            operation.insert(
                "security".to_string(),
                Value::Array(vec![serde_json::json!({ scheme_name: security })]),
            );
        }

        path_object.insert(method, Value::Object(operation));
    }

    let mut root = Map::new();
    root.insert("openapi".to_string(), Value::String("3.0.3".to_string()));
    root.insert(
        "info".to_string(),
        serde_json::json!({
            "title": collection.name,
            "description": collection.description,
            "version": "1.0.0",
        }),
    );
    root.insert(
        "servers".to_string(),
        Value::Array(vec![serde_json::json!({ "url": server_url })]),
    );
    root.insert("paths".to_string(), Value::Object(paths));

    if !security_schemes.is_empty() {
        root.insert(
            "components".to_string(),
            serde_json::json!({ "securitySchemes": security_schemes }),
        );
    }

    let json = serde_json::to_string_pretty(&Value::Object(root))?;
    std::fs::write(path, json)?;
    Ok(())
}

fn collection_server_url(requests: &[SavedRequest]) -> String {
    requests
        .iter()
        .find_map(|request| origin_from_url(&request.url))
        .unwrap_or_else(|| "{{baseUrl}}".to_string())
}

fn origin_from_url(raw_url: &str) -> Option<String> {
    if let Ok(parsed) = url::Url::parse(raw_url) {
        let host = parsed.host_str()?;
        let mut origin = format!("{}://{}", parsed.scheme(), host);
        if let Some(port) = parsed.port() {
            origin.push(':');
            origin.push_str(&port.to_string());
        }
        return Some(origin);
    }

    let scheme_index = raw_url.find("://")?;
    let authority_start = scheme_index + 3;
    let authority_end = raw_url[authority_start..]
        .find(['/', '?', '#'])
        .map(|offset| authority_start + offset)
        .unwrap_or(raw_url.len());
    if authority_end <= authority_start {
        return None;
    }
    Some(raw_url[..authority_end].to_string())
}

fn request_path_and_query(raw_url: &str) -> (String, Vec<KeyValue>) {
    if raw_url.contains("{{") {
        if let Some((path, query)) = manual_path_and_query(raw_url) {
            return (api900_path_to_openapi(&path), query);
        }
    }

    if let Ok(parsed) = url::Url::parse(raw_url) {
        let query = parsed
            .query_pairs()
            .map(|(key, value)| KeyValue {
                key: key.to_string(),
                value: value.to_string(),
                enabled: true,
            })
            .collect();
        return (api900_path_to_openapi(parsed.path()), query);
    }

    if let Some((path, query)) = manual_path_and_query(raw_url) {
        return (api900_path_to_openapi(&path), query);
    }

    let stripped = raw_url
        .strip_prefix("{{baseUrl}}")
        .or_else(|| raw_url.strip_prefix("{{base_url}}"))
        .unwrap_or(raw_url);
    let path = stripped
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(stripped);
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    (api900_path_to_openapi(&path), Vec::new())
}

fn manual_path_and_query(raw_url: &str) -> Option<(String, Vec<KeyValue>)> {
    let path_and_query = if let Some(scheme_index) = raw_url.find("://") {
        let authority_start = scheme_index + 3;
        let path_start = raw_url[authority_start..]
            .find('/')
            .map(|offset| authority_start + offset);
        match path_start {
            Some(index) => &raw_url[index..],
            None => "/",
        }
    } else if let Some(stripped) = raw_url
        .strip_prefix("{{baseUrl}}")
        .or_else(|| raw_url.strip_prefix("{{base_url}}"))
    {
        stripped
    } else if raw_url.starts_with('/') {
        raw_url
    } else {
        return None;
    };

    let (path, query) = path_and_query
        .split_once('?')
        .map(|(path, query)| (path, parse_query_string(query)))
        .unwrap_or((path_and_query, Vec::new()));
    let path = if path.is_empty() { "/" } else { path };
    Some((path.to_string(), query))
}

fn parse_query_string(query: &str) -> Vec<KeyValue> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            KeyValue {
                key: key.to_string(),
                value: value.to_string(),
                enabled: true,
            }
        })
        .collect()
}

fn api900_path_to_openapi(path: &str) -> String {
    let mut output = String::new();
    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' || chars.peek() != Some(&'{') {
            output.push(ch);
            continue;
        }
        chars.next();

        let mut name = String::new();
        while let Some(next) = chars.next() {
            if next == '}' && chars.peek() == Some(&'}') {
                chars.next();
                break;
            }
            name.push(next);
        }
        output.push('{');
        output.push_str(name.trim());
        output.push('}');
    }
    output
}

fn operation_id(name: &str, method: &str) -> String {
    let mut id = String::new();
    let mut uppercase_next = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if id.is_empty() {
                id.push(ch.to_ascii_lowercase());
            } else if uppercase_next {
                id.push(ch.to_ascii_uppercase());
            } else {
                id.push(ch);
            }
            uppercase_next = false;
        } else {
            uppercase_next = true;
        }
    }
    if id.is_empty() {
        format!("{}Request", method.to_ascii_lowercase())
    } else {
        id
    }
}

fn export_parameters(request: &SavedRequest, query_from_url: Vec<KeyValue>) -> Vec<Value> {
    let mut parameters = Vec::new();
    for row in query_from_url
        .into_iter()
        .chain(parse_key_values(&request.params))
        .filter(|row| row.enabled && !row.key.trim().is_empty())
    {
        parameters.push(serde_json::json!({
            "name": row.key,
            "in": "query",
            "required": false,
            "schema": schema_from_string(&row.value),
            "example": row.value,
        }));
    }

    for row in parse_key_values(&request.headers)
        .into_iter()
        .filter(|row| row.enabled && !row.key.trim().is_empty())
    {
        parameters.push(serde_json::json!({
            "name": row.key,
            "in": "header",
            "required": false,
            "schema": schema_from_string(&row.value),
            "example": row.value,
        }));
    }

    parameters
}

fn parse_key_values(json: &str) -> Vec<KeyValue> {
    serde_json::from_str::<Vec<KeyValue>>(json).unwrap_or_default()
}

fn schema_from_string(value: &str) -> Value {
    if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
        serde_json::json!({ "type": "boolean" })
    } else if value.parse::<i64>().is_ok() {
        serde_json::json!({ "type": "integer" })
    } else if value.parse::<f64>().is_ok() {
        serde_json::json!({ "type": "number" })
    } else {
        serde_json::json!({ "type": "string" })
    }
}

fn export_request_body(request: &SavedRequest) -> Option<Value> {
    if request.body.trim().is_empty() || request.body_type == "none" {
        return None;
    }

    let content = match request.body_type.as_str() {
        "json" => {
            let example = serde_json::from_str::<Value>(&request.body)
                .unwrap_or_else(|_| Value::String(request.body.clone()));
            serde_json::json!({
                "application/json": {
                    "example": example
                }
            })
        }
        "x_www_form_urlencoded" => form_content("application/x-www-form-urlencoded", &request.body),
        "form_data" => form_content("multipart/form-data", &request.body),
        _ => {
            serde_json::json!({
                "text/plain": {
                    "example": request.body
                }
            })
        }
    };

    Some(serde_json::json!({
        "required": true,
        "content": content
    }))
}

fn form_content(media_type: &str, body: &str) -> Value {
    let rows = parse_key_values(body);
    let mut properties = Map::new();
    let mut example = Map::new();
    for row in rows.into_iter().filter(|row| !row.key.trim().is_empty()) {
        properties.insert(row.key.clone(), schema_from_string(&row.value));
        example.insert(row.key, Value::String(row.value));
    }
    serde_json::json!({
        media_type: {
            "schema": {
                "type": "object",
                "properties": properties
            },
            "example": example
        }
    })
}

fn export_security(
    request: &SavedRequest,
    security_schemes: &mut Map<String, Value>,
) -> Option<(String, Value)> {
    let config = serde_json::from_str::<Value>(&request.auth_config).unwrap_or(Value::Null);
    match request.auth_type.as_str() {
        "bearer" => {
            security_schemes.insert(
                "BearerAuth".to_string(),
                serde_json::json!({ "type": "http", "scheme": "bearer" }),
            );
            Some(("BearerAuth".to_string(), Value::Array(Vec::new())))
        }
        "basic" => {
            security_schemes.insert(
                "BasicAuth".to_string(),
                serde_json::json!({ "type": "http", "scheme": "basic" }),
            );
            Some(("BasicAuth".to_string(), Value::Array(Vec::new())))
        }
        "api_key" => {
            let key_name = config
                .get("api_key_name")
                .and_then(Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or("X-API-Key");
            let key_location = config
                .get("api_key_in")
                .and_then(Value::as_str)
                .unwrap_or("header");
            let scheme_name = format!("ApiKey{}", clean_component_name(key_name));
            security_schemes.insert(
                scheme_name.clone(),
                serde_json::json!({
                    "type": "apiKey",
                    "name": key_name,
                    "in": if key_location == "query" { "query" } else { "header" },
                }),
            );
            Some((scheme_name, Value::Array(Vec::new())))
        }
        "o_auth2" => {
            security_schemes.insert(
                "OAuth2Bearer".to_string(),
                serde_json::json!({ "type": "http", "scheme": "bearer" }),
            );
            Some(("OAuth2Bearer".to_string(), Value::Array(Vec::new())))
        }
        _ => None,
    }
}

fn clean_component_name(value: &str) -> String {
    let cleaned = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if cleaned.is_empty() {
        "Auth".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_export_and_import_collection() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-export-test-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let collection = Collection {
            id: "test-id".to_string(),
            name: "Test Collection".to_string(),
            description: Some("A test".to_string()),
            parent_id: None,
            sort_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let requests = vec![SavedRequest {
            id: "req-1".to_string(),
            collection_id: "test-id".to_string(),
            name: "Get Users".to_string(),
            method: "GET".to_string(),
            url: "https://api.example.com/users".to_string(),
            headers: "[]".to_string(),
            params: "[]".to_string(),
            body_type: "none".to_string(),
            body: "".to_string(),
            auth_type: "none".to_string(),
            auth_config: "{}".to_string(),
            settings: "{}".to_string(),
            pre_request_script: "var before = api900.response.status;".to_string(),
            test_script: "if (api900.response.status !== 200) { throw new Error('bad'); }"
                .to_string(),
            sort_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }];

        export_collection(&collection, &requests, &path).unwrap();
        assert!(path.exists());

        let imported = import_collection(&path).unwrap();
        assert_eq!(imported.name, "Test Collection");
        assert_eq!(imported.requests.len(), 1);
        assert_eq!(imported.requests[0].name, "Get Users");
        assert_eq!(imported.requests[0].method, "GET");
        assert_eq!(
            imported.requests[0].pre_request_script,
            "var before = api900.response.status;"
        );
        assert_eq!(
            imported.requests[0].test_script,
            "if (api900.response.status !== 200) { throw new Error('bad'); }"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_collection_with_response_examples() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-export-examples-test-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let collection = Collection {
            id: "test-id".to_string(),
            name: "Test Collection".to_string(),
            description: None,
            parent_id: None,
            sort_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let requests = vec![SavedRequest {
            id: "req-1".to_string(),
            collection_id: "test-id".to_string(),
            name: "Get Users".to_string(),
            method: "GET".to_string(),
            url: "https://api.example.com/users".to_string(),
            headers: "[]".to_string(),
            params: "[]".to_string(),
            body_type: "none".to_string(),
            body: "".to_string(),
            auth_type: "none".to_string(),
            auth_config: "{}".to_string(),
            settings: "{}".to_string(),
            pre_request_script: "".to_string(),
            test_script: "".to_string(),
            sort_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }];

        let mut examples = HashMap::new();
        examples.insert(
            "req-1".to_string(),
            vec![ResponseExample {
                id: "example-1".to_string(),
                request_id: "req-1".to_string(),
                name: "200 OK".to_string(),
                status: 200,
                status_text: "OK".to_string(),
                headers: r#"{"content-type":"application/json"}"#.to_string(),
                body: r#"{"ok":true}"#.to_string(),
                time_ms: 30,
                size_bytes: 11,
                created_at: "2026-01-01T00:00:00Z".to_string(),
            }],
        );

        export_collection_with_examples(&collection, &requests, &examples, &path).unwrap();

        let imported = import_collection(&path).unwrap();
        assert_eq!(imported.requests[0].response_examples.len(), 1);
        assert_eq!(imported.requests[0].response_examples[0].name, "200 OK");
        assert_eq!(
            imported.requests[0].response_examples[0].body,
            r#"{"ok":true}"#
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_import_invalid_json() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-import-invalid-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "{{ invalid json").unwrap();

        let result = import_collection(&path);
        assert!(result.is_err());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_openapi_collection() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-openapi-export-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let collection = Collection {
            id: "test-id".to_string(),
            name: "Inventory API".to_string(),
            description: Some("Warehouse operations".to_string()),
            parent_id: None,
            sort_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let requests = vec![SavedRequest {
            id: "req-1".to_string(),
            collection_id: "test-id".to_string(),
            name: "Create Item".to_string(),
            method: "POST".to_string(),
            url: "https://api.example.com/items/{{id}}".to_string(),
            headers: r#"[{"key":"X-Trace-Id","value":"trace-123","enabled":true}]"#.to_string(),
            params: r#"[{"key":"dryRun","value":"true","enabled":true}]"#.to_string(),
            body_type: "json".to_string(),
            body: r#"{"name":"Widget"}"#.to_string(),
            auth_type: "bearer".to_string(),
            auth_config: r#"{"auth_type":"bearer","token":"{{token}}"}"#.to_string(),
            settings: "{}".to_string(),
            pre_request_script: "".to_string(),
            test_script: "".to_string(),
            sort_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }];

        export_openapi_collection(&collection, &requests, &path).unwrap();
        let exported: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

        assert_eq!(exported["openapi"], "3.0.3");
        assert_eq!(exported["info"]["title"], "Inventory API");
        assert!(exported["paths"]["/items/{id}"]["post"]["parameters"]
            .to_string()
            .contains("dryRun"));
        assert!(exported["paths"]["/items/{id}"]["post"]["parameters"]
            .to_string()
            .contains("X-Trace-Id"));
        assert_eq!(
            exported["paths"]["/items/{id}"]["post"]["requestBody"]["content"]["application/json"]
                ["example"]["name"],
            "Widget"
        );
        assert_eq!(
            exported["components"]["securitySchemes"]["BearerAuth"]["scheme"],
            "bearer"
        );

        let _ = std::fs::remove_file(&path);
    }
}
