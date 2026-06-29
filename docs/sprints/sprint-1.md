# Sprint 1: Project Foundation & Architecture

## Scope
- Initialized Tauri v2 + Svelte 5 + TailwindCSS v4 project
- Set up Rust workspace with `api900-cli` crate
- Created SQLite schema and migration system (collections, requests, environments, history, settings)
- Established project structure matching 900 Labs ecosystem conventions
- Created CI/CD workflow (`.github/workflows/ci.yml`)
- Created quality gate scripts (`verify-local.sh`, `verify-public-release.sh`)
- Wrote initial documentation: README, ARCHITECTURE, API, PRIVACY_MODEL, THREAT_MODEL, QUALITY_GATE, SPRINT_PROCESS, ROADMAP, PUBLIC_RELEASE, CONTRIBUTING, SECURITY, LICENSE
- Wrote ADRs 001–003 (tech stack, git-native storage, sandboxed JS)
- Built app shell with sidebar navigation and placeholder views
- Built request builder UI with method selector, URL bar, params/headers/body tabs
- Built collection tree UI placeholder
- Implemented Rust HTTP engine with `reqwest`
- Implemented Tauri IPC commands: `get_app_version`, `send_request`, `list_collections`, `create_collection`, `delete_collection`, `list_environments`, `create_environment`, `delete_environment`, `list_history`, `clear_history`
- Implemented CLI crate with `run` and `export` commands

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` passes ✅
- Project structure matches ecosystem conventions (src/, src-tauri/, crates/, docs/, scripts/, .github/) ✅
- Documentation covers all required areas (build, architecture, API, privacy, security, process, roadmap) ✅
- ADRs document key architectural decisions ✅

## Decisions
- ADR-001: Tauri v2 + Rust + Svelte 5 + TailwindCSS v4 (matches ecosystem)
- ADR-002: Hybrid storage (SQLite + JSON export for Git)
- ADR-003: `boa_engine` for sandboxed JS scripting
- Package name is `api900` (Cargo doesn't allow names starting with digits)
- CLI binary name is `900api` (the user-facing command)

## Known Issues
- Tauri icons are placeholder (generated from simple SVG)
- Frontend views for environments, tests, docs, settings are placeholders
- `cargo tauri dev` not yet tested end-to-end (requires system WebView dependencies)

## Next Sprint
- Sprint 2: Full HTTP engine testing against real APIs, auth types, response rendering for all content types
