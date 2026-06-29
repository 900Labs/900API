# Sprint 6: Import/Export, CLI Runner & Documentation Generation

## Scope
- Created Postman v2.1 collection import module (`src-tauri/src/import/mod.rs`)
  - Parses Postman JSON format with info, items, requests, folders
  - Converts Postman headers, body modes, and auth types (Bearer, Basic, API Key)
  - Recursively flattens folder structures into flat request list
  - 3 unit tests covering basic import, folders, and bearer auth
- Added `import_postman` Tauri IPC command
  - Creates a new collection from imported Postman data
  - All requests inserted with proper auth config
- Updated CollectionTree import to auto-detect format (900API native or Postman)
- Enhanced CLI runner (`crates/900api-cli`) with:
  - Environment file support (`-e`/`--environment` flag)
  - `{{var}}` variable resolution in URL, headers, and body
  - Environment file can be full `EnvironmentFile` JSON or raw variables array
  - `docs` subcommand for API documentation generation
    - Markdown format with endpoint tables, headers, body, test scripts
    - HTML format with styled method badges and syntax-highlighted code blocks
  - `export` subcommand supports Postman v2.1 and cURL formats
  - `run` subcommand supports console, JSON, and JUnit reporters
  - Exit code 0 for all pass, 1 for failures, 2 for errors
- Updated API docs with `import_postman` command

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 30 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅
- `cargo check -p api900-cli` — CLI compiles cleanly ✅

## CLI Usage

```bash
# Run a collection with environment
900api run collection.json -e environment.json -r json

# Export to Postman format
900api export collection.json -f postman -o output.json

# Export to cURL commands
900api export collection.json -f curl

# Generate Markdown documentation
900api docs collection.json -f markdown -o API.md

# Generate HTML documentation
900api docs collection.json -f html -o api-docs.html
```

## Decisions
- Postman import flattens folders — 900API uses flat request lists within collections
- CLI environment files accept both `{name, variables}` and raw `[{key, value, enabled}]` formats
- Documentation generation supports Markdown and HTML — HTML includes inline CSS for standalone use
- CLI uses `reqwest` directly (not the Tauri HTTP engine) for headless operation without Tauri runtime
- Variable resolution in CLI is simple string replacement (no nesting) for performance

## Known Issues
- OpenAPI/Swagger import not yet implemented (post-MVP)
- CLI test script execution not wired (requires boa_engine in CLI crate)
- No watch mode for CLI
- Documentation doesn't include response examples (post-MVP)

## MVP Complete
All 6 MVP sprints are now complete. The 900API client supports:
- REST and GraphQL request building
- Collections with save/load/export/import (900API + Postman formats)
- Environment variable management with `{{var}}` resolution
- Sandboxed JS test scripts with `api900.response` API
- CLI runner for CI/CD with console/JSON/JUnit reporters
- API documentation generation (Markdown/HTML)
- Full privacy: no telemetry, no cloud sync, local SQLite storage
