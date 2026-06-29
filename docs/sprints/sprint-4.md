# Sprint 4: GraphQL Support

## Scope
- Added GraphQL models: `GraphQLRequest`, `GraphQLResponse`
- Implemented `send_graphql` function in Rust HTTP engine
  - Sends POST with `query`, `variables`, and optional `operationName`
  - Supports environment variable resolution in URL and headers
  - Supports auth (Basic, Bearer, API Key)
  - Logs to history
- Added `send_graphql` Tauri IPC command
- Created `GraphQLBuilder.svelte` UI component with:
  - URL bar with POST indicator
  - Query editor textarea
  - Operation name input
  - Variables JSON editor
  - Headers tab with key-value editor
  - Auth tab (None, Bearer)
  - Response viewer with JSON formatting and copy button
  - Environment variable support via store
- Added GraphQL nav item to Sidebar (Network icon)
- Updated App.svelte with GraphQL view routing
- Updated API docs with `send_graphql` command

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 22 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- GraphQL always uses POST (standard approach, supports queries and mutations)
- Variables sent as JSON string, parsed in Rust — frontend doesn't need to validate
- Auth limited to None and Bearer in GraphQL UI (most common for GraphQL APIs)
- Schema introspection and auto-complete deferred to post-MVP (requires GraphQL schema parsing)

## Known Issues
- No schema introspection or explorer (post-MVP)
- No query auto-complete (post-MVP)
- GraphQL requests not saveable to collections yet (Sprint 6 or post-MVP)
- No syntax highlighting in query editor (post-MVP)

## Next Sprint
- Sprint 5: Test Scripts & Pre-Request Scripts — sandboxed JS runtime, assertion library
