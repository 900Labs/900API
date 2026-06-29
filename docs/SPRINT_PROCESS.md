# Sprint Process

## Sprint Cycle

900API is developed in sprints. Each sprint has a defined scope, deliverables, and a mandatory review before the next sprint begins.

## Sprint Workflow

1. **Sprint Planning**: Scope is defined in the build plan. Tasks are tracked via todo items.
2. **Implementation**: Code is written following the coding standards in [CONTRIBUTING.md](../CONTRIBUTING.md).
3. **Quality Gate**: `./scripts/verify-local.sh` must pass before sprint review.
4. **Sprint Review**: All deliverables are verified against the sprint plan (see below).
5. **Sprint Record**: A record is written to `docs/sprints/sprint-N.md` before sprint closure.
6. **Squash-Merge**: All PRs are squash-merged to keep history clean.

## Sprint Review Checklist

Every sprint must pass this review before the next sprint begins:

### 1. Functional Verification
- [ ] All new features tested against real scenarios
- [ ] Tested against real public APIs (not just mocks)
- [ ] Tested on minimum target hardware (4-year-old laptop, 4GB RAM)

### 2. Quality Gate
- [ ] `./scripts/verify-local.sh` passes
- [ ] No compiler warnings in Rust
- [ ] No console errors in frontend
- [ ] Memory usage within budget (< 100MB idle)

### 3. Documentation Review
- [ ] New Tauri commands documented in `docs/API.md`
- [ ] New ADRs written for architectural decisions
- [ ] User-facing features documented
- [ ] Sprint record written in `docs/sprints/sprint-N.md`

### 4. Privacy Audit
- [ ] No new network calls except user-initiated API requests
- [ ] No telemetry, analytics, or tracking added
- [ ] No data written outside app data directory
- [ ] `./scripts/verify-public-release.sh` passes

### 5. Code Review
- [ ] Code follows ecosystem conventions
- [ ] No `unwrap()` or `expect()` in production Rust code
- [ ] All public functions documented with rustdoc
- [ ] Svelte components follow Runes patterns

## Sprint Record Template

```markdown
# Sprint N: [Title]

## Scope
- [What was delivered]

## Validation
- [Test scenarios run and results]

## Decisions
- [ADRs referenced or created]

## Known Issues
- [Carried forward items]

## Next Sprint
- [What's queued]
```
