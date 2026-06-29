# Sprint 14: API Documentation Enhancements

## Scope
- Created docs module (`src-tauri/src/docs/mod.rs`) with:
  - `ApiDoc` — collection-level documentation with name, description, and endpoints
  - `EndpointDoc` — per-endpoint documentation with method, URL, headers, params, body, auth
  - `generate_collection_docs` — generates docs from a saved collection in the database
  - `generate_all_docs` — generates docs from all saved collections
  - `docs_to_markdown` — converts ApiDoc to formatted Markdown with tables and code blocks
  - `docs_to_html` — converts ApiDoc to styled HTML with method-colored badges and tables
  - 3 unit tests covering markdown and HTML generation
- Added 5 Tauri IPC commands: `generate_collection_docs`, `generate_all_docs`, `docs_to_markdown`, `docs_to_html`, `write_text_file`
- Created `ApiDocs.svelte` UI component with:
  - Collection selector sidebar (shown when multiple collections exist)
  - Endpoint list with expandable cards showing method badge, URL, and name
  - Per-endpoint details: headers table, params table, body code block, auth type
  - Export to Markdown and HTML via file save dialog
  - Refresh button to reload docs from database
  - Empty state with icon and instructions
  - Color-coded HTTP method badges (GET=blue, POST=green, PUT=orange, DELETE=red, etc.)
- Updated App.svelte to route Docs view to ApiDocs component (replaced placeholder)
- Updated API docs with docs commands and types

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 62 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Docs generated from saved collections in the database (not from live requests)
- Headers and params parsed from JSON strings stored in `SavedRequest`
- HTML export includes inline CSS styling with method-colored badges
- Markdown export uses standard tables for headers/params and code blocks for body
- `write_text_file` command added for file export (avoids needing plugin-fs)
- Docs view replaces the previous "coming in Sprint 6" placeholder

## Known Issues
- No OpenAPI/Swagger format export
- No live preview of HTML export in app
- No search/filter for endpoints
- No request/response examples
- No schema documentation
- No try-it-from-docs functionality
- No custom branding/styling for HTML export

## Next Sprint
- Sprint 15: Performance & Polish
