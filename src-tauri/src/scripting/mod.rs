pub use api900_core::scripting::{run_test_script, ScriptOutput};

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
