# Sprint 2: HTTP Engine & Request Builder (REST)

## Scope
- Enhanced Rust HTTP engine with auth support (Basic, Bearer, API Key)
- Added AuthConfig model and auth handling in HTTP engine
- Added Auth tab to RequestBuilder UI with type selector and credential fields
- Added response content-type detection (JSON, XML, HTML, text) with appropriate formatting
- Added copy-to-clipboard button for response body
- Added request CRUD operations to SQLite database (create, list, update, delete)
- Added environment variable update support to database
- Added Tauri IPC commands: `list_requests`, `create_request`, `update_request`, `delete_request`, `update_environment`
- Enhanced CollectionTree UI with expand/collapse, request listing, create/delete collection, delete request
- Added 9 Rust unit tests (4 model tests, 9 DB tests = 13 total)
- Updated API documentation with all new commands and types

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 13 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅
- `./scripts/verify-public-release.sh` — all privacy gate checks pass ✅

## Decisions
- Auth types: None, Basic, Bearer, API Key (header or query param) — covers most common REST API auth patterns
- Response formatting: JSON pretty-printed, XML/HTML get basic line-break indentation, plain text shown as-is
- Request storage: headers/params/auth stored as JSON strings in SQLite for flexibility
- Sort order: auto-incremented per collection for drag-and-drop support in future

## Known Issues
- "Save to Collection" button not yet wired in RequestBuilder UI (Sprint 3)
- Environment variable resolution (`{{var}}`) not yet implemented (Sprint 3)
- No request loading from collection click yet (Sprint 3)

## Next Sprint
- Sprint 3: Collections & Environments — variable resolution engine, save/load requests from UI, environment manager UI, collection export/import
