# Sprint 11: Mock Server

## Scope
- Added `axum` and `tower-http` dependencies for HTTP server
- Created mock server module (`src-tauri/src/mock/mod.rs`) with:
  - `MockServer` — manages server lifecycle and graceful shutdown
  - `MockManager` — thread-safe `Arc<RwLock<HashMap<u16, MockServer>>>` for managing multiple servers
  - `MockRoute` — route definition with method, path pattern, status, headers, body, delay
  - `MockServerConfig` — port + routes configuration
  - `start_mock_server` — spawns axum server with catch-all fallback handler
  - `stop_mock_server` — graceful shutdown via oneshot channel
  - `get_mock_server_state` — returns running state and request count
  - `list_mock_servers` — returns all running server ports
  - Path matching with `:param` segments and `*` wildcard support
  - Method matching including `*`/`ANY` for any method
  - Configurable response delay per route
  - CORS support via `tower-http`
  - 5 unit tests for path matching (exact, params, no-match, wildcard, complex params)
- Added 4 Tauri IPC commands: `mock_start`, `mock_stop`, `mock_get_state`, `mock_list_servers`
- Added `MockManager` to `AppState`
- Created `MockServer.svelte` UI component with:
  - Server controls: port input, start/stop buttons, status indicator
  - Request count polling every 2 seconds while running
  - Route list sidebar with add/remove and method badges
  - Route editor: method selector, path input, status code, delay
  - Response headers editor with add/remove
  - Response body textarea
  - All fields disabled while server is running
  - Default sample route (`GET /api/hello`)
- Added Mock Server nav item to Sidebar (Server icon)
- Updated App.svelte with Mock Server view routing
- Updated API docs with mock server commands and types

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 46 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Used `axum` 0.7 for HTTP server (lightweight, async, tokio-compatible)
- Catch-all fallback handler matches all paths — routes matched in application code
- Path patterns support `:param` segments and `*` wildcard
- Multiple mock servers can run simultaneously on different ports
- Graceful shutdown via `tokio::sync::oneshot` channel
- Routes are immutable while server is running (must stop to edit)
- CORS enabled by default for cross-origin testing
- Default Content-Type is `application/json` if not specified in route headers

## Known Issues
- No mock route persistence (routes are in-memory only)
- No request logging/history
- No dynamic response templating (e.g., echoing path params)
- No conditional responses based on request headers/body
- No mock server configuration export/import
- No HTTPS support for mock server

## Next Sprint
- Sprint 12: Git-Native Sync
