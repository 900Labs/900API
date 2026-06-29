# Audit Remediation Report

Date: 2026-06-29

This is the tracked builder handoff for the audit remediation pass. It documents the verified issues that were fixed, documentation that was updated, validation that passed, and residual non-blocking dependency warning debt.

## Executive Summary

The public-release blockers identified in the audit were remediated. The codebase now has safer behavior for generated HTML docs, mock server exposure, script execution, test runner execution, docs file writes, CLI networking, and generated CLI output.

Current readiness after this pass: **Beta-ready for public open-source release**, assuming maintainers accept the documented transitive RustSec warnings from Tauri/Wry, Boa, and Tauri URL pattern dependencies.

Release artifacts were rebuilt successfully:
- `target/release/bundle/macos/900API.app`
- `target/release/bundle/dmg/900API_0.1.0_aarch64.dmg`

## Fixed Issues

### Generated HTML Documentation Escaping

Files:
- `src-tauri/src/docs/mod.rs`
- `crates/900api-cli/src/main.rs`

Fixed:
- Escaped collection names, descriptions, endpoint names, URLs, headers, params, body content, body types, and auth labels before rendering HTML.
- Added a safe `OTHER` CSS class for unknown HTTP methods.
- Added regression tests for malicious HTML/script content.

Docs updated:
- `docs/API.md`
- `docs/THREAT_MODEL.md`
- `docs/sprints/sprint-14.md`

### Mock Server Exposure

Files:
- `src-tauri/src/mock/mod.rs`
- `src/components/requests/MockServer.svelte`

Fixed:
- Default bind host changed to `127.0.0.1`.
- LAN exposure now requires explicit `bind_host: "0.0.0.0"`.
- Permissive CORS now requires explicit `cors_permissive: true`.
- Mock server state now returns `bind_host`, `cors_permissive`, and the live request counter.
- UI exposes LAN and CORS toggles while keeping them disabled during runtime.
- Added tests for loopback default, localhost normalization, explicit LAN binding, and invalid host rejection.

Docs updated:
- `README.md`
- `SECURITY.md`
- `docs/API.md`
- `docs/THREAT_MODEL.md`
- `docs/sprints/sprint-11.md`

### Test Runner Environment and Script Execution

Files:
- `src-tauri/src/test_runner/mod.rs`
- `src-tauri/src/http/variables.rs`
- `src/components/requests/TestRunner.svelte`

Fixed:
- Test requests now resolve environment variables in URL, headers, params, body, and auth fields.
- Main request sending also resolves auth fields.
- Test runner UI passes the active environment into `run_test_suites`.
- Pre-request scripts execute before the HTTP request and fail the suite on script error.
- Test scripts execute after assertions and fail the suite on script error.
- Added regression coverage for variable and auth resolution.

Docs updated:
- `docs/API.md`
- `docs/sprints/sprint-13.md`
- `docs/adr/ADR-003-sandboxed-js-scripting.md`

### JavaScript Runtime Limits

File:
- `src-tauri/src/scripting/mod.rs`

Fixed:
- Configured Boa loop, recursion, and stack limits before evaluating user scripts.
- Added regression coverage for an infinite loop returning an execution error.

Docs updated:
- `SECURITY.md`
- `docs/THREAT_MODEL.md`
- `docs/sprints/sprint-5.md`

### Safer Text File Writes

File:
- `src-tauri/src/commands/mod.rs`

Fixed:
- Rejects parent-directory traversal components.
- Canonicalizes the user home directory and target parent directory.
- Requires the target parent directory to exist inside the user's home directory.
- Refuses writes through symbolic-link targets.

Docs updated:
- `SECURITY.md`
- `docs/API.md`
- `docs/PRIVACY_MODEL.md`
- `docs/sprints/sprint-14.md`

### CLI Network and Output Safety

File:
- `crates/900api-cli/src/main.rs`

Fixed:
- Added CLI HTTP timeout and pool configuration matching the desktop client policy.
- Escaped JUnit suite and test case names.
- Added safe shell quoting for cURL export.
- Escaped CLI-generated HTML docs.
- Added CLI regression tests for HTML escaping, shell quoting, and malicious docs content.

## Documentation Sweep

Updated durable docs:
- `README.md`
- `SECURITY.md`
- `docs/API.md`
- `docs/ARCHITECTURE.md`
- `docs/PRIVACY_MODEL.md`
- `docs/PUBLIC_RELEASE.md`
- `docs/QUALITY_GATE.md`
- `docs/README.md`
- `docs/ROADMAP.md`
- `docs/THREAT_MODEL.md`
- `docs/adr/ADR-003-sandboxed-js-scripting.md`
- `docs/sprints/sprint-5.md`
- `docs/sprints/sprint-11.md`
- `docs/sprints/sprint-13.md`
- `docs/sprints/sprint-14.md`
- `docs/sprints/sprint-18.md`

Corrected stale claims:
- Removed current-release claims that cURL import and OpenAPI import/export are implemented.
- Documented current import/export support:
  - App: 900API native JSON import/export and Postman v2.1 import.
  - CLI: Postman v2.1 and cURL export.
  - OpenAPI and cURL import remain planned.
- Updated mock server config/state docs for `bind_host` and `cors_permissive`.
- Updated script docs to reflect the actual `api900.response` API and runtime limits.
- Removed stale `api900.expect` usage from the in-app script example.
- Added public-release checks for `npm audit`, `cargo audit`, and `npm run tauri:build`.

## Verification

| Command | Result | Notes |
|---|---|---|
| `cargo check` | PASS | Backend compiles |
| `cargo test` | PASS | 89 backend tests passed |
| `cargo test` in `crates/900api-cli` | PASS | 3 CLI tests passed |
| `cargo clippy` | PASS | No clippy failures |
| `npm run check` | PASS | Svelte and TypeScript passed |
| `npm run build` | PASS | Frontend production build passed with the existing Vite dynamic import warning |
| `./scripts/verify-local.sh` | PASS | Local quality gate passed |
| `./scripts/verify-public-release.sh` | PASS | Privacy release gate passed |
| `npm audit --audit-level=high` | PASS | 0 vulnerabilities |
| `cargo audit` | PASS with allowed warnings | 19 existing allowed warnings |
| `npm run tauri:build` | PASS | `.app` and `.dmg` produced |
| `git diff --check` | PASS | No whitespace errors |

## Residual Non-Blocking Warnings

`cargo audit` still reports 19 allowed warnings:
- GTK/glib warnings are transitive through Tauri/Wry's Linux GTK stack.
- `paste` is transitive through Boa.
- `unic-*` warnings are transitive through Tauri URL pattern dependencies.

These were not fixed because they are upstream/transitive dependency issues. Replacing them would require a broader dependency or runtime strategy change and should be tracked separately.

## Builder Follow-Up Guidance

Do not regress these controls:
- Do not re-enable default mock binding to `0.0.0.0`.
- Do not make permissive CORS the default.
- Do not write unescaped user-controlled content into generated HTML.
- Do not evaluate scripts without Boa runtime limits.
- Do not bypass `write_text_file` parent canonicalization and symlink refusal.
- Do not document planned formats such as OpenAPI export or cURL import as implemented.

Recommended future work:
- Add CLI JavaScript test execution only if the CLI can share or safely reuse sandboxed scripting behavior.
- Add wall-clock cancellation for scripts if Boa support or an execution wrapper makes that reliable.
- Add mock request logging/history if mock server observability becomes important.
- Track upstream RustSec transitive warnings and revisit them when Tauri/Wry/Boa releases update those dependency chains.
