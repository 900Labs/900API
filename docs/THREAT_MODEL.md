# Threat Model

## Assets

1. **User's API data** — Collections, environments, request/response history, API keys, tokens
2. **User's filesystem** — The SQLite database and exported files
3. **User's network traffic** — API requests sent through 900API

## Threats and Mitigations

### 1. Data Exfiltration via Network
**Threat:** An attacker could modify the app to send user data to a remote server.
**Mitigation:** The app makes no network connections except user-initiated API requests. The privacy gate script checks for telemetry/analytics code. The Rust HTTP engine (`reqwest`) only sends requests when the user explicitly triggers them. No background network activity.

### 2. Script Injection via Pre-Request/Test Scripts
**Threat:** Malicious scripts could access the filesystem or make unauthorized network calls.
**Mitigation:** Pre-request and test scripts run in a sandboxed JS engine (`boa_engine` or `deno_core`) embedded in Rust. The sandbox has no access to the filesystem, network, or OS APIs. Scripts can only interact with the request/response through a controlled API (`900api.expect`, `900api.response`, etc.).

### 3. Malicious API Responses
**Threat:** An API could return a response designed to exploit the response viewer.
**Mitigation:** Response bodies are displayed as text with no script execution. The WebView CSP restricts script sources. Response rendering uses text content, not HTML parsing.

### 4. SQLite Database Corruption
**Threat:** Power failure or crash could corrupt the database.
**Mitigation:** SQLite WAL mode provides crash recovery. UUIDs prevent ID conflicts. The database is a single file that can be backed up by copying.

### 5. Import of Malicious Collection Files
**Threat:** A malicious collection file could contain scripts or data designed to exploit the app.
**Mitigation:** Imported collections are parsed as data only. Scripts in imported collections run in the sandbox. Import parsers validate file structure before processing.

### 6. Sensitive Data in Exported Files
**Threat:** Exported files may contain API keys or tokens that users accidentally share.
**Mitigation:** Documentation warns users about this risk. Environment variables can be used for secrets. Users are responsible for managing exported file security.

## Not in Scope

- **Attacks against target APIs**: 900API is a client tool. Security of target APIs is the API provider's responsibility.
- **Local filesystem attacks**: If an attacker has filesystem access, they can read the SQLite database directly. This is outside 900API's threat model.
- **Supply chain attacks**: Dependencies are managed via Cargo and npm. We rely on the Rust and npm ecosystems for dependency security.
