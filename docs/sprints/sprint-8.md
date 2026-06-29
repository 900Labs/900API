# Sprint 8: Server-Sent Events (SSE)

## Scope
- Created SSE module (`src-tauri/src/sse/mod.rs`) with:
  - `SseManager` — thread-safe connection manager using `Arc<Mutex<HashMap>>`
  - `connect_sse` — establishes SSE connection via `reqwest` streaming, parses SSE protocol
  - `disconnect_sse` — cancels connection via oneshot channel
  - `get_sse_state` — returns current state and event count
  - SSE protocol parsing: `data:`, `event:`, `id:`, `retry:` fields, empty line = event boundary
  - Tauri event emission for real-time state and event updates (`sse-{id}-state`, `sse-{id}-event`)
  - Cancellation support via `tokio::select!` on stream and cancel channel
- Added 3 Tauri IPC commands: `sse_connect`, `sse_disconnect`, `sse_get_state`
- Added `SseManager` to `AppState`
- Created `SseClient.svelte` UI component with:
  - URL bar with connect/disconnect buttons
  - Custom headers support (collapsible panel with add/remove)
  - Connection status indicator
  - Real-time event log with event type badges, timestamps, retry info
  - Event count in footer
  - Clear events button
  - Event listeners for Tauri events
- Added SSE nav item to Sidebar (Download icon)
- Updated App.svelte with SSE view routing
- Updated API docs with SSE commands and types

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 30 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- SSE uses `reqwest` streaming (already a dependency) — no new crates needed
- SSE protocol parsed manually from byte stream chunks (line-by-line)
- Custom headers supported for auth (e.g., Bearer token)
- Event type defaults to "message" when not specified by server
- `retry` field parsed and forwarded to UI but not used for auto-reconnect yet
- Cancellation via `oneshot` channel + `tokio::select!` for clean disconnect

## Known Issues
- No auto-reconnect using `retry` field
- No Last-Event-ID header on reconnect
- No event filtering by type
- SSE connections not saveable to collections

## Next Sprint
- Sprint 9: gRPC Support
