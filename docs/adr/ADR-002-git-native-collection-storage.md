# ADR-002: Git-Native Collection Storage

## Date
2026-06-29

## Status
Accepted

## Context

API collections need to be shareable and version-controllable. Options:

1. **Cloud sync** (Postman model) — Requires accounts, servers, and internet. Violates offline-first principle.
2. **SQLite only** — Fast, simple, but not version-controllable. Can't be diffed in Git.
3. **Git-native files** (Bruno model) — Collections stored as plain-text JSON files. Version-controllable. No cloud needed.
4. **Hybrid** — SQLite for app state, exportable to JSON files for Git.

## Decision

Use **hybrid storage**: SQLite for the running app (fast queries, history, settings) with export/import to plain-text JSON files for Git version control.

Collections are stored in SQLite for performance but can be exported as JSON files that live alongside code in a Git repository. The CLI runner reads these JSON files directly.

## Consequences

- Users must explicitly export collections to get Git-friendly files
- The JSON export format must be stable and well-documented
- The CLI runner depends on the JSON format, not SQLite
- Future work: file-watching to auto-sync JSON files back to SQLite
