# Sprint 15: Performance & Polish

## Scope
- **HTTP Connection Pooling**: Replaced per-request `Client::builder()` with a shared `reqwest::Client` using `OnceLock`
  - Pool idle timeout: 90 seconds
  - Max idle connections per host: 20
  - Request timeout: 120 seconds
  - Connect timeout: 30 seconds
  - TCP_NODELAY enabled for lower latency
  - Applied to both `send_request` and `send_graphql_request` functions
- **Settings Page**: Created `Settings.svelte` with:
  - Application info (version, platform, engine)
  - HTTP engine configuration display (pooling, timeouts, HTTP/2)
  - Storage info (SQLite, location)
  - Feature list with status indicators
  - Replaced "coming soon" placeholder in App.svelte
- **UI Polish**: All placeholder views now have functional components

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 62 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Used `OnceLock<Client>` for thread-safe lazy initialization of shared HTTP client
- Connection pool settings chosen for typical API testing workloads
- TCP_NODELAY enabled to reduce latency for small request/response pairs
- Settings page is read-only (displays current configuration, no editable settings yet)

## Known Issues
- No configurable timeout/pool settings (hardcoded)
- No response caching
- No request retry logic
- No HTTP/3 support
- No theme switching (dark mode only)
- No keyboard shortcuts
- No recent requests history

## Next Sprint
- Sprint 16: Internationalization
