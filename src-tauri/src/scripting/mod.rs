use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{js_string, Context, Source};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const SCRIPT_LOOP_ITERATION_LIMIT: u64 = 100_000;
const SCRIPT_RECURSION_LIMIT: usize = 128;
const SCRIPT_STACK_SIZE_LIMIT: usize = 4096;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("JS execution error: {0}")]
    Execution(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptOutput {
    pub logs: Vec<String>,
    pub test_results: Vec<TestResult>,
    pub error: Option<String>,
}

pub fn run_test_script(
    script: &str,
    response_body: &str,
    response_status: u16,
    response_headers: &str,
) -> Result<ScriptOutput, ScriptError> {
    if script.trim().is_empty() {
        return Ok(ScriptOutput {
            logs: vec![],
            test_results: vec![],
            error: None,
        });
    }

    let mut context = Context::default();
    let limits = context.runtime_limits_mut();
    limits.set_loop_iteration_limit(SCRIPT_LOOP_ITERATION_LIMIT);
    limits.set_recursion_limit(SCRIPT_RECURSION_LIMIT);
    limits.set_stack_size_limit(SCRIPT_STACK_SIZE_LIMIT);

    // Build the 900api.response object
    let response_obj = ObjectInitializer::new(&mut context)
        .property(
            js_string!("status"),
            response_status as i32,
            Attribute::all(),
        )
        .property(
            js_string!("body"),
            js_string!(response_body),
            Attribute::all(),
        )
        .property(
            js_string!("headers"),
            js_string!(response_headers),
            Attribute::all(),
        )
        .build();

    let api_obj = ObjectInitializer::new(&mut context)
        .property(js_string!("response"), response_obj, Attribute::all())
        .build();

    // Register api900 in the global scope
    context
        .register_global_property(js_string!("api900"), api_obj, Attribute::all())
        .map_err(|e| ScriptError::Execution(e.to_string()))?;

    // Execute the script
    let result = context.eval(Source::from_bytes(script));
    match result {
        Ok(_) => Ok(ScriptOutput {
            logs: vec![],
            test_results: vec![],
            error: None,
        }),
        Err(e) => Ok(ScriptOutput {
            logs: vec![],
            test_results: vec![],
            error: Some(e.to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_script() {
        let output = run_test_script("", "{}", 200, "{}").unwrap();
        assert!(output.error.is_none());
        assert!(output.logs.is_empty());
    }

    #[test]
    fn test_syntax_error() {
        let output = run_test_script("var x = ;", "{}", 200, "{}").unwrap();
        assert!(output.error.is_some());
    }

    #[test]
    fn test_simple_script() {
        let output = run_test_script("var x = 1 + 2;", "{}", 200, "{}").unwrap();
        assert!(output.error.is_none());
    }

    #[test]
    fn test_response_access() {
        let script = "var s = api900.response.status;";
        let output = run_test_script(script, "{\"ok\":true}", 200, "{}").unwrap();
        assert!(output.error.is_none());
    }

    #[test]
    fn test_response_body_access() {
        let script = "var b = api900.response.body;";
        let output = run_test_script(script, "{\"ok\":true}", 200, "{}").unwrap();
        assert!(output.error.is_none());
    }

    #[test]
    fn test_infinite_loop_hits_runtime_limit() {
        let output = run_test_script("while (true) {}", "{}", 200, "{}").unwrap();
        assert!(output.error.is_some());
    }
}
