use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread;

fn temp_collection_path(label: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("900api-cli-{label}-{unique}.json"))
}

fn run_http_case(status_line: &str, test_script: &str, label: &str) -> Option<Output> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("Loopback bind is not permitted in this test environment; test skipped");
            return None;
        }
        Err(error) => panic!("Could not start loopback test server: {error}"),
    };
    let address = listener.local_addr().unwrap();
    let status_line = status_line.to_string();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 2048];
        let _ = stream.read(&mut buffer);
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{{\"ok\":true}}"
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let collection = serde_json::json!({
        "schema": api900_core::format::COLLECTION_SCHEMA,
        "name": "CLI exit test",
        "requests": [{
            "name": "HTTP request",
            "method": "GET",
            "url": format!("http://{address}/health"),
            "body_type": "none",
            "test_script": test_script
        }]
    });
    let path = temp_collection_path(label);
    std::fs::write(&path, serde_json::to_vec_pretty(&collection).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_900api"))
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();

    server.join().unwrap();
    let _ = std::fs::remove_file(path);
    Some(output)
}

#[test]
fn failing_saved_script_returns_exit_one_for_http_200() {
    let Some(output) = run_http_case(
        "200 OK",
        "throw new Error('deliberate failure')",
        "script-failure",
    ) else {
        return;
    };

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Test script failed"));
}

#[test]
fn non_success_http_status_returns_exit_one() {
    let Some(output) = run_http_case("500 Internal Server Error", "", "http-failure") else {
        return;
    };

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Expected 2xx, got 500"));
}

#[test]
fn unknown_reporter_returns_exit_two_before_file_access() {
    let output = Command::new(env!("CARGO_BIN_EXE_900api"))
        .args(["run", "missing-collection.json", "--reporter", "tap"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid value 'tap'"));
}

#[test]
fn invalid_request_configuration_returns_exit_two() {
    let collection = serde_json::json!({
        "schema": api900_core::format::COLLECTION_SCHEMA,
        "name": "Invalid request configuration",
        "requests": [{
            "name": "Invalid URL",
            "method": "GET",
            "url": "not a valid URL",
            "body_type": "none"
        }]
    });
    let path = temp_collection_path("invalid-config");
    std::fs::write(&path, serde_json::to_vec_pretty(&collection).unwrap()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_900api"))
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(path);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("Invalid request configuration for 'Invalid URL'"));
}
