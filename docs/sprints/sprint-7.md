# Sprint 7: WebSocket Support

## Scope
- Added `tokio-tungstenite` and `futures-util` dependencies for WebSocket support
- Created WebSocket module (`src-tauri/src/websocket/mod.rs`) with:
  - `WsManager` — thread-safe connection manager using `Arc<Mutex<HashMap>>`
  - `connect_websocket` — establishes WS connection, spawns read/write tasks
  - `send_websocket_message` — sends text messages via unbounded channel
  - `disconnect_websocket` — drops connection and cleans up
  - `get_websocket_state` — returns current state and message history
  - Tauri event emission for real-time state and message updates
- Added 4 Tauri IPC commands: `ws_connect`, `ws_send`, `ws_disconnect`, `ws_get_state`
- Added `WsManager` to `AppState` for shared connection management
- Created `WebSocketClient.svelte` UI component with:
  - URL bar with connect/disconnect buttons
  - Connection status indicator (disconnected/connecting/connected/error)
  - Real-time message log with direction arrows (→ sent, ← received)
  - Message type labels (text, binary, ping, pong, close)
  - Timestamps for each message
  - Message input with send button
  - Clear messages button
  - Event listeners for `ws-{id}-state` and `ws-{id}-message` Tauri events
- Added WebSocket nav item to Sidebar (Radio icon)
- Updated App.svelte with WebSocket view routing
- Updated API docs with WebSocket commands and types

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 30 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Used `tokio-tungstenite` with `rustls-tls-native-roots` for secure WS (wss://) support
- Connection management via `Arc<Mutex<HashMap>>` for thread-safe access from Tauri commands
- Real-time updates via Tauri events (`ws-{id}-state`, `ws-{id}-message`) — frontend listens
- Each connection gets a UUID for unique event namespacing
- Read and write tasks spawned as separate tokio tasks for concurrent operation
- Binary messages displayed as `[binary: N bytes]` in UI (not decoded)

## Known Issues
- No binary message sending (text only currently)
- No custom headers/subprotocols on WebSocket connect
- No reconnection logic
- No message filtering or search
- WebSocket connections not saveable to collections

## Next Sprint
- Sprint 8: Server-Sent Events (SSE) support
