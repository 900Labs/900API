use api900_core::format::{parse_collection, CollectionFile, KeyValue, RequestItem};
use api900_core::scripting::run_test_script;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "900api")]
#[command(version)]
#[command(about = "900API CLI, a headless API collection runner for CI/CD")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a collection of API requests and tests
    Run {
        /// Path to the collection JSON file
        collection: PathBuf,

        /// Path to an environment JSON file
        #[arg(short, long)]
        environment: Option<PathBuf>,

        /// Reporter format: console, json, junit
        #[arg(short, long, value_enum, default_value = "console")]
        reporter: Reporter,
    },
    /// Export a collection to a different format
    Export {
        /// Path to the collection JSON file
        collection: PathBuf,

        /// Output format: postman, curl
        #[arg(short, long)]
        format: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate API documentation from a collection
    Docs {
        /// Path to the collection JSON file
        collection: PathBuf,

        /// Output format: markdown, html
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Reporter {
    Console,
    Json,
    Junit,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentFile {
    #[serde(default)]
    name: String,
    #[serde(default)]
    variables: Vec<EnvironmentVariableFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentVariableFile {
    key: String,
    value: String,
    #[serde(default = "default_true")]
    enabled: bool,
}

fn default_true() -> bool {
    true
}

struct TestResult {
    name: String,
    passed: bool,
    error: Option<String>,
    status: u16,
    time_ms: u64,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            collection,
            environment,
            reporter,
        } => run_collection(collection, environment, reporter).await,
        Commands::Export {
            collection,
            format,
            output,
        } => export_collection(collection, format, output),
        Commands::Docs {
            collection,
            format,
            output,
        } => generate_docs(collection, format, output),
    }
}

async fn run_collection(
    collection_path: PathBuf,
    environment_path: Option<PathBuf>,
    reporter: Reporter,
) -> ExitCode {
    let collection_str = match std::fs::read_to_string(&collection_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading collection file: {}", e);
            return ExitCode::from(2);
        }
    };

    let collection = match parse_collection(&collection_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing collection JSON: {}", e);
            return ExitCode::from(2);
        }
    };

    // Load environment file if provided
    let env_vars: Vec<EnvironmentVariableFile> = if let Some(ref env_path) = environment_path {
        match std::fs::read_to_string(env_path) {
            Ok(content) => {
                match serde_json::from_str::<EnvironmentFile>(&content) {
                    Ok(env) => env.variables,
                    Err(_) => {
                        // Try parsing as just a variables array
                        match serde_json::from_str::<Vec<EnvironmentVariableFile>>(&content) {
                            Ok(vars) => vars,
                            Err(e) => {
                                eprintln!("Error parsing environment JSON: {}", e);
                                return ExitCode::from(2);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading environment file: {}", e);
                return ExitCode::from(2);
            }
        }
    } else {
        Vec::new()
    };

    for request in &collection.requests {
        if let Err(error) = validate_request_config(request, &env_vars) {
            eprintln!(
                "Invalid request configuration for '{}': {}",
                request.name, error
            );
            return ExitCode::from(2);
        }
    }

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .tcp_nodelay(true)
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error building HTTP client: {}", e);
            return ExitCode::from(2);
        }
    };
    let mut results = Vec::new();
    let mut all_passed = true;

    for req in &collection.requests {
        let result = execute_request(&client, req, &env_vars).await;
        let passed = result.passed;
        if !passed {
            all_passed = false;
        }
        results.push(result);
    }

    match reporter {
        Reporter::Json => {
            let json = serde_json::to_string_pretty(
                &results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "name": r.name,
                            "passed": r.passed,
                            "status": r.status,
                            "time_ms": r.time_ms,
                            "error": r.error,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap_or_default();
            println!("{}", json);
        }
        Reporter::Junit => {
            println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
            println!(
                r#"<testsuite name="{}" tests="{}">"#,
                xml_escape(&collection.name),
                results.len()
            );
            for r in &results {
                println!(
                    r#"  <testcase name="{}" time="{}">"#,
                    xml_escape(&r.name),
                    r.time_ms as f64 / 1000.0
                );
                if !r.passed {
                    if let Some(ref err) = r.error {
                        println!(r#"    <failure>{}</failure>"#, xml_escape(err));
                    }
                }
                println!(r#"  </testcase>"#);
            }
            println!("</testsuite>");
        }
        Reporter::Console => {
            // Console reporter
            for r in &results {
                let status_icon = if r.passed { "✓" } else { "✗" };
                println!("{} {} [{}] {}ms", status_icon, r.name, r.status, r.time_ms);
                if let Some(ref err) = r.error {
                    println!("  Error: {}", err);
                }
            }
            println!(
                "\n{} passed, {} failed, {} total",
                results.iter().filter(|r| r.passed).count(),
                results.iter().filter(|r| !r.passed).count(),
                results.len()
            );
        }
    }

    if all_passed {
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}

fn validate_request_config(
    request: &RequestItem,
    env_vars: &[EnvironmentVariableFile],
) -> Result<(), String> {
    let resolved_url = resolve_variables(&request.url, env_vars);
    reqwest::Url::parse(&resolved_url).map_err(|error| format!("Invalid URL: {error}"))?;

    match request.method.to_ascii_uppercase().as_str() {
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS" => {}
        other => return Err(format!("Unsupported HTTP method: {other}")),
    }

    for header in request
        .headers
        .iter()
        .filter(|header| header.enabled && !header.key.is_empty())
    {
        let key = resolve_variables(&header.key, env_vars);
        let value = resolve_variables(&header.value, env_vars);
        reqwest::header::HeaderName::from_bytes(key.as_bytes())
            .map_err(|error| format!("Invalid header name '{key}': {error}"))?;
        reqwest::header::HeaderValue::from_str(&value)
            .map_err(|error| format!("Invalid value for header '{key}': {error}"))?;
    }

    validate_auth_config(request, env_vars)?;
    validate_body_config(request, env_vars)?;
    validate_request_settings(request)?;
    Ok(())
}

fn validate_auth_config(
    request: &RequestItem,
    env_vars: &[EnvironmentVariableFile],
) -> Result<(), String> {
    let config = request
        .auth_config
        .as_object()
        .ok_or_else(|| "Authentication configuration must be a JSON object".to_string())?;
    let value = |key: &str| {
        config
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(|value| resolve_variables(value, env_vars))
            .unwrap_or_default()
    };

    match request.auth_type.as_str() {
        "" | "none" | "basic" | "bearer" | "o_auth2" => Ok(()),
        "api_key" => {
            let name = value("api_key_name");
            if name.is_empty() {
                return Err("API key authentication requires an API key name".to_string());
            }
            match value("api_key_in").as_str() {
                "query" => Ok(()),
                "" | "header" => {
                    reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                        format!("Invalid API key header name '{name}': {error}")
                    })?;
                    reqwest::header::HeaderValue::from_str(&value("api_key")).map_err(|error| {
                        format!("Invalid value for API key header '{name}': {error}")
                    })?;
                    Ok(())
                }
                location => Err(format!(
                    "API key location must be 'header' or 'query', received '{location}'"
                )),
            }
        }
        unsupported => Err(format!(
            "Authentication type '{unsupported}' is not supported by the CLI"
        )),
    }
}

fn validate_body_config(
    request: &RequestItem,
    env_vars: &[EnvironmentVariableFile],
) -> Result<(), String> {
    let body = resolve_variables(&request.body, env_vars);
    match request.body_type.as_str() {
        "none" | "raw" => Ok(()),
        "json" => {
            if !body.trim().is_empty() {
                serde_json::from_str::<serde_json::Value>(&body)
                    .map_err(|error| format!("Invalid JSON request body: {error}"))?;
            }
            Ok(())
        }
        "form_data" => parse_form_fields(&body, "multipart form-data").map(|_| ()),
        "x_www_form_urlencoded" => parse_form_fields(&body, "URL-encoded form").map(|_| ()),
        unsupported => Err(format!("Unsupported request body type: {unsupported}")),
    }
}

fn validate_request_settings(request: &RequestItem) -> Result<(), String> {
    let settings = request
        .settings
        .as_object()
        .ok_or_else(|| "Request settings must be a JSON object".to_string())?;
    let timeout_ms = optional_u64_setting(settings, "timeout_ms")?.unwrap_or(120_000);
    if timeout_ms == 0 || timeout_ms > 600_000 {
        return Err("Request timeout must be between 1 ms and 600000 ms".to_string());
    }
    if let Some(connect_timeout_ms) = optional_u64_setting(settings, "connect_timeout_ms")? {
        if connect_timeout_ms == 0 || connect_timeout_ms > timeout_ms {
            return Err("Connect timeout must be between 1 ms and the request timeout".to_string());
        }
    }
    Ok(())
}

fn optional_u64_setting(
    settings: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<u64>, String> {
    settings
        .get(key)
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| format!("Request setting '{key}' must be a positive integer"))
        })
        .transpose()
}

async fn execute_request(
    client: &reqwest::Client,
    req: &RequestItem,
    env_vars: &[EnvironmentVariableFile],
) -> TestResult {
    if let Ok(output) = run_test_script(&req.pre_request_script, "", 0, "{}") {
        if let Some(error) = output.error {
            return failed_result(req, 0, 0, format!("Pre-request script failed: {error}"));
        }
    } else {
        return failed_result(
            req,
            0,
            0,
            "Pre-request script could not be executed".to_string(),
        );
    }

    let resolved_url = resolve_variables(&req.url, env_vars);
    let mut url = match reqwest::Url::parse(&resolved_url) {
        Ok(url) => url,
        Err(error) => return failed_result(req, 0, 0, format!("Invalid URL: {error}")),
    };
    for param in req
        .params
        .iter()
        .filter(|param| param.enabled && !param.key.is_empty())
    {
        url.query_pairs_mut().append_pair(
            &resolve_variables(&param.key, env_vars),
            &resolve_variables(&param.value, env_vars),
        );
    }
    let method = match req.method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "PATCH" => reqwest::Method::PATCH,
        "DELETE" => reqwest::Method::DELETE,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => return failed_result(req, 0, 0, format!("Unsupported method: {}", req.method)),
    };

    if let Err(error) = apply_query_auth(req, &mut url, env_vars) {
        return failed_result(req, 0, 0, error);
    }

    let mut request = client.request(method, url.clone());

    for h in &req.headers {
        if h.enabled && !h.key.is_empty() {
            let key = resolve_variables(&h.key, env_vars);
            let value = resolve_variables(&h.value, env_vars);
            request = request.header(key, value);
        }
    }

    request = match apply_auth(request, req, env_vars) {
        Ok(request) => request,
        Err(error) => return failed_result(req, 0, 0, error),
    };

    request = match apply_body(request, req, env_vars) {
        Ok(request) => request,
        Err(error) => return failed_result(req, 0, 0, error),
    };

    if let Some(timeout_ms) = req
        .settings
        .get("timeout_ms")
        .and_then(serde_json::Value::as_u64)
    {
        if timeout_ms == 0 || timeout_ms > 600_000 {
            return failed_result(
                req,
                0,
                0,
                "Request timeout must be between 1 ms and 600000 ms".to_string(),
            );
        }
        request = request.timeout(Duration::from_millis(timeout_ms));
    }

    let start = std::time::Instant::now();
    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            return failed_result(req, 0, start.elapsed().as_millis() as u64, e.to_string());
        }
    };
    let elapsed = start.elapsed();

    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(key, value)| {
            (
                key.to_string(),
                value.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();
    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => {
            return failed_result(
                req,
                status,
                elapsed.as_millis() as u64,
                format!("Could not read response body: {error}"),
            )
        }
    };
    let headers_json = serde_json::to_string(&headers).unwrap_or_else(|_| "{}".to_string());

    let mut errors = Vec::new();
    if !(200..300).contains(&status) {
        errors.push(format!("Expected 2xx, got {status}"));
    }
    match run_test_script(&req.test_script, &body, status, &headers_json) {
        Ok(output) => {
            if let Some(error) = output.error {
                errors.push(format!("Test script failed: {error}"));
            }
        }
        Err(error) => errors.push(format!("Test script could not be executed: {error}")),
    }

    TestResult {
        name: req.name.clone(),
        passed: errors.is_empty(),
        error: (!errors.is_empty()).then(|| errors.join("; ")),
        status,
        time_ms: elapsed.as_millis() as u64,
    }
}

fn apply_auth(
    mut request: reqwest::RequestBuilder,
    req: &RequestItem,
    env_vars: &[EnvironmentVariableFile],
) -> Result<reqwest::RequestBuilder, String> {
    let value = |key: &str| {
        req.auth_config
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(|value| resolve_variables(value, env_vars))
            .unwrap_or_default()
    };
    match req.auth_type.as_str() {
        "" | "none" => {}
        "basic" => request = request.basic_auth(value("username"), Some(value("password"))),
        "bearer" => request = request.bearer_auth(value("token")),
        "o_auth2" => request = request.bearer_auth(value("oauth2_access_token")),
        "api_key" => {
            let name = value("api_key_name");
            if name.is_empty() {
                return Err("API key authentication requires an API key name".to_string());
            }
            let api_key = value("api_key");
            if value("api_key_in") != "query" {
                request = request.header(name, api_key);
            }
        }
        unsupported => {
            return Err(format!(
                "Authentication type '{unsupported}' is not supported by the CLI"
            ))
        }
    }
    Ok(request)
}

fn apply_query_auth(
    req: &RequestItem,
    url: &mut reqwest::Url,
    env_vars: &[EnvironmentVariableFile],
) -> Result<(), String> {
    if req.auth_type != "api_key" {
        return Ok(());
    }
    let value = |key: &str| {
        req.auth_config
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(|value| resolve_variables(value, env_vars))
            .unwrap_or_default()
    };
    if value("api_key_in") != "query" {
        return Ok(());
    }
    let name = value("api_key_name");
    if name.is_empty() {
        return Err("API key authentication requires an API key name".to_string());
    }
    url.query_pairs_mut().append_pair(&name, &value("api_key"));
    Ok(())
}

fn apply_body(
    request: reqwest::RequestBuilder,
    req: &RequestItem,
    env_vars: &[EnvironmentVariableFile],
) -> Result<reqwest::RequestBuilder, String> {
    let body = resolve_variables(&req.body, env_vars);
    match req.body_type.as_str() {
        "none" => Ok(request),
        "json" => {
            if !body.trim().is_empty() {
                serde_json::from_str::<serde_json::Value>(&body)
                    .map_err(|error| format!("Invalid JSON request body: {error}"))?;
            }
            Ok(request
                .header("Content-Type", "application/json")
                .body(body))
        }
        "raw" => Ok(request.body(body)),
        "form_data" => {
            let fields = parse_form_fields(&body, "multipart form-data")?;
            let form = fields
                .into_iter()
                .filter(|field| field.enabled && !field.key.is_empty())
                .fold(reqwest::multipart::Form::new(), |form, field| {
                    form.text(field.key, field.value)
                });
            Ok(request.multipart(form))
        }
        "x_www_form_urlencoded" => {
            let fields = parse_form_fields(&body, "URL-encoded form")?;
            let enabled = fields
                .into_iter()
                .filter(|field| field.enabled && !field.key.is_empty())
                .map(|field| (field.key, field.value))
                .collect::<Vec<_>>();
            Ok(request.form(&enabled))
        }
        unsupported => Err(format!("Unsupported request body type: {unsupported}")),
    }
}

fn parse_form_fields(body: &str, label: &str) -> Result<Vec<KeyValue>, String> {
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(body).map_err(|error| format!("Invalid {label} fields: {error}"))
}

fn failed_result(req: &RequestItem, status: u16, time_ms: u64, error: String) -> TestResult {
    TestResult {
        name: req.name.clone(),
        passed: false,
        error: Some(error),
        status,
        time_ms,
    }
}

fn export_collection(
    collection_path: PathBuf,
    format: String,
    output: Option<PathBuf>,
) -> ExitCode {
    let collection_str = match std::fs::read_to_string(&collection_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading collection file: {}", e);
            return ExitCode::from(2);
        }
    };

    let collection: CollectionFile = match serde_json::from_str(&collection_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing collection JSON: {}", e);
            return ExitCode::from(2);
        }
    };

    let output_str = match format.as_str() {
        "postman" => {
            let postman = serde_json::json!({
                "info": {
                    "name": collection.name,
                    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
                },
                "item": collection.requests.iter().map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "request": {
                            "method": r.method,
                            "header": r.headers.iter().filter(|h| h.enabled).map(|h| {
                                serde_json::json!({"key": h.key, "value": h.value, "type": "text"})
                            }).collect::<Vec<_>>(),
                            "url": {
                                "raw": r.url,
                                "url": r.url
                            },
                            "body": (!r.body.is_empty()).then(|| {
                                serde_json::json!({
                                    "mode": "raw",
                                    "raw": r.body
                                })
                            })
                        }
                    })
                }).collect::<Vec<_>>()
            });
            serde_json::to_string_pretty(&postman).unwrap_or_default()
        }
        "curl" => {
            let mut curls = Vec::new();
            for r in &collection.requests {
                let mut cmd = format!("curl -X {} {}", r.method, shell_quote(&r.url));
                for h in &r.headers {
                    if h.enabled {
                        cmd.push_str(&format!(
                            " -H {}",
                            shell_quote(&format!("{}: {}", h.key, h.value))
                        ));
                    }
                }
                if !r.body.is_empty() {
                    cmd.push_str(&format!(" -d {}", shell_quote(&r.body)));
                }
                curls.push(format!("# {}", r.name));
                curls.push(cmd);
                curls.push(String::new());
            }
            curls.join("\n")
        }
        _ => {
            eprintln!("Unknown format: {}. Supported: postman, curl", format);
            return ExitCode::from(2);
        }
    };

    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, &output_str) {
                eprintln!("Error writing output file: {}", e);
                return ExitCode::from(2);
            }
            println!("Exported to {}", path.display());
        }
        None => println!("{}", output_str),
    }

    ExitCode::from(0)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn html_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn method_class(method: &str) -> &'static str {
    match method.to_uppercase().as_str() {
        "GET" => "GET",
        "POST" => "POST",
        "PUT" => "PUT",
        "PATCH" => "PATCH",
        "DELETE" => "DELETE",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    }
}

fn resolve_variables(input: &str, vars: &[EnvironmentVariableFile]) -> String {
    let mut result = input.to_string();
    for var in vars.iter().filter(|v| v.enabled) {
        let pattern = format!("{{{{{}}}}}", var.key);
        result = result.replace(&pattern, &var.value);
    }
    result
}

fn generate_docs(collection_path: PathBuf, format: String, output: Option<PathBuf>) -> ExitCode {
    let collection_str = match std::fs::read_to_string(&collection_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading collection file: {}", e);
            return ExitCode::from(2);
        }
    };

    let collection: CollectionFile = match serde_json::from_str(&collection_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing collection JSON: {}", e);
            return ExitCode::from(2);
        }
    };

    let output_str = match format.as_str() {
        "markdown" => generate_markdown_docs(&collection),
        "html" => generate_html_docs(&collection),
        _ => {
            eprintln!("Unknown format: {}. Supported: markdown, html", format);
            return ExitCode::from(2);
        }
    };

    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, &output_str) {
                eprintln!("Error writing output file: {}", e);
                return ExitCode::from(2);
            }
            println!("Documentation written to {}", path.display());
        }
        None => println!("{}", output_str),
    }

    ExitCode::from(0)
}

fn generate_markdown_docs(collection: &CollectionFile) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {}\n\n", collection.name));

    if let Some(ref desc) = collection.description {
        md.push_str(desc);
        md.push_str("\n\n");
    }

    md.push_str("## Endpoints\n\n");

    for req in &collection.requests {
        md.push_str(&format!("### {} `{}`\n\n", req.method, req.name));
        md.push_str(&format!("**URL:** `{}`\n\n", req.url));

        if !req.headers.is_empty() {
            md.push_str("**Headers:**\n\n");
            md.push_str("| Key | Value | Enabled |\n");
            md.push_str("|-----|-------|---------|\n");
            for h in &req.headers {
                md.push_str(&format!("| {} | {} | {} |\n", h.key, h.value, h.enabled));
            }
            md.push('\n');
        }

        if !req.body.is_empty() {
            md.push_str("**Body:**\n\n");
            md.push_str(&format!("```text\n{}\n```\n\n", req.body));
        }

        if !req.test_script.is_empty() {
            md.push_str("**Test Script:**\n\n");
            md.push_str(&format!("```javascript\n{}\n```\n\n", req.test_script));
        }

        md.push_str("---\n\n");
    }

    md
}

fn generate_html_docs(collection: &CollectionFile) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str(&format!(
        "<title>{} | API Documentation</title>\n",
        html_escape(&collection.name)
    ));
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem; color: #333; }\n");
    html.push_str("h1 { color: #1a1a2e; }\n");
    html.push_str(
        "h2 { color: #16213e; border-bottom: 2px solid #e94560; padding-bottom: 0.5rem; }\n",
    );
    html.push_str("h3 { color: #0f3460; }\n");
    html.push_str("code { background: #f4f4f4; padding: 0.2rem 0.4rem; border-radius: 3px; font-family: 'SF Mono', monospace; }\n");
    html.push_str("pre { background: #1a1a2e; color: #e0e0e0; padding: 1rem; border-radius: 8px; overflow-x: auto; }\n");
    html.push_str("pre code { background: none; color: inherit; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 0.5rem; text-align: left; }\n");
    html.push_str("th { background: #f8f8f8; }\n");
    html.push_str(".method { display: inline-block; padding: 0.2rem 0.6rem; border-radius: 4px; font-weight: bold; font-size: 0.85rem; }\n");
    html.push_str(".GET { background: #d4edda; color: #155724; }\n");
    html.push_str(".POST { background: #fff3cd; color: #856404; }\n");
    html.push_str(".PUT { background: #cce5ff; color: #004085; }\n");
    html.push_str(".PATCH { background: #d1ecf1; color: #0c5460; }\n");
    html.push_str(".DELETE { background: #f8d7da; color: #721c24; }\n");
    html.push_str(".HEAD { background: #e2d9f3; color: #3d1766; }\n");
    html.push_str(".OPTIONS { background: #d6e4ff; color: #003a8c; }\n");
    html.push_str(".OTHER { background: #e5e7eb; color: #374151; }\n");
    html.push_str("</style>\n</head>\n<body>\n");

    html.push_str(&format!("<h1>{}</h1>\n", html_escape(&collection.name)));

    if let Some(ref desc) = collection.description {
        html.push_str(&format!("<p>{}</p>\n", html_escape(desc)));
    }

    html.push_str("<h2>Endpoints</h2>\n");

    for req in &collection.requests {
        let method_class = method_class(&req.method);
        html.push_str(&format!(
            "<h3><span class=\"method {}\">{}</span> {}</h3>\n",
            method_class,
            html_escape(&req.method),
            html_escape(&req.name)
        ));
        html.push_str(&format!(
            "<p><strong>URL:</strong> <code>{}</code></p>\n",
            html_escape(&req.url)
        ));

        if !req.headers.is_empty() {
            html.push_str("<p><strong>Headers:</strong></p>\n<table>\n<tr><th>Key</th><th>Value</th><th>Enabled</th></tr>\n");
            for h in &req.headers {
                html.push_str(&format!(
                    "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td></tr>\n",
                    html_escape(&h.key),
                    html_escape(&h.value),
                    h.enabled
                ));
            }
            html.push_str("</table>\n");
        }

        if !req.body.is_empty() {
            html.push_str("<p><strong>Body:</strong></p>\n<pre><code>");
            html.push_str(&html_escape(&req.body));
            html.push_str("</code></pre>\n");
        }

        html.push_str("<hr>\n");
    }

    html.push_str("</body>\n</html>\n");

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape_escapes_markup() {
        assert_eq!(
            html_escape("<script>alert(\"x\")</script>"),
            "&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_shell_quote_handles_single_quotes() {
        assert_eq!(
            shell_quote("https://example.com/a'b"),
            "'https://example.com/a'\\''b'"
        );
    }

    #[test]
    fn test_generate_html_docs_escapes_user_content() {
        let collection = CollectionFile {
            schema: api900_core::format::COLLECTION_SCHEMA.to_string(),
            id: None,
            name: "<script>alert(1)</script>".to_string(),
            description: Some("\"><img src=x onerror=alert(1)>".to_string()),
            parent_id: None,
            sort_order: 0,
            exported_at: None,
            requests: vec![RequestItem {
                id: None,
                name: "<b>Create</b>".to_string(),
                method: "TRACE\"><script>".to_string(),
                url: "https://example.com/<script>".to_string(),
                headers: vec![KeyValue {
                    key: "X-Test".to_string(),
                    value: "<svg onload=alert(1)>".to_string(),
                    enabled: true,
                }],
                params: Vec::new(),
                body_type: "json".to_string(),
                body: "{\"x\":\"</script>\"}".to_string(),
                auth_type: "none".to_string(),
                auth_config: serde_json::json!({}),
                pre_request_script: String::new(),
                test_script: String::new(),
                settings: serde_json::json!({}),
                sort_order: 0,
                response_examples: Vec::new(),
            }],
        };

        let html = generate_html_docs(&collection);
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("<svg onload=alert(1)>"));
        assert!(!html.contains("TRACE\"><script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("class=\"method OTHER\""));
        assert!(html.contains("&lt;svg onload=alert(1)&gt;"));
    }
}
