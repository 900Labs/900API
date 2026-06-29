# Privacy Model

## Core Guarantees

1. **No cloud connections** — The only network calls are API requests you explicitly make. 900API never connects to any 900 Labs server, analytics service, or third-party API.

2. **No accounts** — No login, no registration, no authentication with any server. The app works entirely without internet after download.

3. **No telemetry** — Zero analytics, tracking, remote logging, crash reporting, or usage statistics. We do not know you exist.

4. **Local storage only** — All data (collections, environments, history, settings) is stored in a local SQLite file at `{APP_DATA_DIR}/900api.db`. No data is transmitted anywhere unless you explicitly export it.

5. **No third-party services** — No CDN dependencies at runtime, no external fonts, no external resources loaded in the WebView.

## Data Flow

```
User Input → Svelte UI → Tauri IPC → Rust Backend
                                        ├── HTTP Engine → Target API (user-initiated only)
                                        └── SQLite → Local file (no network)
```

The only outbound network traffic is HTTP/HTTPS requests to the API endpoints you explicitly configure and send. These requests go directly from the Rust `reqwest` client to the target URL — they do not pass through any 900 Labs infrastructure.

## What We Don't Do

- We don't collect your email, name, or any personal information
- We don't track which APIs you test
- We don't store your API keys, tokens, or secrets anywhere except your local machine
- We don't send crash reports
- We don't use cookies
- We don't load external resources (fonts, scripts, stylesheets)
- We don't integrate with any analytics platform

## Verification

The privacy gate script (`scripts/verify-public-release.sh`) checks for:
- Hardcoded secrets in source code
- Telemetry/analytics code patterns
- Hardcoded local paths that could leak developer information

Run it before any release:
```bash
./scripts/verify-public-release.sh
```

## Export Safety

When you export collections or environments, the exported files may contain sensitive data (API keys, tokens, URLs). These files are written to the path you choose. Be careful when sharing exported files or committing them to version control — use environment variables for secrets and never commit `.env` files.
