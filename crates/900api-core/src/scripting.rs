use boa_engine::object::builtins::JsArray;
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{
    js_string, Context, JsResult, JsString, JsValue as JsVal, NativeFunction, Source,
};
use serde::{Deserialize, Serialize};

const SCRIPT_LOOP_ITERATION_LIMIT: u64 = 100_000;
const SCRIPT_RECURSION_LIMIT: usize = 128;
const SCRIPT_STACK_SIZE_LIMIT: usize = 4096;
const MAX_LOGS: u64 = 500;
const MAX_TEST_RESULTS: u64 = 500;

const LOGS_GLOBAL: &str = "__900api_logs";
const RESULTS_GLOBAL: &str = "__900api_results";

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

    let logs_array = JsArray::new(&mut context);
    let results_array = JsArray::new(&mut context);
    context
        .register_global_property(
            js_string!(LOGS_GLOBAL),
            logs_array.clone(),
            Attribute::all(),
        )
        .map_err(|error| ScriptError::Execution(error.to_string()))?;
    context
        .register_global_property(
            js_string!(RESULTS_GLOBAL),
            results_array.clone(),
            Attribute::all(),
        )
        .map_err(|error| ScriptError::Execution(error.to_string()))?;

    let console = ObjectInitializer::new(&mut context)
        .function(
            NativeFunction::from_fn_ptr(console_log_native),
            js_string!("log"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(console_info_native),
            js_string!("info"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(console_warn_native),
            js_string!("warn"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(console_error_native),
            js_string!("error"),
            0,
        )
        .build();
    context
        .register_global_property(js_string!("console"), console, Attribute::all())
        .map_err(|error| ScriptError::Execution(error.to_string()))?;

    context
        .register_global_callable(
            js_string!("test"),
            2,
            NativeFunction::from_fn_ptr(test_native),
        )
        .map_err(|error| ScriptError::Execution(error.to_string()))?;

    let mut output = empty_output();
    match context.eval(Source::from_bytes(script)) {
        Ok(_) => {}
        Err(error) => output.error = Some(error.to_string()),
    }

    output.logs = read_logs(&logs_array, &mut context);
    output.test_results = read_results(&results_array, &mut context);
    Ok(output)
}

fn console_log_native(_this: &JsVal, args: &[JsVal], context: &mut Context) -> JsResult<JsVal> {
    console_push("log", args, context)
}

fn console_info_native(_this: &JsVal, args: &[JsVal], context: &mut Context) -> JsResult<JsVal> {
    console_push("info", args, context)
}

fn console_warn_native(_this: &JsVal, args: &[JsVal], context: &mut Context) -> JsResult<JsVal> {
    console_push("warn", args, context)
}

fn console_error_native(_this: &JsVal, args: &[JsVal], context: &mut Context) -> JsResult<JsVal> {
    console_push("error", args, context)
}

fn console_push(level: &str, args: &[JsVal], context: &mut Context) -> JsResult<JsVal> {
    let binding = context
        .global_object()
        .get(js_string!(LOGS_GLOBAL), context)?;
    let logs = binding
        .as_object()
        .and_then(|object| JsArray::from_object(object.clone()).ok());
    let Some(logs) = logs else {
        return Ok(JsVal::undefined());
    };
    if logs.length(context)? < MAX_LOGS {
        let rendered = args
            .iter()
            .map(|value| render_value(value, context))
            .collect::<Vec<_>>()
            .join(" ");
        logs.push(js_string!(format!("{level}: {rendered}")), context)?;
    }
    Ok(JsVal::undefined())
}

fn test_native(_this: &JsVal, args: &[JsVal], context: &mut Context) -> JsResult<JsVal> {
    let binding = context
        .global_object()
        .get(js_string!(RESULTS_GLOBAL), context)?;
    let results = binding
        .as_object()
        .and_then(|object| JsArray::from_object(object.clone()).ok());
    let Some(results) = results else {
        return Ok(JsVal::from(false));
    };

    let name = match args.first() {
        Some(value) => value.to_string(context)?.to_std_string_escaped(),
        None => "<unnamed>".to_string(),
    };

    let outcome = match args.get(1).and_then(JsVal::as_callable) {
        Some(callable) => match callable.call(&JsVal::undefined(), &[], context) {
            Ok(result) => Ok(result.to_boolean()),
            Err(error) => Err(error.to_string()),
        },
        None => Ok(args.get(1).is_some_and(|value| value.to_boolean())),
    };

    let (passed, message) = match outcome {
        Ok(true) => (true, String::new()),
        Ok(false) => (false, "assertion failed".to_string()),
        Err(error) => (false, error),
    };

    if results.length(context)? < MAX_TEST_RESULTS {
        let record = ObjectInitializer::new(context)
            .property(js_string!("name"), js_string!(name), Attribute::all())
            .property(js_string!("passed"), passed, Attribute::all())
            .property(js_string!("message"), js_string!(message), Attribute::all())
            .build();
        results.push(record, context)?;
    }

    Ok(JsVal::from(passed))
}

fn read_logs(logs: &JsArray, context: &mut Context) -> Vec<String> {
    let length = logs.length(context).unwrap_or(0);
    (0..length)
        .filter_map(|index: u64| {
            logs.at(index as i64, context)
                .ok()
                .and_then(|value| value.as_string().map(JsString::to_std_string_escaped))
        })
        .collect()
}

fn read_results(results: &JsArray, context: &mut Context) -> Vec<TestResult> {
    let length = results.length(context).unwrap_or(0);
    (0..length)
        .filter_map(|index: u64| {
            let object = results
                .at(index as i64, context)
                .ok()
                .and_then(|value| value.as_object().cloned())?;
            let name = object
                .get(js_string!("name"), context)
                .ok()
                .and_then(|value| value.as_string().map(JsString::to_std_string_escaped))
                .unwrap_or_default();
            let passed = object
                .get(js_string!("passed"), context)
                .ok()
                .is_some_and(|value| value.to_boolean());
            let message = object
                .get(js_string!("message"), context)
                .ok()
                .and_then(|value| value.as_string().map(JsString::to_std_string_escaped))
                .unwrap_or_default();
            Some(TestResult {
                name,
                passed,
                message,
            })
        })
        .collect()
}

fn render_value(value: &JsVal, context: &mut Context) -> String {
    value
        .to_string(context)
        .map(|string| string.to_std_string_escaped())
        .unwrap_or_else(|_| "<unprintable>".to_string())
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

    #[test]
    fn console_log_is_captured() {
        let output = run_test_script(
            "console.log('hello'); console.warn('careful'); console.error('bad');",
            "{}",
            200,
            "{}",
        )
        .unwrap();
        assert!(output.error.is_none());
        assert_eq!(output.logs.len(), 3);
        assert!(output.logs[0].contains("hello"));
        assert!(output.logs[1].starts_with("warn:"));
        assert!(output.logs[2].starts_with("error:"));
    }

    #[test]
    fn passing_test_is_recorded() {
        let output = run_test_script(
            "test('status is 200', () => api900.response.status === 200); test('body parses', () => { JSON.parse(api900.response.body); return true; });",
            "{\"ok\":true}",
            200,
            "{}",
        )
        .unwrap();
        assert!(output.error.is_none());
        assert_eq!(output.test_results.len(), 2);
        assert!(output.test_results.iter().all(|result| result.passed));
    }

    #[test]
    fn failing_test_is_recorded() {
        let output = run_test_script(
            "test('status is 201', api900.response.status === 201);",
            "{}",
            200,
            "{}",
        )
        .unwrap();
        assert!(output.error.is_none());
        assert_eq!(output.test_results.len(), 1);
        assert!(!output.test_results[0].passed);
    }

    #[test]
    fn throwing_test_body_is_recorded_as_failure() {
        let output = run_test_script(
            "test('explodes', () => { throw new Error('boom'); });",
            "{}",
            200,
            "{}",
        )
        .unwrap();
        assert!(output.error.is_none());
        assert_eq!(output.test_results.len(), 1);
        assert!(!output.test_results[0].passed);
    }
}
