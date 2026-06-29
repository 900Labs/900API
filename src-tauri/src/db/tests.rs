#[cfg(test)]
mod tests {
    use crate::db::Database;

    fn temp_db(suffix: &str) -> Database {
        let path = std::env::temp_dir().join(format!(
            "900api-test-{}-{}-{}.db",
            std::process::id(),
            suffix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&path);
        Database::open(&path).expect("failed to open test db")
    }

    #[test]
    fn test_create_and_list_collections() {
        let db = temp_db("collections");
        let col = db
            .create_collection("Test Collection", Some("A test"))
            .unwrap();
        assert_eq!(col.name, "Test Collection");
        assert_eq!(col.description, Some("A test".to_string()));

        let collections = db.list_collections().unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].name, "Test Collection");
    }

    #[test]
    fn test_delete_collection() {
        let db = temp_db("delete");
        let col = db.create_collection("To Delete", None).unwrap();
        db.delete_collection(&col.id).unwrap();
        let collections = db.list_collections().unwrap();
        assert_eq!(collections.len(), 0);
    }

    #[test]
    fn test_create_and_list_environments() {
        let db = temp_db("envs");
        let env = db.create_environment("Production").unwrap();
        assert_eq!(env.name, "Production");

        let envs = db.list_environments().unwrap();
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].name, "Production");
    }

    #[test]
    fn test_history_add_and_list() {
        let db = temp_db("history");
        db.add_history("GET", "https://example.com", 200, 50, 1024)
            .unwrap();
        db.add_history("POST", "https://example.com/api", 201, 120, 512)
            .unwrap();

        let history = db.list_history(100).unwrap();
        assert_eq!(history.len(), 2);
        // Most recent first
        assert_eq!(history[0].method, "POST");
        assert_eq!(history[1].method, "GET");
    }

    #[test]
    fn test_clear_history() {
        let db = temp_db("clear");
        db.add_history("GET", "https://example.com", 200, 50, 1024)
            .unwrap();
        db.clear_history().unwrap();
        let history = db.list_history(100).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_create_and_list_requests() {
        let db = temp_db("requests");
        let col = db.create_collection("Test", None).unwrap();
        db.create_request(
            &col.id, "Get Users", "GET", "https://api.example.com/users",
            "[]", "[]", "none", "", "none", "{}", "", "",
        ).unwrap();
        db.create_request(
            &col.id, "Create User", "POST", "https://api.example.com/users",
            "[]", "[]", "json", "{}", "none", "{}", "", "",
        ).unwrap();

        let reqs = db.list_requests(&col.id).unwrap();
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].name, "Get Users");
        assert_eq!(reqs[1].name, "Create User");
    }

    #[test]
    fn test_update_request() {
        let db = temp_db("update_req");
        let col = db.create_collection("Test", None).unwrap();
        let req = db.create_request(
            &col.id, "Original", "GET", "https://example.com",
            "[]", "[]", "none", "", "none", "{}", "", "",
        ).unwrap();

        db.update_request(
            &req.id, "Updated", "POST", "https://example.com/updated",
            "[]", "[]", "json", "{}", "bearer", "{}", "console.log('hi')", "",
        ).unwrap();

        let reqs = db.list_requests(&col.id).unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].name, "Updated");
        assert_eq!(reqs[0].method, "POST");
        assert_eq!(reqs[0].pre_request_script, "console.log('hi')");
    }

    #[test]
    fn test_delete_request() {
        let db = temp_db("delete_req");
        let col = db.create_collection("Test", None).unwrap();
        let req = db.create_request(
            &col.id, "To Delete", "GET", "https://example.com",
            "[]", "[]", "none", "", "none", "{}", "", "",
        ).unwrap();

        db.delete_request(&req.id).unwrap();
        let reqs = db.list_requests(&col.id).unwrap();
        assert_eq!(reqs.len(), 0);
    }

    #[test]
    fn test_update_environment() {
        let db = temp_db("update_env");
        let env = db.create_environment("Test").unwrap();
        db.update_environment(&env.id, r#"[{"key":"url","value":"https://api.example.com","enabled":true}]"#).unwrap();

        let envs = db.list_environments().unwrap();
        assert_eq!(envs.len(), 1);
        assert!(envs[0].variables.contains("api.example.com"));
    }
}
