use crate::models::{AuthConfig, EnvironmentVariable, KeyValue};
use std::collections::HashMap;

/// Resolves `{{variable_name}}` patterns in a string using the provided environment variables.
pub fn resolve_variables(input: &str, variables: &[EnvironmentVariable]) -> String {
    let map: HashMap<&str, &str> = variables
        .iter()
        .filter(|v| v.enabled)
        .map(|v| (v.key.as_str(), v.value.as_str()))
        .collect();

    resolve_with_map(input, &map)
}

/// Resolves `{{variable_name}}` patterns using a simple key-value map.
pub fn resolve_with_map(input: &str, map: &HashMap<&str, &str>) -> String {
    let mut result = input.to_string();
    let mut changed = true;

    // Resolve up to 5 levels of nested variables
    let mut iterations = 0;
    while changed && iterations < 5 {
        changed = false;
        iterations += 1;

        let mut start = 0;
        while let Some(open) = result[start..].find("{{") {
            let open_abs = start + open;
            if let Some(close_rel) = result[open_abs + 2..].find("}}") {
                let close_abs = open_abs + 2 + close_rel;
                let var_name = result[open_abs + 2..close_abs].trim();

                if let Some(value) = map.get(var_name) {
                    result = format!(
                        "{}{}{}",
                        &result[..open_abs],
                        value,
                        &result[close_abs + 2..]
                    );
                    changed = true;
                    start = open_abs + value.len();
                } else {
                    // Variable not found — skip past this occurrence
                    start = close_abs + 2;
                }
            } else {
                break;
            }
        }
    }

    result
}

/// Resolves variables in a list of KeyValue pairs.
pub fn resolve_key_values(items: &[KeyValue], variables: &[EnvironmentVariable]) -> Vec<KeyValue> {
    items
        .iter()
        .map(|kv| KeyValue {
            key: resolve_variables(&kv.key, variables),
            value: resolve_variables(&kv.value, variables),
            enabled: kv.enabled,
        })
        .collect()
}

/// Resolves variables in every string field of an authentication config.
pub fn resolve_auth_config(auth: &AuthConfig, variables: &[EnvironmentVariable]) -> AuthConfig {
    let mut resolved = auth.clone();
    resolved.username = resolve_variables(&auth.username, variables);
    resolved.password = resolve_variables(&auth.password, variables);
    resolved.token = resolve_variables(&auth.token, variables);
    resolved.api_key = resolve_variables(&auth.api_key, variables);
    resolved.api_key_name = resolve_variables(&auth.api_key_name, variables);
    resolved.api_key_in = resolve_variables(&auth.api_key_in, variables);
    resolved.oauth2_access_token = resolve_variables(&auth.oauth2_access_token, variables);
    resolved.oauth2_token_type = resolve_variables(&auth.oauth2_token_type, variables);
    resolved.oauth2_refresh_token = resolve_variables(&auth.oauth2_refresh_token, variables);
    resolved.oauth1_consumer_key = resolve_variables(&auth.oauth1_consumer_key, variables);
    resolved.oauth1_consumer_secret = resolve_variables(&auth.oauth1_consumer_secret, variables);
    resolved.oauth1_token = resolve_variables(&auth.oauth1_token, variables);
    resolved.oauth1_token_secret = resolve_variables(&auth.oauth1_token_secret, variables);
    resolved.aws_access_key_id = resolve_variables(&auth.aws_access_key_id, variables);
    resolved.aws_secret_access_key = resolve_variables(&auth.aws_secret_access_key, variables);
    resolved.aws_region = resolve_variables(&auth.aws_region, variables);
    resolved.aws_service = resolve_variables(&auth.aws_service, variables);
    resolved.hawk_id = resolve_variables(&auth.hawk_id, variables);
    resolved.hawk_key = resolve_variables(&auth.hawk_key, variables);
    resolved.hawk_algorithm = resolve_variables(&auth.hawk_algorithm, variables);
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_resolution() {
        let vars = vec![EnvironmentVariable {
            key: "baseUrl".to_string(),
            value: "https://api.example.com".to_string(),
            enabled: true,
        }];
        let result = resolve_variables("{{baseUrl}}/users", &vars);
        assert_eq!(result, "https://api.example.com/users");
    }

    #[test]
    fn test_no_variables() {
        let result = resolve_variables("https://example.com", &[]);
        assert_eq!(result, "https://example.com");
    }

    #[test]
    fn test_multiple_variables() {
        let vars = vec![
            EnvironmentVariable {
                key: "host".to_string(),
                value: "api.example.com".to_string(),
                enabled: true,
            },
            EnvironmentVariable {
                key: "port".to_string(),
                value: "8080".to_string(),
                enabled: true,
            },
        ];
        let result = resolve_variables("https://{{host}}:{{port}}/api", &vars);
        assert_eq!(result, "https://api.example.com:8080/api");
    }

    #[test]
    fn test_disabled_variable_not_resolved() {
        let vars = vec![EnvironmentVariable {
            key: "secret".to_string(),
            value: "hidden".to_string(),
            enabled: false,
        }];
        let result = resolve_variables("{{secret}}", &vars);
        assert_eq!(result, "{{secret}}");
    }

    #[test]
    fn test_nested_variables() {
        let vars = vec![
            EnvironmentVariable {
                key: "domain".to_string(),
                value: "example.com".to_string(),
                enabled: true,
            },
            EnvironmentVariable {
                key: "baseUrl".to_string(),
                value: "https://{{domain}}".to_string(),
                enabled: true,
            },
        ];
        let result = resolve_variables("{{baseUrl}}/api", &vars);
        assert_eq!(result, "https://example.com/api");
    }

    #[test]
    fn test_variable_in_header_value() {
        let vars = vec![EnvironmentVariable {
            key: "token".to_string(),
            value: "abc123".to_string(),
            enabled: true,
        }];
        let items = vec![KeyValue {
            key: "Authorization".to_string(),
            value: "Bearer {{token}}".to_string(),
            enabled: true,
        }];
        let resolved = resolve_key_values(&items, &vars);
        assert_eq!(resolved[0].value, "Bearer abc123");
    }

    #[test]
    fn test_variable_in_auth_token() {
        let vars = vec![EnvironmentVariable {
            key: "token".to_string(),
            value: "abc123".to_string(),
            enabled: true,
        }];
        let token_placeholder = format!("{{{{{}}}}}", "token");
        let auth = AuthConfig {
            token: token_placeholder,
            ..AuthConfig::default()
        };
        let resolved = resolve_auth_config(&auth, &vars);
        assert_eq!(resolved.token, "abc123");
    }

    #[test]
    fn test_unresolved_variable_preserved() {
        let result = resolve_variables("{{unknown}}/path", &[]);
        assert_eq!(result, "{{unknown}}/path");
    }
}
