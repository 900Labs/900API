use api900_core::format::{parse_collection, to_pretty_json, CollectionFile};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Sync error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Collection format error: {0}")]
    Format(#[from] api900_core::format::FormatError),
    #[error("No sync directory configured")]
    NoSyncDir,
    #[error("Git error: {0}")]
    Git(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncConfig {
    pub directory: String,
    pub auto_sync: bool,
    pub author_name: String,
    pub author_email: String,
}

pub struct SyncManager {
    config: Mutex<Option<SyncConfig>>,
    storage_path: Mutex<Option<PathBuf>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(None),
            storage_path: Mutex::new(None),
        }
    }

    pub fn set_storage_path(&self, path: PathBuf) -> Result<(), SyncError> {
        let config: Option<SyncConfig> = crate::persistence::load_json_or_default(&path)?;
        *self.config.lock().unwrap_or_else(|e| e.into_inner()) = config;
        *self.storage_path.lock().unwrap_or_else(|e| e.into_inner()) = Some(path);
        Ok(())
    }

    fn persist_config(&self, config: &SyncConfig) -> Result<(), SyncError> {
        let path = self
            .storage_path
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if let Some(path) = path {
            let json = serde_json::to_string_pretty(config)?;
            crate::persistence::atomic_write(&path, json.as_bytes())?;
        }
        Ok(())
    }

    pub fn set_config(&self, config: SyncConfig) -> Result<(), SyncError> {
        self.persist_config(&config)?;
        *self.config.lock().unwrap_or_else(|e| e.into_inner()) = Some(config);
        Ok(())
    }

    pub fn get_config(&self) -> Option<SyncConfig> {
        self.config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn export_collection(&self, collection: &CollectionFile) -> Result<PathBuf, SyncError> {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;

        let dir = PathBuf::from(&config.directory);
        std::fs::create_dir_all(&dir)?;

        let safe_name = collection
            .name
            .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let stable_id = collection
            .id
            .as_deref()
            .unwrap_or("collection")
            .chars()
            .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
            .take(12)
            .collect::<String>();
        let file_path = dir.join(format!("{}-{}.json", safe_name, stable_id));

        let json = to_pretty_json(collection)?;
        crate::persistence::atomic_write(&file_path, json.as_bytes())?;

        Ok(file_path)
    }

    pub fn import_collection(&self, name: &str) -> Result<CollectionFile, SyncError> {
        if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
            return Err(SyncError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid synced collection name",
            )));
        }
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;
        let file_path = PathBuf::from(&config.directory).join(format!("{}.json", name));
        let content = std::fs::read_to_string(file_path)?;
        Ok(parse_collection(&content)?)
    }

    pub fn list_collections(&self) -> Result<Vec<String>, SyncError> {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;

        let dir = PathBuf::from(&config.directory);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    files.push(name.to_string());
                }
            }
        }
        files.sort();
        Ok(files)
    }

    pub fn git_init(&self) -> Result<(), SyncError> {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;

        let dir = PathBuf::from(&config.directory);
        std::fs::create_dir_all(&dir)?;

        let output = std::process::Command::new("git")
            .arg("init")
            .current_dir(&dir)
            .output()
            .map_err(|e| SyncError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(SyncError::Git(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub fn git_status(&self) -> Result<GitStatus, SyncError> {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;

        let dir = PathBuf::from(&config.directory);

        if !dir.exists() {
            return Ok(GitStatus {
                is_repo: false,
                changed_files: Vec::new(),
            });
        }

        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&dir)
            .output()
            .map_err(|e| SyncError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(SyncError::Git(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let changed: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let is_repo = !changed.is_empty() || {
            let check = std::process::Command::new("git")
                .args(["rev-parse", "--is-inside-work-tree"])
                .current_dir(&dir)
                .output()
                .map_err(|e| SyncError::Git(e.to_string()))?;
            check.status.success()
        };

        Ok(GitStatus {
            is_repo,
            changed_files: changed,
        })
    }

    pub fn git_commit(&self, message: &str) -> Result<(), SyncError> {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;

        let dir = PathBuf::from(&config.directory);

        // Set author if provided
        if !config.author_name.is_empty() {
            let _ = std::process::Command::new("git")
                .args(["config", "user.name", &config.author_name])
                .current_dir(&dir)
                .output();
        }
        if !config.author_email.is_empty() {
            let _ = std::process::Command::new("git")
                .args(["config", "user.email", &config.author_email])
                .current_dir(&dir)
                .output();
        }

        // Add all
        let add_output = std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&dir)
            .output()
            .map_err(|e| SyncError::Git(e.to_string()))?;

        if !add_output.status.success() {
            return Err(SyncError::Git(
                String::from_utf8_lossy(&add_output.stderr).to_string(),
            ));
        }

        // Commit
        let commit_output = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&dir)
            .output()
            .map_err(|e| SyncError::Git(e.to_string()))?;

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            let stdout = String::from_utf8_lossy(&commit_output.stdout);
            // "nothing to commit" is not an error
            if stderr.contains("nothing to commit") || stdout.contains("nothing to commit") {
                return Ok(());
            }
            return Err(SyncError::Git(stderr.to_string()));
        }

        Ok(())
    }

    pub fn git_pull(&self) -> Result<String, SyncError> {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;

        let dir = PathBuf::from(&config.directory);

        let output = std::process::Command::new("git")
            .args(["pull", "--rebase"])
            .current_dir(&dir)
            .output()
            .map_err(|e| SyncError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(SyncError::Git(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn git_push(&self) -> Result<String, SyncError> {
        let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
        let config = config.as_ref().ok_or(SyncError::NoSyncDir)?;

        let dir = PathBuf::from(&config.directory);

        let output = std::process::Command::new("git")
            .args(["push"])
            .current_dir(&dir)
            .output()
            .map_err(|e| SyncError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(SyncError::Git(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub is_repo: bool,
    pub changed_files: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_manager_new() {
        let manager = SyncManager::new();
        assert!(manager.get_config().is_none());
    }

    #[test]
    fn test_sync_manager_set_get_config() {
        let manager = SyncManager::new();
        let config = SyncConfig {
            directory: "/tmp/test-sync".to_string(),
            auto_sync: false,
            author_name: "Test".to_string(),
            author_email: "test@test.com".to_string(),
        };
        manager.set_config(config.clone()).unwrap();
        let retrieved = manager.get_config().unwrap();
        assert_eq!(retrieved.directory, "/tmp/test-sync");
    }

    #[test]
    fn test_export_collection_serialization() {
        let collection = CollectionFile {
            schema: api900_core::format::COLLECTION_SCHEMA.to_string(),
            id: None,
            name: "Test API".to_string(),
            description: Some("Test description".to_string()),
            parent_id: None,
            sort_order: 0,
            requests: vec![],
            exported_at: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&collection).unwrap();
        let deserialized: CollectionFile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test API");
    }

    #[test]
    fn test_no_sync_dir_error() {
        let manager = SyncManager::new();
        let result = manager.list_collections();
        assert!(matches!(result, Err(SyncError::NoSyncDir)));
    }

    #[test]
    fn malformed_config_is_backed_up_without_blocking_initialization() {
        let path =
            std::env::temp_dir().join(format!("900api-sync-config-{}.json", uuid::Uuid::new_v4()));
        std::fs::write(&path, "{broken").unwrap();

        let manager = SyncManager::new();
        manager.set_storage_path(path.clone()).unwrap();

        assert!(manager.get_config().is_none());
        assert!(!path.exists());
        let prefix = format!("{}.", path.file_name().unwrap().to_string_lossy());
        let backups = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        for backup in backups {
            let _ = std::fs::remove_file(backup.path());
        }
    }
}
