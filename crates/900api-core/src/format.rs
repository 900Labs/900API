use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub const COLLECTION_SCHEMA: &str = "900api.collection/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionFile {
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exported_at: Option<String>,
    #[serde(default)]
    pub requests: Vec<RequestItem>,
}

impl CollectionFile {
    pub fn validate(&self) -> Result<(), FormatError> {
        if self.schema != COLLECTION_SCHEMA {
            return Err(FormatError::UnsupportedSchema(self.schema.clone()));
        }
        if self.name.trim().is_empty() {
            return Err(FormatError::Invalid(
                "collection name is required".to_string(),
            ));
        }
        for request in &self.requests {
            request.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestItem {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default, deserialize_with = "deserialize_vec_or_json_string")]
    pub headers: Vec<KeyValue>,
    #[serde(default, deserialize_with = "deserialize_vec_or_json_string")]
    pub params: Vec<KeyValue>,
    #[serde(default = "default_body_type")]
    pub body_type: String,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    pub body: String,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    #[serde(
        default = "default_object",
        deserialize_with = "deserialize_value_or_json_string"
    )]
    pub auth_config: Value,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    pub pre_request_script: String,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    pub test_script: String,
    #[serde(
        default = "default_object",
        deserialize_with = "deserialize_value_or_json_string"
    )]
    pub settings: Value,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub response_examples: Vec<ResponseExample>,
}

impl RequestItem {
    fn validate(&self) -> Result<(), FormatError> {
        if self.name.trim().is_empty() {
            return Err(FormatError::Invalid("request name is required".to_string()));
        }
        match self.method.to_ascii_uppercase().as_str() {
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS" => {}
            _ => {
                return Err(FormatError::Invalid(format!(
                    "unsupported HTTP method '{}'",
                    self.method
                )))
            }
        }
        match self.body_type.as_str() {
            "none" | "json" | "form_data" | "x_www_form_urlencoded" | "raw" => {}
            "binary" => {
                return Err(FormatError::Invalid(
                    "binary request bodies are not supported in portable collections".to_string(),
                ))
            }
            other => {
                return Err(FormatError::Invalid(format!(
                    "unsupported body type '{}'",
                    other
                )))
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseExample {
    pub name: String,
    pub status: u16,
    #[serde(default)]
    pub status_text: String,
    #[serde(
        default = "default_object",
        deserialize_with = "deserialize_value_or_json_string"
    )]
    pub headers: Value,
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    pub body: String,
    #[serde(default)]
    pub time_ms: u64,
    #[serde(default)]
    pub size_bytes: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("unsupported collection schema: {0}")]
    UnsupportedSchema(String),
    #[error("invalid collection: {0}")]
    Invalid(String),
    #[error("collection JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn parse_collection(input: &str) -> Result<CollectionFile, FormatError> {
    let collection: CollectionFile = serde_json::from_str(input)?;
    collection.validate()?;
    Ok(collection)
}

pub fn to_pretty_json(collection: &CollectionFile) -> Result<String, FormatError> {
    collection.validate()?;
    Ok(serde_json::to_string_pretty(collection)?)
}

pub fn value_to_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}

fn default_true() -> bool {
    true
}

fn default_schema() -> String {
    COLLECTION_SCHEMA.to_string()
}

fn default_body_type() -> String {
    "none".to_string()
}

fn default_auth_type() -> String {
    "none".to_string()
}

fn default_object() -> Value {
    Value::Object(Default::default())
}

fn deserialize_vec_or_json_string<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(Vec::new()),
        Value::String(raw) if raw.trim().is_empty() => Ok(Vec::new()),
        Value::String(raw) => serde_json::from_str(&raw).map_err(serde::de::Error::custom),
        other => serde_json::from_value(other).map_err(serde::de::Error::custom),
    }
}

fn deserialize_value_or_json_string<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(default_object()),
        Value::String(raw) if raw.trim().is_empty() => Ok(default_object()),
        Value::String(raw) => serde_json::from_str(&raw).map_err(serde::de::Error::custom),
        other => Ok(other),
    }
}

fn deserialize_string_or_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(String::new()),
        Value::String(raw) => Ok(raw),
        other => serde_json::to_string(&other).map_err(serde::de::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_collection_round_trips_structured_fields() {
        let collection = CollectionFile {
            schema: COLLECTION_SCHEMA.to_string(),
            id: Some("collection-1".to_string()),
            name: "Example".to_string(),
            description: Some("Portable collection".to_string()),
            parent_id: None,
            sort_order: 2,
            exported_at: None,
            requests: vec![RequestItem {
                id: Some("request-1".to_string()),
                name: "Health".to_string(),
                method: "GET".to_string(),
                url: "https://example.test/health".to_string(),
                headers: vec![KeyValue {
                    key: "Accept".to_string(),
                    value: "application/json".to_string(),
                    enabled: true,
                }],
                params: vec![],
                body_type: "none".to_string(),
                body: String::new(),
                auth_type: "none".to_string(),
                auth_config: serde_json::json!({}),
                pre_request_script: String::new(),
                test_script: "if (api900.response.status !== 200) { throw new Error('bad'); }"
                    .to_string(),
                settings: serde_json::json!({"timeout_ms": 5000}),
                sort_order: 1,
                response_examples: vec![ResponseExample {
                    name: "OK".to_string(),
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: serde_json::json!({"content-type": "application/json"}),
                    body: "{}".to_string(),
                    time_ms: 10,
                    size_bytes: 2,
                }],
            }],
        };

        let json = to_pretty_json(&collection).unwrap();
        assert!(json.contains("\"headers\": ["));
        assert_eq!(parse_collection(&json).unwrap(), collection);
    }

    #[test]
    fn imports_legacy_string_encoded_fields() {
        let legacy = r#"{
            "name": "Legacy",
            "requests": [{
                "name": "Health",
                "method": "GET",
                "url": "https://example.test",
                "headers": "[{\"key\":\"Accept\",\"value\":\"application/json\",\"enabled\":true}]",
                "params": "[]",
                "body_type": "none",
                "body": "",
                "auth_type": "none",
                "auth_config": "{}",
                "settings": "{\"timeout_ms\":120000}"
            }]
        }"#;

        let parsed = parse_collection(legacy).unwrap();
        assert_eq!(parsed.schema, COLLECTION_SCHEMA);
        assert_eq!(parsed.requests[0].headers[0].key, "Accept");
        assert_eq!(parsed.requests[0].settings["timeout_ms"], 120_000);
    }

    #[test]
    fn rejects_non_portable_binary_body() {
        let input = r#"{
            "name": "Binary",
            "requests": [{
                "name": "Upload", "method": "POST", "url": "https://example.test",
                "body_type": "binary", "body": "private.bin"
            }]
        }"#;
        assert!(parse_collection(input).is_err());
    }
}
