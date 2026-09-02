use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] yaml_serde::Error),
    #[error("OpenAPI import error: {0}")]
    OpenApi(String),
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

pub fn import_postman_collection(
    path: &Path,
) -> Result<(String, Option<String>, Vec<ImportedRequest>), ImportError> {
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

pub fn import_openapi_collection(
    path: &Path,
) -> Result<(String, Option<String>, Vec<ImportedRequest>), ImportError> {
    let content = std::fs::read_to_string(path)?;
    let document = parse_json_or_yaml(&content)?;

    if document.get("openapi").is_none() && document.get("swagger").is_none() {
        return Err(ImportError::OpenApi(
            "File is not an OpenAPI 3.x or Swagger 2.0 document".to_string(),
        ));
    }

    let fallback_name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.trim().is_empty())
        .unwrap_or("OpenAPI Import");
    let name = document
        .pointer("/info/title")
        .and_then(Value::as_str)
        .filter(|title| !title.trim().is_empty())
        .unwrap_or(fallback_name)
        .to_string();
    let description = document
        .pointer("/info/description")
        .and_then(Value::as_str)
        .map(str::to_string);

    let paths = document
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| ImportError::OpenApi("Missing or invalid paths object".to_string()))?;
    let default_server = default_server_url(&document);
    let mut requests = Vec::new();

    for (path_template, path_item_value) in paths {
        let path_item = resolve_ref(&document, path_item_value);
        let Some(path_item_obj) = path_item.as_object() else {
            continue;
        };
        let path_params = collect_openapi_parameters(&document, path_item.get("parameters"));

        for method in ["get", "post", "put", "patch", "delete", "head", "options"] {
            let Some(operation_value) = path_item_obj.get(method) else {
                continue;
            };
            let operation = resolve_ref(&document, operation_value);
            if !operation.is_object() {
                continue;
            }

            let mut operation_params = path_params.clone();
            operation_params.extend(collect_openapi_parameters(
                &document,
                operation.get("parameters"),
            ));

            let server = operation_server_url(operation)
                .or_else(|| operation_server_url(path_item))
                .unwrap_or_else(|| default_server.clone());
            let url = join_server_and_path(&server, path_template);
            let headers = key_values_json(
                operation_params
                    .iter()
                    .filter(|param| param.location == "header")
                    .map(openapi_param_to_key_value),
            );
            let params = key_values_json(
                operation_params
                    .iter()
                    .filter(|param| param.location == "query")
                    .map(openapi_param_to_key_value),
            );
            let (body_type, body) = openapi_request_body(&document, operation);
            let (auth_type, auth_config) = openapi_auth(&document, operation);
            let request_name = operation
                .get("summary")
                .and_then(Value::as_str)
                .or_else(|| operation.get("operationId").and_then(Value::as_str))
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path_template));

            requests.push(ImportedRequest {
                name: request_name,
                method: method.to_uppercase(),
                url,
                headers,
                params,
                body_type,
                body,
                auth_type,
                auth_config,
            });
        }
    }

    if requests.is_empty() {
        return Err(ImportError::OpenApi(
            "No HTTP operations were found in paths".to_string(),
        ));
    }

    Ok((name, description, requests))
}

fn parse_json_or_yaml(content: &str) -> Result<Value, ImportError> {
    match serde_json::from_str::<Value>(content) {
        Ok(value) => Ok(value),
        Err(json_error) => yaml_serde::from_str::<Value>(content).map_err(|yaml_error| {
            ImportError::OpenApi(format!(
                "Could not parse as JSON or YAML. JSON: {}; YAML: {}",
                json_error, yaml_error
            ))
        }),
    }
}

fn resolve_ref<'a>(root: &'a Value, value: &'a Value) -> &'a Value {
    let Some(reference) = value.get("$ref").and_then(Value::as_str) else {
        return value;
    };
    let Some(pointer) = reference.strip_prefix('#') else {
        return value;
    };
    root.pointer(pointer).unwrap_or(value)
}

#[derive(Clone, Debug)]
struct OpenApiParameter {
    name: String,
    location: String,
    required: bool,
    value: String,
}

fn collect_openapi_parameters(root: &Value, parameters: Option<&Value>) -> Vec<OpenApiParameter> {
    parameters
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|parameter| {
            let parameter = resolve_ref(root, parameter);
            let name = parameter.get("name")?.as_str()?.to_string();
            let location = parameter.get("in")?.as_str()?.to_string();
            if !matches!(location.as_str(), "query" | "header") {
                return None;
            }
            let required = parameter
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let value = parameter_sample(root, parameter)
                .map(|value| value_to_field_string(&value))
                .unwrap_or_default();
            Some(OpenApiParameter {
                name,
                location,
                required,
                value,
            })
        })
        .collect()
}

fn parameter_sample(root: &Value, parameter: &Value) -> Option<Value> {
    parameter
        .get("example")
        .cloned()
        .or_else(|| first_example_value(parameter.get("examples")))
        .or_else(|| {
            parameter
                .get("schema")
                .map(|schema| sample_from_schema(root, schema))
        })
}

fn first_example_value(examples: Option<&Value>) -> Option<Value> {
    let examples = examples?.as_object()?;
    examples.values().next().map(|example| {
        let example = example.get("value").unwrap_or(example);
        example.clone()
    })
}

fn openapi_param_to_key_value(param: &OpenApiParameter) -> Value {
    serde_json::json!({
        "key": param.name,
        "value": param.value,
        "enabled": param.required || !param.value.is_empty(),
    })
}

fn key_values_json<I>(rows: I) -> String
where
    I: IntoIterator<Item = Value>,
{
    serde_json::to_string(&rows.into_iter().collect::<Vec<_>>())
        .unwrap_or_else(|_| "[]".to_string())
}

fn default_server_url(root: &Value) -> String {
    operation_server_url(root)
        .or_else(|| swagger_server_url(root))
        .unwrap_or_else(|| "{{baseUrl}}".to_string())
}

fn operation_server_url(value: &Value) -> Option<String> {
    let server = value
        .get("servers")
        .and_then(Value::as_array)?
        .iter()
        .find_map(|server| server.get("url").and_then(Value::as_str))?;
    Some(apply_server_variables(
        server,
        value.get("servers")?.as_array()?.first()?,
    ))
}

fn apply_server_variables(url: &str, server: &Value) -> String {
    let Some(variables) = server.get("variables").and_then(Value::as_object) else {
        return url.to_string();
    };

    let mut resolved = url.to_string();
    for (name, config) in variables {
        let replacement = config
            .get("default")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("{{{{{}}}}}", name));
        resolved = resolved.replace(&format!("{{{}}}", name), &replacement);
    }
    resolved
}

fn swagger_server_url(root: &Value) -> Option<String> {
    let host = root.get("host").and_then(Value::as_str)?;
    let scheme = root
        .get("schemes")
        .and_then(Value::as_array)
        .and_then(|schemes| schemes.first())
        .and_then(Value::as_str)
        .unwrap_or("https");
    let base_path = root.get("basePath").and_then(Value::as_str).unwrap_or("");
    Some(format!("{}://{}{}", scheme, host, base_path))
}

fn join_server_and_path(server: &str, path_template: &str) -> String {
    let path = openapi_path_to_900api(path_template);
    if server.trim().is_empty() {
        return path;
    }
    format!(
        "{}{}",
        server.trim_end_matches('/'),
        if path.starts_with('/') {
            path
        } else {
            format!("/{}", path)
        }
    )
}

fn openapi_path_to_900api(path: &str) -> String {
    let mut output = String::new();
    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' {
            output.push(ch);
            continue;
        }

        let mut name = String::new();
        while let Some(next) = chars.peek().copied() {
            chars.next();
            if next == '}' {
                break;
            }
            name.push(next);
        }

        if name.is_empty() {
            output.push_str("{}");
        } else {
            output.push_str("{{");
            output.push_str(&name);
            output.push_str("}}");
        }
    }
    output
}

fn openapi_request_body(root: &Value, operation: &Value) -> (String, String) {
    let Some(body) = operation
        .get("requestBody")
        .map(|value| resolve_ref(root, value))
    else {
        return ("none".to_string(), String::new());
    };
    let Some(content) = body.get("content").and_then(Value::as_object) else {
        return ("none".to_string(), String::new());
    };
    let Some((media_type, media)) = select_media_type(content) else {
        return ("none".to_string(), String::new());
    };

    let media = resolve_ref(root, media);
    let schema = media.get("schema").map(|schema| resolve_ref(root, schema));
    let sample = media_sample(root, media);

    if media_type.contains("json") {
        let sample = sample
            .or_else(|| schema.map(|schema| sample_from_schema(root, schema)))
            .unwrap_or_else(|| serde_json::json!({}));
        return ("json".to_string(), value_to_body_string(&sample, true));
    }

    if media_type == "application/x-www-form-urlencoded" || media_type == "multipart/form-data" {
        let rows = form_rows_from_sample(root, sample.as_ref(), schema);
        let body = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string());
        let body_type = if media_type == "multipart/form-data" {
            "form_data"
        } else {
            "x_www_form_urlencoded"
        };
        return (body_type.to_string(), body);
    }

    let sample = sample
        .or_else(|| schema.map(|schema| sample_from_schema(root, schema)))
        .unwrap_or_else(|| Value::String(String::new()));
    ("raw".to_string(), value_to_body_string(&sample, false))
}

fn select_media_type(content: &Map<String, Value>) -> Option<(&str, &Value)> {
    for preferred in [
        "application/json",
        "application/x-www-form-urlencoded",
        "multipart/form-data",
        "text/plain",
    ] {
        if let Some(value) = content.get(preferred) {
            return Some((preferred, value));
        }
    }
    content
        .iter()
        .next()
        .map(|(key, value)| (key.as_str(), value))
}

fn media_sample(root: &Value, media: &Value) -> Option<Value> {
    media
        .get("example")
        .cloned()
        .or_else(|| first_example_value(media.get("examples")))
        .or_else(|| {
            media
                .get("schema")
                .map(|schema| sample_from_schema(root, schema))
        })
}

const SAMPLE_SCHEMA_MAX_DEPTH: usize = 32;

fn sample_from_schema(root: &Value, schema: &Value) -> Value {
    sample_from_schema_at_depth(root, schema, 0)
}

fn sample_from_schema_at_depth(root: &Value, schema: &Value, depth: usize) -> Value {
    if depth >= SAMPLE_SCHEMA_MAX_DEPTH {
        return Value::Null;
    }
    let next_depth = depth + 1;
    let schema = resolve_ref(root, schema);
    if let Some(example) = schema.get("example").or_else(|| schema.get("default")) {
        return example.clone();
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        if let Some(first) = values.first() {
            return first.clone();
        }
    }
    if let Some(all_of) = schema.get("allOf").and_then(Value::as_array) {
        let mut merged = Map::new();
        for part in all_of {
            if let Value::Object(object) = sample_from_schema_at_depth(root, part, next_depth) {
                merged.extend(object);
            }
        }
        return Value::Object(merged);
    }
    if let Some(choice) = schema
        .get("oneOf")
        .or_else(|| schema.get("anyOf"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
    {
        return sample_from_schema_at_depth(root, choice, next_depth);
    }

    let schema_type = schema.get("type").and_then(Value::as_str);
    if schema_type == Some("object") || schema.get("properties").is_some() {
        let mut object = Map::new();
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (name, property_schema) in properties {
                object.insert(
                    name.clone(),
                    sample_from_schema_at_depth(root, property_schema, next_depth),
                );
            }
        }
        return Value::Object(object);
    }
    if schema_type == Some("array") {
        let item = schema
            .get("items")
            .map(|items| sample_from_schema_at_depth(root, items, next_depth))
            .unwrap_or_else(|| Value::String(String::new()));
        return Value::Array(vec![item]);
    }

    match schema_type {
        Some("integer") => serde_json::json!(0),
        Some("number") => serde_json::json!(0.0),
        Some("boolean") => serde_json::json!(false),
        _ => Value::String(sample_string_for_schema(schema)),
    }
}

fn sample_string_for_schema(schema: &Value) -> String {
    match schema.get("format").and_then(Value::as_str) {
        Some("date-time") => "2026-01-01T00:00:00Z".to_string(),
        Some("date") => "2026-01-01".to_string(),
        Some("email") => "user@example.com".to_string(),
        Some("uuid") => "00000000-0000-0000-0000-000000000000".to_string(),
        _ => String::new(),
    }
}

fn form_rows_from_sample(
    root: &Value,
    sample: Option<&Value>,
    schema: Option<&Value>,
) -> Vec<Value> {
    if let Some(Value::Object(object)) = sample {
        return object
            .iter()
            .map(|(key, value)| {
                serde_json::json!({
                    "key": key,
                    "value": value_to_field_string(value),
                    "enabled": true,
                })
            })
            .collect();
    }

    schema
        .and_then(|schema| resolve_ref(root, schema).get("properties"))
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(key, property_schema)| {
                    serde_json::json!({
                        "key": key,
                        "value": value_to_field_string(&sample_from_schema(root, property_schema)),
                        "enabled": true,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn value_to_body_string(value: &Value, pretty_json: bool) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    if pretty_json {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    } else {
        value_to_field_string(value)
    }
}

fn value_to_field_string(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    if value.is_number() || value.is_boolean() {
        return value.to_string();
    }
    serde_json::to_string(value).unwrap_or_default()
}

fn openapi_auth(root: &Value, operation: &Value) -> (String, String) {
    let Some(requirements) = operation
        .get("security")
        .or_else(|| root.get("security"))
        .and_then(Value::as_array)
    else {
        return none_auth();
    };
    let Some(requirement) = requirements
        .iter()
        .filter_map(Value::as_object)
        .find(|requirement| !requirement.is_empty())
    else {
        return none_auth();
    };
    let Some((scheme_name, _scopes)) = requirement.iter().next() else {
        return none_auth();
    };
    let Some(scheme) = security_scheme(root, scheme_name) else {
        return none_auth();
    };

    let scheme_type = scheme
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let http_scheme = scheme
        .get("scheme")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let variable = clean_variable_name(scheme_name);

    match (scheme_type.as_str(), http_scheme.as_str()) {
        ("http", "bearer") => (
            "bearer".to_string(),
            serde_json::json!({
                "auth_type": "bearer",
                "token": format!("{{{{{}_token}}}}", variable),
                "username": "",
                "password": "",
                "api_key": "",
                "api_key_name": "",
                "api_key_in": "header",
            })
            .to_string(),
        ),
        ("http", "basic") | ("basic", _) => (
            "basic".to_string(),
            serde_json::json!({
                "auth_type": "basic",
                "token": "",
                "username": format!("{{{{{}_username}}}}", variable),
                "password": format!("{{{{{}_password}}}}", variable),
                "api_key": "",
                "api_key_name": "",
                "api_key_in": "header",
            })
            .to_string(),
        ),
        ("apikey", _) => {
            let key_name = scheme
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or(scheme_name);
            let key_in = scheme.get("in").and_then(Value::as_str).unwrap_or("header");
            (
                "api_key".to_string(),
                serde_json::json!({
                    "auth_type": "api_key",
                    "token": "",
                    "username": "",
                    "password": "",
                    "api_key": format!("{{{{{}_key}}}}", variable),
                    "api_key_name": key_name,
                    "api_key_in": if key_in == "query" { "query" } else { "header" },
                })
                .to_string(),
            )
        }
        ("oauth2", _) | ("openidconnect", _) => (
            "o_auth2".to_string(),
            serde_json::json!({
                "auth_type": "o_auth2",
                "oauth2_access_token": format!("{{{{{}_token}}}}", variable),
                "oauth2_token_type": "Bearer",
                "oauth2_refresh_token": "",
            })
            .to_string(),
        ),
        _ => none_auth(),
    }
}

fn security_scheme<'a>(root: &'a Value, name: &str) -> Option<&'a Value> {
    root.pointer(&format!(
        "/components/securitySchemes/{}",
        escape_json_pointer(name)
    ))
    .or_else(|| {
        root.pointer(&format!(
            "/securityDefinitions/{}",
            escape_json_pointer(name)
        ))
    })
}

fn escape_json_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn clean_variable_name(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if cleaned.is_empty() {
        "auth".to_string()
    } else {
        cleaned
    }
}

fn none_auth() -> (String, String) {
    (
        "none".to_string(),
        serde_json::json!({"auth_type": "none"}).to_string(),
    )
}

fn collect_items(item: &PostmanItem, requests: &mut Vec<ImportedRequest>) {
    // If this item has a request, it's a leaf node
    if let Some(ref req) = item.request {
        let headers_json = serde_json::to_string(
            &req.header
                .iter()
                .map(|h| {
                    serde_json::json!({
                        "key": h.key,
                        "value": h.value,
                        "enabled": !h.disabled,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|_| "[]".to_string());

        let (body_type, body) = req
            .body
            .as_ref()
            .map(|b| {
                let bt = match b.mode.as_str() {
                    "raw" => "raw",
                    "urlencoded" => "x_www_form_urlencoded",
                    "formdata" => "form_data",
                    _ => "none",
                };
                (bt.to_string(), b.raw.clone().unwrap_or_default())
            })
            .unwrap_or(("none".to_string(), String::new()));

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
    fn test_sample_from_schema_recursive_ref_terminates() {
        let root: Value = serde_json::from_str(
            r##"{
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "children": {
                        "type": "array",
                        "items": {"$ref": "#"}
                    }
                }
            }"##,
        )
        .unwrap();
        let sample = sample_from_schema(&root, &root);
        assert!(sample.is_object());
        assert_eq!(sample["name"], serde_json::json!(""));
        assert!(sample["children"].is_array());
    }

    #[test]
    fn test_import_postman_collection() {
        let json = r##"{
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
        }"##;

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

    #[test]
    fn test_import_openapi_json() {
        let json = r##"{
          "openapi": "3.0.3",
          "info": {"title": "Inventory API", "description": "Warehouse operations"},
          "servers": [{"url": "https://api.example.com/v1"}],
          "security": [{"BearerAuth": []}],
          "components": {
            "securitySchemes": {
              "BearerAuth": {"type": "http", "scheme": "bearer"}
            },
            "schemas": {
              "CreateItem": {
                "type": "object",
                "properties": {
                  "name": {"type": "string", "example": "Widget"},
                  "quantity": {"type": "integer", "example": 2}
                }
              }
            }
          },
          "paths": {
            "/items/{id}": {
              "get": {
                "summary": "Get item",
                "parameters": [
                  {"name": "includeHistory", "in": "query", "schema": {"type": "boolean", "default": true}},
                  {"name": "X-Trace-Id", "in": "header", "schema": {"type": "string", "example": "trace-123"}}
                ]
              }
            },
            "/items": {
              "post": {
                "operationId": "createItem",
                "requestBody": {
                  "content": {
                    "application/json": {
                      "schema": {"$ref": "#/components/schemas/CreateItem"}
                    }
                  }
                }
              }
            }
          }
        }"##;

        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-openapi-json-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, json).unwrap();

        let (name, description, requests) = import_openapi_collection(&path).unwrap();
        assert_eq!(name, "Inventory API");
        assert_eq!(description, Some("Warehouse operations".to_string()));
        assert_eq!(requests.len(), 2);

        let get_item = requests.iter().find(|req| req.method == "GET").unwrap();
        assert_eq!(get_item.name, "Get item");
        assert_eq!(get_item.url, "https://api.example.com/v1/items/{{id}}");
        assert!(get_item.params.contains("includeHistory"));
        assert!(get_item.headers.contains("X-Trace-Id"));
        assert_eq!(get_item.auth_type, "bearer");
        assert!(get_item.auth_config.contains("{{BearerAuth_token}}"));

        let create_item = requests.iter().find(|req| req.method == "POST").unwrap();
        assert_eq!(create_item.name, "createItem");
        assert_eq!(create_item.body_type, "json");
        assert!(create_item.body.contains("\"name\": \"Widget\""));
        assert!(create_item.body.contains("\"quantity\": 2"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_import_openapi_yaml() {
        let yaml = r#"
openapi: 3.0.3
info:
  title: YAML API
servers:
  - url: https://yaml.example.test
paths:
  /search:
    get:
      summary: Search records
      parameters:
        - name: q
          in: query
          required: true
          schema:
            type: string
            example: invoices
"#;

        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "900api-openapi-yaml-{}.yaml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, yaml).unwrap();

        let (name, _, requests) = import_openapi_collection(&path).unwrap();
        assert_eq!(name, "YAML API");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].name, "Search records");
        assert_eq!(requests[0].url, "https://yaml.example.test/search");
        assert!(requests[0].params.contains("invoices"));

        let _ = std::fs::remove_file(&path);
    }
}
