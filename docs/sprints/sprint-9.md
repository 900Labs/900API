# Sprint 9: gRPC Support

## Scope
- Added `hex` crate and `http2` feature to `reqwest` for gRPC support
- Created gRPC module (`src-tauri/src/grpc/mod.rs`) with:
  - `send_grpc_unary` — sends unary gRPC calls over HTTP/2 with proper gRPC framing
  - 5-byte gRPC frame: [compressed flag (1 byte)] [length (4 bytes BE)] [protobuf message]
  - Supports both TLS (https) and plaintext (h2c with `http2_prior_knowledge`)
  - Custom metadata headers support
  - Response parsing with gRPC frame extraction
  - Hex encoding/decoding for raw protobuf bytes
  - 6 unit tests covering hex decode/encode, empty input, spaces, invalid input, roundtrip
- Added `send_grpc` Tauri IPC command
- Created `GrpcClient.svelte` UI component with:
  - Address bar with h2c/TLS selector
  - Service/method path input
  - Hex-encoded protobuf body editor
  - Metadata (headers) panel with add/remove
  - Response viewer with tabs: Response (hex formatted), Headers, Trailers
  - HTTP status, gRPC status, timing, and size display
  - Copy response hex button
- Added gRPC nav item to Sidebar (Cable icon)
- Updated App.svelte with gRPC view routing
- Updated API docs with `send_grpc` command and `GrpcResponse` type

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 36 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Raw protobuf bytes (hex-encoded) — no proto schema parsing required
- This is a "raw gRPC" mode that works without .proto files
- `http2_prior_knowledge` used for plaintext gRPC (h2c) connections
- TLS gRPC uses standard HTTPS with HTTP/2 negotiation
- Unary calls only — streaming gRPC deferred to future sprint
- Proto file parsing and JSON-to-protobuf conversion deferred to future enhancement

## Known Issues
- No .proto file parsing or schema-aware message construction
- No JSON-to-protobuf conversion (requires proto descriptor)
- No gRPC streaming (server-streaming, client-streaming, bidi-streaming)
- No gRPC-Web support
- No gRPC reflection/service discovery
- gRPC trailers not fully parsed from response

## Next Sprint
- Sprint 10: Advanced Authentication (OAuth 2.0, OAuth 1.0a, AWS Sig v4, Hawk, NTLM)
