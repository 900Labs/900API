use crate::db::Database;
use crate::export;
use crate::http;
use crate::import;
use crate::models::{
    AuthConfig, Collection, Environment, EnvironmentVariable, GraphQLSchema, HistoryEntry,
    KeyValue, RequestConfig, ResponseData, ResponseExample, SavedRequest,
};
use crate::scripting::ScriptOutput;
use crate::AppState;

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn send_request(
    state: tauri::State<'_, AppState>,
    config: RequestConfig,
    environment_variables: Option<Vec<EnvironmentVariable>>,
) -> Result<ResponseData, String> {
    let request_snapshot = serde_json::to_string(&config).unwrap_or_else(|_| "{}".to_string());

    // Resolve environment variables if provided
    let resolved_config = if let Some(vars) = &environment_variables {
        let url = http::variables::resolve_variables(&config.url, vars);
        let headers = http::variables::resolve_key_values(&config.headers, vars);
        let params = http::variables::resolve_key_values(&config.params, vars);
        let body = http::variables::resolve_variables(&config.body, vars);
        let auth = http::variables::resolve_auth_config(&config.auth, vars);
        RequestConfig {
            url,
            headers,
            params,
            body,
            auth,
            ..config
        }
    } else {
        config
    };

    let method_str = resolved_config.method.to_string();
    let url_str = resolved_config.url.clone();

    let response = http::send_request(&resolved_config)
        .await
        .map_err(|e| e.to_string())?;

    // Log to history
    if let Some(ref db) = *state.db.lock().unwrap_or_else(|e| e.into_inner()) {
        let _ = db.add_history_with_snapshot(
            &method_str,
            &url_str,
            response.status,
            response.time_ms,
            response.size_bytes,
            &request_snapshot,
        );
    }

    Ok(response)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn send_graphql(
    state: tauri::State<'_, AppState>,
    url: String,
    query: String,
    variables: String,
    operation_name: Option<String>,
    headers: Vec<KeyValue>,
    auth: AuthConfig,
    environment_variables: Option<Vec<EnvironmentVariable>>,
) -> Result<ResponseData, String> {
    let env_vars = environment_variables.unwrap_or_default();

    let response = http::send_graphql(
        &url,
        &query,
        &variables,
        operation_name.as_deref(),
        &headers,
        &auth,
        &env_vars,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Log to history
    if let Some(ref db) = *state.db.lock().unwrap_or_else(|e| e.into_inner()) {
        let _ = db.add_history(
            "POST",
            &url,
            response.status,
            response.time_ms,
            response.size_bytes,
        );
    }

    Ok(response)
}

#[tauri::command]
pub async fn introspect_graphql_schema(
    url: String,
    headers: Vec<KeyValue>,
    auth: AuthConfig,
    environment_variables: Option<Vec<EnvironmentVariable>>,
) -> Result<GraphQLSchema, String> {
    let env_vars = environment_variables.unwrap_or_default();
    http::introspect_graphql_schema(&url, &headers, &auth, &env_vars)
        .await
        .map_err(|e| e.to_string())
}

fn with_db<F, T>(state: &tauri::State<'_, AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&Database) -> Result<T, crate::db::DbError>,
{
    let guard = state.db.lock().unwrap_or_else(|e| e.into_inner());
    match *guard {
        Some(ref db) => f(db).map_err(|e| e.to_string()),
        None => Err("Database not initialized".to_string()),
    }
}

fn portable_collection(
    db: &Database,
    collection_id: &str,
) -> Result<api900_core::format::CollectionFile, crate::db::DbError> {
    let collection = db.get_collection(collection_id)?;
    let requests = db.list_requests(collection_id)?;
    let mut response_examples = std::collections::HashMap::new();
    for request in &requests {
        response_examples.insert(request.id.clone(), db.list_response_examples(&request.id)?);
    }
    export::collection_to_file(&collection, &requests, &response_examples)
        .map_err(|error| crate::db::DbError::NotFound(error.to_string()))
}

#[tauri::command]
pub fn list_collections(state: tauri::State<'_, AppState>) -> Result<Vec<Collection>, String> {
    with_db(&state, |db| db.list_collections())
}

#[tauri::command]
pub fn create_collection(
    state: tauri::State<'_, AppState>,
    name: String,
    description: Option<String>,
    parent_id: Option<String>,
) -> Result<Collection, String> {
    with_db(&state, |db| {
        db.create_collection_with_parent(&name, description.as_deref(), parent_id.as_deref())
    })
}

#[tauri::command]
pub fn update_collection(
    state: tauri::State<'_, AppState>,
    id: String,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    with_db(&state, |db| {
        db.update_collection(&id, &name, description.as_deref())
    })
}

#[tauri::command]
pub fn move_collection(
    state: tauri::State<'_, AppState>,
    id: String,
    parent_id: Option<String>,
) -> Result<(), String> {
    with_db(&state, |db| db.move_collection(&id, parent_id.as_deref()))
}

#[tauri::command]
pub fn delete_collection(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    with_db(&state, |db| db.delete_collection(&id))
}

#[tauri::command]
pub fn list_environments(state: tauri::State<'_, AppState>) -> Result<Vec<Environment>, String> {
    with_db(&state, |db| db.list_environments())
}

#[tauri::command]
pub fn create_environment(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<Environment, String> {
    with_db(&state, |db| db.create_environment(&name))
}

#[tauri::command]
pub fn delete_environment(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    with_db(&state, |db| db.delete_environment(&id))
}

#[tauri::command]
pub fn list_history(
    state: tauri::State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<HistoryEntry>, String> {
    with_db(&state, |db| db.list_history(limit.unwrap_or(100)))
}

#[tauri::command]
pub fn clear_history(state: tauri::State<'_, AppState>) -> Result<(), String> {
    with_db(&state, |db| db.clear_history())
}

#[tauri::command]
pub fn list_requests(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<Vec<SavedRequest>, String> {
    with_db(&state, |db| db.list_requests(&collection_id))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_request(
    state: tauri::State<'_, AppState>,
    collection_id: String,
    name: String,
    method: String,
    url: String,
    headers: String,
    params: String,
    body_type: String,
    body: String,
    auth_type: String,
    auth_config: String,
    pre_request_script: Option<String>,
    test_script: Option<String>,
    settings: Option<String>,
) -> Result<SavedRequest, String> {
    with_db(&state, |db| {
        db.create_request_with_settings(
            &collection_id,
            &name,
            &method,
            &url,
            &headers,
            &params,
            &body_type,
            &body,
            &auth_type,
            &auth_config,
            pre_request_script.as_deref().unwrap_or(""),
            test_script.as_deref().unwrap_or(""),
            settings.as_deref().unwrap_or("{}"),
        )
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_request(
    state: tauri::State<'_, AppState>,
    id: String,
    name: String,
    method: String,
    url: String,
    headers: String,
    params: String,
    body_type: String,
    body: String,
    auth_type: String,
    auth_config: String,
    pre_request_script: Option<String>,
    test_script: Option<String>,
    settings: Option<String>,
) -> Result<(), String> {
    with_db(&state, |db| {
        db.update_request_with_settings(
            &id,
            &name,
            &method,
            &url,
            &headers,
            &params,
            &body_type,
            &body,
            &auth_type,
            &auth_config,
            pre_request_script.as_deref().unwrap_or(""),
            test_script.as_deref().unwrap_or(""),
            settings.as_deref().unwrap_or("{}"),
        )
    })
}

#[tauri::command]
pub fn delete_request(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    with_db(&state, |db| db.delete_request(&id))
}

#[tauri::command]
pub fn list_response_examples(
    state: tauri::State<'_, AppState>,
    request_id: String,
) -> Result<Vec<ResponseExample>, String> {
    with_db(&state, |db| db.list_response_examples(&request_id))
}

#[tauri::command]
pub fn create_response_example(
    state: tauri::State<'_, AppState>,
    request_id: String,
    name: String,
    response: ResponseData,
) -> Result<ResponseExample, String> {
    let headers = serde_json::to_string(&response.headers).map_err(|e| e.to_string())?;
    with_db(&state, |db| {
        db.create_response_example(
            &request_id,
            &name,
            response.status,
            &response.status_text,
            &headers,
            &response.body,
            response.time_ms,
            response.size_bytes,
        )
    })
}

#[tauri::command]
pub fn delete_response_example(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    with_db(&state, |db| db.delete_response_example(&id))
}

#[tauri::command]
pub fn move_request(
    state: tauri::State<'_, AppState>,
    id: String,
    collection_id: String,
) -> Result<(), String> {
    with_db(&state, |db| db.move_request(&id, &collection_id))
}

#[tauri::command]
pub fn update_environment(
    state: tauri::State<'_, AppState>,
    id: String,
    variables: String,
) -> Result<(), String> {
    with_db(&state, |db| db.update_environment(&id, &variables))
}

#[tauri::command]
pub async fn export_collection(
    state: tauri::State<'_, AppState>,
    collection_id: String,
    path: String,
) -> Result<(), String> {
    let target = validate_user_file_path(&path)?;
    let json = with_db(&state, |db| {
        let portable = portable_collection(db, &collection_id)?;
        api900_core::format::to_pretty_json(&portable)
            .map_err(|error| crate::db::DbError::Serialization(error.to_string()))
    })?;
    tokio::task::spawn_blocking(move || {
        std::fs::write(&target, json).map_err(|error| crate::db::DbError::Io(error.to_string()))
    })
    .await
    .map_err(|error| format!("Export task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_openapi(
    state: tauri::State<'_, AppState>,
    collection_id: String,
    path: String,
) -> Result<(), String> {
    let target = validate_user_file_path(&path)?;
    let (collection, requests) = with_db(&state, |db| {
        let collections = db.list_collections()?;
        let collection = collections
            .into_iter()
            .find(|c| c.id == collection_id)
            .ok_or_else(|| crate::db::DbError::NotFound(format!("Collection {}", collection_id)))?;
        let requests = db.list_requests(&collection.id)?;
        Ok((collection, requests))
    })?;
    tokio::task::spawn_blocking(move || {
        export::export_openapi_collection(&collection, &requests, &target)
            .map_err(|error| crate::db::DbError::Io(error.to_string()))
    })
    .await
    .map_err(|error| format!("Export task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn import_collection_file(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<Collection, String> {
    let source = validate_user_file_path(&path)?;
    let imported = tokio::task::spawn_blocking(move || {
        export::import_collection(&source).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Import task failed: {error}"))??;
    with_db(&state, |db| db.import_collection_file(&imported))
}

#[tauri::command]
pub fn run_test_script(
    script: String,
    response_body: String,
    response_status: u16,
    response_headers: String,
) -> Result<ScriptOutput, String> {
    crate::scripting::run_test_script(&script, &response_body, response_status, &response_headers)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ws_connect(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
    url: String,
) -> Result<(), String> {
    crate::websocket::connect_websocket(app, state.ws_manager.clone(), id, url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ws_send(
    state: tauri::State<'_, AppState>,
    id: String,
    message: String,
) -> Result<(), String> {
    crate::websocket::send_websocket_message(&state.ws_manager, &id, message)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ws_disconnect(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    crate::websocket::disconnect_websocket(&state.ws_manager, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ws_get_state(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<crate::websocket::WsConnectionState, String> {
    crate::websocket::get_websocket_state(&state.ws_manager, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sse_connect(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
    url: String,
    headers: Vec<KeyValue>,
) -> Result<(), String> {
    crate::sse::connect_sse(app, state.sse_manager.clone(), id, url, headers)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sse_disconnect(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    crate::sse::disconnect_sse(&state.sse_manager, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sse_get_state(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<crate::sse::SseConnectionState, String> {
    crate::sse::get_sse_state(&state.sse_manager, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_grpc(
    address: String,
    service_method: String,
    body_hex: String,
    headers: Vec<KeyValue>,
    use_tls: bool,
) -> Result<crate::grpc::GrpcResponse, String> {
    crate::grpc::send_grpc_unary(&address, &service_method, &body_hex, &headers, use_tls)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mock_start(
    state: tauri::State<'_, AppState>,
    config: crate::mock::MockServerConfig,
) -> Result<(), String> {
    crate::mock::start_mock_server(state.mock_manager.clone(), config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mock_stop(state: tauri::State<'_, AppState>, port: u16) -> Result<(), String> {
    crate::mock::stop_mock_server(&state.mock_manager, port).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mock_get_state(
    state: tauri::State<'_, AppState>,
    port: u16,
) -> Result<crate::mock::MockServerState, String> {
    crate::mock::get_mock_server_state(&state.mock_manager, port).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mock_list_servers(state: tauri::State<'_, AppState>) -> Vec<u16> {
    crate::mock::list_mock_servers(&state.mock_manager)
}

#[tauri::command]
pub fn sync_set_config(
    state: tauri::State<'_, AppState>,
    config: crate::sync::SyncConfig,
) -> Result<(), String> {
    state
        .sync_manager
        .set_config(config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_get_config(state: tauri::State<'_, AppState>) -> Option<crate::sync::SyncConfig> {
    state.sync_manager.get_config()
}

#[tauri::command]
pub fn sync_export_collection(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<String, String> {
    let collection = with_db(&state, |db| portable_collection(db, &collection_id))?;
    state
        .sync_manager
        .export_collection(&collection)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_import_collection(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<Collection, String> {
    let imported = state
        .sync_manager
        .import_collection(&name)
        .map_err(|e| e.to_string())?;
    with_db(&state, |db| db.import_collection_file(&imported))
}

#[tauri::command]
pub fn sync_list_collections(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .sync_manager
        .list_collections()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_git_init(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let manager = state.sync_manager.clone();
    tokio::task::spawn_blocking(move || manager.git_init())
        .await
        .map_err(|error| format!("Git task failed: {error}"))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_git_status(
    state: tauri::State<'_, AppState>,
) -> Result<crate::sync::GitStatus, String> {
    let manager = state.sync_manager.clone();
    tokio::task::spawn_blocking(move || manager.git_status())
        .await
        .map_err(|error| format!("Git task failed: {error}"))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_git_commit(
    state: tauri::State<'_, AppState>,
    message: String,
) -> Result<(), String> {
    let manager = state.sync_manager.clone();
    tokio::task::spawn_blocking(move || manager.git_commit(&message))
        .await
        .map_err(|error| format!("Git task failed: {error}"))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_git_pull(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let manager = state.sync_manager.clone();
    tokio::task::spawn_blocking(move || manager.git_pull())
        .await
        .map_err(|error| format!("Git task failed: {error}"))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_git_push(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let manager = state.sync_manager.clone();
    tokio::task::spawn_blocking(move || manager.git_push())
        .await
        .map_err(|error| format!("Git task failed: {error}"))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_test_suite(
    suite: crate::test_runner::TestSuite,
    environment_variables: Option<Vec<crate::models::EnvironmentVariable>>,
) -> Result<crate::test_runner::TestSuiteResult, String> {
    let env_vars = environment_variables.unwrap_or_default();
    Ok(crate::test_runner::run_test_suite(&suite, &env_vars).await)
}

#[tauri::command]
pub async fn run_test_suites(
    suites: Vec<crate::test_runner::TestSuite>,
    environment_variables: Option<Vec<crate::models::EnvironmentVariable>>,
) -> Result<crate::test_runner::TestRunResult, String> {
    let env_vars = environment_variables.unwrap_or_default();
    Ok(crate::test_runner::run_test_suites(suites, &env_vars).await)
}

#[tauri::command]
pub fn generate_collection_docs(
    state: tauri::State<'_, AppState>,
    collection_id: String,
) -> Result<crate::docs::ApiDoc, String> {
    let db_lock = state.db.lock().unwrap_or_else(|e| e.into_inner());
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    crate::docs::generate_collection_docs(db, &collection_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn generate_all_docs(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::docs::ApiDoc>, String> {
    let db_lock = state.db.lock().unwrap_or_else(|e| e.into_inner());
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    crate::docs::generate_all_docs(db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn docs_to_markdown(doc: crate::docs::ApiDoc) -> String {
    crate::docs::docs_to_markdown(&doc)
}

#[tauri::command]
pub fn docs_to_html(doc: crate::docs::ApiDoc) -> String {
    crate::docs::docs_to_html(&doc)
}

#[tauri::command]
pub fn validate_user_file_path(path: &str) -> Result<std::path::PathBuf, String> {
    let target = std::path::Path::new(path);

    if !target.is_absolute() {
        return Err("Path must be absolute".to_string());
    }

    if target
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err("Path traversal is not allowed".to_string());
    }

    let home = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .canonicalize()
        .map_err(|e| format!("Could not resolve home directory: {}", e))?;

    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let parent = parent.canonicalize().map_err(|e| {
        format!(
            "Parent directory does not exist or is not accessible: {}",
            e
        )
    })?;

    if !parent.starts_with(&home) {
        return Err(format!("Path '{}' is outside the allowed directory", path));
    }

    let file_name = target
        .file_name()
        .ok_or_else(|| "Path must include a file name".to_string())?;
    let safe_target = parent.join(file_name);

    if let Ok(metadata) = std::fs::symlink_metadata(&safe_target) {
        if metadata.file_type().is_symlink() {
            return Err("Refusing to access a symbolic link path".to_string());
        }
    }

    Ok(safe_target)
}

#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
    let target = validate_user_file_path(&path)?;
    tokio::task::spawn_blocking(move || std::fs::write(&target, content))
        .await
        .map_err(|error| format!("Write task failed: {error}"))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn plugin_list(state: tauri::State<'_, AppState>) -> Vec<crate::plugins::Plugin> {
    state.plugin_manager.list_plugins()
}

#[tauri::command]
pub fn plugin_get(state: tauri::State<'_, AppState>, id: String) -> Option<crate::plugins::Plugin> {
    state.plugin_manager.get_plugin(&id)
}

#[tauri::command]
pub fn plugin_install(
    state: tauri::State<'_, AppState>,
    manifest: crate::plugins::PluginManifest,
) -> Result<crate::plugins::Plugin, String> {
    state
        .plugin_manager
        .install_plugin(manifest)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn plugin_uninstall(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .plugin_manager
        .uninstall_plugin(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn plugin_enable(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .plugin_manager
        .enable_plugin(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn plugin_disable(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .plugin_manager
        .disable_plugin(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn plugin_update_config(
    state: tauri::State<'_, AppState>,
    id: String,
    config: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    state
        .plugin_manager
        .update_plugin_config(&id, config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn team_list_workspaces(state: tauri::State<'_, AppState>) -> Vec<crate::team::Workspace> {
    state.team_manager.list_workspaces()
}

#[tauri::command]
pub fn team_get_workspace(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Option<crate::team::Workspace> {
    state.team_manager.get_workspace(&id)
}

#[tauri::command]
pub fn team_create_workspace(
    state: tauri::State<'_, AppState>,
    name: String,
    description: Option<String>,
    owner: crate::team::TeamMember,
) -> Result<crate::team::Workspace, String> {
    state
        .team_manager
        .create_workspace(&name, description.as_deref(), owner)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn team_delete_workspace(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .team_manager
        .delete_workspace(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn team_add_member(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    member: crate::team::TeamMember,
) -> Result<(), String> {
    state
        .team_manager
        .add_member(&workspace_id, member)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn team_remove_member(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    member_id: String,
) -> Result<(), String> {
    state
        .team_manager
        .remove_member(&workspace_id, &member_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn team_update_member_role(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    member_id: String,
    role: crate::team::TeamRole,
) -> Result<(), String> {
    state
        .team_manager
        .update_member_role(&workspace_id, &member_id, role)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn team_get_activity(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    limit: Option<usize>,
) -> Vec<crate::team::ActivityEvent> {
    state
        .team_manager
        .get_activity(&workspace_id, limit.unwrap_or(50))
}

#[tauri::command]
pub fn team_share_collection(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    collection_id: String,
) -> Result<(), String> {
    state
        .team_manager
        .share_collection(&workspace_id, &collection_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn team_unshare_collection(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    collection_id: String,
) -> Result<(), String> {
    state
        .team_manager
        .unshare_collection(&workspace_id, &collection_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_postman(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<Collection, String> {
    let source = validate_user_file_path(&path)?;
    let (name, description, requests) = tokio::task::spawn_blocking(move || {
        import::import_postman_collection(&source).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Import task failed: {error}"))??;

    with_db(&state, |db| {
        db.import_requests_collection(&name, description.as_deref(), &requests)
    })
}

#[tauri::command]
pub async fn import_openapi(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<Collection, String> {
    let source = validate_user_file_path(&path)?;
    let (name, description, requests) = tokio::task::spawn_blocking(move || {
        import::import_openapi_collection(&source).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Import task failed: {error}"))??;

    with_db(&state, |db| {
        db.import_requests_collection(&name, description.as_deref(), &requests)
    })
}
