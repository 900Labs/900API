use crate::db::Database;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocsError {
    #[error("Database error: {0}")]
    Db(#[from] crate::db::DbError),
    #[error("Collection not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDoc {
    pub collection_name: String,
    pub collection_description: Option<String>,
    pub endpoints: Vec<EndpointDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDoc {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<crate::models::KeyValue>,
    pub params: Vec<crate::models::KeyValue>,
    pub body_type: String,
    pub body: String,
    pub auth_type: String,
    pub description: String,
}

pub fn generate_collection_docs(
    db: &Database,
    collection_id: &str,
) -> Result<ApiDoc, DocsError> {
    let collections = db.list_collections()?;
    let collection = collections
        .iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| DocsError::NotFound(collection_id.to_string()))?;

    let requests = db.list_requests(collection_id)?;

    let endpoints: Vec<EndpointDoc> = requests
        .iter()
        .map(|req| {
            let auth_type = match req.auth_type.as_str() {
                "basic" => "Basic Auth",
                "bearer" => "Bearer Token",
                "api_key" => "API Key",
                "o_auth2" => "OAuth 2.0",
                "o_auth1" => "OAuth 1.0a",
                "aws_sig_v4" => "AWS Signature v4",
                "hawk" => "Hawk",
                _ => "None",
            };

            let headers: Vec<crate::models::KeyValue> =
                serde_json::from_str(&req.headers).unwrap_or_default();
            let params: Vec<crate::models::KeyValue> =
                serde_json::from_str(&req.params).unwrap_or_default();

            let description = format!(
                "### {} {}\n\n**Method:** {}\n\n**Auth:** {}",
                req.method, req.url, req.method, auth_type
            );

            EndpointDoc {
                name: req.name.clone(),
                method: req.method.clone(),
                url: req.url.clone(),
                headers: headers
                    .iter()
                    .filter(|h| h.enabled && !h.key.is_empty())
                    .cloned()
                    .collect(),
                params: params
                    .iter()
                    .filter(|p| p.enabled && !p.key.is_empty())
                    .cloned()
                    .collect(),
                body_type: req.body_type.clone(),
                body: req.body.clone(),
                auth_type: auth_type.to_string(),
                description,
            }
        })
        .collect();

    Ok(ApiDoc {
        collection_name: collection.name.clone(),
        collection_description: collection.description.clone(),
        endpoints,
    })
}

pub fn generate_all_docs(db: &Database) -> Result<Vec<ApiDoc>, DocsError> {
    let collections = db.list_collections()?;
    let mut docs = Vec::new();

    for collection in &collections {
        let doc = generate_collection_docs(db, &collection.id)?;
        docs.push(doc);
    }

    Ok(docs)
}

pub fn docs_to_markdown(doc: &ApiDoc) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {}\n\n", doc.collection_name));

    if let Some(desc) = &doc.collection_description {
        md.push_str(desc);
        md.push_str("\n\n");
    }

    md.push_str("## Endpoints\n\n");

    for endpoint in &doc.endpoints {
        md.push_str(&format!(
            "### {} `{}` {}\n\n",
            endpoint.method, endpoint.method, endpoint.url
        ));

        md.push_str(&format!("**Name:** {}\n\n", endpoint.name));

        if !endpoint.headers.is_empty() {
            md.push_str("**Headers:**\n\n");
            md.push_str("| Key | Value |\n|-----|-------|\n");
            for h in &endpoint.headers {
                md.push_str(&format!("| {} | {} |\n", h.key, h.value));
            }
            md.push('\n');
        }

        if !endpoint.params.is_empty() {
            md.push_str("**Query Parameters:**\n\n");
            md.push_str("| Key | Value |\n|-----|-------|\n");
            for p in &endpoint.params {
                md.push_str(&format!("| {} | {} |\n", p.key, p.value));
            }
            md.push('\n');
        }

        if endpoint.body_type != "none" && !endpoint.body.is_empty() {
            md.push_str(&format!("**Body** (`{}`):**\n\n", endpoint.body_type));
            md.push_str(&format!("```json\n{}\n```\n\n", endpoint.body));
        }

        md.push_str(&format!("**Authentication:** {}\n\n", endpoint.auth_type));
        md.push_str("---\n\n");
    }

    md
}

pub fn docs_to_html(doc: &ApiDoc) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str(&format!("<title>{} - API Documentation</title>\n", doc.collection_name));
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem; color: #333; }\n");
    html.push_str("h1 { color: #1a1a2e; }\n");
    html.push_str("h2 { color: #16213e; border-bottom: 2px solid #e94560; padding-bottom: 0.5rem; }\n");
    html.push_str("h3 { color: #0f3460; }\n");
    html.push_str(".method { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 0.85em; font-weight: bold; color: white; }\n");
    html.push_str(".GET { background: #61affe; }\n");
    html.push_str(".POST { background: #49cc90; }\n");
    html.push_str(".PUT { background: #fca130; }\n");
    html.push_str(".PATCH { background: #50e3c2; }\n");
    html.push_str(".DELETE { background: #f93e3e; }\n");
    html.push_str(".HEAD { background: #9012fe; }\n");
    html.push_str(".OPTIONS { background: #0d5aa7; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; margin: 1rem 0; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px 12px; text-align: left; }\n");
    html.push_str("th { background: #f5f5f5; }\n");
    html.push_str("pre { background: #f8f8f8; padding: 1rem; border-radius: 4px; overflow-x: auto; }\n");
    html.push_str("code { font-family: 'SF Mono', Monaco, monospace; }\n");
    html.push_str(".endpoint { margin-bottom: 2rem; padding-bottom: 1rem; border-bottom: 1px solid #eee; }\n");
    html.push_str("</style>\n</head>\n<body>\n");

    html.push_str(&format!("<h1>{}</h1>\n", doc.collection_name));

    if let Some(desc) = &doc.collection_description {
        html.push_str(&format!("<p>{}</p>\n", desc));
    }

    html.push_str("<h2>Endpoints</h2>\n");

    for endpoint in &doc.endpoints {
        let method_class = endpoint.method.to_uppercase();
        html.push_str("<div class=\"endpoint\">\n");
        html.push_str(&format!(
            "<h3><span class=\"method {}\">{}</span> {}</h3>\n",
            method_class, endpoint.method, endpoint.url
        ));

        html.push_str(&format!("<p><strong>Name:</strong> {}</p>\n", endpoint.name));

        if !endpoint.headers.is_empty() {
            html.push_str("<p><strong>Headers:</strong></p>\n<table><tr><th>Key</th><th>Value</th></tr>\n");
            for h in &endpoint.headers {
                html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>\n", h.key, h.value));
            }
            html.push_str("</table>\n");
        }

        if !endpoint.params.is_empty() {
            html.push_str("<p><strong>Query Parameters:</strong></p>\n<table><tr><th>Key</th><th>Value</th></tr>\n");
            for p in &endpoint.params {
                html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>\n", p.key, p.value));
            }
            html.push_str("</table>\n");
        }

        if endpoint.body_type != "none" && !endpoint.body.is_empty() {
            html.push_str(&format!("<p><strong>Body</strong> ({})</p>\n<pre><code>{}</code></pre>\n", endpoint.body_type, endpoint.body));
        }

        html.push_str(&format!("<p><strong>Authentication:</strong> {}</p>\n", endpoint.auth_type));
        html.push_str("</div>\n");
    }

    html.push_str("</body>\n</html>\n");

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_to_markdown_empty() {
        let doc = ApiDoc {
            collection_name: "Test API".to_string(),
            collection_description: Some("A test API".to_string()),
            endpoints: vec![],
        };
        let md = docs_to_markdown(&doc);
        assert!(md.contains("# Test API"));
        assert!(md.contains("A test API"));
    }

    #[test]
    fn test_docs_to_markdown_with_endpoint() {
        let doc = ApiDoc {
            collection_name: "Test API".to_string(),
            collection_description: None,
            endpoints: vec![EndpointDoc {
                name: "Get users".to_string(),
                method: "GET".to_string(),
                url: "https://api.example.com/users".to_string(),
                headers: vec![],
                params: vec![],
                body_type: "none".to_string(),
                body: "".to_string(),
                auth_type: "Bearer Token".to_string(),
                description: "".to_string(),
            }],
        };
        let md = docs_to_markdown(&doc);
        assert!(md.contains("GET"));
        assert!(md.contains("https://api.example.com/users"));
        assert!(md.contains("Bearer Token"));
    }

    #[test]
    fn test_docs_to_html_with_endpoint() {
        let doc = ApiDoc {
            collection_name: "Test API".to_string(),
            collection_description: None,
            endpoints: vec![EndpointDoc {
                name: "Create user".to_string(),
                method: "POST".to_string(),
                url: "https://api.example.com/users".to_string(),
                headers: vec![],
                params: vec![],
                body_type: "json".to_string(),
                body: "{\"name\":\"John\"}".to_string(),
                auth_type: "None".to_string(),
                description: "".to_string(),
            }],
        };
        let html = docs_to_html(&doc);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("POST"));
        assert!(html.contains("Create user"));
        assert!(html.contains("class=\"method POST\""));
    }
}
