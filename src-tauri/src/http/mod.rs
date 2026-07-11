use crate::models::{
    BodyType, EnvironmentVariable, GraphQLEnumValue, GraphQLField, GraphQLInputValue,
    GraphQLSchema, GraphQLSchemaType, GraphQLTypeRef, HttpMethod, KeyValue, RequestConfig,
    RequestSettings, ResponseData,
};
use reqwest::{cookie::Jar, redirect, Client};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use thiserror::Error;

pub mod variables;

const GRAPHQL_INTROSPECTION_QUERY: &str = r#"
query IntrospectionQuery {
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    types {
      kind
      name
      description
      fields(includeDeprecated: true) {
        name
        description
        args {
          name
          description
          type { ...TypeRef }
          defaultValue
        }
        type { ...TypeRef }
        isDeprecated
        deprecationReason
      }
      inputFields {
        name
        description
        type { ...TypeRef }
        defaultValue
      }
      interfaces { ...TypeRef }
      enumValues(includeDeprecated: true) {
        name
        description
        isDeprecated
        deprecationReason
      }
      possibleTypes { ...TypeRef }
    }
  }
}

fragment TypeRef on __Type {
  kind
  name
  ofType {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
              }
            }
          }
        }
      }
    }
  }
}
"#;

static SHARED_CLIENT: OnceLock<Result<Client, String>> = OnceLock::new();
static COOKIE_CLIENT: OnceLock<Result<Client, String>> = OnceLock::new();
static COOKIE_JAR: OnceLock<Arc<Jar>> = OnceLock::new();

fn get_client() -> Result<&'static Client, HttpError> {
    SHARED_CLIENT
        .get_or_init(|| {
            Client::builder()
                .danger_accept_invalid_certs(false)
                .pool_idle_timeout(Duration::from_secs(90))
                .pool_max_idle_per_host(20)
                .timeout(Duration::from_secs(120))
                .connect_timeout(Duration::from_secs(30))
                .tcp_nodelay(true)
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| HttpError::RequestFailed(format!("Could not build HTTP client: {error}")))
}

fn get_cookie_jar() -> Arc<Jar> {
    COOKIE_JAR.get_or_init(|| Arc::new(Jar::default())).clone()
}

fn get_cookie_client() -> Result<&'static Client, HttpError> {
    COOKIE_CLIENT
        .get_or_init(|| {
            Client::builder()
                .cookie_provider(get_cookie_jar())
                .danger_accept_invalid_certs(false)
                .pool_idle_timeout(Duration::from_secs(90))
                .pool_max_idle_per_host(20)
                .timeout(Duration::from_secs(120))
                .connect_timeout(Duration::from_secs(30))
                .tcp_nodelay(true)
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| {
            HttpError::RequestFailed(format!("Could not build cookie HTTP client: {error}"))
        })
}

fn validate_settings(settings: &RequestSettings) -> Result<(), HttpError> {
    if settings.timeout_ms == 0 || settings.timeout_ms > 600_000 {
        return Err(HttpError::RequestFailed(
            "Request timeout must be between 1 ms and 600000 ms".to_string(),
        ));
    }
    if settings.connect_timeout_ms == 0 || settings.connect_timeout_ms > settings.timeout_ms {
        return Err(HttpError::RequestFailed(
            "Connect timeout must be between 1 ms and the request timeout".to_string(),
        ));
    }
    Ok(())
}

fn build_client(settings: &RequestSettings) -> Result<Client, HttpError> {
    validate_settings(settings)?;

    if settings == &RequestSettings::default() {
        return Ok(get_client()?.clone());
    }

    if settings.use_cookie_jar
        && settings.timeout_ms == RequestSettings::default().timeout_ms
        && settings.connect_timeout_ms == RequestSettings::default().connect_timeout_ms
        && settings.follow_redirects
        && settings.verify_ssl
        && settings.proxy_url.trim().is_empty()
    {
        return Ok(get_cookie_client()?.clone());
    }

    let mut builder = Client::builder()
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .timeout(Duration::from_millis(settings.timeout_ms))
        .connect_timeout(Duration::from_millis(settings.connect_timeout_ms))
        .danger_accept_invalid_certs(!settings.verify_ssl)
        .tcp_nodelay(true);

    if !settings.follow_redirects {
        builder = builder.redirect(redirect::Policy::none());
    }

    if settings.use_cookie_jar {
        builder = builder.cookie_provider(get_cookie_jar());
    }

    let proxy_url = settings.proxy_url.trim();
    if !proxy_url.is_empty() {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| HttpError::RequestFailed(format!("Invalid proxy URL: {}", e)))?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(HttpError::Reqwest)
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
    let client = build_client(&config.settings)?;

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
            let name =
                reqwest::header::HeaderName::from_bytes(header.key.as_bytes()).map_err(|e| {
                    HttpError::RequestFailed(format!("Invalid header name '{}': {}", header.key, e))
                })?;
            let value = reqwest::header::HeaderValue::from_str(&header.value).map_err(|e| {
                HttpError::RequestFailed(format!(
                    "Invalid header value for '{}': {}",
                    header.key, e
                ))
            })?;
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
            if !config.body.trim().is_empty() {
                serde_json::from_str::<Value>(&config.body).map_err(|error| {
                    HttpError::RequestFailed(format!("Invalid JSON request body: {error}"))
                })?;
            }
            request
                .header("Content-Type", "application/json")
                .body(config.body.clone())
        }
        BodyType::Raw => request.body(config.body.clone()),
        BodyType::FormData => {
            let mut form = reqwest::multipart::Form::new();
            // Parse body as key=value lines or JSON array
            let entries = parse_form_fields(&config.body, "multipart form-data")?;
            for entry in entries
                .into_iter()
                .filter(|entry| entry.enabled && !entry.key.trim().is_empty())
            {
                form = form.text(entry.key, entry.value);
            }
            request.multipart(form)
        }
        BodyType::XWwwFormUrlencoded => {
            let form_data = parse_form_fields(&config.body, "URL-encoded form")?
                .into_iter()
                .filter(|entry| entry.enabled && !entry.key.trim().is_empty())
                .map(|entry| (entry.key, entry.value))
                .collect::<Vec<_>>();
            request.form(&form_data)
        }
    };

    let start = Instant::now();
    let response = request.send().await?;
    let elapsed = start.elapsed();

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();

    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response.text().await?;
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

fn parse_form_fields(body: &str, label: &str) -> Result<Vec<KeyValue>, HttpError> {
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(body)
        .map_err(|error| HttpError::RequestFailed(format!("Invalid {label} fields: {error}")))
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

    let client = get_client()?;

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
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();

    let resp_headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = response.text().await?;
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

pub async fn introspect_graphql_schema(
    url: &str,
    headers: &[KeyValue],
    auth: &crate::models::AuthConfig,
    env_vars: &[EnvironmentVariable],
) -> Result<GraphQLSchema, HttpError> {
    let response = send_graphql(
        url,
        GRAPHQL_INTROSPECTION_QUERY,
        "{}",
        Some("IntrospectionQuery"),
        headers,
        auth,
        env_vars,
    )
    .await?;

    if response.status >= 400 {
        return Err(HttpError::RequestFailed(format!(
            "GraphQL introspection failed with HTTP {} {}",
            response.status, response.status_text
        )));
    }

    parse_graphql_schema_response(&response.body)
}

pub fn parse_graphql_schema_response(body: &str) -> Result<GraphQLSchema, HttpError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|e| HttpError::RequestFailed(format!("Invalid GraphQL JSON response: {}", e)))?;

    if let Some(errors) = value.get("errors").and_then(Value::as_array) {
        if !errors.is_empty() {
            let message = errors
                .iter()
                .filter_map(|error| error.get("message").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(HttpError::RequestFailed(if message.is_empty() {
                "GraphQL introspection returned errors".to_string()
            } else {
                format!("GraphQL introspection returned errors: {}", message)
            }));
        }
    }

    let schema = value
        .pointer("/data/__schema")
        .ok_or_else(|| HttpError::RequestFailed("Missing data.__schema in response".to_string()))?;

    let query_type = schema
        .pointer("/queryType/name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let mutation_type = schema
        .pointer("/mutationType/name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let subscription_type = schema
        .pointer("/subscriptionType/name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let types = schema
        .get("types")
        .and_then(Value::as_array)
        .map(|types| {
            types
                .iter()
                .filter_map(parse_graphql_schema_type)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(GraphQLSchema {
        query_type,
        mutation_type,
        subscription_type,
        types,
    })
}

fn parse_graphql_schema_type(value: &Value) -> Option<GraphQLSchemaType> {
    Some(GraphQLSchemaType {
        kind: value.get("kind")?.as_str()?.to_string(),
        name: value.get("name")?.as_str()?.to_string(),
        description: optional_string(value.get("description")),
        fields: value
            .get("fields")
            .and_then(Value::as_array)
            .map(|fields| fields.iter().filter_map(parse_graphql_field).collect())
            .unwrap_or_default(),
        input_fields: value
            .get("inputFields")
            .and_then(Value::as_array)
            .map(|fields| {
                fields
                    .iter()
                    .filter_map(parse_graphql_input_value)
                    .collect()
            })
            .unwrap_or_default(),
        enum_values: value
            .get("enumValues")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(parse_graphql_enum_value).collect())
            .unwrap_or_default(),
        possible_types: value
            .get("possibleTypes")
            .and_then(Value::as_array)
            .map(|types| types.iter().filter_map(parse_graphql_type_ref).collect())
            .unwrap_or_default(),
    })
}

fn parse_graphql_field(value: &Value) -> Option<GraphQLField> {
    Some(GraphQLField {
        name: value.get("name")?.as_str()?.to_string(),
        description: optional_string(value.get("description")),
        args: value
            .get("args")
            .and_then(Value::as_array)
            .map(|args| args.iter().filter_map(parse_graphql_input_value).collect())
            .unwrap_or_default(),
        field_type: parse_graphql_type_ref(value.get("type")?)?,
        is_deprecated: value
            .get("isDeprecated")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        deprecation_reason: optional_string(value.get("deprecationReason")),
    })
}

fn parse_graphql_input_value(value: &Value) -> Option<GraphQLInputValue> {
    Some(GraphQLInputValue {
        name: value.get("name")?.as_str()?.to_string(),
        description: optional_string(value.get("description")),
        value_type: parse_graphql_type_ref(value.get("type")?)?,
        default_value: optional_string(value.get("defaultValue")),
    })
}

fn parse_graphql_enum_value(value: &Value) -> Option<GraphQLEnumValue> {
    Some(GraphQLEnumValue {
        name: value.get("name")?.as_str()?.to_string(),
        description: optional_string(value.get("description")),
        is_deprecated: value
            .get("isDeprecated")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        deprecation_reason: optional_string(value.get("deprecationReason")),
    })
}

fn parse_graphql_type_ref(value: &Value) -> Option<GraphQLTypeRef> {
    Some(GraphQLTypeRef {
        kind: value.get("kind")?.as_str()?.to_string(),
        name: optional_string(value.get("name")),
        of_type: value.get("ofType").and_then(|of_type| {
            if of_type.is_null() {
                None
            } else {
                parse_graphql_type_ref(of_type).map(Box::new)
            }
        }),
    })
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_use_shared_client() {
        let client = build_client(&RequestSettings::default());
        assert!(client.is_ok());
    }

    #[test]
    fn test_cookie_settings_use_cookie_client() {
        let settings = RequestSettings {
            use_cookie_jar: true,
            ..RequestSettings::default()
        };
        let client = build_client(&settings);
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_timeout_is_rejected() {
        let settings = RequestSettings {
            timeout_ms: 0,
            ..RequestSettings::default()
        };
        let err = build_client(&settings).unwrap_err().to_string();
        assert!(err.contains("Request timeout"));
    }

    #[test]
    fn test_invalid_proxy_is_rejected() {
        let settings = RequestSettings {
            proxy_url: "not a proxy url".to_string(),
            ..RequestSettings::default()
        };
        let err = build_client(&settings).unwrap_err().to_string();
        assert!(err.contains("Invalid proxy URL"));
    }

    #[test]
    fn test_parse_graphql_schema_response() {
        let body = r#"{
          "data": {
            "__schema": {
              "queryType": {"name": "Query"},
              "mutationType": {"name": "Mutation"},
              "subscriptionType": null,
              "types": [
                {
                  "kind": "OBJECT",
                  "name": "Query",
                  "description": "Root query",
                  "fields": [
                    {
                      "name": "country",
                      "description": "Find country",
                      "args": [
                        {
                          "name": "code",
                          "description": "Country code",
                          "type": {
                            "kind": "NON_NULL",
                            "name": null,
                            "ofType": {"kind": "SCALAR", "name": "ID", "ofType": null}
                          },
                          "defaultValue": null
                        }
                      ],
                      "type": {"kind": "OBJECT", "name": "Country", "ofType": null},
                      "isDeprecated": false,
                      "deprecationReason": null
                    }
                  ],
                  "inputFields": null,
                  "interfaces": [],
                  "enumValues": null,
                  "possibleTypes": null
                },
                {
                  "kind": "OBJECT",
                  "name": "Country",
                  "description": null,
                  "fields": [
                    {
                      "name": "name",
                      "description": null,
                      "args": [],
                      "type": {"kind": "SCALAR", "name": "String", "ofType": null},
                      "isDeprecated": false,
                      "deprecationReason": null
                    }
                  ],
                  "inputFields": null,
                  "interfaces": [],
                  "enumValues": null,
                  "possibleTypes": null
                }
              ]
            }
          }
        }"#;

        let schema = parse_graphql_schema_response(body).unwrap();
        assert_eq!(schema.query_type, Some("Query".to_string()));
        assert_eq!(schema.mutation_type, Some("Mutation".to_string()));
        assert_eq!(schema.types.len(), 2);
        assert_eq!(schema.types[0].fields[0].name, "country");
        assert_eq!(
            schema.types[0].fields[0].args[0].value_type.kind,
            "NON_NULL"
        );
    }

    #[test]
    fn test_parse_graphql_schema_response_rejects_errors() {
        let result = parse_graphql_schema_response(
            r#"{"errors":[{"message":"Introspection is disabled"}],"data":null}"#,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Introspection is disabled"));
    }
}
