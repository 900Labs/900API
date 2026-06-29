# Sprint 12: Git-Native Sync

## Scope
- Created sync module (`src-tauri/src/sync/mod.rs`) with:
  - `SyncManager` — manages sync config and git operations via `std::process::Command`
  - `SyncConfig` — directory, auto_sync flag, author name/email
  - `ExportCollection` / `ExportRequest` — JSON-serializable collection format
  - `GitStatus` — is_repo flag + changed files list
  - `export_collection` — writes collection as pretty JSON to sync directory
  - `import_collection` — reads collection JSON from file path
  - `list_collections` — lists all `.json` files in sync directory
  - `git_init` — runs `git init` in sync directory
  - `git_status` — runs `git status --porcelain` and parses output
  - `git_commit` — stages all (`git add -A`) and commits with message
  - `git_pull` — runs `git pull --rebase`
  - `git_push` — runs `git push`
  - 4 unit tests covering config management, serialization, and error cases
- Added 10 Tauri IPC commands: `sync_set_config`, `sync_get_config`, `sync_export_collection`, `sync_import_collection`, `sync_list_collections`, `sync_git_init`, `sync_git_status`, `sync_git_commit`, `sync_git_pull`, `sync_git_push`
- Added `SyncManager` to `AppState`
- Created `GitSync.svelte` UI component with:
  - Sync directory configuration with folder picker (tauri dialog plugin)
  - Author name/email configuration
  - Git status display (repo indicator, changed files list)
  - Git init button (shown when not a repo)
  - Commit message input + commit button
  - Pull and Push buttons
  - Synced collections list with import button
  - Git command output display
  - Loading states and error/success messages
- Added Git Sync nav item to Sidebar (GitBranch icon)
- Updated App.svelte with Git Sync view routing
- Updated API docs with sync commands and types

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 50 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Git operations via `std::process::Command` (shells out to system `git`)
- Collections exported as pretty JSON files (one per collection, named `<collection_name>.json`)
- File names sanitized (replacing `/`, `\`, `:`, etc. with `_`)
- Git author configured per-repo via `git config user.name/email`
- `git pull --rebase` used to avoid merge commits
- No git library dependency (libgit2) — uses system git binary
- Sync config stored in memory (not persisted to DB yet)

## Known Issues
- Sync config not persisted across app restarts
- No auto-sync on collection changes (manual export required)
- No diff viewer for changed files
- No branch management (create, switch, merge)
- No conflict resolution UI
- No .gitignore management
- Requires system `git` binary to be installed

## Next Sprint
- Sprint 13: Advanced Test Runner
