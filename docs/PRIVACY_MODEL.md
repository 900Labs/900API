# Privacy Model

900API does not require an account and does not contain telemetry, advertising, hosted sync, remote logging, crash reporting, or an automatic update service.

## Data Kept on the Device

The desktop app keeps working data in its operating-system application data directory:

- SQLite database with collections, saved requests, response examples, environments, request history, and settings
- Git Sync configuration
- plugin manifest metadata
- local workspace planning metadata

Optional JSON state uses recoverable file writes. A malformed optional file is preserved as a timestamped corrupt backup and the app continues with defaults.

The app can keep HTTP cookies locally when a request explicitly enables the cookie jar. Cookie persistence is off by default.

## Data That Can Leave the Device

Data leaves the device only through an action the user initiates or configures:

- sending REST, GraphQL, WebSocket, SSE, or gRPC traffic to a chosen endpoint
- exposing a mock server to the LAN after changing the default bind setting
- exporting a collection, environment, response, documentation file, or other local content
- committing or pushing collection files through a configured Git repository
- using a configured HTTP proxy

900API sends request data directly from the local Rust process. It does not proxy requests through 900 Labs infrastructure.

## Credentials

Literal API keys, tokens, passwords, and private URLs can be stored in local requests, environments, history snapshots, exported collections, generated snippets, and documentation. 900API does not claim to be a secret manager.

For repositories and shared exports:

- reference environment variables such as `{{token}}` instead of storing literal values
- review exported JSON, cURL, generated code, and documentation before sharing
- keep private environment files outside version control
- remove sensitive history entries when they are no longer needed

## Runtime Resources

The application bundles its interface resources. It does not load remote fonts, scripts, stylesheets, or analytics resources into the WebView.

## Verification

Run:

```bash
./scripts/verify-public-release.sh
```

The gate scans every tracked regular file and every untracked, nonignored regular file, regardless of its name or extension. This includes environment files, key files, text files, extensionless files, source, configuration, scripts, public documentation, and GitHub templates. Binary detection keeps non-text payloads from producing noisy false failures. The gate checks text files for known credential formats, likely assigned credentials, personal local paths, known developer identifiers, unintended email addresses, telemetry dependencies, and em dashes. A disposable self-test proves that representative file types are included without leaving credential fixtures in the working tree. Pattern scanning reduces accidental disclosure but does not replace human review.

The gate checks file content only. It does not inspect commit authors, committer identities, commit messages, branch names, tag annotations, remotes, or earlier Git history. Publication owners must review that metadata separately. The release preparation process does not rewrite Git history automatically.
