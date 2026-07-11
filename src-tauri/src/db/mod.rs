use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
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

#[derive(Debug, PartialEq, Eq)]
enum ParentValidation {
    Valid,
    Missing,
    Cycle,
}

fn validate_collection_parent(
    conn: &Connection,
    collection_id: &str,
    parent_id: &str,
) -> Result<ParentValidation, DbError> {
    let mut current = Some(parent_id.to_string());
    let mut visited = HashSet::new();

    while let Some(candidate) = current {
        if candidate == collection_id || !visited.insert(candidate.clone()) {
            return Ok(ParentValidation::Cycle);
        }

        let parent = conn
            .query_row(
                "SELECT parent_id FROM collections WHERE id = ?1",
                params![candidate],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?;
        match parent {
            Some(next) => current = next,
            None => return Ok(ParentValidation::Missing),
        }
    }

    Ok(ParentValidation::Valid)
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
                settings TEXT DEFAULT '{}',
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

            CREATE TABLE IF NOT EXISTS response_examples (
                id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                name TEXT NOT NULL,
                status INTEGER NOT NULL,
                status_text TEXT NOT NULL,
                headers TEXT DEFAULT '{}',
                body TEXT DEFAULT '',
                time_ms INTEGER DEFAULT 0,
                size_bytes INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_requests_collection ON requests(collection_id);
            CREATE INDEX IF NOT EXISTS idx_history_created ON history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_response_examples_request ON response_examples(request_id, created_at DESC);
            ",
        )?;
        self.ensure_column(
            "collections",
            "parent_id",
            "ALTER TABLE collections ADD COLUMN parent_id TEXT",
        )?;
        self.ensure_column(
            "collections",
            "sort_order",
            "ALTER TABLE collections ADD COLUMN sort_order INTEGER DEFAULT 0",
        )?;
        self.ensure_column(
            "requests",
            "settings",
            "ALTER TABLE requests ADD COLUMN settings TEXT DEFAULT '{}'",
        )?;
        Ok(())
    }

    fn ensure_column(&self, table: &str, column: &str, alter_sql: &str) -> Result<(), DbError> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({})", table))?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for existing in columns {
            if existing? == column {
                return Ok(());
            }
        }
        self.conn.execute(alter_sql, [])?;
        Ok(())
    }

    pub fn list_collections(&self) -> Result<Vec<crate::models::Collection>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, parent_id, sort_order, created_at, updated_at FROM collections ORDER BY parent_id, sort_order, name")?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                parent_id: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut collections = Vec::new();
        for row in rows {
            collections.push(row?);
        }
        Ok(collections)
    }

    pub fn get_collection(&self, id: &str) -> Result<crate::models::Collection, DbError> {
        self.conn
            .query_row(
                "SELECT id, name, description, parent_id, sort_order, created_at, updated_at FROM collections WHERE id = ?1",
                params![id],
                |row| {
                    Ok(crate::models::Collection {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        parent_id: row.get(3)?,
                        sort_order: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| DbError::NotFound(format!("Collection {}", id)))
    }

    pub fn import_collection_file(
        &self,
        imported: &api900_core::format::CollectionFile,
    ) -> Result<crate::models::Collection, DbError> {
        let transaction = self.conn.unchecked_transaction()?;
        let id = imported
            .id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().to_rfc3339();
        let created_at = transaction
            .query_row(
                "SELECT created_at FROM collections WHERE id = ?1",
                params![id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_else(|| now.clone());
        let parent_id = match imported
            .parent_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
        {
            Some(parent_id)
                if validate_collection_parent(&transaction, &id, parent_id)?
                    == ParentValidation::Valid =>
            {
                Some(parent_id.to_string())
            }
            _ => None,
        };

        transaction.execute(
            "INSERT INTO collections (id, name, description, parent_id, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               description = excluded.description,
               parent_id = excluded.parent_id,
               sort_order = excluded.sort_order,
               updated_at = excluded.updated_at",
            params![
                id,
                imported.name,
                imported.description,
                parent_id,
                imported.sort_order,
                created_at,
                now
            ],
        )?;
        transaction.execute("DELETE FROM requests WHERE collection_id = ?1", params![id])?;

        for (index, request) in imported.requests.iter().enumerate() {
            let request_id = request
                .id
                .as_deref()
                .filter(|request_id| !request_id.trim().is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let sort_order = if request.sort_order == 0 {
                index as i32 + 1
            } else {
                request.sort_order
            };
            let headers = serde_json::to_string(&request.headers)
                .map_err(|error| DbError::NotFound(error.to_string()))?;
            let params_json = serde_json::to_string(&request.params)
                .map_err(|error| DbError::NotFound(error.to_string()))?;
            let auth_config = api900_core::format::value_to_json(&request.auth_config);
            let settings = api900_core::format::value_to_json(&request.settings);

            transaction.execute(
                "INSERT INTO requests (
                    id, collection_id, name, method, url, headers, params, body_type, body,
                    auth_type, auth_config, pre_request_script, test_script, settings,
                    sort_order, created_at, updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17
                 )",
                params![
                    request_id,
                    id,
                    request.name,
                    request.method.to_ascii_uppercase(),
                    request.url,
                    headers,
                    params_json,
                    request.body_type,
                    request.body,
                    request.auth_type,
                    auth_config,
                    request.pre_request_script,
                    request.test_script,
                    settings,
                    sort_order,
                    now,
                    now
                ],
            )?;

            for example in &request.response_examples {
                transaction.execute(
                    "INSERT INTO response_examples (
                        id, request_id, name, status, status_text, headers, body,
                        time_ms, size_bytes, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        request_id,
                        example.name,
                        example.status,
                        example.status_text,
                        api900_core::format::value_to_json(&example.headers),
                        example.body,
                        example.time_ms,
                        example.size_bytes,
                        now
                    ],
                )?;
            }
        }

        transaction.commit()?;
        Ok(crate::models::Collection {
            id,
            name: imported.name.clone(),
            description: imported.description.clone(),
            parent_id,
            sort_order: imported.sort_order,
            created_at,
            updated_at: now,
        })
    }

    pub fn create_collection(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<crate::models::Collection, DbError> {
        self.create_collection_with_parent(name, description, None)
    }

    pub fn create_collection_with_parent(
        &self,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<crate::models::Collection, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM collections WHERE parent_id IS ?1",
            params![parent_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO collections (id, name, description, parent_id, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, name, description, parent_id, sort_order, now, now],
        )?;
        Ok(crate::models::Collection {
            id,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            parent_id: parent_id.map(|s| s.to_string()),
            sort_order,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn update_collection(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE collections SET name = ?2, description = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, name, description, now],
        )?;
        Ok(())
    }

    pub fn move_collection(&self, id: &str, parent_id: Option<&str>) -> Result<(), DbError> {
        if let Some(parent) = parent_id {
            match validate_collection_parent(&self.conn, id, parent)? {
                ParentValidation::Valid => {}
                ParentValidation::Missing => {
                    return Err(DbError::NotFound(format!("Collection {}", parent)));
                }
                ParentValidation::Cycle if parent == id => {
                    return Err(DbError::NotFound(
                        "A collection cannot be moved into itself".to_string(),
                    ));
                }
                ParentValidation::Cycle => {
                    return Err(DbError::NotFound(
                        "A collection cannot be moved into one of its descendants".to_string(),
                    ));
                }
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let sort_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM collections WHERE parent_id IS ?1",
            params![parent_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "UPDATE collections SET parent_id = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, parent_id, sort_order, now],
        )?;
        Ok(())
    }

    pub fn delete_collection(&self, id: &str) -> Result<(), DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM collections WHERE parent_id = ?1")?;
        let child_rows = stmt.query_map(params![id], |row| row.get::<_, String>(0))?;
        let mut child_ids = Vec::new();
        for child in child_rows {
            child_ids.push(child?);
        }
        drop(stmt);

        for child_id in child_ids {
            self.delete_collection(&child_id)?;
        }
        self.conn
            .execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_environments(&self) -> Result<Vec<crate::models::Environment>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, variables, created_at, updated_at FROM environments ORDER BY name",
        )?;
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
        self.conn
            .execute("DELETE FROM environments WHERE id = ?1", params![id])?;
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
        self.add_history_with_snapshot(method, url, status, time_ms, size_bytes, "{}")
    }

    pub fn add_history_with_snapshot(
        &self,
        method: &str,
        url: &str,
        status: u16,
        time_ms: u64,
        size_bytes: usize,
        request_snapshot: &str,
    ) -> Result<(), DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO history (id, method, url, status, time_ms, size_bytes, request_snapshot, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, method, url, status, time_ms, size_bytes, request_snapshot, now],
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
            "SELECT id, method, url, status, time_ms, size_bytes, request_snapshot, created_at FROM history ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(crate::models::HistoryEntry {
                id: row.get(0)?,
                method: row.get(1)?,
                url: row.get(2)?,
                status: row.get(3)?,
                time_ms: row.get(4)?,
                size_bytes: row.get(5)?,
                request_snapshot: row.get(6)?,
                created_at: row.get(7)?,
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

    pub fn list_requests(
        &self,
        collection_id: &str,
    ) -> Result<Vec<crate::models::SavedRequest>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, collection_id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, settings, sort_order, created_at, updated_at
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
                settings: row.get(13)?,
                sort_order: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
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
        self.create_request_with_settings(
            collection_id,
            name,
            method,
            url,
            headers,
            params,
            body_type,
            body,
            auth_type,
            auth_config,
            pre_request_script,
            test_script,
            "{}",
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_request_with_settings(
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
        settings: &str,
    ) -> Result<crate::models::SavedRequest, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM requests WHERE collection_id = ?1",
            params![collection_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO requests (id, collection_id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, settings, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![id, collection_id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, settings, sort_order, now, now],
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
            settings: settings.to_string(),
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
        self.update_request_with_settings(
            id,
            name,
            method,
            url,
            headers,
            params,
            body_type,
            body,
            auth_type,
            auth_config,
            pre_request_script,
            test_script,
            "{}",
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_request_with_settings(
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
        settings: &str,
    ) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE requests SET name = ?2, method = ?3, url = ?4, headers = ?5, params = ?6, body_type = ?7, body = ?8, auth_type = ?9, auth_config = ?10, pre_request_script = ?11, test_script = ?12, settings = ?13, updated_at = ?14 WHERE id = ?1",
            params![id, name, method, url, headers, params, body_type, body, auth_type, auth_config, pre_request_script, test_script, settings, now],
        )?;
        Ok(())
    }

    pub fn delete_request(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM requests WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_response_examples(
        &self,
        request_id: &str,
    ) -> Result<Vec<crate::models::ResponseExample>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, request_id, name, status, status_text, headers, body, time_ms, size_bytes, created_at
             FROM response_examples WHERE request_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![request_id], |row| {
            Ok(crate::models::ResponseExample {
                id: row.get(0)?,
                request_id: row.get(1)?,
                name: row.get(2)?,
                status: row.get(3)?,
                status_text: row.get(4)?,
                headers: row.get(5)?,
                body: row.get(6)?,
                time_ms: row.get(7)?,
                size_bytes: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        let mut examples = Vec::new();
        for row in rows {
            examples.push(row?);
        }
        Ok(examples)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_response_example(
        &self,
        request_id: &str,
        name: &str,
        status: u16,
        status_text: &str,
        headers: &str,
        body: &str,
        time_ms: u64,
        size_bytes: usize,
    ) -> Result<crate::models::ResponseExample, DbError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO response_examples (id, request_id, name, status, status_text, headers, body, time_ms, size_bytes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id, request_id, name, status, status_text, headers, body, time_ms, size_bytes, now],
        )?;
        Ok(crate::models::ResponseExample {
            id,
            request_id: request_id.to_string(),
            name: name.to_string(),
            status,
            status_text: status_text.to_string(),
            headers: headers.to_string(),
            body: body.to_string(),
            time_ms,
            size_bytes,
            created_at: now,
        })
    }

    pub fn delete_response_example(&self, id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM response_examples WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn move_request(&self, id: &str, collection_id: &str) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM requests WHERE collection_id = ?1",
            params![collection_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "UPDATE requests SET collection_id = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, collection_id, sort_order, now],
        )?;
        Ok(())
    }

    pub fn update_environment(&self, id: &str, variables: &str) -> Result<(), DbError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE environments SET variables = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, variables, now],
        )?;
        Ok(())
    }
}
