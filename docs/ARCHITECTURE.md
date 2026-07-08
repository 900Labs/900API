# Architecture Overview

## System Design

900API is a Tauri v2 desktop application with a Rust backend and Svelte 5 frontend. The architecture follows the 900 Labs ecosystem pattern established by 900Invoice and 900Word.

### High-Level Data Flow

```
User → Svelte 5 UI → Tauri IPC → Rust Backend
                                    ├── HTTP Engine (reqwest) → Target API
                                    ├── GraphQL Engine → Target GraphQL endpoint / schema introspection
                                    ├── Script Sandbox (boa_engine) → REST pre-request / test scripts and Test Runner scripts
                                    ├── SQLite DB → Collections, Requests, Response Examples, Environments, History
                                    ├── Local JSON State → Git sync config, plugin manifest registry, team workspaces
                                    └── File I/O → Export/Import (900API JSON, Postman, OpenAPI; REST cURL import, CLI cURL export)
```

### Layers

1. **Frontend (Svelte 5 + TailwindCSS)**: UI components for the REST workbench, request building, collections, environments, tests, and documentation. Uses Svelte 5 Runes plus small shared stores for cross-panel state such as loaded requests and active environments. Communicates with the backend via Tauri IPC (`invoke`).

2. **Tauri IPC Layer**: Bridges frontend and backend. Commands are defined in `src-tauri/src/commands/` and registered in `lib.rs`.

3. **Rust Backend**: Handles HTTP requests (`reqwest`), SQLite database operations, file I/O for import/export, local workflow-state persistence, and sandboxed script execution.

4. **SQLite Database**: Local file at `{APP_DATA_DIR}/900api.db`. Stores collections, requests, saved response examples, environments, history, and settings.

5. **Local JSON Workflow State**: Local files under `{APP_DATA_DIR}` store Git sync configuration, the plugin manifest registry, and team workspaces. These surfaces are local-first and do not imply cloud collaboration.

6. **CLI Crate (`api900-cli`)**: Standalone binary for headless collection execution and format export in CI/CD pipelines.

## Data Model

### SQLite Schema

```sql
collections (id, name, description, parent_id, sort_order, created_at, updated_at)
requests (id, collection_id, name, method, url, headers, params, body_type, body,
          auth_type, auth_config, pre_request_script, test_script, settings, sort_order,
          created_at, updated_at)
environments (id, name, variables, created_at, updated_at)
history (id, method, url, status, time_ms, size_bytes, request_snapshot, created_at)
response_examples (id, request_id, name, status, status_text, headers, body,
                   time_ms, size_bytes, created_at)
settings (key, value)
```

### Key Design Decisions

- **IDs**: UUID v4 for offline-safe creation (no server coordination needed)
- **Headers/Params**: Stored as JSON arrays in SQLite
- **Request settings**: Stored as JSON per saved request. Defaults keep the pooled client behavior; custom timeout, redirect, TLS, proxy, or cookie settings select a request-specific `reqwest` client.
- **Request scripts**: `pre_request_script` and `test_script` are stored with saved requests. The REST workbench runs them through the Rust Boa sandbox around sends; Test Runner imports the same saved scripts when converting saved requests into suites.
- **Response examples**: Stored as stable fixtures attached to saved requests, not transient history entries. They cascade when the parent request is deleted and are included in generated docs and native 900API JSON import/export.
- **Variables**: Stored as JSON in environments table
- **History**: Capped at 500 entries (auto-pruned). REST sends persist the original request snapshot before active environment variable resolution so history replay can reopen headers, params, body, auth, and `{{variable}}` placeholders.
- **WAL mode**: SQLite WAL journal mode for concurrent read/write

## Frontend Workbench Model

The REST client is composed as a workbench rather than a single form:

- `App.svelte` owns global navigation, the app menu bar, command palette state, keyboard shortcuts, and the local search index for saved requests, collections, environments, and recent history.
- `components/workspace/WorkspaceShell.svelte` lays out the REST workbench: collection/history/variable rail on the left, tabbed request editor on the right, and the active environment selector in the toolbar.
- `components/collections/CollectionTree.svelte` renders nested collections from `collections.parent_id`, exposes context menus, and supports moving requests or collections by drag/drop.
- `components/requests/RequestBuilder.svelte` owns request tabs, dirty state, send/save behavior, pre-request/test script editing and sandbox invocation, response rendering, formatted/raw/preview response modes, response examples, cURL import, code snippet generation, response body search, and environment-variable autocomplete.
- `components/requests/TestRunner.svelte` owns local persisted test suites and can import saved collection requests, including saved scripts, into runnable suites.
- `components/workspace/HistoryPanel.svelte` reads request history and opens stored request snapshots as new request tabs, falling back to method/URL for older rows without snapshots.
- `components/requests/GraphQLBuilder.svelte` owns GraphQL query execution, variables, headers, auth, response rendering, schema introspection, schema-powered query assist, searchable schema exploration, and root operation insertion from fields.
- `components/requests/GrpcClient.svelte` owns unary gRPC execution, raw hex/text request bodies, pasted `.proto` parsing, scalar protobuf field body generation, and hex/text response inspection. Full server reflection is not implemented in the backend yet.
- `components/workspace/EnvironmentSelector.svelte` sets the active environment used by request, GraphQL, and test execution paths.

Cross-component actions use narrow browser events with the `900api:*` prefix. Examples: `900api:new-request`, `900api:send-request`, `900api:save-request`, `900api:collections-changed`, `900api:history-changed`, and `900api:select-environment`. This keeps the workbench panels decoupled while avoiding a large global state framework.

Frontend code calls backend commands and event listeners through `src/lib/tauri.ts` rather than importing Tauri APIs directly. In the desktop app or `npm run tauri:dev`, the wrapper delegates to Tauri `invoke`/`listen`. In plain `npm run dev` browser preview, there is no Tauri IPC bridge, so the wrapper supplies harmless empty values for read-only list/version commands, no-op event listeners for stream previews, and a clear desktop-runtime-required error for actions that need the Rust backend.

## Offline Model

900API is offline-first by design:

1. **No cloud connections**: The only network calls are API requests the user explicitly makes
2. **No accounts**: No login, no registration, no authentication with any server
3. **No telemetry**: Zero analytics, tracking, or remote logging
4. **Local storage**: Core API data lives in SQLite; workflow configuration lives in local app-data JSON files
5. **Git-native collections**: Collections export as plain-text JSON for version control
6. **Manifest-only plugins**: Installed plugin manifests are persisted locally, but hook code is not executed in this release

## Project Structure

```
900API/
├── src/                          # Svelte 5 frontend
│   ├── components/               # UI components by feature
│   │   ├── workspace/            # App menu, command palette, REST workbench shell, history, environment selector
│   │   ├── requests/             # REST, GraphQL, WebSocket, SSE, gRPC, mocks, docs, sync, settings
│   │   ├── collections/          # Nested collection tree, context menus, drag/move behavior
│   │   ├── environments/         # Full environment editor and variable manager
│   │   └── tests/                # Test script editor and runner UI
│   └── lib/                      # Shared stores and i18n helpers
├── src-tauri/                    # Rust backend (Tauri v2)
│   └── src/
│       ├── commands/             # Tauri IPC command handlers
│       ├── models/               # Data structures
│       ├── db/                   # SQLite schema, migrations, queries
│       ├── http/                 # REST and GraphQL execution, variables, client settings, GraphQL introspection
│       ├── scripting/            # JS sandbox for scripts
│       ├── import/               # Postman and OpenAPI importers
│       ├── export/               # 900API JSON and OpenAPI exporters
│       └── docs/                 # API documentation generator
├── crates/
│   └── 900api-cli/               # Standalone CLI binary for CI/CD
├── docs/                         # Documentation
│   ├── adr/                      # Architecture Decision Records
│   └── sprints/                  # Sprint records
├── scripts/                      # Validation and release scripts
└── .github/                      # CI/CD workflows
```

## Performance Targets

| Metric | Target |
|---|---|
| App binary size | < 15MB |
| Idle RAM usage | < 100MB |
| Cold start time | < 2 seconds (4-year-old hardware) |
| Request send latency | < 50ms overhead over raw curl |
| Collection load (100 requests) | < 500ms |
| CLI collection run (50 requests) | < 10 seconds |
