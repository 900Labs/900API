use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Plugin already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub permissions: Vec<PluginPermission>,
    pub hooks: Vec<PluginHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    Network,
    FileSystem,
    Environment,
    Clipboard,
    Notifications,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginHook {
    BeforeRequest,
    AfterRequest,
    BeforeTest,
    AfterTest,
    OnCollectionLoad,
    OnEnvironmentChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub installed_at: String,
    pub config: HashMap<String, String>,
}

pub struct PluginManager {
    plugins: Mutex<Vec<Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Mutex::new(Vec::new()),
        }
    }

    pub fn list_plugins(&self) -> Vec<Plugin> {
        self.plugins.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_plugin(&self, id: &str) -> Option<Plugin> {
        self.plugins
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .find(|p| p.manifest.id == id)
            .cloned()
    }

    pub fn install_plugin(&self, manifest: PluginManifest) -> Result<Plugin, PluginError> {
        let mut plugins = self.plugins.lock().unwrap_or_else(|e| e.into_inner());

        if plugins.iter().any(|p| p.manifest.id == manifest.id) {
            return Err(PluginError::AlreadyExists(manifest.id));
        }

        if manifest.id.is_empty() || manifest.name.is_empty() {
            return Err(PluginError::InvalidManifest(
                "id and name are required".to_string(),
            ));
        }

        let plugin = Plugin {
            enabled: true,
            installed_at: chrono::Utc::now().to_rfc3339(),
            config: HashMap::new(),
            manifest,
        };

        plugins.push(plugin.clone());
        Ok(plugin)
    }

    pub fn uninstall_plugin(&self, id: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.lock().unwrap_or_else(|e| e.into_inner());
        let len_before = plugins.len();
        plugins.retain(|p| p.manifest.id != id);
        if plugins.len() == len_before {
            return Err(PluginError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn enable_plugin(&self, id: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.lock().unwrap_or_else(|e| e.into_inner());
        let plugin = plugins
            .iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.enabled = true;
        Ok(())
    }

    pub fn disable_plugin(&self, id: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.lock().unwrap_or_else(|e| e.into_inner());
        let plugin = plugins
            .iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.enabled = false;
        Ok(())
    }

    pub fn update_plugin_config(
        &self,
        id: &str,
        config: HashMap<String, String>,
    ) -> Result<(), PluginError> {
        let mut plugins = self.plugins.lock().unwrap_or_else(|e| e.into_inner());
        let plugin = plugins
            .iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.config = config;
        Ok(())
    }

    pub fn get_enabled_plugins_for_hook(&self, hook: &PluginHook) -> Vec<Plugin> {
        self.plugins
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|p| p.enabled && p.manifest.hooks.contains(hook))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest() -> PluginManifest {
        PluginManifest {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            homepage: None,
            permissions: vec![PluginPermission::Network],
            hooks: vec![PluginHook::BeforeRequest, PluginHook::AfterRequest],
        }
    }

    #[test]
    fn test_install_and_list() {
        let manager = PluginManager::new();
        let manifest = test_manifest();
        manager.install_plugin(manifest).unwrap();

        let plugins = manager.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.id, "test-plugin");
        assert!(plugins[0].enabled);
    }

    #[test]
    fn test_install_duplicate() {
        let manager = PluginManager::new();
        manager.install_plugin(test_manifest()).unwrap();
        let result = manager.install_plugin(test_manifest());
        assert!(matches!(result, Err(PluginError::AlreadyExists(_))));
    }

    #[test]
    fn test_uninstall() {
        let manager = PluginManager::new();
        manager.install_plugin(test_manifest()).unwrap();
        manager.uninstall_plugin("test-plugin").unwrap();
        assert_eq!(manager.list_plugins().len(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let manager = PluginManager::new();
        manager.install_plugin(test_manifest()).unwrap();

        manager.disable_plugin("test-plugin").unwrap();
        assert!(!manager.get_plugin("test-plugin").unwrap().enabled);

        manager.enable_plugin("test-plugin").unwrap();
        assert!(manager.get_plugin("test-plugin").unwrap().enabled);
    }

    #[test]
    fn test_get_enabled_for_hook() {
        let manager = PluginManager::new();
        manager.install_plugin(test_manifest()).unwrap();

        let before_request = manager.get_enabled_plugins_for_hook(&PluginHook::BeforeRequest);
        assert_eq!(before_request.len(), 1);

        manager.disable_plugin("test-plugin").unwrap();
        let before_request = manager.get_enabled_plugins_for_hook(&PluginHook::BeforeRequest);
        assert_eq!(before_request.len(), 0);
    }

    #[test]
    fn test_update_config() {
        let manager = PluginManager::new();
        manager.install_plugin(test_manifest()).unwrap();

        let mut config = HashMap::new();
        config.insert("key".to_string(), "value".to_string());
        manager.update_plugin_config("test-plugin", config).unwrap();

        let plugin = manager.get_plugin("test-plugin").unwrap();
        assert_eq!(plugin.config.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_invalid_manifest() {
        let manager = PluginManager::new();
        let manifest = PluginManifest {
            id: "".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            homepage: None,
            permissions: vec![],
            hooks: vec![],
        };
        let result = manager.install_plugin(manifest);
        assert!(matches!(result, Err(PluginError::InvalidManifest(_))));
    }

    #[test]
    fn test_not_found() {
        let manager = PluginManager::new();
        assert!(manager.get_plugin("nonexistent").is_none());
        assert!(matches!(
            manager.uninstall_plugin("nonexistent"),
            Err(PluginError::NotFound(_))
        ));
    }
}
