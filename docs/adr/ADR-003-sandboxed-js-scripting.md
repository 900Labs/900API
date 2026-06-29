# ADR-003: Sandboxed JS Scripting

## Date
2026-06-29

## Status
Accepted

## Context

Pre-request scripts and test assertions need a JavaScript runtime (Postman-compatible syntax). Options:

1. **WebView JS engine** — Run scripts in the Svelte frontend's JS context. Risk: scripts can access the DOM, make fetch calls, and interfere with the app.
2. **`boa_engine`** — Pure Rust JS engine. No filesystem, no network, no DOM. Fully sandboxed by design. Slower than V8 but sufficient for test scripts.
3. **`deno_core`** — V8-based, fast, but heavier. Can be configured with no permissions but adds binary size.

## Decision

Use **`boa_engine`** (pure Rust JS engine) for pre-request and test scripts.

The sandbox exposes a controlled API:
- `900api.expect` — assertion library (status, header, body, time)
- `900api.response` — read-only access to the last response
- `900api.request` — read-only access to the current request config
- `900api.environment` — get/set environment variables
- `900api.log` — logging for debug output

No access to: filesystem, network, DOM, `require`, `import`, `process`.

## Consequences

- Scripts are Postman-compatible in syntax but not in API (different namespace)
- `boa_engine` may not support all JS features (no async/await initially)
- Binary size stays small (no V8 dependency)
- Security: scripts cannot exfiltrate data or modify the filesystem
- If `boa_engine` proves too limited, can switch to `deno_core` with `--no-permissions`
