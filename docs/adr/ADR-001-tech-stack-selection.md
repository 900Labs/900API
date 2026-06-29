# ADR-001: Tech Stack Selection

## Date
2026-06-29

## Status
Accepted

## Context

900API needs a desktop application stack that:
- Runs on older hardware (4-year-old laptops, 4GB RAM)
- Produces small binaries for download in low-bandwidth regions
- Works completely offline
- Matches the 900 Labs ecosystem conventions (900Invoice, 900Word)

Options considered:
1. **Electron + React** — Industry standard for API clients (Postman, Insomnia, Bruno). But ~150MB binaries, ~500MB RAM. Too heavy for target hardware.
2. **Tauri v2 + Svelte 5** — Native WebView, ~15MB binaries, ~100MB RAM. Matches ecosystem. Excellent for older hardware.
3. **Pure native (Qt/GTK)** — Lightest option but requires platform-specific UI code and doesn't match ecosystem conventions.

## Decision

Use **Tauri v2 + Rust + Svelte 5 + TailwindCSS v4**.

This matches the 900 Labs ecosystem (900Invoice, 900Word) and provides:
- ~15MB binary vs ~150MB (Electron)
- ~100MB RAM vs ~500MB (Electron)
- Shared conventions, libraries, and contributor base
- Rust backend for HTTP engine performance and safety
- Svelte 5 Runes for minimal frontend runtime

## Consequences

- Contributors must know Rust and Svelte (smaller pool than React)
- Platform-specific WebView differences require testing on all platforms
- Tauri v2 is newer than Electron — fewer community plugins
- All ecosystem benefits (shared patterns, shared docs, shared contributors) outweigh the costs
