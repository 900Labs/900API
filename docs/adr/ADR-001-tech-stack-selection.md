# ADR 001: Technology Stack

Date: 2026-06-29

Status: Accepted

## Context

900API needs a cross-platform desktop interface, a reliable local request engine, low runtime overhead, and a contributor workflow that can produce Windows, Linux, and macOS builds from one codebase. It should remain useful on older hardware and after the initial download when internet access is intermittent.

The main options were Electron with a web framework, Tauri with a web framework, and separate native interfaces for each platform.

## Decision

Use Tauri 2, Rust, Svelte 5, and Tailwind CSS.

- Tauri uses the operating-system WebView instead of packaging a separate browser engine.
- Rust provides the HTTP, persistence, scripting, and local server boundaries.
- Svelte provides a compact reactive interface without requiring separate platform implementations.
- The same repository can build desktop packages and a standalone Rust CLI.

No fixed binary size, memory, startup, or latency number is promised by this decision. Those values depend on platform, build target, WebView, and workload and must be measured when they are used in release claims.

## Consequences

- Contributors need Rust and Svelte knowledge.
- WebView behavior needs platform testing.
- Native packaging and signing differ by operating system.
- Shared request and collection behavior must live below the interface layer so the CLI and desktop app do not diverge.
