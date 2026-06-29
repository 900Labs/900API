# Public Release Checklist

Before changing repository visibility or publishing a release, verify:

## Repository
- [ ] `README.md` is accurate and up-to-date
- [ ] `CONTRIBUTING.md` reflects current process
- [ ] `SECURITY.md` is accurate
- [ ] `LICENSE` file is present and correct (MIT)
- [ ] `.gitignore` covers all build artifacts
- [ ] No hardcoded local paths in source code
- [ ] No hardcoded secrets in source code

## Documentation
- [ ] `docs/README.md` index is complete
- [ ] `docs/ARCHITECTURE.md` is current
- [ ] `docs/API.md` documents all Tauri commands
- [ ] `docs/PRIVACY_MODEL.md` is accurate
- [ ] `docs/THREAT_MODEL.md` is current
- [ ] `docs/QUALITY_GATE.md` reflects current checks
- [ ] `docs/SPRINT_PROCESS.md` is current
- [ ] `docs/ROADMAP.md` is up-to-date
- [ ] All ADRs are complete

## Code Quality
- [ ] `./scripts/verify-local.sh` passes
- [ ] `./scripts/verify-public-release.sh` passes
- [ ] No compiler warnings (Rust)
- [ ] No TypeScript errors or warnings
- [ ] No `unwrap()` or `expect()` in production Rust code

## CI/CD
- [ ] `.github/workflows/ci.yml` is configured
- [ ] CI passes on `main` branch

## Privacy
- [ ] No telemetry or analytics code
- [ ] No third-party network calls
- [ ] No external resources loaded at runtime
- [ ] All data storage is local (SQLite)
