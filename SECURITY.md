# Security Policy

## Reporting a Vulnerability

To report a security vulnerability, email **security@900labs.com** with:
- A description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge receipt within 48 hours and provide a timeline for a fix within 7 days.

## Disclosure

- We follow responsible disclosure
- We will credit reporters in release notes (unless they prefer to remain anonymous)
- Please do not publicly disclose vulnerabilities until a fix is released

## Security Architecture

900API is designed with privacy and security as foundational principles:

- **Offline-first**: No cloud connections. The only network calls are API requests you explicitly make.
- **No telemetry**: Zero analytics, tracking, or remote logging.
- **Local storage**: All data in a local SQLite file. No remote data storage.
- **Sandboxed scripting**: Pre-request and test scripts run in a sandboxed JS engine with no filesystem, network, DOM, `require`, `import`, or `process` access. Loop, recursion, and stack limits are configured before execution.
- **Local mock server defaults**: Mock servers bind to `127.0.0.1` by default. LAN exposure and permissive CORS are explicit opt-in settings.
- **Safe generated docs**: HTML documentation exports escape collection, request, header, parameter, body, and auth content before rendering.
- **Constrained docs writes**: The docs export write command requires an existing parent directory inside the user's home directory and refuses symbolic-link targets.
- **TLS by default**: All HTTPS requests use Rust's native TLS with certificate validation.

## Threat Model

See [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) for the complete threat model.
