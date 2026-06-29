# Sprint 13: Advanced Test Runner

## Scope
- Created test runner module (`src-tauri/src/test_runner/mod.rs`) with:
  - `Assertion` — assertion definition with type, target, operator, expected value
  - `AssertionType` — status, header, body, body_json_path, response_time, body_contains
  - `AssertionOperator` — equals, not_equals, contains, not_contains, greater_than, less_than, exists, not_exists
  - `TestSuite` — suite with name, request config, assertions, pre-request script, test script
  - `TestRequest` — serializable request config (method, url, headers, params, body, auth)
  - `TestSuiteResult` — per-suite result with assertion results, response status, time, errors
  - `TestRunResult` — aggregate result with total/passed/failed counts and duration
  - `run_test_suite` — sends HTTP request and evaluates all assertions
  - `run_test_suites` — runs multiple suites sequentially, returns aggregate results
  - `evaluate_assertion` — extracts value from response and checks against expected
  - `extract_json_path` — dot-notation JSON path extraction (e.g., `user.name`, `items.0`)
  - `check_assertion` — applies operator to compare actual vs expected
  - 9 unit tests covering JSON path extraction, assertion checking (equals, contains, greater_than, exists)
- Added 2 Tauri IPC commands: `run_test_suite`, `run_test_suites`
- Created `TestRunner.svelte` UI component with:
  - Suite list sidebar with pass/fail status icons and add/remove
  - Suite editor: name, method selector, URL input
  - Assertion builder: type selector, target input, operator selector, expected value
  - Per-assertion pass/fail icons and failure messages
  - Run All button with loading state
  - Aggregate result summary (passed/total, duration)
  - Per-suite result display (status, time, errors)
  - Default sample suite testing jsonplaceholder API
- Updated App.svelte to route Tests view to TestRunner component
- Updated API docs with test runner commands and types

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 59 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Assertions are declarative (type + target + operator + expected) rather than code-based
- JSON path extraction uses dot notation with array index support
- Test suites run sequentially (not parallel) to avoid rate limiting
- TestRunner replaces the old TestScripts view in the Tests tab

## Post-Audit Update
- Environment variables are now applied to test request URL, headers, params, body, and auth fields.
- `TestRunner.svelte` passes the active environment into `run_test_suites`.
- Pre-request scripts execute before the HTTP request; script errors fail the suite without sending the request.
- Test scripts execute after assertions; script errors fail the suite and are returned in `TestSuiteResult.error`.

## Known Issues
- No test suite persistence (in-memory only)
- No parallel test execution
- No test report export (JUnit, HTML)
- No chaining (using response from one test as input to next)
- No data-driven testing (CSV/JSON data sources)

## Next Sprint
- Sprint 14: API Documentation Enhancements
