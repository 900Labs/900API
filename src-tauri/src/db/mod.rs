use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                parent_id TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS requests (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                name TEXT NOT NULL,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                headers TEXT DEFAULT '[]',
                params TEXT DEFAULT '[]',
                body_type TEXT DEFAULT 'none',
                body TEXT DEFAULT '',
                auth_type TEXT DEFAULT 'none',
                auth_config TEXT DEFAULT '{}',
                pre_request_script TEXT DEFAULT '',
                test_script TEXT DEFAULT '',
                sort_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS environments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                variables TEXT DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER DEFAULT 0,
                time_ms INTEGER DEFAULT 0,
                size_bytes INTEGER DEFAULT 0,
                request_snapshot TEXT DEFAULT '{}',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_requests_collection ON requests(collection_id);
            CREATE INDEX IF NOT EXISTS idx_history_created ON history(created_at DESC);
            ",
        )?;
        Ok(())
    }

    pub fn list_collections(&self) -> Result<Vec<crate::models::Collection>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, created_at, updated_at FROM collections ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        let mut collections = Vec::new();
        for row in rows {
            collections.push(row?);
        }
        Ok(collections)
    }

    pub fn create_collection(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<crate::models::Collection, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO collections (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, description, now, now],
        )?;
        Ok(crate::models::Collection {
            id,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn delete_collection(&self, id: &str) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_environments(&self) -> Result<Vec<crate::models::Environment>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, variables, created_at, updated_at FROM environments ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                variables: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        let mut environments = Vec::new();
        for row in rows {
            environments.push(row?);
        }
        Ok(environments)
    }

    pub fn create_environment(&self, name: &str) -> Result<crate::models::Environment, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO environments (id, name, variables, created_at, updated_at) VALUES (?1, ?2, '[]', ?3, ?4)",
            params![id, name, now, now],
        )?;
        Ok(crate::models::Environment {
            id,
            name: name.to_string(),
            variables: "[]".to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn delete_environment(&self, id: &str) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_history(
        &self,
        method: &str,
        url: &str,
        status: u16,
        time_ms: u64,
        size_bytes: usize,
    ) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO history (id, method, url, status, time_ms, size_bytes, request_snapshot, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '{}', ?7)",
            params![id, method, url, status, time_ms, size_bytes, now],
        )?;
        // Keep only the last 500 entries
        self.conn.execute(
            "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY created_at DESC LIMIT 500)",
            [],
        )?;
        Ok(())
    }

    pub fn list_history(&self, limit: u32) -> Result<Vec<crate::models::HistoryEntry>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, method, url, status, time_ms, created_at FROM history ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(crate::models::HistoryEntry {
                id: row.get(0)?,
                method: row.get(1)?,
                url: row.get(2)?,
                status: row.get(3)?,
                time_ms: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn clear_history(&self) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    pub fn list_requests(&self, collection_id: &str) -> Result<Vec<crate::models::SavedRequest>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, sort_order, created_at, updated_at
             FROM requests WHERE collection_id = ?1 ORDER BY sort_order, name",
        )?;
        let rows = stmt.query_map(params![collection_id], |row| {
            Ok(crate::models::SavedRequest {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                name: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                headers: row.get(5)?,
                params: row.get(6)?,
                body_type: row.get(7)?,
                body: row.get(8)?,
                auth_type: row.get(9)?,
                auth_config: row.get(10)?,
                pre_request_script: row.get(11)?,
                test_script: row.get(12)?,
                sort_order: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?;
        let mut requests = Vec::new();
        for row in rows {
            requests.push(row?);
        }
        Ok(requests)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_request(
        &self,
        collection_id: &str,
        name: &str,
        method: &str,
        url: &str,
        headers: &str,
        params: &str,
        body_type: &str,
        body: &str,
        auth_type: &str,
        auth_config: &str,
        pre_request_script: &str,
        test_script: &str,
    ) -> Result<crate::models::SavedRequest, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM requests WHERE collection_id = ?1",
            params![collection_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO requests (id, collection_id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![id, collection_id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, sort_order, now, now],
        )?;
        Ok(crate::models::SavedRequest {
            id,
            collection_id: collection_id.to_string(),
            name: name.to_string(),
            method: method.to_string(),
            url: url.to_string(),
            headers: headers.to_string(),
            params: params.to_string(),
            body_type: body_type.to_string(),
            body: body.to_string(),
            auth_type: auth_type.to_string(),
            auth_config: auth_config.to_string(),
            pre_request_script: pre_request_script.to_string(),
            test_script: test_script.to_string(),
            sort_order,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_request(
        &self,
        id: &str,
        name: &str,
        method: &str,
        url: &str,
        headers: &str,
        params: &str,
        body_type: &str,
        body: &str,
        auth_type: &str,
        auth_config: &str,
        pre_request_script: &str,
        test_script: &str,
    ) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE requests SET name = ?2, method = ?3, url = ?4, headers = ?5, params = ?6, body_type = ?7, body = ?8, auth_type = ?9, auth_config = ?10, pre_request_script = ?11, test_script = ?12, updated_at = ?13 WHERE id = ?1",
            params![id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, now],
        )?;
        Ok(())
    }

    pub fn delete_request(&self, id: &str) -> Result<(), DbError> {
        self.conn.execute("DELETE FROM requests WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_environment(
        &self,
        id: &str,
        variables: &str,
    ) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE environments SET variables = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, variables, now],
        )?;
        Ok(())
    }
}
