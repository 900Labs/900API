# ADR 002: Git-Native Collection Storage

Date: 2026-06-29

Status: Accepted, updated for 0.2.0

## Context

The desktop app needs fast local queries and history, while teams also need collection files that can be reviewed and versioned without a hosted account.

SQLite alone is efficient but is not suitable for source review. Cloud sync would require accounts, servers, and dependable connectivity. Plain files are reviewable but are less convenient for application state and related records.

## Decision

Use hybrid storage:

- SQLite is the working store for collections, requests, examples, environments, and history.
- Git Sync exports complete collections as readable JSON files into a directory selected by the user.
- Git Sync imports selected files back into SQLite in one transaction.
- Desktop export, Git Sync, and CLI use `900api.collection/v1` from `crates/900api-core`.
- Legacy 0.1 files with JSON-encoded string fields remain importable.

Portable files include stable collection and request IDs, request data, scripts, settings, and response examples. They do not include machine-specific binary or multipart file paths.

## Consequences

- Export is explicit. The app does not watch files or silently overwrite local collections.
- Import replaces an existing collection with the same portable ID, including its requests and response examples.
- Literal credentials saved in a request can appear in exported JSON. Users should use environment variables and review files before committing.
- Future schema changes require compatibility tests in the shared core crate.
