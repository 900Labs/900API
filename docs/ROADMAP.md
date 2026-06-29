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
- Request builder UI (method, URL, headers, params, body)
- Response viewer (status, time, size, headers, body)
- Request history
- Basic auth, Bearer token, API key auth

### Sprint 3: Collections & Environments
- Collection tree UI (folders, requests, drag-and-drop)
- Collection CRUD
- Environment manager UI
- Variable resolution engine (`{{var}}` syntax)
- Collection export/import (JSON)

### Sprint 4: GraphQL Support
- GraphQL request type with query editor
- Variables panel
- Schema introspection and explorer
- Auto-complete from introspected schema
- Response viewer for GraphQL

### Sprint 5: Test Scripts & Pre-Request Scripts
- Sandboxed JS runtime (`boa_engine`)
- Pre-request script editor
- Test script editor with assertion support through the test runner
- Test results panel
- Collection-level test runner

### Sprint 6: Import/Export, CLI Runner & Documentation Generation
- Postman Collection v2.1 import
- 900API native JSON import/export
- CLI export to Postman Collection v2.1 and cURL commands
- CLI runner (`900api run`, `900api export`)
- API documentation generation (HTML, Markdown)

## Post-MVP

### Phase 2: Real-Time Protocols & Advanced Features
- **Sprint 7**: WebSocket support
- **Sprint 8**: Server-Sent Events (SSE)
- **Sprint 9**: gRPC support
- **Sprint 10**: Advanced authentication (OAuth 2.0, OAuth 1.0a, AWS Sig v4, Hawk, NTLM)
- **Sprint 11**: Mock server

### Phase 3: Collaboration & Productivity
- **Sprint 12**: Git-native sync (file watching, diff viewer, merge conflict resolution)
- **Sprint 13**: Advanced test runner (data-driven testing, request chaining, scheduled runs, HTML reports)
- **Sprint 14**: API documentation enhancements (interactive docs, custom branding, OpenAPI live editing)
- **Sprint 14+**: cURL import, OpenAPI 3.x / Swagger 2.0 import, and OpenAPI export
- **Sprint 15**: Performance & polish (memory optimization, cold start, keyboard shortcuts, command palette, search)

### Phase 4: Ecosystem & Community
- **Sprint 16**: Internationalization (6 languages: English, French, Spanish, Arabic RTL, Swahili, Hindi)
- **Sprint 17**: Plugin system (custom auth, response viewers, import/export formats)
- **Sprint 18**: Team workflows (shared environment templates, collection templates, workspace bundles)
