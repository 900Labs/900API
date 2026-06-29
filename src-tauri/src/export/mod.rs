use crate::models::{Collection, SavedRequest};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedCollection {
    pub name: String,
    pub description: Option<String>,
    pub requests: Vec<ExportedRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportedRequest {
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

pub fn export_collection(
    collection: &Collection,
    requests: &[SavedRequest],
    path: &Path,
) -> Result<(), ExportError> {
    let exported = ExportedCollection {
        name: collection.name.clone(),
        description: collection.description.clone(),
        requests: requests
            .iter()
            .map(|r| ExportedRequest {
                name: r.name.clone(),
                method: r.method.clone(),
                url: r.url.clone(),
                headers: r.headers.clone(),
                params: r.params.clone(),
                body_type: r.body_type.clone(),
                body: r.body.clone(),
                auth_type: r.auth_type.clone(),
                auth_config: r.auth_config.clone(),
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&exported)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn import_collection(path: &Path) -> Result<ExportedCollection, ExportError> {
    let content = std::fs::read_to_string(path)?;
    let imported: ExportedCollection = serde_json::from_str(&content)?;
    Ok(imported)
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
            pre_request_script: "".to_string(),
            test_script: "".to_string(),
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
}
