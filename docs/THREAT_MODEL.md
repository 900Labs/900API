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
**Mitigation:** Pre-request and test scripts run in a sandboxed JS engine (`boa_engine`) embedded in Rust. The sandbox has no access to the filesystem, network, DOM, or OS APIs. Scripts can only read the controlled `api900.response` object. Loop, recursion, and stack limits are configured before script evaluation.

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

### 7. Generated Documentation Injection
**Threat:** Collection names, request names, URLs, headers, params, or bodies could contain HTML or script content that executes when exported docs are opened.
**Mitigation:** HTML documentation generation escapes user-controlled content before writing it into the document. Unknown HTTP methods are rendered with a safe fallback CSS class instead of being used directly as arbitrary class names.

### 8. Mock Server Exposure
**Threat:** A local mock server could be reachable from the LAN or accept arbitrary browser origins without the user intending it.
**Mitigation:** Mock servers bind to `127.0.0.1` by default and permissive CORS is disabled by default. LAN binding (`0.0.0.0`) and permissive CORS are explicit opt-in settings.

## Not in Scope

- **Attacks against target APIs**: 900API is a client tool. Security of target APIs is the API provider's responsibility.
- **Local filesystem attacks**: If an attacker has filesystem access, they can read the SQLite database directly. This is outside 900API's threat model.
- **Supply chain attacks**: Dependencies are managed via Cargo and npm. We rely on the Rust and npm ecosystems for dependency security.
