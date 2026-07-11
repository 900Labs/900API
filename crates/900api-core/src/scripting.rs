use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{js_string, Context, Source};
use serde::{Deserialize, Serialize};

const SCRIPT_LOOP_ITERATION_LIMIT: u64 = 100_000;
const SCRIPT_RECURSION_LIMIT: usize = 128;
const SCRIPT_STACK_SIZE_LIMIT: usize = 4096;

#[derive(Debug, thiserror::Error)]
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
        return Ok(empty_output());
    }

    let mut context = Context::default();
    let limits = context.runtime_limits_mut();
    limits.set_loop_iteration_limit(SCRIPT_LOOP_ITERATION_LIMIT);
    limits.set_recursion_limit(SCRIPT_RECURSION_LIMIT);
    limits.set_stack_size_limit(SCRIPT_STACK_SIZE_LIMIT);

    let response = ObjectInitializer::new(&mut context)
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
    let api = ObjectInitializer::new(&mut context)
        .property(js_string!("response"), response, Attribute::all())
        .build();
    context
        .register_global_property(js_string!("api900"), api, Attribute::all())
        .map_err(|error| ScriptError::Execution(error.to_string()))?;

    match context.eval(Source::from_bytes(script)) {
        Ok(_) => Ok(empty_output()),
        Err(error) => Ok(ScriptOutput {
            error: Some(error.to_string()),
            ..empty_output()
        }),
    }
}

fn empty_output() -> ScriptOutput {
    ScriptOutput {
        logs: Vec::new(),
        test_results: Vec::new(),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_response_and_reports_failure() {
        let output = run_test_script(
            "if (api900.response.status !== 201) { throw new Error('expected 201'); }",
            "{}",
            200,
            "{}",
        )
        .unwrap();
        assert!(output.error.unwrap().contains("expected 201"));
    }

    #[test]
    fn infinite_loop_hits_runtime_limit() {
        let output = run_test_script("while (true) {}", "{}", 200, "{}").unwrap();
        assert!(output.error.is_some());
    }
}
