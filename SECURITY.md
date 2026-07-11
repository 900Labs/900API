# Security Policy

## Supported Versions

Security fixes are applied to the latest public release and the current `main` branch. Older binaries may not receive patches.

## Report a Vulnerability

Do not open a public GitHub issue for a vulnerability, exposed credential, or private endpoint.

Email `security@900labs.com` with:

- affected version and operating system
- a clear description of the issue
- minimal reproduction steps or a proof of concept
- expected impact
- any suggested mitigation

Avoid sending real customer data, production credentials, or unrelated personal information. The maintainers will confirm receipt and coordinate disclosure based on severity and release availability.

## Security Boundaries

- API traffic goes directly from the local process to endpoints configured by the user.
- The app has no hosted account service, telemetry, remote logging, or automatic crash reporting.
- Pre-request and test scripts run in a bounded Boa context without filesystem, network, DOM, process, or module access.
- Mock servers bind to `127.0.0.1` by default. LAN binding and permissive CORS require explicit settings.
- HTTPS certificate verification is enabled by default but can be disabled per request.
- Generated HTML documentation escapes user-controlled content.
- Portable collection files can contain credentials if users save literal values. They are not encrypted secret containers.
- Git commands run in the sync directory selected by the user and can contact remotes configured in that repository.

See [Threat Model](docs/THREAT_MODEL.md) and [Privacy Model](docs/PRIVACY_MODEL.md) for more detail.
