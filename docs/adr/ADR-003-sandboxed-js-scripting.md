# ADR-003: Sandboxed JS Scripting

## Date
2026-06-29

## Status
Accepted

## Context

Pre-request scripts and test assertions need a JavaScript runtime. The project does not promise Postman script API compatibility.

1. **WebView JS engine**  -  Run scripts in the Svelte frontend's JS context. Risk: scripts can access the DOM, make fetch calls, and interfere with the app.
2. **`boa_engine`**  -  Pure Rust JS engine. A newly created context has no application filesystem, network, or DOM bindings unless the host adds them.
3. **`deno_core`**  -  V8-based, fast, but heavier. Can be configured with no permissions but adds binary size.

## Decision

Use **`boa_engine`** for pre-request and test scripts. Keep the runner in `api900-core` so desktop and CLI use the same limits and exposed values.

The sandbox currently exposes a controlled API:
- `api900.response.status`  -  read-only response status
- `api900.response.body`  -  read-only response body string
- `api900.response.headers`  -  read-only response headers JSON string

No access to: filesystem, network, DOM, `require`, `import`, `process`.
Loop, recursion, and stack limits are configured on the Boa context before evaluating user scripts.

## Consequences

- Scripts are JavaScript-compatible in syntax but not Postman-compatible in API
- `boa_engine` may not support all JS features (no async/await initially)
- No V8 runtime is added to the application bundle
- The current host bindings do not give scripts filesystem or network access
- If `boa_engine` proves too limited, can switch to `deno_core` with `--no-permissions`
- Richer helpers such as assertions, logging, request inspection, and environment mutation remain future API work.
