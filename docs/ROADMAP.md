# Roadmap

## MVP (Sprints 1–6)

### Sprint 1: Project Foundation & Architecture ✅
- Tauri v2 + Svelte 5 + TailwindCSS project scaffold
- Rust workspace with `api900-cli` crate
- SQLite schema and migration system
- Project structure matching ecosystem conventions
- CI/CD workflows
- Quality gate scripts
- Initial documentation (README, ARCHITECTURE, CONTRIBUTING, PRIVACY_MODEL, THREAT_MODEL)
- ADRs 001–003

### Sprint 2: HTTP Engine & Request Builder (REST)
- Rust HTTP engine with `reqwest`
- REST workbench UI with request tabs, collection/history/variable rail, method, URL, headers, params, body, auth, response panel, response search, response examples, saved-example comparison, cURL import, export/copy response actions, and code snippet generation
- Per-request runtime settings for timeout, redirects, SSL verification, proxy URL, and app-local cookie jar opt-in
- App menu bar, global command palette, and keyboard shortcuts for common actions, saved requests, collections, environments, and recent history
- Response viewer (status, time, size, headers, formatted/raw body, sandboxed HTML preview, saved-example comparison)
- Request history with stored request snapshots for replay
- Basic auth, Bearer token, API key auth

### Sprint 3: Collections & Environments
- Collection tree UI with nested folders/subcollections, context menus, request duplication, rename actions, and drag-to-move requests/collections
- Collection CRUD
- Environment manager UI
- Variable resolution engine (`{{var}}` syntax)
- Active environment selector and variable autocomplete/copy chips in the REST workbench
- Collection export/import (JSON)

### Sprint 4: GraphQL Support
- GraphQL request type with query editor
- Variables panel
- Response viewer for GraphQL
- Schema introspection and searchable schema explorer
- Schema-powered query assist for inserting root fields into the editor
- Root operation insertion from introspected fields

### Sprint 5: Test Scripts & Pre-Request Scripts
- Sandboxed JS runtime (`boa_engine`)
- Pre-request script editor
- Test script editor with assertion support through the test runner
- Test results panel
- Collection-level test runner

### Sprint 6: Import/Export, CLI Runner & Documentation Generation
- Postman Collection v2.1 import
- 900API native JSON import/export, including saved response examples
- OpenAPI 3.x / Swagger 2.0 JSON/YAML import and OpenAPI 3.0.3 JSON export
- CLI export to Postman Collection v2.1 and cURL commands
- CLI runner (`900api run`, `900api export`)
- API documentation generation (HTML, Markdown) with saved response examples

## Post-MVP

### Phase 2: Real-Time Protocols & Advanced Features
- **Sprint 7**: WebSocket support
- **Sprint 8**: Server-Sent Events (SSE)
- **Sprint 9**: gRPC support with unary h2c/TLS calls, raw hex mode, proto paste/parser helper, scalar field body builder, and hex/text response viewing
- **Sprint 10**: Advanced authentication (OAuth 2.0, OAuth 1.0a, AWS Sig v4, Hawk, NTLM)
- **Sprint 11**: Mock server

### Phase 3: Collaboration & Productivity
- **Sprint 12**: Git-native sync (file watching, diff viewer, merge conflict resolution)
- **Sprint 13**: Advanced test runner (data-driven testing, request chaining, scheduled runs, HTML reports)
- **Sprint 14**: API documentation enhancements (interactive docs, custom branding, OpenAPI live editing)
- **Sprint 15**: Performance & polish (memory optimization, cold start, full gRPC server reflection)

### Phase 4: Ecosystem & Community
- **Sprint 16**: Internationalization (6 languages: English, French, Spanish, Arabic RTL, Swahili, Hindi)
- **Sprint 17**: Plugin system (custom auth, response viewers, import/export formats)
- **Sprint 18**: Team workflows (shared environment templates, collection templates, workspace bundles)
