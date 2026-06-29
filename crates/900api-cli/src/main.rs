use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "900api")]
#[command(version = "0.1.0")]
#[command(about = "900API CLI — headless API collection runner for CI/CD")]
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
        #[arg(short, long, default_value = "console")]
        reporter: String,
    },
    /// Export a collection to a different format
    Export {
        /// Path to the collection JSON file
        collection: PathBuf,

        /// Output format: postman, openapi, curl
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

#[derive(Debug, Serialize, Deserialize)]
struct CollectionFile {
    name: String,
    #[serde(default)]
    description: Option<String>,
    requests: Vec<RequestItem>,
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

#[derive(Debug, Serialize, Deserialize)]
struct RequestItem {
    name: String,
    method: String,
    url: String,
    #[serde(default)]
    headers: Vec<KeyValueFile>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    test_script: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeyValueFile {
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
    reporter: String,
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

    match reporter.as_str() {
        "json" => {
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
        "junit" => {
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
        _ => {
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

async fn execute_request(
    client: &reqwest::Client,
    req: &RequestItem,
    env_vars: &[EnvironmentVariableFile],
) -> TestResult {
    let resolved_url = resolve_variables(&req.url, env_vars);
    let method = match req.method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "PATCH" => reqwest::Method::PATCH,
        "DELETE" => reqwest::Method::DELETE,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let mut request = client.request(method, &resolved_url);

    for h in &req.headers {
        if h.enabled && !h.key.is_empty() {
            let key = resolve_variables(&h.key, env_vars);
            let value = resolve_variables(&h.value, env_vars);
            request = request.header(key, value);
        }
    }

    if let Some(ref body) = req.body {
        let resolved_body = resolve_variables(body, env_vars);
        request = request.body(resolved_body);
    }

    let start = std::time::Instant::now();
    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            return TestResult {
                name: req.name.clone(),
                passed: false,
                error: Some(e.to_string()),
                status: 0,
                time_ms: start.elapsed().as_millis() as u64,
            };
        }
    };
    let elapsed = start.elapsed();

    let status = response.status().as_u16();

    // Basic test: check if status is 2xx
    let passed = (200..300).contains(&status);
    let error = if passed {
        None
    } else {
        Some(format!("Expected 2xx, got {}", status))
    };

    TestResult {
        name: req.name.clone(),
        passed,
        error,
        status,
        time_ms: elapsed.as_millis() as u64,
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
                            "body": r.body.as_ref().map(|b| {
                                serde_json::json!({
                                    "mode": "raw",
                                    "raw": b
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
                if let Some(ref body) = r.body {
                    cmd.push_str(&format!(" -d {}", shell_quote(body)));
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

        if let Some(ref body) = req.body {
            md.push_str("**Body:**\n\n");
            md.push_str(&format!("```json\n{}\n```\n\n", body));
        }

        if let Some(ref script) = req.test_script {
            md.push_str("**Test Script:**\n\n");
            md.push_str(&format!("```javascript\n{}\n```\n\n", script));
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
        "<title>{} — API Documentation</title>\n",
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

        if let Some(ref body) = req.body {
            html.push_str("<p><strong>Body:</strong></p>\n<pre><code>");
            html.push_str(&html_escape(body));
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
            name: "<script>alert(1)</script>".to_string(),
            description: Some("\"><img src=x onerror=alert(1)>".to_string()),
            requests: vec![RequestItem {
                name: "<b>Create</b>".to_string(),
                method: "TRACE\"><script>".to_string(),
                url: "https://example.com/<script>".to_string(),
                headers: vec![KeyValueFile {
                    key: "X-Test".to_string(),
                    value: "<svg onload=alert(1)>".to_string(),
                    enabled: true,
                }],
                body: Some("{\"x\":\"</script>\"}".to_string()),
                test_script: None,
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
