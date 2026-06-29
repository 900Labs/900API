# Sprint 17: Plugin System

## Scope
- Created plugins module (`src-tauri/src/plugins/mod.rs`) with:
  - `PluginManifest` — id, name, version, description, author, homepage, permissions, hooks
  - `PluginPermission` — Network, FileSystem, Environment, Clipboard, Notifications
  - `PluginHook` — BeforeRequest, AfterRequest, BeforeTest, AfterTest, OnCollectionLoad, OnEnvironmentChange
  - `Plugin` — manifest + enabled flag + installed_at + config map
  - `PluginManager` — thread-safe manager with Mutex<Vec<Plugin>>
  - `install_plugin` — validates manifest, prevents duplicates
  - `uninstall_plugin` — removes by id
  - `enable_plugin` / `disable_plugin` — toggle enabled state
  - `update_plugin_config` — update key-value config
  - `get_enabled_plugins_for_hook` — filter enabled plugins by hook type
  - 8 unit tests covering install, duplicate, uninstall, enable/disable, hooks, config, invalid manifest, not found
- Added 7 Tauri IPC commands: `plugin_list`, `plugin_get`, `plugin_install`, `plugin_uninstall`, `plugin_enable`, `plugin_disable`, `plugin_update_config`
- Added `PluginManager` to `AppState`
- Created `PluginManager.svelte` UI component with:
  - Plugin list with name, version, description, author, enabled/disabled status
  - Hook badges showing registered hooks
  - Permission badges with warning styling
  - Enable/Disable toggle button
  - Remove button with error styling
  - Install from JSON manifest textarea with sample manifest placeholder
  - Empty state with icon and instructions
  - Error/success message banners
- Added Plugins nav item to Sidebar (Puzzle icon)
- Updated App.svelte with Plugins view routing

## Validation
- `cargo check` passes with zero warnings ✅
- `npm run check` (svelte-check) passes with zero errors and zero warnings ✅
- `cargo test` — 70 tests, all passing ✅
- `./scripts/verify-local.sh` — all quality gate checks pass ✅

## Decisions
- Plugins defined by manifest JSON (no dynamic code loading for security)
- Plugin hooks are declarative (manifest specifies which hooks, execution TBD)
- Permissions are declarative (manifest specifies, not enforced yet)
- Plugin config stored as HashMap<String, String> for flexibility
- In-memory storage (plugins not persisted across restarts yet)
- `PluginHook` derives `PartialEq` for hook matching

## Known Issues
- Plugins not persisted to database
- Hook execution not implemented (plugins registered but not called)
- Permission enforcement not implemented
- No plugin marketplace or discovery
- No plugin code execution (manifest-only, no JS/WASM runtime for plugins)
- No plugin settings UI (config update via API only)

## Next Sprint
- Sprint 18: Team Workflows
