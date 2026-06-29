# Quality Gate

All pull requests must pass the quality gate before merging.

## Required Checks

### 1. Rust Compilation
```bash
cargo check
```
Must pass with zero warnings.

### 2. Rust Tests
```bash
cargo test
```
All tests must pass.

### 3. Svelte/TypeScript Check
```bash
npm run check
```
Must pass with zero errors and zero warnings.

### 4. Privacy Gate (for releases)
```bash
./scripts/verify-public-release.sh
```
Checks for hardcoded secrets, telemetry code, and local paths.

## Automated CI

The `.github/workflows/ci.yml` workflow runs the quality gate and privacy gate on every push and pull request to `main`.

## Local Verification

Run the full quality gate locally before opening a PR:
```bash
./scripts/verify-local.sh
```

## Public Release Verification

Run these before publishing binaries or making a release public:

```bash
./scripts/verify-public-release.sh
npm audit --audit-level=high
cargo audit
npm run tauri:build
```

Current dependency-audit policy:
- `npm audit --audit-level=high` must report zero high or critical vulnerabilities.
- `cargo audit` must report no blocking vulnerabilities. Existing allowed warnings are transitive RustSec warnings from Tauri/Wry's Linux GTK stack, Boa's `paste` dependency, and Tauri URL pattern dependencies.
- `npm run tauri:build` must produce the platform app bundle and installer artifact.

## Coding Standards

- **Rust**: No `unwrap()` or `expect()` in production code. Use `thiserror` for error types. All public functions need rustdoc.
- **Svelte**: Use Svelte 5 Runes. TypeScript for all new files. No `any` types without justification.
- **CSS**: Use TailwindCSS utility classes and theme variables.
- **Documentation**: Behavioral changes must include matching documentation updates.
