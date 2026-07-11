# Threat Model

## Protected Assets

- collections, environments, scripts, history, response examples, and workspace metadata
- literal API keys, tokens, passwords, private URLs, and response bodies stored by the user
- exported files and configured Git repositories
- network traffic sent to target APIs, proxies, Git remotes, and local mock clients

## Trust Boundaries

The Svelte interface is not trusted to perform privileged work directly. Filesystem, database, network, script, mock-server, and Git operations cross the Tauri command boundary into Rust.

The user controls target URLs, request data, imports, scripts, export destinations, proxy settings, mock-server exposure, and Git repository configuration. Target APIs, imported files, API responses, and Git remotes are untrusted inputs.

## Main Threats and Controls

### Script access to the machine

Imported or saved scripts could try to read files, start network requests, or interfere with the interface. Scripts run in a bounded Boa context shared by desktop and CLI. The context exposes only `api900.response` values and has no filesystem, network, DOM, process, or module API. Loop, recursion, and stack limits reduce denial-of-service risk from runaway scripts.

The limits do not make arbitrary scripts harmless. Users should review scripts before importing a collection.

### Malicious API responses

Response text and structured views do not execute scripts. Optional HTML preview uses an iframe with an empty `sandbox` attribute. The application content security policy limits script and resource sources. Generated HTML documentation escapes user-controlled collection and request content.

Large or endless responses can still consume memory or bandwidth. Response size limits and streaming large-body views remain roadmap work.

### Malicious imports

Importers parse data into typed structures. Portable collection imports validate schema, HTTP methods, and body types before a database transaction. Legacy structured fields are parsed as JSON rather than concatenated into commands. Imported scripts remain inert until the user runs the request or suite.

### Credential disclosure

Literal credentials can appear in SQLite, history snapshots, collection exports, environment files, generated cURL or source snippets, documentation, and Git commits. 900API does not encrypt these fields or act as a secret manager.

Documentation directs users to use variable references, keep private environment files outside Git, and review every export. The repository privacy gate protects project source, not a user's exported collection.

### Optional-state corruption

Interrupted writes could damage Git settings, plugin metadata, or workspace plans. These files use a recoverable temporary-file flow. Malformed state is preserved as a timestamped corrupt backup and defaults are loaded. SQLite uses WAL mode and transactions for collection replacement.

### Mock server exposure

Mock servers bind to `127.0.0.1` and do not enable permissive CORS by default. LAN binding and permissive CORS require explicit settings. A user who enables LAN exposure is responsible for local network trust and firewall rules.

### Git command execution

Git commands run in the sync directory selected by the user. Commit stages files in that directory, and pull or push can contact its configured remotes. 900API does not create credentials or validate remote ownership. Users should choose a dedicated repository and review Git status before committing.

### Transport configuration

HTTPS certificate verification is enabled by default. Users can disable it or configure a proxy per request, which changes the trust boundary. The interface exposes these choices in request settings.

### Supply chain

Cargo and npm dependencies are locked. CI runs npm and Rust dependency audits, but automated advisories are not complete protection. Release review includes lockfile changes, action versions, generated bundles, and known transitive warnings.

## Out of Scope

- protection from an attacker who already controls the user's operating-system account
- security of target APIs and Git providers
- encrypted storage or enterprise secret management
- code signing guarantees for unsigned initial release artifacts
- preventing a user from intentionally sending data to a configured endpoint or remote
