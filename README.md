# 900API

900API is a local-first desktop client for building, sending, testing, and documenting API requests. It supports REST, GraphQL, WebSocket, Server-Sent Events, unary gRPC, local mock servers, collections, environments, saved response examples, and headless collection runs from the command line.

No account is required. There is no subscription, telemetry, hosted workspace, or vendor lock-in. Request data stays on your computer unless you choose to send a request, export a file, or push a configured Git repository.

## Why 900API Exists

[900 Labs](https://www.900labs.com) builds open source tools for people and organizations constrained by software cost, bandwidth, or older hardware. 900API is part of [900 Open](https://www.900labs.com/impact), guided by four practical principles:

- Free forever under a permissive license
- Quality that can support real work
- Local-first operation for low-bandwidth connections, intermittent internet, and older hardware
- Open development maintained with the community

900API uses Tauri, Rust, and the operating system WebView so it does not need to bundle a full browser engine. The app is designed to remain useful offline after installation. Network access is used only for endpoints and Git remotes that you configure.

## What You Can Do

- Send REST requests with query parameters, headers, authentication, JSON, raw text, text-only multipart fields, and URL-encoded forms
- Organize requests in collections and nested folders
- Switch between local environments with `{{variable}}` substitution
- Save response examples and compare later responses
- Run bounded JavaScript pre-request and test scripts
- Run exported collections in CI with the `900api` command
- Query GraphQL APIs and inspect schemas
- Connect to WebSocket and SSE endpoints with bounded local event history
- Send unary gRPC requests from raw protobuf bytes or the scalar field helper
- Run local mock endpoints bound to `127.0.0.1` by default
- Import Postman, OpenAPI, Swagger, cURL, and 900API collection data where supported by the relevant screen
- Export portable 900API JSON or OpenAPI 3.0.3 JSON
- Keep portable collection files in a Git repository without a hosted sync account

Multipart file parts and binary request bodies are not supported in 0.2.1. This avoids storing machine-specific file paths in portable collection files.

## Install

Download the build for your operating system from the [GitHub Releases page](https://github.com/900Labs/900API/releases). The `v0.2.1` release workflow produces macOS arm64 and Intel DMGs, Linux AppImage, Debian, and RPM packages, plus Windows MSI and setup packages. Each completed release includes `SHA256SUMS.txt`.

Before creating checksums, the workflow requires exactly one nonempty canonical 900API file for each of the seven package types. Only exact current-version macOS app archives and an existing checksum file from a rerun are also allowed. This validates the published artifact set and package generation. It does not test installation or upgrade behavior.

The first public CI builds are unsigned. Windows SmartScreen and macOS Gatekeeper may warn before opening them. macOS builds use an ad-hoc identity so downloaded test builds are not treated as damaged, but they are not Apple-notarized. See [Public Releases](docs/PUBLIC_RELEASE.md) for verification and platform notes.

### Build From Source

You need:

- Rustup, using the repository-pinned Rust 1.97.0 toolchain
- Node.js 22
- The [Tauri 2 system prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system

```bash
git clone https://github.com/900Labs/900API.git
cd 900API
npm ci
npm run tauri:dev
```

Build a desktop bundle:

```bash
npm run tauri:build
```

`npm run dev` opens the interface in a normal browser for layout work. Sending requests, local persistence, imports, exports, mocks, and Git operations require `npm run tauri:dev` or a built desktop app.

## Send Your First Request

1. Open **REST** in the left navigation.
2. Choose `GET` and enter `https://httpbin.org/get`, or use an endpoint you control.
3. Add query parameters, headers, authentication, or an environment if needed.
4. Select **Send**.
5. Review the response status, time, headers, and body.
6. Select **Save** to add the request to a collection.

For low-bandwidth use, choose a small endpoint for the first test. 900API does not contact a 900 Labs service during this flow.

## Collections, Environments, and Git

Collections store reusable requests, scripts, request settings, and saved response examples in local SQLite. Environments store local values that can be referenced as `{{baseUrl}}`, `{{token}}`, or another key.

The Git Sync screen exports a selected local collection into a versioned `900api.collection/v1` JSON file. Importing a synced file writes the complete collection back into SQLite. Existing collections with the same portable ID are replaced in one database transaction, which keeps Git round trips predictable.

Collection files can contain credentials if you save literal authentication values. Use environment variables for secrets and review files before committing them.

## CLI

Build the runner:

```bash
cargo build --release -p api900-cli
```

Run a collection exported by the desktop app:

```bash
./target/release/900api run ./my-collection.json
./target/release/900api run ./my-collection.json --environment ./environment.json --reporter junit
```

The runner resolves variables, sends request headers and parameters, supports portable request body modes and common authentication types, and executes saved scripts in the same bounded Boa sandbox used by the desktop app. Reporter values are `console`, `json`, and `junit`. It exits with code `1` for HTTP failures and thrown saved scripts. It exits with code `2` for malformed files, unknown reporters, invalid request configuration, and other runner input errors.

Export options:

```bash
./target/release/900api export ./my-collection.json --format postman --output postman.json
./target/release/900api export ./my-collection.json --format curl
./target/release/900api docs ./my-collection.json --format markdown --output API.md
```

## Local Data and Privacy

The desktop app stores working data in its operating-system application data directory:

- `900api.db` for collections, requests, examples, environments, and history
- `sync-config.json` for Git Sync settings
- `plugins.json` for plugin manifest metadata
- `team-workspaces.json` for local workspace planning metadata

Optional JSON state is written through a recoverable temporary-file flow. If an optional state file is malformed, 900API moves it to a timestamped `.corrupt-*` backup, starts with defaults, and logs the problem.

Read [Privacy Model](docs/PRIVACY_MODEL.md) and [Security Policy](SECURITY.md) before using production credentials.

## Contributing

Community contributions are welcome, including bug reports from less common operating systems, accessibility fixes, import compatibility, translations, tests, and documentation improvements.

Start with [CONTRIBUTING.md](CONTRIBUTING.md), [SUPPORT.md](SUPPORT.md), and the [Documentation Index](docs/README.md). All pull requests must pass:

```bash
./scripts/verify-local.sh
```

## License

900API is free software under the [MIT License](LICENSE). You may use, modify, and distribute it, including for commercial work.

## Security

Do not open a public issue for a vulnerability or exposed credential. Follow [SECURITY.md](SECURITY.md) and contact `security@900labs.com`.
