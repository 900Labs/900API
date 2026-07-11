# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-07-11

### Added
- Strict cross-platform release asset validation for one canonical macOS arm64 and Intel DMG, Linux AppImage, Debian and RPM package, and Windows MSI and setup package.
- Vitest coverage for complete, missing, empty, stale, duplicate, masquerading, and unexpected release artifact sets.

### Changed
- Updated application, npm, Tauri, and Rust workspace versions to `0.2.1`.
- Pinned local and automated Rust builds to Rust `1.97.0` with the minimal rustup profile, Rustfmt, and Clippy.
- Updated official GitHub Actions versions used by CI and release workflows.
- Added a manual release workflow entry point for rebuilding an existing tag without creating or moving it.
- Bound every release job to the immutable commit resolved from the explicit tag by the quality job, with a fresh remote tag check before each tag-addressed release operation.
- Made artifact-set validation a prerequisite for generating and uploading `SHA256SUMS.txt`.

### Documentation
- Documented the `v0.2.1` packages, pinned toolchain, manual rerun path, artifact-set checks, and unsigned signing status.
- Clarified that package validation does not test installation or upgrade behavior.

## [0.2.0] - 2026-07-11

### Added
- Shared `api900-core` crate with the versioned `900api.collection/v1` format and bounded Boa script runner.
- Structured editors for text-only multipart fields and URL-encoded request forms.
- Browser-wrapper unit tests and a CLI process regression test for failing saved scripts.
- Recoverable atomic persistence for Git Sync settings, plugin manifests, and local workspace plans.
- GitHub issue forms, pull request template, support guide, code of conduct, and repository metadata guide.
- Tag-driven release workflow for macOS arm64, macOS x86_64, Ubuntu 22.04, and Windows x86_64, with SHA-256 checksums.

### Fixed
- Git Sync now exports actual SQLite collections and imports complete synced collections back into SQLite in one transaction.
- Portable nested collections import at the root when their parent is absent in the destination database.
- Desktop export, Git Sync, and CLI now use one compatible collection schema, while legacy string-encoded fields remain importable.
- The CLI executes saved scripts and exits with code `1` when a script fails, even when the HTTP response is successful.
- CLI help now lists only implemented export formats.
- Invalid structured request bodies now return clear errors instead of sending empty forms.
- Removed unsupported binary body and multipart file claims from the interface and documentation.
- Malformed optional JSON state is backed up and no longer prevents application startup.
- Collapsed navigation buttons now have accessible names and tooltips.
- WebSocket and SSE failures are visible in the interface, and displayed stream history is capped at 500 entries.
- SSE decoding now preserves fields, UTF-8 values, CRLF boundaries, multi-line data, and final buffered events across arbitrary network chunks.
- Live WebSocket and SSE backends are disconnected when their interface components are destroyed.
- CLI reporter and request configuration errors now exit with code `2`, while HTTP and saved-script failures remain exit code `1`.
- Workspace planning is labeled as local metadata rather than live collaboration.

### Changed
- Updated all application and package versions to `0.2.0`.
- Expanded the release gate to enforce Rust formatting, warnings-as-errors Clippy, clean frontend installation, frontend tests, type checks, production build, workspace tests, and privacy scanning.
- Expanded privacy scanning to tracked source, configuration, scripts, documentation, and GitHub templates.
- Updated the release matrix to the current Intel macOS runner and added a pre-publication tag and package-version guard.

### Documentation
- Rewrote the README around the 900 Labs and 900 Open mission, supported workflows, first request, collections, environments, Git Sync, CLI, privacy, and contribution paths.
- Added truthful unsigned-build, ad-hoc macOS signing, checksum, and release workflow guidance.
- Removed internal remediation and sprint diary documents from the public documentation set.

## [0.1.1] - 2026-06-29

### Security
- Fix HTML injection in docs generation (desktop and CLI)
- Bind mock server to loopback-only by default; disable permissive CORS unless explicit
- Add Boa runtime limits for sandboxed pre-request and test scripts
- Harden docs file writes against path traversal and symlink attacks
- Add CLI request timeouts to prevent hanging
- Add XML escaping in CLI docs output
- Add shell-safe cURL export (proper quoting)
- Add HTML escaping in generated API docs

### Fixed
- Wire test runner environments, pre-request scripts, and test scripts end-to-end
- Correct stale docs that claimed OpenAPI/cURL import/export and `api900.expect` were implemented

### Documentation
- Update README, security, API, architecture, threat model, and privacy documentation.

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
- Plugin manifest registry with permission and hook metadata
- Local workspace planning with members, roles, and activity notes
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
