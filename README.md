# 900API

**API testing and documentation client — a Postman alternative that respects your privacy.**

Build, test, and document REST and GraphQL APIs with collections, environments, and automated testing. All offline. All local. No cloud. No accounts. No telemetry.

Built by [900 Labs](https://www.900labs.com) — building enterprise-grade open source tools for the 900 million+ people in developing economies who are priced out of the software that modern businesses depend on.

## The Problem

A developer in Lagos testing APIs pays the same $14/month for Postman as a developer in San Francisco — but in an economy where that's 10% of a monthly salary. Existing tools are cloud-first, telemetry-heavy, and bloated for older hardware. Every existing open-source API client is either Electron-based (heavy RAM), browser-only (limited offline), or lacks essential features like GraphQL support and CI/CD integration.

## The Solution

900API is a desktop API testing client that works completely offline. It runs on the hardware you already own. No subscriptions. No cloud dependencies. No internet required after the first download.

## Features

### Request Builder
- Full HTTP method support: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- Request headers, query parameters, and body types (JSON, form-data, x-www-form-urlencoded, raw, binary)
- Response viewer with status, timing, size, headers, and pretty-printed body
- Request history (last 500 requests)

### Collections
- Organize requests into collections and folders
- Drag-and-drop reordering
- Git-native JSON export for version control

### Environments
- Create and manage multiple environments (dev, staging, prod)
- Variable resolution with `{{variable_name}}` syntax
- Active environment selector

### GraphQL
- Full GraphQL query editor with syntax highlighting
- Schema introspection and explorer
- Auto-complete from introspected schema
- Variables panel for GraphQL variables

### Test Scripts
- Sandboxed JavaScript pre-request and test scripts
- Assertion library: status code, headers, body (JSON path, text, regex), response time
- Collection-level test runner with pass/fail summary

### Import / Export
- Import from Postman Collection v2.1, cURL, OpenAPI 3.x / Swagger 2.0
- Export to Postman Collection v2.1, cURL, OpenAPI 3.0

### CLI Runner
- `900api run <collection.json>` — headless collection execution for CI/CD
- Reporter formats: console, JSON, JUnit XML
- Environment file support: `-e <environment.json>`
- Exit code 0 on all tests pass, 1 on any failure

### API Documentation
- Generate static HTML or Markdown API docs from collections
- Include request examples, response examples, and descriptions

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri v2 |
| Backend | Rust 1.88+ |
| Frontend | Svelte 5 (Runes) |
| Styling | TailwindCSS v4 |
| Icons | Lucide |
| Data storage | SQLite (local file) |
| HTTP engine | `reqwest` (Rust) |
| Scripting | Sandboxed JS engine (Rust-embedded) |

**Why Tauri v2?** Tauri uses the OS native WebView instead of bundling Chromium (Electron). The result: ~15MB binary vs ~150MB, and ~100MB RAM vs ~500MB. On a 4-year-old laptop with 4 GB of RAM running three browser tabs, this difference is everything.

## Installation

### Releases
Tagged releases are published on the [releases page](https://github.com/900Labs/900API/releases/latest) when available. Until platform binaries are published, build from source.

### Build from Source
Prerequisites:
- Rust 1.88+ — install from [rustup.rs](https://rustup.rs)
- Node.js 20.19+, 22.12+, or 24+ — install from [nodejs.org](https://nodejs.org)
- Tauri CLI v2: `cargo install tauri-cli --version "^2"`
- Tauri v2 system dependencies — see [v2.tauri.app/start/prerequisites](https://v2.tauri.app/start/prerequisites/)

Linux (Ubuntu/Debian):
```bash
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
```

Build and run:
```bash
git clone https://github.com/900Labs/900API.git
cd 900API
npm install
cargo tauri dev          # Run in development mode (hot-reload)
cargo tauri build        # Build for production
```

Production app bundles are written under `src-tauri/target/release/bundle/`.

### CLI Only
```bash
cargo build --release -p api900-cli
# Binary at target/release/900api
```

## Data Storage

All data is stored locally in a single SQLite file. No cloud. No server.
- Location: `{APP_DATA_DIR}/900api.db`
- IDs: UUID v4 for offline-safe creation

Your data never leaves your machine unless you explicitly export it.

## Documentation

- [Documentation Index](docs/README.md) — sorted guide to public docs, ADRs, and sprint records
- [Architecture Overview](docs/ARCHITECTURE.md) — system design, data flow, and offline model
- [API Documentation](docs/API.md) — complete Tauri command reference
- [Privacy Model](docs/PRIVACY_MODEL.md) — privacy guarantees and data flow
- [Threat Model](docs/THREAT_MODEL.md) — security threats and mitigations
- [Quality Gate](docs/QUALITY_GATE.md) — required pre-merge validation
- [Sprint Process](docs/SPRINT_PROCESS.md) — sprint workflow and review policy
- [Roadmap](docs/ROADMAP.md) — feature roadmap and post-MVP plans

## Contributing

We welcome contributions from developers worldwide — especially those in the regions 900API serves. Every line of code from a developer in Lagos, Nairobi, Accra, or Mumbai makes this tool better for the people it's built for.

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, coding standards, sprint rules, and the PR process.

Quick contribution ideas:
- Add a translation for your language to `src/i18n/`
- Report bugs in your operating environment
- Improve documentation clarity
- Add import/export format support

## License

MIT License — see [LICENSE](LICENSE) for details.
You are free to use, modify, and distribute this software — including commercially. You do not owe us anything.

## Security

To report a vulnerability, email security@900labs.com. See [SECURITY.md](SECURITY.md) for the full process.

## Part of the 900 Labs Ecosystem

900API is part of the 900 Labs open-source portfolio:
- [900PDF](https://github.com/900-labs/900pdf)
- [900CRM](https://github.com/900-labs/900crm)
- [900Invoice](https://github.com/900Labs/900Invoice)
- [900Word](https://github.com/900Labs/900Word)
- **900API** (this project)

All tools are built on the same Tauri v2 + Rust + Svelte 5 stack. They share conventions, libraries, and the same commitment: free forever, offline-first, open source.

Learn more at [900labs.com/open-source](https://www.900labs.com/open-source).
