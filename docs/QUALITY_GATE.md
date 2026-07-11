# Quality Gate

Every pull request and release commit must pass:

```bash
./scripts/verify-local.sh
```

The script runs these checks in order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `npm ci`
4. `npm test`
5. `npm run check`
6. `npm run build`
7. `cargo test --workspace --locked`
8. `npm run check:docs`
9. `./scripts/test-privacy-gate.sh`
10. `./scripts/verify-public-release.sh`

The repository pins Rust 1.97.0, Rustfmt, and Clippy in `rust-toolchain.toml`. The test suite includes collection schema compatibility, root fallback for missing or cyclic portable parents, SQLite collection replacement, malformed optional state recovery, incremental SSE decoding with standards-correct end-of-stream handling, bounded JavaScript execution, browser-safe Tauri wrappers, delayed connection teardown races, privacy scanner file coverage, CLI process regressions for exit codes `1` and `2`, and positive and negative release artifact-set validation.

## Dependency Review

Before a public release, also run:

```bash
npm audit --audit-level=high
cargo audit
```

High and critical npm findings block release. Rust advisories and unmaintained transitive dependencies must be reviewed against actual reachability and documented before a release decision. An allowlist must be specific and kept in version control.

## Native Build

CI checks the portable code paths. A release also requires a native Tauri bundle smoke test on at least one available platform before tagging. The tag workflow then builds all supported release targets.

On macOS:

```bash
npm run tauri:build:dmg
```

The deterministic DMG helper is provided for local builds. The GitHub release workflow uses Tauri Action for platform packaging.

After all release build jobs finish, the workflow runs:

```bash
npm run validate:release-assets -- release-assets v0.2.1
```

The validator requires exactly one nonempty canonical file for each package type:

- `900API_<version>_aarch64.dmg`
- `900API_<version>_x64.dmg`
- `900API_<version>_amd64.AppImage`
- `900API_<version>_amd64.deb`
- `900API-<version>-<rpm release>.x86_64.rpm`
- `900API_<version>_x64_<locale>.msi`
- `900API_<version>_x64-setup.exe`

The only additional regular files allowed are nonempty `900API_<version>_aarch64.app.tar.gz` and `900API_<version>_x64.app.tar.gz` archives, plus an existing `SHA256SUMS.txt` from a rerun. The checksum file is ignored because the job regenerates it. Every stale, duplicate, differently named, or unexpected regular asset fails validation. This gate validates the release artifact set and package generation, not installation or upgrade behavior.

## Review Standard

- User-facing behavior and documentation must agree.
- Errors must be visible and actionable.
- Imports and persistence changes need round-trip or recovery tests.
- New network behavior requires a privacy and threat-model review.
- No generated files, personal paths, credentials, or unrelated local state may enter a commit.
- A passing build does not replace functional review.
