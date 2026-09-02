mod auth;
mod commands;
mod db;
mod docs;
mod export;
mod grpc;
mod http;
mod import;
mod mock;
mod models;
mod persistence;
mod plugins;
mod scripting;
mod sse;
mod sync;
mod team;
mod test_runner;
mod websocket;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Option<db::Database>>,
    pub ws_manager: websocket::WsManager,
    pub sse_manager: sse::SseManager,
    pub mock_manager: mock::MockManager,
    pub sync_manager: std::sync::Arc<sync::SyncManager>,
    pub plugin_manager: plugins::PluginManager,
    pub team_manager: team::TeamManager,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(None),
            ws_manager: websocket::create_ws_manager(),
            sse_manager: sse::create_sse_manager(),
            mock_manager: mock::create_mock_manager(),
            sync_manager: std::sync::Arc::new(sync::SyncManager::new()),
            plugin_manager: plugins::PluginManager::new(),
            team_manager: team::TeamManager::new(),
        })
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|error| {
                std::io::Error::other(format!("Could not locate the app data directory: {error}"))
            })?;
            std::fs::create_dir_all(&app_data_dir).map_err(|error| {
                std::io::Error::other(format!("Could not create the app data directory: {error}"))
            })?;

            let db_path = app_data_dir.join("900api.db");
            let database = db::Database::open(&db_path).map_err(|error| {
                std::io::Error::other(format!("Could not open the local database: {error}"))
            })?;

            let state: tauri::State<AppState> = app.state();
            *state.db.lock().unwrap_or_else(|e| e.into_inner()) = Some(database);
            if let Err(error) = state
                .sync_manager
                .set_storage_path(app_data_dir.join("sync-config.json"))
            {
                log::error!("Could not load optional Git Sync settings: {error}");
            }
            if let Err(error) = state
                .plugin_manager
                .set_storage_path(app_data_dir.join("plugins.json"))
            {
                log::error!("Could not load optional plugin registry: {error}");
            }
            if let Err(error) = state
                .team_manager
                .set_storage_path(app_data_dir.join("team-workspaces.json"))
            {
                log::error!("Could not load optional local workspace plans: {error}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_version,
            commands::send_request,
            commands::send_graphql,
            commands::introspect_graphql_schema,
            commands::list_collections,
            commands::create_collection,
            commands::update_collection,
            commands::move_collection,
            commands::delete_collection,
            commands::list_environments,
            commands::create_environment,
            commands::delete_environment,
            commands::update_environment,
            commands::list_history,
            commands::clear_history,
            commands::list_requests,
            commands::create_request,
            commands::update_request,
            commands::delete_request,
            commands::list_response_examples,
            commands::create_response_example,
            commands::delete_response_example,
            commands::move_request,
            commands::export_collection,
            commands::export_openapi,
            commands::import_collection_file,
            commands::run_test_script,
            commands::import_postman,
            commands::import_openapi,
            commands::ws_connect,
            commands::ws_send,
            commands::ws_disconnect,
            commands::ws_get_state,
            commands::sse_connect,
            commands::sse_disconnect,
            commands::sse_get_state,
            commands::send_grpc,
            commands::mock_start,
            commands::mock_stop,
            commands::mock_get_state,
            commands::mock_list_servers,
            commands::sync_set_config,
            commands::sync_get_config,
            commands::sync_export_collection,
            commands::sync_import_collection,
            commands::sync_list_collections,
            commands::sync_git_init,
            commands::sync_git_status,
            commands::sync_git_commit,
            commands::sync_git_pull,
            commands::sync_git_push,
            commands::run_test_suite,
            commands::run_test_suites,
            commands::generate_collection_docs,
            commands::generate_all_docs,
            commands::docs_to_markdown,
            commands::docs_to_html,
            commands::write_text_file,
            commands::plugin_list,
            commands::plugin_get,
            commands::plugin_install,
            commands::plugin_uninstall,
            commands::plugin_enable,
            commands::plugin_disable,
            commands::plugin_update_config,
            commands::team_list_workspaces,
            commands::team_get_workspace,
            commands::team_create_workspace,
            commands::team_delete_workspace,
            commands::team_add_member,
            commands::team_remove_member,
            commands::team_update_member_role,
            commands::team_get_activity,
            commands::team_share_collection,
            commands::team_unshare_collection,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("900API could not start: {error}");
            std::process::exit(1);
        });
}
