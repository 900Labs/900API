# Architecture

900API is a Tauri 2 desktop application with a Svelte 5 interface and a Rust backend. It is local-first: the desktop app does not require a 900 Labs account or hosted service.

## Runtime Boundaries

```text
Svelte interface
  -> Tauri command bridge
    -> Rust request engines -> endpoint chosen by the user
    -> SQLite -> local API workspace data
    -> optional JSON state -> Git settings, plugin metadata, workspace plans
    -> filesystem -> explicit import, export, docs, and Git operations

900API CLI
  -> shared collection parser and Boa sandbox
    -> endpoint described by the collection
```

The frontend calls Rust commands through `src/lib/tauri.ts`. In a Tauri runtime, the wrapper delegates to the real command and event APIs. In a plain Vite browser preview, read-only commands return isolated empty values and desktop-only actions return a clear runtime error. Browser preview is for interface work, not API execution.

## Main Modules

- `src/components/requests/RequestBuilder.svelte`: REST tabs, request editing, save/send flow, scripts, response rendering, examples, cURL import, and snippets
- `src/components/collections/CollectionTree.svelte`: nested collection navigation and request movement
- `src/components/workspace/`: workbench layout, menu, command palette, history, and environment selection
- `src-tauri/src/commands/mod.rs`: Tauri command boundary
- `src-tauri/src/db/`: SQLite schema, migrations, transactions, and queries
- `src-tauri/src/http/`: REST, GraphQL, variables, request settings, and shared clients
- `src-tauri/src/export/` and `src-tauri/src/import/`: portable formats and external format adapters
- `src-tauri/src/sync/`: configured Git directory and Git command execution
- `src-tauri/src/persistence.rs`: recoverable atomic writes for optional local JSON state
- `crates/900api-core/`: versioned collection format and bounded Boa script runner shared by desktop and CLI
- `crates/900api-cli/`: headless collection runner, exports, and documentation generation

## Storage

SQLite stores:

```text
collections(id, name, description, parent_id, sort_order, created_at, updated_at)
requests(id, collection_id, name, method, url, headers, params, body_type, body,
         auth_type, auth_config, pre_request_script, test_script, settings,
         sort_order, created_at, updated_at)
response_examples(id, request_id, name, status, status_text, headers, body,
                  time_ms, size_bytes, created_at)
environments(id, name, variables, created_at, updated_at)
history(id, method, url, status, time_ms, size_bytes, request_snapshot, created_at)
settings(key, value)
```

SQLite uses foreign keys and WAL mode. Collection imports run in one transaction. Importing a portable collection with an existing collection ID replaces its requests and examples together. Request and collection deletion cascades to owned data.

Request history is capped at 500 entries. WebSocket messages and SSE events shown by the frontend are also capped at 500 per connection.

Optional JSON state includes Git Sync configuration, plugin manifest metadata, and local workspace plans. Writes use a temporary file and recoverable previous file. Malformed JSON is moved to a timestamped `.corrupt-*` backup and defaults are loaded.

## Portable Collection Format

Desktop export, Git Sync, and CLI use `900api.collection/v1` from `api900-core`. The format keeps these fields as structured JSON:

- collection ID, name, description, parent ID, and sort order
- request ID, method, URL, headers, parameters, body type, and body
- authentication type and configuration
- pre-request and test scripts
- request timeout and related settings
- response examples

The parser accepts the legacy 0.1 format where headers, parameters, settings, authentication, and example headers were JSON-encoded strings. New exports always use structured values.

Binary bodies and multipart file paths are rejected by the portable schema. Supported request bodies are JSON, raw text, text-only multipart fields, URL-encoded fields, and no body.

## Script Runtime

The desktop and CLI call the same bounded Boa runner. Scripts can read:

- `api900.response.status`
- `api900.response.body`
- `api900.response.headers`

The runtime has no filesystem, network, DOM, `process`, `require`, or module access. Loop, recursion, and stack limits are set before evaluation. A thrown error fails the request test. Pre-request scripts use the same runtime with an empty response and status `0`.

## Network Behavior

900API can connect to:

- REST, GraphQL, WebSocket, SSE, and gRPC endpoints entered by the user
- Git remotes configured by the user in the selected sync repository
- local mock server ports started by the user

The application does not include telemetry, hosted account sync, remote logging, automatic update checks, or a 900 Labs API dependency.

Mock servers bind to `127.0.0.1` unless the user explicitly selects LAN exposure. TLS certificate validation is enabled by default. Per-request settings can change timeouts, redirects, proxy use, cookie persistence, and certificate verification.

## Release Architecture

CI runs the same `scripts/verify-local.sh` gate used by contributors. Tags matching `v*` start a Tauri matrix build for macOS arm64, macOS x86_64, Ubuntu 22.04, and Windows x86_64. The workflow publishes platform assets and a SHA-256 checksum file.

Repository signing secrets are not required for the first release. Without them, Windows and macOS artifacts are unsigned. macOS CI uses an ad-hoc identity for bundle integrity, not Apple notarization.
