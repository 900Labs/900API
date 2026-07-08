#[cfg(test)]
mod tests {
    use crate::models::{BodyType, HttpMethod, KeyValue, RequestConfig, RequestSettings};

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::GET.to_string(), "GET");
        assert_eq!(HttpMethod::POST.to_string(), "POST");
        assert_eq!(HttpMethod::DELETE.to_string(), "DELETE");
    }

    #[test]
    fn test_http_method_from_str() {
        use std::str::FromStr;
        assert!(matches!(HttpMethod::from_str("get"), Ok(HttpMethod::GET)));
        assert!(matches!(HttpMethod::from_str("POST"), Ok(HttpMethod::POST)));
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[test]
    fn test_request_config_serialization() {
        let config = RequestConfig {
            method: HttpMethod::POST,
            url: "https://api.example.com/users".to_string(),
            headers: vec![KeyValue {
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
                enabled: true,
            }],
            params: vec![],
            body_type: BodyType::Json,
            body: r#"{"name":"test"}"#.to_string(),
            auth: Default::default(),
            settings: RequestSettings::default(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RequestConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.url, config.url);
        assert_eq!(deserialized.headers.len(), 1);
        assert_eq!(deserialized.headers[0].key, "Content-Type");
        assert_eq!(deserialized.settings.timeout_ms, 120_000);
    }

    #[test]
    fn test_request_config_default_settings() {
        let json = r#"{
            "method":"get",
            "url":"https://api.example.com/users",
            "headers":[],
            "params":[],
            "body_type":"none",
            "body":"",
            "auth":{"auth_type":"none"}
        }"#;
        let config: RequestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.settings, RequestSettings::default());
    }

    #[test]
    fn test_key_value_default_enabled() {
        let json = r#"{"key":"x","value":"y"}"#;
        let kv: KeyValue = serde_json::from_str(json).unwrap();
        assert!(kv.enabled);
    }
}
