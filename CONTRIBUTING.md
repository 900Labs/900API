# Contributing to 900API

900API is community-maintained software from 900 Labs. Contributions are welcome from any region and experience level. Reports from low-bandwidth networks, intermittent connections, older hardware, and less common operating-system setups are especially useful because those environments are part of the product requirement.

Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Set Up the Project

You need Rustup, Node.js 22, Git, ripgrep, and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system. The repository pins Rust 1.97.0 with Rustfmt and Clippy in `rust-toolchain.toml`.

```bash
git clone https://github.com/900Labs/900API.git
cd 900API
npm ci
npm run tauri:dev
```

Use `npm run dev` only for browser layout work. Backend commands require Tauri.

## Choose Work

- Search existing issues before opening or implementing a duplicate.
- For a substantial change, open an issue describing the user problem and proposed boundary.
- Keep pull requests focused. Separate unrelated cleanup from behavior changes.
- Do not add hosted accounts, telemetry, analytics, or a required cloud service.

## Code Expectations

Rust changes should use `Result` and meaningful error messages at runtime boundaries. Avoid `unwrap()` and `expect()` in production paths where failure can come from user input, files, state, or the network.

Svelte changes should use the existing Svelte 5 and TypeScript patterns. Keep controls accessible at the 900px minimum width and test both the desktop runtime and browser preview when changing the Tauri wrapper.

Collection format changes belong in `crates/900api-core`. Desktop export, Git Sync, and CLI must not grow separate schemas.

Update documentation in the same pull request when behavior, commands, setup, supported formats, or release requirements change.

## Run the Gate

```bash
./scripts/verify-local.sh
```

The gate formats and lints Rust, performs a clean npm install, runs frontend and Rust tests, checks TypeScript and Svelte, builds the frontend, and scans public files for privacy leaks. The npm tests include positive and negative coverage for release artifact-set validation.

For dependency changes, also run:

```bash
npm audit --audit-level=high
cargo audit
```

## Pull Requests

1. Create a branch from current `main`.
2. Add focused tests for the behavior and failure path.
3. Run the full local gate.
4. Complete the pull request template with verification evidence and screenshots for visible changes.
5. Wait for CI and review before merging.

Maintainers may ask for a smaller patch when the proposed scope raises review or regression risk.

## Privacy and Security

Never commit real API keys, tokens, customer URLs, personal filesystem paths, private collection exports, environment files, or local databases.

Do not report vulnerabilities in a public issue. Follow [SECURITY.md](SECURITY.md) and contact `security@900labs.com`.
