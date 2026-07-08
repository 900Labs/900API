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
    fn test_nested_collection_move_rejects_cycles() {
        let db = temp_db("nested");
        let root = db.create_collection("Root", None).unwrap();
        let child = db
            .create_collection_with_parent("Child", None, Some(&root.id))
            .unwrap();

        let collections = db.list_collections().unwrap();
        let saved_child = collections.iter().find(|c| c.id == child.id).unwrap();
        assert_eq!(saved_child.parent_id, Some(root.id.clone()));

        let cycle = db.move_collection(&root.id, Some(&child.id));
        assert!(cycle.is_err());

        db.move_collection(&child.id, None).unwrap();
        let collections = db.list_collections().unwrap();
        let moved_child = collections.iter().find(|c| c.id == child.id).unwrap();
        assert_eq!(moved_child.parent_id, None);
    }

    #[test]
    fn test_delete_collection_removes_descendants() {
        let db = temp_db("delete_nested");
        let root = db.create_collection("Root", None).unwrap();
        let child = db
            .create_collection_with_parent("Child", None, Some(&root.id))
            .unwrap();
        db.create_request(
            &child.id,
            "Nested Request",
            "GET",
            "https://example.com",
            "[]",
            "[]",
            "none",
            "",
            "none",
            "{}",
            "",
            "",
        )
        .unwrap();

        db.delete_collection(&root.id).unwrap();

        assert_eq!(db.list_collections().unwrap().len(), 0);
        assert_eq!(db.list_requests(&child.id).unwrap().len(), 0);
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
        assert_eq!(history[0].size_bytes, 512);
        assert_eq!(history[1].request_snapshot, "{}");
    }

    #[test]
    fn test_history_stores_request_snapshot() {
        let db = temp_db("history_snapshot");
        let snapshot = r#"{"method":"POST","url":"{{baseUrl}}/users","headers":[{"key":"Content-Type","value":"application/json","enabled":true}],"params":[],"body_type":"json","body":"{\"name\":\"Ada\"}","auth":{"auth_type":"none"}}"#;

        db.add_history_with_snapshot("POST", "{{baseUrl}}/users", 201, 75, 128, snapshot)
            .unwrap();

        let history = db.list_history(100).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].request_snapshot, snapshot);
        assert_eq!(history[0].size_bytes, 128);
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
            &col.id,
            "Get Users",
            "GET",
            "https://api.example.com/users",
            "[]",
            "[]",
            "none",
            "",
            "none",
            "{}",
            "",
            "",
        )
        .unwrap();
        db.create_request(
            &col.id,
            "Create User",
            "POST",
            "https://api.example.com/users",
            "[]",
            "[]",
            "json",
            "{}",
            "none",
            "{}",
            "",
            "",
        )
        .unwrap();

        let reqs = db.list_requests(&col.id).unwrap();
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].name, "Get Users");
        assert_eq!(reqs[1].name, "Create User");
    }

    #[test]
    fn test_request_settings_persist() {
        let db = temp_db("request_settings");
        let col = db.create_collection("Test", None).unwrap();
        let settings = r#"{"timeout_ms":45000,"connect_timeout_ms":10000,"follow_redirects":false,"verify_ssl":false,"proxy_url":"http://127.0.0.1:8080","use_cookie_jar":true}"#;
        let req = db
            .create_request_with_settings(
                &col.id,
                "Runtime Settings",
                "GET",
                "https://example.com",
                "[]",
                "[]",
                "none",
                "",
                "none",
                "{}",
                "",
                "",
                settings,
            )
            .unwrap();

        let reqs = db.list_requests(&col.id).unwrap();
        assert_eq!(reqs[0].settings, settings);

        let updated = r#"{"timeout_ms":120000,"connect_timeout_ms":30000,"follow_redirects":true,"verify_ssl":true,"proxy_url":"","use_cookie_jar":false}"#;
        db.update_request_with_settings(
            &req.id,
            "Runtime Settings",
            "POST",
            "https://example.com",
            "[]",
            "[]",
            "none",
            "",
            "none",
            "{}",
            "",
            "",
            updated,
        )
        .unwrap();

        let reqs = db.list_requests(&col.id).unwrap();
        assert_eq!(reqs[0].settings, updated);
    }

    #[test]
    fn test_update_request() {
        let db = temp_db("update_req");
        let col = db.create_collection("Test", None).unwrap();
        let req = db
            .create_request(
                &col.id,
                "Original",
                "GET",
                "https://example.com",
                "[]",
                "[]",
                "none",
                "",
                "none",
                "{}",
                "",
                "",
            )
            .unwrap();

        db.update_request(
            &req.id,
            "Updated",
            "POST",
            "https://example.com/updated",
            "[]",
            "[]",
            "json",
            "{}",
            "bearer",
            "{}",
            "console.log('hi')",
            "",
        )
        .unwrap();

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
        let req = db
            .create_request(
                &col.id,
                "To Delete",
                "GET",
                "https://example.com",
                "[]",
                "[]",
                "none",
                "",
                "none",
                "{}",
                "",
                "",
            )
            .unwrap();

        db.delete_request(&req.id).unwrap();
        let reqs = db.list_requests(&col.id).unwrap();
        assert_eq!(reqs.len(), 0);
    }

    #[test]
    fn test_response_examples_lifecycle() {
        let db = temp_db("response_examples");
        let col = db.create_collection("Test", None).unwrap();
        let req = db
            .create_request(
                &col.id,
                "Get User",
                "GET",
                "https://example.com/user",
                "[]",
                "[]",
                "none",
                "",
                "none",
                "{}",
                "",
                "",
            )
            .unwrap();

        let example = db
            .create_response_example(
                &req.id,
                "200 OK",
                200,
                "OK",
                r#"{"content-type":"application/json"}"#,
                r#"{"id":1}"#,
                42,
                8,
            )
            .unwrap();

        let examples = db.list_response_examples(&req.id).unwrap();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].name, "200 OK");
        assert_eq!(examples[0].status, 200);
        assert_eq!(examples[0].body, r#"{"id":1}"#);

        db.delete_response_example(&example.id).unwrap();
        assert_eq!(db.list_response_examples(&req.id).unwrap().len(), 0);
    }

    #[test]
    fn test_delete_request_removes_response_examples() {
        let db = temp_db("response_examples_cascade");
        let col = db.create_collection("Test", None).unwrap();
        let req = db
            .create_request(
                &col.id,
                "Get User",
                "GET",
                "https://example.com/user",
                "[]",
                "[]",
                "none",
                "",
                "none",
                "{}",
                "",
                "",
            )
            .unwrap();

        db.create_response_example(&req.id, "200 OK", 200, "OK", "{}", "ok", 25, 2)
            .unwrap();

        db.delete_request(&req.id).unwrap();

        assert_eq!(db.list_requests(&col.id).unwrap().len(), 0);
        assert_eq!(db.list_response_examples(&req.id).unwrap().len(), 0);
    }

    #[test]
    fn test_move_request_between_collections() {
        let db = temp_db("move_req");
        let first = db.create_collection("First", None).unwrap();
        let second = db.create_collection("Second", None).unwrap();
        let req = db
            .create_request(
                &first.id,
                "Move Me",
                "GET",
                "https://example.com",
                "[]",
                "[]",
                "none",
                "",
                "none",
                "{}",
                "",
                "",
            )
            .unwrap();

        db.move_request(&req.id, &second.id).unwrap();

        assert_eq!(db.list_requests(&first.id).unwrap().len(), 0);
        let moved = db.list_requests(&second.id).unwrap();
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].id, req.id);
    }

    #[test]
    fn test_update_environment() {
        let db = temp_db("update_env");
        let env = db.create_environment("Test").unwrap();
        db.update_environment(
            &env.id,
            r#"[{"key":"url","value":"https://api.example.com","enabled":true}]"#,
        )
        .unwrap();

        let envs = db.list_environments().unwrap();
        assert_eq!(envs.len(), 1);
        assert!(envs[0].variables.contains("api.example.com"));
    }
}
