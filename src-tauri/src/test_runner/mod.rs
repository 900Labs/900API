use crate::http;
use crate::models::{AuthConfig, HttpMethod, KeyValue, RequestConfig, ResponseData};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum TestRunnerError {
    #[error("Test runner error: {0}")]
    Http(String),
    #[error("Script error: {0}")]
    Script(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub id: String,
    pub assertion_type: AssertionType,
    pub target: String,
    pub operator: AssertionOperator,
    pub expected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionType {
    Status,
    Header,
    Body,
    BodyJsonPath,
    ResponseTime,
    BodyContains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    Exists,
    NotExists,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub id: String,
    pub name: String,
    pub request: TestRequest,
    pub assertions: Vec<Assertion>,
    pub pre_request_script: String,
    pub test_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<KeyValue>,
    pub params: Vec<KeyValue>,
    pub body_type: String,
    pub body: String,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    pub suite_id: String,
    pub suite_name: String,
    pub passed: bool,
    pub assertions: Vec<AssertionResult>,
    pub response_status: u16,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion_id: String,
    pub passed: bool,
    pub actual: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub results: Vec<TestSuiteResult>,
}

impl TestRequest {
    fn to_request_config(
        &self,
        env_vars: &[crate::models::EnvironmentVariable],
    ) -> Result<RequestConfig, String> {
        let method = match self.method.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            other => return Err(format!("Unsupported HTTP method: {other}")),
        };

        let body_type = match self.body_type.as_str() {
            "json" => crate::models::BodyType::Json,
            "form_data" => crate::models::BodyType::FormData,
            "x_www_form_urlencoded" => crate::models::BodyType::XWwwFormUrlencoded,
            "raw" => crate::models::BodyType::Raw,
            "none" | "" => crate::models::BodyType::None,
            other => return Err(format!("Unsupported request body type: {other}")),
        };

        Ok(RequestConfig {
            method,
            url: crate::http::variables::resolve_variables(&self.url, env_vars),
            headers: crate::http::variables::resolve_key_values(&self.headers, env_vars),
            params: crate::http::variables::resolve_key_values(&self.params, env_vars),
            body_type,
            body: crate::http::variables::resolve_variables(&self.body, env_vars),
            auth: crate::http::variables::resolve_auth_config(&self.auth, env_vars),
            settings: Default::default(),
        })
    }
}

pub async fn run_test_suite(
    suite: &TestSuite,
    env_vars: &[crate::models::EnvironmentVariable],
) -> TestSuiteResult {
    if !suite.pre_request_script.trim().is_empty() {
        match crate::scripting::run_test_script(&suite.pre_request_script, "", 0, "{}") {
            Ok(output) => {
                if let Some(error) = output.error {
                    return TestSuiteResult {
                        suite_id: suite.id.clone(),
                        suite_name: suite.name.clone(),
                        passed: false,
                        assertions: vec![],
                        response_status: 0,
                        response_time_ms: 0,
                        error: Some(format!("Pre-request script failed: {}", error)),
                    };
                }
            }
            Err(e) => {
                return TestSuiteResult {
                    suite_id: suite.id.clone(),
                    suite_name: suite.name.clone(),
                    passed: false,
                    assertions: vec![],
                    response_status: 0,
                    response_time_ms: 0,
                    error: Some(format!("Pre-request script failed: {}", e)),
                };
            }
        }
    }

    let config = match suite.request.to_request_config(env_vars) {
        Ok(config) => config,
        Err(error) => {
            return TestSuiteResult {
                suite_id: suite.id.clone(),
                suite_name: suite.name.clone(),
                passed: false,
                assertions: vec![],
                response_status: 0,
                response_time_ms: 0,
                error: Some(error),
            }
        }
    };

    let response = match http::send_request(&config).await {
        Ok(r) => r,
        Err(e) => {
            return TestSuiteResult {
                suite_id: suite.id.clone(),
                suite_name: suite.name.clone(),
                passed: false,
                assertions: vec![],
                response_status: 0,
                response_time_ms: 0,
                error: Some(e.to_string()),
            };
        }
    };

    let status = response.status;
    let time_ms = response.time_ms;

    let mut assertion_results = Vec::new();
    for assertion in &suite.assertions {
        let result = evaluate_assertion(assertion, &response);
        assertion_results.push(result);
    }

    let script_error = if suite.test_script.trim().is_empty() {
        None
    } else {
        let response_headers =
            serde_json::to_string(&response.headers).unwrap_or_else(|_| "{}".to_string());
        match crate::scripting::run_test_script(
            &suite.test_script,
            &response.body,
            response.status,
            &response_headers,
        ) {
            Ok(output) => output.error.map(|e| format!("Test script failed: {}", e)),
            Err(e) => Some(format!("Test script failed: {}", e)),
        }
    };

    let all_passed = assertion_results.iter().all(|r| r.passed) && script_error.is_none();

    TestSuiteResult {
        suite_id: suite.id.clone(),
        suite_name: suite.name.clone(),
        passed: all_passed,
        assertions: assertion_results,
        response_status: status,
        response_time_ms: time_ms,
        error: script_error,
    }
}

pub async fn run_test_suites(
    suites: Vec<TestSuite>,
    env_vars: &[crate::models::EnvironmentVariable],
) -> TestRunResult {
    let start = std::time::Instant::now();
    let mut results = Vec::with_capacity(suites.len());

    for suite in &suites {
        let result = run_test_suite(suite, env_vars).await;
        results.push(result);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    TestRunResult {
        total,
        passed,
        failed,
        duration_ms,
        results,
    }
}

fn evaluate_assertion(assertion: &Assertion, response: &ResponseData) -> AssertionResult {
    let actual = get_assertion_value(assertion, response);
    let passed = check_assertion(assertion, &actual);

    let message = if passed {
        format!(
            "Expected {} {} {}",
            assertion.target,
            format_operator(&assertion.operator),
            assertion.expected
        )
    } else {
        format!(
            "Expected {} {} '{}' but got '{}'",
            assertion.target,
            format_operator(&assertion.operator),
            assertion.expected,
            actual
        )
    };

    AssertionResult {
        assertion_id: assertion.id.clone(),
        passed,
        actual,
        message,
    }
}

fn get_assertion_value(assertion: &Assertion, response: &ResponseData) -> String {
    match assertion.assertion_type {
        AssertionType::Status => response.status.to_string(),
        AssertionType::Header => response
            .headers
            .get(&assertion.target)
            .cloned()
            .unwrap_or_default(),
        AssertionType::Body => response.body.clone(),
        AssertionType::BodyContains => response.body.clone(),
        AssertionType::BodyJsonPath => {
            let body: serde_json::Value =
                serde_json::from_str(&response.body).unwrap_or(serde_json::Value::Null);
            extract_json_path(&body, &assertion.target)
        }
        AssertionType::ResponseTime => response.time_ms.to_string(),
    }
}

fn extract_json_path(value: &serde_json::Value, path: &str) -> String {
    let parts: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
    let mut current = value;

    for part in parts {
        if let Ok(idx) = part.parse::<usize>() {
            if let Some(arr) = current.as_array() {
                if idx < arr.len() {
                    current = &arr[idx];
                } else {
                    return String::new();
                }
            } else {
                return String::new();
            }
        } else if let Some(v) = current.get(part) {
            current = v;
        } else {
            return String::new();
        }
    }

    match current {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn check_assertion(assertion: &Assertion, actual: &str) -> bool {
    match assertion.operator {
        AssertionOperator::Equals => actual == assertion.expected,
        AssertionOperator::NotEquals => actual != assertion.expected,
        AssertionOperator::Contains => actual.contains(&assertion.expected),
        AssertionOperator::NotContains => !actual.contains(&assertion.expected),
        AssertionOperator::GreaterThan => {
            let actual_num: f64 = actual.parse().unwrap_or(0.0);
            let expected_num: f64 = assertion.expected.parse().unwrap_or(0.0);
            actual_num > expected_num
        }
        AssertionOperator::LessThan => {
            let actual_num: f64 = actual.parse().unwrap_or(0.0);
            let expected_num: f64 = assertion.expected.parse().unwrap_or(0.0);
            actual_num < expected_num
        }
        AssertionOperator::Exists => !actual.is_empty(),
        AssertionOperator::NotExists => actual.is_empty(),
    }
}

fn format_operator(operator: &AssertionOperator) -> &'static str {
    match operator {
        AssertionOperator::Equals => "equals",
        AssertionOperator::NotEquals => "not equals",
        AssertionOperator::Contains => "contains",
        AssertionOperator::NotContains => "not contains",
        AssertionOperator::GreaterThan => "greater than",
        AssertionOperator::LessThan => "less than",
        AssertionOperator::Exists => "exists",
        AssertionOperator::NotExists => "not exists",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AuthConfig, AuthType, EnvironmentVariable};

    #[test]
    fn test_extract_json_path_simple() {
        let value = serde_json::json!({"name": "test"});
        assert_eq!(extract_json_path(&value, "name"), "test");
    }

    #[test]
    fn test_extract_json_path_nested() {
        let value = serde_json::json!({"user": {"name": "John"}});
        assert_eq!(extract_json_path(&value, "user.name"), "John");
    }

    #[test]
    fn test_extract_json_path_array() {
        let value = serde_json::json!({"items": ["a", "b", "c"]});
        assert_eq!(extract_json_path(&value, "items.0"), "a");
        assert_eq!(extract_json_path(&value, "items.1"), "b");
    }

    #[test]
    fn test_extract_json_path_missing() {
        let value = serde_json::json!({"name": "test"});
        assert_eq!(extract_json_path(&value, "missing"), "");
    }

    #[test]
    fn test_extract_json_path_number() {
        let value = serde_json::json!({"count": 42});
        assert_eq!(extract_json_path(&value, "count"), "42");
    }

    #[test]
    fn test_check_assertion_equals() {
        let assertion = Assertion {
            id: "1".to_string(),
            assertion_type: AssertionType::Status,
            target: "status".to_string(),
            operator: AssertionOperator::Equals,
            expected: "200".to_string(),
        };
        assert!(check_assertion(&assertion, "200"));
        assert!(!check_assertion(&assertion, "404"));
    }

    #[test]
    fn test_check_assertion_contains() {
        let assertion = Assertion {
            id: "1".to_string(),
            assertion_type: AssertionType::BodyContains,
            target: "body".to_string(),
            operator: AssertionOperator::Contains,
            expected: "hello".to_string(),
        };
        assert!(check_assertion(&assertion, "hello world"));
        assert!(!check_assertion(&assertion, "goodbye"));
    }

    #[test]
    fn test_check_assertion_greater_than() {
        let assertion = Assertion {
            id: "1".to_string(),
            assertion_type: AssertionType::ResponseTime,
            target: "time".to_string(),
            operator: AssertionOperator::GreaterThan,
            expected: "100".to_string(),
        };
        assert!(check_assertion(&assertion, "200"));
        assert!(!check_assertion(&assertion, "50"));
    }

    #[test]
    fn test_check_assertion_exists() {
        let assertion = Assertion {
            id: "1".to_string(),
            assertion_type: AssertionType::Header,
            target: "x-custom".to_string(),
            operator: AssertionOperator::Exists,
            expected: "".to_string(),
        };
        assert!(check_assertion(&assertion, "some-value"));
        assert!(!check_assertion(&assertion, ""));
    }

    #[test]
    fn test_request_config_resolves_environment_variables() {
        let request = TestRequest {
            method: "POST".to_string(),
            url: "{{baseUrl}}/users".to_string(),
            headers: vec![KeyValue {
                key: "Authorization".to_string(),
                value: "Bearer {{token}}".to_string(),
                enabled: true,
            }],
            params: vec![KeyValue {
                key: "q".to_string(),
                value: "{{query}}".to_string(),
                enabled: true,
            }],
            body_type: "json".to_string(),
            body: "{\"name\":\"{{name}}\"}".to_string(),
            auth: AuthConfig {
                auth_type: AuthType::Bearer,
                token: "{{token}}".to_string(),
                ..AuthConfig::default()
            },
        };
        let vars = vec![
            EnvironmentVariable {
                key: "baseUrl".to_string(),
                value: "https://api.example.com".to_string(),
                enabled: true,
            },
            EnvironmentVariable {
                key: "token".to_string(),
                value: "secret".to_string(),
                enabled: true,
            },
            EnvironmentVariable {
                key: "query".to_string(),
                value: "active".to_string(),
                enabled: true,
            },
            EnvironmentVariable {
                key: "name".to_string(),
                value: "Example".to_string(),
                enabled: true,
            },
        ];

        let config = request.to_request_config(&vars).unwrap();
        assert_eq!(config.url, "https://api.example.com/users");
        assert_eq!(config.headers[0].value, "Bearer secret");
        assert_eq!(config.params[0].value, "active");
        assert_eq!(config.body, "{\"name\":\"Example\"}");
        assert_eq!(config.auth.token, "secret");
    }
}
