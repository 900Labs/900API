use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    #[serde(rename = "GET", alias = "get", alias = "g_e_t")]
    GET,
    #[serde(rename = "POST", alias = "post", alias = "p_o_s_t")]
    POST,
    #[serde(rename = "PUT", alias = "put", alias = "p_u_t")]
    PUT,
    #[serde(rename = "PATCH", alias = "patch", alias = "p_a_t_c_h")]
    PATCH,
    #[serde(rename = "DELETE", alias = "delete", alias = "d_e_l_e_t_e")]
    DELETE,
    #[serde(rename = "HEAD", alias = "head", alias = "h_e_a_d")]
    HEAD,
    #[serde(rename = "OPTIONS", alias = "options", alias = "o_p_t_i_o_n_s")]
    OPTIONS,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "PATCH" => Ok(HttpMethod::PATCH),
            "DELETE" => Ok(HttpMethod::DELETE),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            _ => Err(format!("Unknown HTTP method: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    None,
    Json,
    FormData,
    XWwwFormUrlencoded,
    Raw,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AuthType {
    #[default]
    None,
    Basic,
    Bearer,
    ApiKey,
    OAuth2,
    OAuth1,
    AwsSigV4,
    Hawk,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub auth_type: AuthType,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_key_name: String,
    #[serde(default)]
    pub api_key_in: String, // "header" or "query"
    // OAuth 2.0
    #[serde(default)]
    pub oauth2_access_token: String,
    #[serde(default)]
    pub oauth2_token_type: String,
    #[serde(default)]
    pub oauth2_refresh_token: String,
    // OAuth 1.0a
    #[serde(default)]
    pub oauth1_consumer_key: String,
    #[serde(default)]
    pub oauth1_consumer_secret: String,
    #[serde(default)]
    pub oauth1_token: String,
    #[serde(default)]
    pub oauth1_token_secret: String,
    // AWS Sig v4
    #[serde(default)]
    pub aws_access_key_id: String,
    #[serde(default)]
    pub aws_secret_access_key: String,
    #[serde(default)]
    pub aws_region: String,
    #[serde(default)]
    pub aws_service: String,
    // Hawk
    #[serde(default)]
    pub hawk_id: String,
    #[serde(default)]
    pub hawk_key: String,
    #[serde(default)]
    pub hawk_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfig {
    pub method: HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<KeyValue>,
    #[serde(default)]
    pub params: Vec<KeyValue>,
    pub body_type: BodyType,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub settings: RequestSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestSettings {
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_connect_timeout_ms")]
    pub connect_timeout_ms: u64,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    #[serde(default = "default_true")]
    pub verify_ssl: bool,
    #[serde(default)]
    pub proxy_url: String,
    #[serde(default)]
    pub use_cookie_jar: bool,
}

impl Default for RequestSettings {
    fn default() -> Self {
        Self {
            timeout_ms: default_timeout_ms(),
            connect_timeout_ms: default_connect_timeout_ms(),
            follow_redirects: true,
            verify_ssl: true,
            proxy_url: String::new(),
            use_cookie_jar: false,
        }
    }
}

fn default_timeout_ms() -> u64 {
    120_000
}

fn default_connect_timeout_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    pub time_ms: u64,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseExample {
    pub id: String,
    pub request_id: String,
    pub name: String,
    pub status: u16,
    pub status_text: String,
    pub headers: String,
    pub body: String,
    pub time_ms: u64,
    pub size_bytes: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub variables: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub time_ms: u64,
    pub size_bytes: usize,
    pub request_snapshot: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRequest {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,
    pub params: String,
    pub body_type: String,
    pub body: String,
    pub auth_type: String,
    pub auth_config: String,
    pub pre_request_script: String,
    pub test_script: String,
    pub settings: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(default)]
    pub variables: String,
    #[serde(default)]
    pub operation_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse {
    pub data: Option<serde_json::Value>,
    pub errors: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLSchema {
    pub query_type: Option<String>,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
    pub types: Vec<GraphQLSchemaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLSchemaType {
    pub kind: String,
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<GraphQLField>,
    pub input_fields: Vec<GraphQLInputValue>,
    pub enum_values: Vec<GraphQLEnumValue>,
    pub possible_types: Vec<GraphQLTypeRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLField {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub args: Vec<GraphQLInputValue>,
    pub field_type: GraphQLTypeRef,
    #[serde(default)]
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLInputValue {
    pub name: String,
    pub description: Option<String>,
    pub value_type: GraphQLTypeRef,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLEnumValue {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLTypeRef {
    pub kind: String,
    pub name: Option<String>,
    pub of_type: Option<Box<GraphQLTypeRef>>,
}
