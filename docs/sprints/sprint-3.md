# Sprint 3: Collections & Environments

## Scope
- Implemented variable resolution engine in Rust (`http::variables`) with `{{var}}` syntax
  - Supports nested variables (up to 5 levels deep)
  - Resolves variables in URL, headers, params, and body
  - Disabled variables are skipped
  - Unresolved variables are preserved as-is
- Added `send_request` environment variable parameter — variables resolved before sending
- Created EnvironmentManager UI component with:
  - Create/delete environments
  - Tab-based environment selector
  - Variable editor (key, value, enabled toggle)
  - Save button to persist variables
  - Active environment published to Svelte store for RequestBuilder
- Enhanced CollectionTree with:
  - Expand/collapse collections
  - Request listing with method-colored labels
  - Click request to load into RequestBuilder via store
  - Export collection to JSON (file dialog)
  - Import collection from JSON (file dialog)
  - Delete collection and individual requests
- Added "Save to Collection" button in RequestBuilder with modal dialog
  - Save new request or update existing
  - Select target collection
  - Auto-generates name from method + URL
- Added collection export/import module (`src-tauri/src/export/mod.rs`)
  - JSON format with collection name, description, and all requests
  - Tauri commands: `export_collection`, `import_collection_file`
- Added `tauri-plugin-dialog` for native file dialogs
- Created Svelte stores for cross-component communication (`src/lib/stores.ts`)
  - `loadRequestStore` — load saved request into RequestBuilder
  - `activeEnvironmentStore` — active environment variables for request resolution
- Added 7 variable resolution tests + 2 export/import tests (22 total tests)

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 22 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅
- `./scripts/verify-public-release.sh` — all privacy gate checks pass ✅

## Decisions
- Variable resolution: `{{var_name}}` syntax (Postman-compatible), up to 5 nesting levels
- Export format: plain JSON with collection metadata and request array — Git-friendly
- Cross-component communication: Svelte stores (not Tauri events) for simplicity
- File dialogs: `tauri-plugin-dialog` for native OS file pickers
- Environment variables sent as optional parameter to `send_request` command

## Known Issues
- No active environment indicator in RequestBuilder UI
- No drag-and-drop reordering of requests within collections
- Export format is 900API-specific (Postman v2.1 import is Sprint 6)

## Next Sprint
- Sprint 4: GraphQL Support — query editor, variables panel, schema introspection
