# Contributing to 900API

We welcome contributions from developers worldwide — especially those in the regions 900API serves. Every line of code from a developer in Lagos, Nairobi, Accra, or Mumbai makes this tool better for the people it's built for.

## Setup

```bash
git clone https://github.com/900Labs/900API.git
cd 900API
npm install
cargo tauri dev
```

Prerequisites:
- Rust 1.88+ — [rustup.rs](https://rustup.rs)
- Node.js 20.19+ — [nodejs.org](https://nodejs.org)
- Tauri v2 system dependencies — [v2.tauri.app/start/prerequisites](https://v2.tauri.app/start/prerequisites/)

## Coding Standards

### Rust
- No `unwrap()` or `expect()` in production code — use proper error handling with `thiserror` and `Result`
- All public functions must have rustdoc comments
- Follow `cargo fmt` formatting (enforced in CI)
- Follow `cargo clippy` lints (enforced in CI)

### Svelte / TypeScript
- Use Svelte 5 Runes syntax (`$state`, `$derived`, `$props`, `$effect`)
- Use TypeScript for all new files
- Follow existing naming conventions
- No `any` types without justification

### CSS
- Use TailwindCSS utility classes
- Use the theme variables defined in `app.css` (`--color-bg`, `--color-surface`, etc.)
- Dark theme is the default

## Sprint Rules

900API is developed in sprints. Each sprint has a defined scope and must pass review before the next sprint begins.

1. **Sprint scope** is defined in the plan and tracked in `docs/sprints/`
2. **Sprint review** must pass all quality gate checks (see [QUALITY_GATE.md](docs/QUALITY_GATE.md))
3. **Sprint record** is written to `docs/sprints/sprint-N.md` before sprint closure
4. **Squash-merge** policy: all PRs are squash-merged to keep history clean

## PR Process

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make your changes following the coding standards above
3. Run the quality gate: `./scripts/verify-local.sh`
4. If adding user-facing behavior, update matching documentation
5. Open a PR with a clear description of what and why
6. Ensure CI passes
7. Address review feedback
8. Squash-merge on approval

## Documentation Updates

Contributions must include matching documentation updates when:
- Behavior or workflows change
- Public APIs (Tauri commands) are added or modified
- Contributor expectations change
- Architecture decisions are made (write an ADR in `docs/adr/`)

## Privacy and Security

- **No telemetry, analytics, or tracking** — ever
- **No network calls** except user-initiated API requests
- **No data collection** — all data stays local
- If you find a vulnerability, email security@900labs.com (see [SECURITY.md](SECURITY.md))

## Quick Contribution Ideas

- Add a translation for your language to `src/i18n/`
- Report bugs in your operating environment
- Improve documentation clarity
- Add import/export format support
- Improve test assertion library
- Add keyboard shortcuts
