# Architecture Overview

## System Design

900API is a Tauri v2 desktop application with a Rust backend and Svelte 5 frontend. The architecture follows the 900 Labs ecosystem pattern established by 900Invoice and 900Word.

### High-Level Data Flow

```
User → Svelte 5 UI → Tauri IPC → Rust Backend
                                    ├── HTTP Engine (reqwest) → Target API
                                    ├── GraphQL Engine → Target GraphQL endpoint
                                    ├── Script Sandbox (boa_engine) → Pre-request / Test scripts
                                    ├── SQLite DB → Collections, Environments, History
                                    └── File I/O → Export/Import (900API JSON, Postman; CLI cURL export)
```

### Layers

1. **Frontend (Svelte 5 + TailwindCSS)**: UI components for request building, collections, environments, tests, and documentation. Uses Svelte 5 Runes for state management. Communicates with the backend via Tauri IPC (`invoke`).

2. **Tauri IPC Layer**: Bridges frontend and backend. Commands are defined in `src-tauri/src/commands/` and registered in `lib.rs`.

3. **Rust Backend**: Handles HTTP requests (`reqwest`), SQLite database operations, file I/O for import/export, and sandboxed script execution.

4. **SQLite Database**: Local file at `{APP_DATA_DIR}/900api.db`. Stores collections, requests, environments, history, and settings.

5. **CLI Crate (`api900-cli`)**: Standalone binary for headless collection execution and format export in CI/CD pipelines.

## Data Model

### SQLite Schema

```sql
collections (id, name, description, parent_id, sort_order, created_at, updated_at)
requests (id, collection_id, name, method, url, headers, params, body_type, body,
          auth_type, auth_config, pre_request_script, test_script, sort_order,
          created_at, updated_at)
environments (id, name, variables, created_at, updated_at)
history (id, method, url, status, time_ms, size_bytes, request_snapshot, created_at)
settings (key, value)
```

### Key Design Decisions

- **IDs**: UUID v4 for offline-safe creation (no server coordination needed)
- **Headers/Params**: Stored as JSON arrays in SQLite
- **Variables**: Stored as JSON in environments table
- **History**: Capped at 500 entries (auto-pruned)
- **WAL mode**: SQLite WAL journal mode for concurrent read/write

## Offline Model

900API is offline-first by design:

1. **No cloud connections**: The only network calls are API requests the user explicitly makes
2. **No accounts**: No login, no registration, no authentication with any server
3. **No telemetry**: Zero analytics, tracking, or remote logging
4. **Local storage**: All data in a single SQLite file
5. **Git-native collections**: Collections export as plain-text JSON for version control

## Project Structure

```
900API/
├── src/                          # Svelte 5 frontend
│   ├── components/               # UI components by feature
│   │   ├── requests/             # Request builder, response viewer
│   │   ├── collections/          # Collection tree, folder management
│   │   ├── environments/         # Environment editor, variable manager
│   │   ├── tests/                # Test script editor, results panel
│   │   ├── docs/                 # API documentation generator/viewer
│   │   ├── import-export/        # Import/export dialogs
│   │   └── settings/             # App settings, themes
│   ├── stores/                   # Svelte 5 Runes state stores
│   ├── i18n/                     # Translation files
│   ├── utils/                    # URL parsing, formatting, validation
│   └── lib/                      # Tauri IPC wrappers
├── src-tauri/                    # Rust backend (Tauri v2)
│   └── src/
│       ├── commands/             # Tauri IPC command handlers
│       ├── models/               # Data structures
│       ├── db/                   # SQLite schema, migrations, queries
│       ├── http/                 # HTTP engine (reqwest wrapper)
│       ├── graphql/              # GraphQL query validation, introspection
│       ├── scripting/            # JS sandbox for scripts
│       ├── import/               # 900API JSON and Postman importers
│       ├── export/               # 900API JSON exporter
│       └── docs/                 # API documentation generator
├── crates/
│   └── 900api-cli/               # Standalone CLI binary for CI/CD
├── docs/                         # Documentation
│   ├── adr/                      # Architecture Decision Records
│   └── sprints/                  # Sprint records
├── scripts/                      # Validation and release scripts
└── .github/                      # CI/CD workflows
```

## Performance Targets

| Metric | Target |
|---|---|
| App binary size | < 15MB |
| Idle RAM usage | < 100MB |
| Cold start time | < 2 seconds (4-year-old hardware) |
| Request send latency | < 50ms overhead over raw curl |
| Collection load (100 requests) | < 500ms |
| CLI collection run (50 requests) | < 10 seconds |
