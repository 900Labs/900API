# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-29

### Added
- HTTP request builder with support for GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- GraphQL query builder with variables and operation names
- WebSocket client with real-time message handling
- Server-Sent Events (SSE) client
- gRPC unary call support with TLS and plaintext
- Mock server with configurable routes, status codes, headers, and delays
- Git sync for collection versioning
- Collection management with import/export (Postman and custom JSON formats)
- Environment variable management with `{{variable}}` interpolation
- Test runner with assertions (status, header, body, JSON path, response time)
- Pre-request and test scripts using sandboxed Boa JS engine
- API documentation generation (Markdown and HTML)
- Plugin system with manifest-based permissions and lifecycle hooks
- Team workflows with workspaces, members, roles, and activity tracking
- Authentication support: Basic, Bearer, API Key, OAuth 1/2, AWS SigV4, Hawk
- Internationalization (i18n) with 6 locales: English, Spanish, French, German, Japanese, Chinese
- CLI tool (`900api`) for headless collection execution, export, and docs generation
- SQLite local storage with WAL mode
- Privacy gate CI check for hardcoded secrets, telemetry, and local paths
- Cargo audit in CI for dependency vulnerability scanning

### Security
- Path traversal prevention in `write_text_file` command (restricted to user home directory)
- Sandboxed JavaScript engine (Boa) for pre-request and test scripts
- TLS certificate validation enabled by default
- Minimal Tauri capabilities (core, shell, dialog only)
- No telemetry or analytics collection
- Mutex lock poison recovery to prevent panics on thread failures
