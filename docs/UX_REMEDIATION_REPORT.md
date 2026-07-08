# UX Remediation Report

Date: 2026-07-02

## Summary

This remediation pass addressed the product review finding that 900API had many feature labels but still felt like a basic set of isolated screens. The main change is a real REST workbench: collections, history, variables, tabs, request editing, sending, saving, and responses now operate in one connected workspace.

## Competitive Baseline Used

The product review compared 900API against current public documentation for:

- Postman collections, scripts, multi-protocol requests, generated docs, mocks, environments, runners, and collaboration: https://learning.postman.com/docs/getting-started/overview/
- Postman collection behavior: https://learning.postman.com/docs/use/use-collections/overview/
- Postman scripting model: https://learning.postman.com/docs/tests-and-scripts/write-scripts/intro-to-scripts/
- Insomnia local vault, Git sync, code generation, environments, and testing: https://developer.konghq.com/insomnia/
- Insomnia environment scopes and autocomplete: https://developer.konghq.com/insomnia/environments/
- Bruno offline-first, Git-friendly, plain-text collection model: https://docs.usebruno.com/introduction/getting-started
- Bruno Git integration: https://docs.usebruno.com/git-integration/overview
- Hoppscotch workspaces, collections, history, spotlight, shortcuts, scripts, importer, cookies, certificates, snippets, and context menus: https://docs.hoppscotch.io/documentation/getting-started/introduction
- Hoppscotch collections and subcollections: https://docs.hoppscotch.io/documentation/features/collections
- Hoppscotch environments and secrets/current values: https://docs.hoppscotch.io/documentation/features/environments

## What Changed

### REST Workbench

Added `src/components/workspace/WorkspaceShell.svelte`.

The REST route now opens into a workbench with:

- Left rail tabs for Collections, History, and Variables
- Main tabbed request editor
- Active environment selector
- Response panel integrated below the request editor
- New and Send toolbar actions

### App Menus and Command Palette

Added:

- `src/components/workspace/AppMenuBar.svelte`
- `src/components/workspace/CommandPalette.svelte`

Updated `src/App.svelte` to provide:

- File, Request, View, Tools, and Help menus
- Global command palette with actions, saved requests, collections, environments, and recent history
- Keyboard shortcuts:
  - `Ctrl/Cmd+K` opens the command palette
  - `Ctrl/Cmd+N` creates a REST request
  - `Ctrl/Cmd+S` saves the active request
  - `Ctrl/Cmd+Enter` sends the active request

The palette now refreshes a local index from SQLite-backed commands when opened and after collection/history/environment changes. Saved request results open directly in request tabs, history results replay stored snapshots, environment results activate the selected environment, and collection results jump to the collection manager.

### Browser Preview Runtime Guard

Added `src/lib/tauri.ts` and routed frontend Tauri command calls through it.

The frontend can now be opened with plain `npm run dev` for layout and browser smoke checks without throwing `TypeError: Cannot read properties of undefined (reading 'invoke')`. In real desktop runtime, calls still delegate to Tauri `invoke`. In browser preview mode, read-only list/version commands return empty preview data, while backend-backed actions return a clear desktop-runtime-required error.

### Request Runtime Settings

Updated `src/components/requests/RequestBuilder.svelte`, `src-tauri/src/http/mod.rs`, `src-tauri/src/models/mod.rs`, and `src-tauri/src/db/mod.rs`.

The REST request editor now includes a Settings tab with:

- Request timeout and connect timeout
- Redirect following toggle
- SSL certificate verification toggle
- Proxy URL field
- App-local cookie jar opt-in

Saved requests persist settings in the `requests.settings` JSON column. Native 900API exports include settings, imports restore them, and older requests without settings use defaults. The backend keeps the pooled default client for default requests, uses a shared in-memory cookie jar for cookie-enabled requests, and builds a request-specific client when custom proxy, TLS, redirect, or timeout behavior is needed.

During this pass, tests also exposed and fixed HTTP method serde names: the backend now accepts current frontend values such as `GET` and old snapshot aliases such as `g_e_t`.

### Request Tabs and Connected Request Flow

Rebuilt `src/components/requests/RequestBuilder.svelte`.

The request editor now supports:

- Multiple request tabs
- Dirty state indicators
- Close and duplicate tab actions
- Opening collection requests into tabs
- Opening history entries as new tabs
- Saving new or existing requests
- Copying cURL commands from the active request
- Copying response bodies
- Environment variable suggestions for URL, params, headers, and auth fields

Follow-up REST workflow pass added:

- cURL import dialog for common commands copied from browser devtools, docs pages, Postman, or Insomnia
- Code snippet dialog for cURL, JavaScript fetch, Python requests, and Go net/http
- Response body search with highlighted matches
- History replay from stored request snapshots so reopened history preserves method, URL, params, headers, body, auth, and unresolved `{{variable}}` placeholders

### Response Examples and Viewer Tools

Updated `src/components/requests/RequestBuilder.svelte`, `src-tauri/src/models/mod.rs`, `src-tauri/src/db/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/docs/mod.rs`, and `src-tauri/src/export/mod.rs`.

The REST response panel now supports formatted and raw body modes, sandboxed HTML preview for HTML responses, copy/export response body actions, response metadata display, search match counts, saved response examples, and a Compare tab. Response examples are attached to saved requests, stored in SQLite, cascade when the parent request is deleted, and can be loaded back into the response panel or compared against the current response.

The Compare tab lets users choose a saved response example as the expected payload, then reviews status, body, header, and size differences against the current response. JSON/XML/HTML bodies use the same formatting path as the normal response viewer before line-level body differences are calculated.

Native 900API JSON import/export now preserves saved response examples. Generated Markdown and HTML API documentation includes saved response examples, and HTML output continues to escape user-controlled collection, request, and response-example content before rendering.

### OpenAPI Import and Export

Updated `src-tauri/src/import/mod.rs`, `src-tauri/src/export/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, and `src/components/collections/CollectionTree.svelte`.

Collections can now import OpenAPI 3.x and Swagger 2.0 documents from JSON, YAML, or YML files. The importer creates editable REST requests from operations, resolves local `$ref` entries for common parameters/request bodies/schemas/security schemes, converts OpenAPI path variables such as `{id}` to 900API variables such as `{{id}}`, imports query/header parameters, generates starter request bodies from examples or schemas, and maps API key/basic/bearer/OAuth2 schemes to 900API auth placeholders.

Collections can also export OpenAPI 3.0.3 JSON from the collection context menu. Export includes request paths/methods, query/header parameters, JSON/form/raw request bodies, and supported security schemes. Native 900API JSON export remains the local backup and Git workflow format and now includes saved response examples.

The YAML parser uses `yaml_serde` from The YAML Organization rather than the deprecated `serde_yaml` crate.

### GraphQL Schema Explorer

Updated `src/components/requests/GraphQLBuilder.svelte`, `src-tauri/src/http/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/models/mod.rs`.

The GraphQL view now includes schema introspection, query assist, and a searchable schema explorer. Users can fetch the schema for the active endpoint using the same headers, bearer auth, and active environment variable resolution as normal GraphQL sends. The Query tab then exposes schema-powered operation suggestions for root query/mutation/subscription fields, including field arguments and return types, so users can insert starter operations without leaving the editor. The Schema tab displays root operation types, object/input/enum metadata, field arguments, deprecation metadata, and GraphQL type references. Root operation fields can be inserted into the query editor with starter selections and argument placeholders.

The backend exposes `introspect_graphql_schema`, runs the standard GraphQL introspection query, rejects GraphQL `errors` with the server-provided message, and returns a compact serializable schema summary instead of exposing raw introspection JSON directly.

### gRPC Proto Helper

Updated `src/components/requests/GrpcClient.svelte`.

The gRPC view is no longer raw hex only. It now supports unary calls over h2c or TLS, raw protobuf hex mode, UTF-8 byte mode, pasted `.proto` parsing for package/service/RPC/message/field discovery, service method population from parsed RPCs, scalar protobuf field composition, and response body viewing as hex or UTF-8 text with headers and trailers.

The field builder intentionally covers common scalar protobuf values, strings, bytes, enums as numeric values, floats/doubles, fixed-width values, and nested messages as already-encoded hex payloads. Full server reflection and descriptor-driven nested message editing remain separate backend-heavy work.

### Nested Collections, Context Menus, and Moves

Rebuilt `src/components/collections/CollectionTree.svelte`.

Collections now support:

- Nested folders/subcollections using `collections.parent_id`
- Context menus for collections and requests
- New request inside a collection
- New folder inside a collection
- Rename collection/request
- Duplicate request
- Export collection as 900API JSON or OpenAPI JSON
- Delete collection/request
- Drag a request into another collection
- Drag a collection into another collection or back to root

### History and Variables Panels

Added:

- `src/components/workspace/HistoryPanel.svelte`
- `src/components/workspace/EnvironmentPanel.svelte`
- `src/components/workspace/EnvironmentSelector.svelte`

The workbench now exposes request history and active environment variables inside the daily workflow rather than hiding them in separate views.

History entries now also display response size and pass their full stored request snapshot back to the request editor. Older history rows without snapshots still reopen by method and URL.

### Backend Commands and Storage

Updated:

- `src-tauri/src/models/mod.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`

Added or exposed:

- `Collection.parent_id`
- `Collection.sort_order`
- `update_collection`
- `move_collection`
- `move_request`
- `HistoryEntry.size_bytes`
- `HistoryEntry.request_snapshot`
- `ResponseExample`
- `list_response_examples`
- `create_response_example`
- `delete_response_example`

The backend rejects collection moves that would create cycles.
REST request history now stores the original `RequestConfig` before environment-variable substitution, which avoids leaking resolved secret values into replay snapshots while keeping history useful.

### Tests

Updated `src-tauri/src/db/tests.rs` with coverage for:

- Nested collection creation and move cycle rejection
- Moving requests between collections
- History request snapshot persistence
- Request runtime settings persistence
- Response example lifecycle, request delete cascade, and native export round trip
- HTTP client settings validation and cookie-client creation
- HTTP method serde compatibility for frontend payloads and older snapshots
- OpenAPI JSON/YAML import and OpenAPI JSON export
- GraphQL schema introspection response parsing and error handling

## Documentation Updated

Updated:

- `README.md`
- `docs/API.md`
- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/README.md`

Important correction: README and roadmap now state that inline cursor-aware GraphQL editor autocomplete remains planned, while schema introspection, query assist, and the schema explorer are current release features.

## Verification

Commands run:

| Command | Result |
|---|---|
| `npm run check` | Passed, 0 Svelte/TypeScript errors |
| `npm run build` | Passed |
| `cargo check` | Passed |
| `cargo test` | Passed, 110 tests total: 107 backend library tests and 3 CLI tests |
| `./scripts/verify-local.sh` | Passed |
| `./scripts/verify-public-release.sh` | Passed |
| `npm run tauri:build` | Passed, produced `target/release/bundle/macos/900API.app` |
| `npm run tauri:build:dmg` | Passed, produced `target/release/bundle/dmg/900API_0.1.1_aarch64.dmg` |
| Browser smoke at `http://127.0.0.1:1431/` | Passed, verified 900API render, REST Settings tab controls, and no plain-browser `undefined.invoke` console error |

Packaging note: Tauri's native Finder-styled DMG path can hang in non-interactive or sandboxed environments during the generated Finder/AppleScript styling step. The project now uses `scripts/build-macos-dmg.sh` for the default macOS DMG release path. The native Tauri DMG path remains available as `npm run tauri:build:native-dmg` for maintainers who explicitly want Finder-styled DMGs in a GUI-capable environment.

Native QA note: `npm run tauri:dev` compiled and launched successfully earlier in this remediation flow, but automated visual inspection of the macOS Tauri window was blocked by missing Screen Recording permission for this terminal/Codex session. Later browser smoke checks used port `1431` because port `1420` was occupied by a different local app during verification.

## Deferred Work

This pass intentionally did not implement:

- Inline cursor-aware GraphQL editor autocomplete from schema data
- Full gRPC server reflection and descriptor-driven nested message editing
- Request body binary file picker
- Client certificate UI
- Additional custom response visualizers
- Real collaborative accounts/cloud sync

Those should be handled as separate focused product slices.

## Builder Notes

The main architectural shift is that the REST client should no longer be treated as a single request form. Future request-related features should be added to the workbench flow first:

1. Left rail for discovery and saved state
2. Request tabs for active work
3. Response panel for immediate feedback
4. Menus/command palette/context menus for fast actions
5. Backend commands kept narrow and test-covered

Avoid adding new isolated screens for core REST workflows unless the workflow is genuinely separate from request editing.
