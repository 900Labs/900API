# Sprint 18: Team Workflows

## Scope
- Created team module (`src-tauri/src/team/mod.rs`) with:
  - `TeamMember` — id, name, email, role, avatar_color, last_active
  - `TeamRole` — Owner, Admin, Editor, Viewer with `can_edit()` and `can_manage()` permission checks
  - `Workspace` — id, name, description, owner_id, members, collection_ids, environment_ids, created_at
  - `ActivityEvent` — id, workspace_id, user info, action, resource_type, resource_name, timestamp
  - `TeamManager` — thread-safe manager with Mutex for workspaces and activities
  - `create_workspace` — creates workspace with owner as first member
  - `delete_workspace` — removes by id
  - `add_member` — adds member, prevents duplicate emails
  - `remove_member` — removes member, prevents removing owner
  - `update_member_role` — changes role, prevents changing owner's role
  - `log_activity` — appends event, caps at 200 entries
  - `get_activity` — returns recent events for a workspace
  - `share_collection` / `unshare_collection` — manage shared collection IDs
  - 10 unit tests covering create, delete, add/remove member, duplicate prevention, owner protection, role update, share/unshare, activity log, role permissions
- Added 11 Tauri IPC commands: `team_list_workspaces`, `team_get_workspace`, `team_create_workspace`, `team_delete_workspace`, `team_add_member`, `team_remove_member`, `team_update_member_role`, `team_get_activity`, `team_share_collection`, `team_unshare_collection`
- Added `TeamManager` to `AppState`
- Created `TeamWorkflows.svelte` UI component with:
  - Workspace list sidebar with member count
  - Create workspace form (name + description)
  - Workspace detail view with members, shared collections, activity feed
  - Member cards with avatar initials, role icons, role selector dropdown
  - Invite member form (name, email, role)
  - Remove member and change role controls (owner protected)
  - Shared collections list with unshare capability
  - Activity feed showing recent events with user avatars
  - Empty state with icon and instructions
  - Error/success message banners
- Added Team nav item to Sidebar (Users icon)
- Updated App.svelte with Team view routing

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 80 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- In-memory storage (workspaces not persisted to database)
- Owner role is immutable (cannot be removed or demoted)
- Activity log capped at 200 entries to prevent unbounded growth
- Role permissions: Owner/Admin can manage, Owner/Admin/Editor can edit, Viewer is read-only
- Avatar colors generated randomly for new members
- Shared collections tracked by ID only (actual collection data in DB)

## Known Issues
- Workspaces not persisted to database (in-memory only)
- No real-time collaboration (no WebSocket sync between clients)
- No authentication/identity system (mock owner "You")
- No collection sharing UI from Collections view
- No environment sharing
- No role-based access enforcement on API operations
- No notifications for team activity
- No audit trail beyond activity log

## Roadmap Complete
All 18 sprints completed. The 900API client now supports:
- REST, GraphQL, WebSocket, SSE, gRPC protocols
- Advanced authentication (OAuth2, OAuth1, AWS SigV4, Hawk)
- Collections & environments with SQLite persistence
- Test scripts with Boa JS engine
- Import/export (Postman, OpenAPI, JSON)
- Mock server with axum
- Git-native sync
- Advanced test runner with assertions
- API documentation generation (Markdown, HTML)
- HTTP connection pooling with shared client
- Internationalization (6 locales)
- Plugin system with manifest-based architecture
- Team workflows with workspaces, members, roles, and activity tracking
