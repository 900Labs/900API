use serde::de::DeserializeOwned;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn load_json_or_default<T>(path: &Path) -> io::Result<T>
where
    T: DeserializeOwned + Default,
{
    recover_previous_file(path)?;
    if !path.exists() {
        return Ok(T::default());
    }

    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(T::default());
    }

    match serde_json::from_str(&content) {
        Ok(value) => Ok(value),
        Err(error) => {
            let backup = corrupt_backup_path(path);
            std::fs::rename(path, &backup)?;
            log::error!(
                "Malformed optional state was moved to {}: {}",
                backup.display(),
                error
            );
            Ok(T::default())
        }
    }
}

pub fn atomic_write(path: &Path, content: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let temp = sibling_path(path, &format!("tmp-{}", uuid::Uuid::new_v4()));
    let previous = sibling_path(path, "previous");
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)?;
    if let Err(error) = (|| {
        file.write_all(content)?;
        file.sync_all()?;
        Ok::<(), io::Error>(())
    })() {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    drop(file);

    if path.exists() {
        let _ = std::fs::remove_file(&previous);
        std::fs::rename(path, &previous)?;
    }

    if let Err(error) = std::fs::rename(&temp, path) {
        if previous.exists() {
            let _ = std::fs::rename(&previous, path);
        }
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    let _ = std::fs::remove_file(previous);
    Ok(())
}

fn recover_previous_file(path: &Path) -> io::Result<()> {
    let previous = sibling_path(path, "previous");
    if !path.exists() && previous.exists() {
        std::fs::rename(previous, path)?;
    } else if path.exists() && previous.exists() {
        std::fs::remove_file(previous)?;
    }
    Ok(())
}

fn sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state.json");
    path.with_file_name(format!("{}.{}", file_name, suffix))
}

fn corrupt_backup_path(path: &Path) -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    sibling_path(
        path,
        &format!("corrupt-{}-{}", timestamp, uuid::Uuid::new_v4()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    struct State {
        value: String,
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("900api-{}-{}.json", name, uuid::Uuid::new_v4()))
    }

    #[test]
    fn atomic_write_replaces_existing_state() {
        let path = temp_path("atomic-state");
        atomic_write(&path, br#"{"value":"first"}"#).unwrap();
        atomic_write(&path, br#"{"value":"second"}"#).unwrap();
        let loaded: State = load_json_or_default(&path).unwrap();
        assert_eq!(loaded.value, "second");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn malformed_state_is_backed_up_and_defaults_are_loaded() {
        let path = temp_path("corrupt-state");
        std::fs::write(&path, b"{broken").unwrap();

        let loaded: State = load_json_or_default(&path).unwrap();
        assert_eq!(loaded, State::default());
        assert!(!path.exists());
        let prefix = format!(
            "{}.",
            path.file_name().and_then(|name| name.to_str()).unwrap()
        );
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
