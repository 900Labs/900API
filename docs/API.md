# API Documentation — Tauri Command Reference

All commands are invoked from the frontend via `@tauri-apps/api/core` `invoke()`.

## Commands

### `get_app_version`
Returns the application version string.

```typescript
const version = await invoke<string>('get_app_version')
// "0.1.0"
```

### `send_request`
Sends an HTTP request and returns the response.

```typescript
const response = await invoke<ResponseData>('send_request', {
  config: {
    method: 'GET',
    url: 'https://api.example.com/users',
    headers: [{ key: 'Authorization', value: 'Bearer token', enabled: true }],
    params: [{ key: 'page', value: '1', enabled: true }],
    body_type: 'none',
    body: ''
  }
})
```

**Parameters:**
- `config: RequestConfig` — the request configuration

**Returns:** `ResponseData` with `status`, `status_text`, `headers`, `body`, `time_ms`, `size_bytes`

### `send_graphql`
Sends a GraphQL query to a GraphQL endpoint.

```typescript
const response = await invoke<ResponseData>('send_graphql', {
  url: 'https://api.example.com/graphql',
  query: 'query { countries { code name } }',
  variables: '{"limit": 10}',
  operationName: null,
  headers: [{ key: 'Authorization', value: 'Bearer token', enabled: true }],
  auth: { auth_type: 'none', username: '', password: '', token: '', api_key: '', api_key_name: '', api_key_in: 'header' },
  environmentVariables: undefined
})
```

**Parameters:**
- `url: string` — the GraphQL endpoint URL
- `query: string` — the GraphQL query/mutation string
- `variables: string` — JSON string of variables
- `operationName: string | null` — optional operation name
- `headers: KeyValue[]` — custom headers
- `auth: AuthConfig` — auth configuration
- `environmentVariables?: EnvironmentVariable[]` — optional env vars for `{{var}}` resolution

**Returns:** `ResponseData`

### `list_collections`
Returns all collections.

```typescript
const collections = await invoke<Collection[]>('list_collections')
```

**Returns:** `Collection[]` with `id`, `name`, `description`, `created_at`, `updated_at`

### `create_collection`
Creates a new collection.

```typescript
const collection = await invoke<Collection>('create_collection', {
  name: 'My API',
  description: 'Production API tests'
})
```

**Parameters:**
- `name: string` — collection name
- `description?: string` — optional description

### `delete_collection`
Deletes a collection and all its requests (cascade).

```typescript
await invoke('delete_collection', { id: 'uuid-here' })
```

### `list_environments`
Returns all environments.

```typescript
const environments = await invoke<Environment[]>('list_environments')
```

### `create_environment`
Creates a new environment.

```typescript
const env = await invoke<Environment>('create_environment', { name: 'Production' })
```

### `delete_environment`
Deletes an environment.

```typescript
await invoke('delete_environment', { id: 'uuid-here' })
```

### `list_history`
Returns recent request history.

```typescript
const history = await invoke<HistoryEntry[]>('list_history', { limit: 100 })
```

**Parameters:**
- `limit?: number` — max entries (default 100, max 500)

### `clear_history`
Clears all request history.

```typescript
await invoke('clear_history')
```

### `list_requests`
Returns all requests in a collection.

```typescript
const requests = await invoke<SavedRequest[]>('list_requests', { collectionId: 'uuid' })
```

**Parameters:**
- `collectionId: string` — the collection ID

**Returns:** `SavedRequest[]` with `id`, `collection_id`, `name`, `method`, `url`, `headers`, `params`, `body_type`, `body`, `auth_type`, `auth_config`, `sort_order`, `created_at`, `updated_at`

### `create_request`
Creates a new request in a collection.

```typescript
const req = await invoke<SavedRequest>('create_request', {
  collectionId: 'uuid',
  name: 'Get Users',
  method: 'GET',
  url: 'https://api.example.com/users',
  headers: '[]',
  params: '[]',
  bodyType: 'none',
  body: '',
  authType: 'none',
  authConfig: '{}'
})
```

### `update_request`
Updates an existing request.

```typescript
await invoke('update_request', {
  id: 'uuid',
  name: 'Updated Name',
  method: 'POST',
  url: 'https://api.example.com/users',
  headers: '[]',
  params: '[]',
  bodyType: 'json',
  body: '{"key":"value"}',
  authType: 'bearer',
  authConfig: '{"token":"abc"}'
})
```

### `delete_request`
Deletes a request.

```typescript
await invoke('delete_request', { id: 'uuid' })
```

### `update_environment`
Updates an environment's variables.

```typescript
await invoke('update_environment', {
  id: 'uuid',
  variables: '[{"key":"baseUrl","value":"https://api.example.com","enabled":true}]'
})
```

### `export_collection`
Exports a collection and all its requests to a JSON file.

```typescript
await invoke('export_collection', { collectionId: 'uuid', path: '/path/to/export.json' })
```

### `import_collection_file`
Imports a collection from a JSON file, creating a new collection with all requests.

```typescript
const collection = await invoke<Collection>('import_collection_file', { path: '/path/to/import.json' })
```

### `run_test_script`
Runs a JavaScript test script in a sandboxed JS engine (boa) with access to `api900.response`.

```typescript
const output = await invoke<ScriptOutput>('run_test_script', {
  script: 'var s = api900.response.status; if (s !== 200) throw new Error("fail");',
  responseBody: '{"ok":true}',
  responseStatus: 200,
  responseHeaders: '{"content-type":"application/json"}'
})
```

**Parameters:**
- `script: string` — JavaScript test script
- `responseBody: string` — response body string
- `responseStatus: number` — HTTP status code
- `responseHeaders: string` — JSON string of response headers

**Returns:** `ScriptOutput` with `logs: string[]`, `test_results: TestResult[]`, `error: string | null`

**Script API:** The `api900` global object provides:
- `api900.response.status` — HTTP status code (number)
- `api900.response.body` — response body (string)
- `api900.response.headers` — response headers (JSON string)

### `import_postman`
Imports a Postman v2.1 collection JSON file, creating a new collection with all requests.

```typescript
const collection = await invoke<Collection>('import_postman', { path: '/path/to/postman-collection.json' })
```

**Parameters:**
- `path: string` — path to the Postman collection JSON file

**Returns:** `Collection` with all imported requests

### `ws_connect`
Connects to a WebSocket server. Emits `ws-{id}-state` and `ws-{id}-message` events.

```typescript
await invoke('ws_connect', { id: 'uuid', url: 'wss://echo.websocket.org' })
```

### `ws_send`
Sends a text message over a WebSocket connection.

```typescript
await invoke('ws_send', { id: 'uuid', message: 'Hello server' })
```

### `ws_disconnect`
Disconnects a WebSocket connection.

```typescript
await invoke('ws_disconnect', { id: 'uuid' })
```

### `ws_get_state`
Returns the current state and message history of a WebSocket connection.

```typescript
const state = await invoke<WsConnectionState>('ws_get_state', { id: 'uuid' })
```

### `sse_connect`
Connects to a Server-Sent Events endpoint. Emits `sse-{id}-state` and `sse-{id}-event` events.

```typescript
await invoke('sse_connect', {
  id: 'uuid',
  url: 'https://api.example.com/events',
  headers: [{ key: 'Authorization', value: 'Bearer token', enabled: true }]
})
```

### `sse_disconnect`
Disconnects an SSE connection.

```typescript
await invoke('sse_disconnect', { id: 'uuid' })
```

### `sse_get_state`
Returns the current state and event count of an SSE connection.

```typescript
const state = await invoke<SseConnectionState>('sse_get_state', { id: 'uuid' })
```

### `send_grpc`
Sends a unary gRPC call with raw protobuf bytes (hex-encoded) over HTTP/2.

```typescript
const response = await invoke<GrpcResponse>('send_grpc', {
  address: 'localhost:50051',
  serviceMethod: '/package.Service/Method',
  bodyHex: '0a0548656c6c6f',
  headers: [{ key: 'authorization', value: 'Bearer token', enabled: true }],
  useTls: false
})
```

**Parameters:**
- `address: string` — host:port (e.g., `localhost:50051`)
- `serviceMethod: string` — gRPC path (e.g., `/package.Service/Method`)
- `bodyHex: string` — hex-encoded protobuf message bytes
- `headers: KeyValue[]` — gRPC metadata
- `useTls: boolean` — use TLS (https) or plaintext h2c

**Returns:** `GrpcResponse` with `status`, `grpc_status`, `grpc_message`, `body_hex`, `body_size`, `time_ms`, `headers`, `trailers`

## Authentication

900API supports the following auth types via `AuthConfig`:

| Type | Description |
|------|-------------|
| `none` | No authentication |
| `basic` | HTTP Basic Auth (username/password) |
| `bearer` | Bearer token |
| `api_key` | API Key in header or query param |
| `o_auth2` | OAuth 2.0 (access token with token type) |
| `o_auth1` | OAuth 1.0a (HMAC-SHA256 signing) |
| `aws_sig_v4` | AWS Signature v4 (HMAC-SHA256) |
| `hawk` | Hawk authentication (HMAC-SHA256) |

Auth is applied automatically by the HTTP engine when sending requests. The `AuthConfig` struct includes fields for each auth type.

### `mock_start`
Starts a local mock server with defined routes.

```typescript
await invoke('mock_start', {
  config: {
    port: 3001,
    routes: [{
      id: 'uuid',
      method: 'GET',
      path: '/api/hello',
      status: 200,
      headers: [{ key: 'Content-Type', value: 'application/json', enabled: true }],
      body: '{"message":"Hello"}',
      delay_ms: 0
    }]
  }
})
```

### `mock_stop`
Stops a running mock server.

```typescript
await invoke('mock_stop', { port: 3001 })
```

### `mock_get_state`
Returns the state of a mock server.

```typescript
const state = await invoke<MockServerState>('mock_get_state', { port: 3001 })
```

### `mock_list_servers`
Returns a list of running mock server ports.

```typescript
const ports = await invoke<number[]>('mock_list_servers')
```

### `sync_set_config`
Configures the git sync directory and author info.

```typescript
await invoke('sync_set_config', {
  config: {
    directory: '/path/to/repo',
    auto_sync: false,
    author_name: 'John Doe',
    author_email: 'john@example.com'
  }
})
```

### `sync_get_config`
Returns the current sync configuration.

```typescript
const config = await invoke<SyncConfig | null>('sync_get_config')
```

### `sync_export_collection`
Exports a collection as JSON to the sync directory.

```typescript
const path = await invoke<string>('sync_export_collection', {
  collection: { name: 'My API', description: null, requests: [], exported_at: '...', version: '1.0' }
})
```

### `sync_import_collection`
Imports a collection from a JSON file.

```typescript
const collection = await invoke<ExportCollection>('sync_import_collection', { filePath: '/path/to/collection.json' })
```

### `sync_list_collections`
Lists collection names in the sync directory.

```typescript
const names = await invoke<string[]>('sync_list_collections')
```

### `sync_git_init`
Initializes a git repository in the sync directory.

### `sync_git_status`
Returns git status (is_repo, changed_files).

### `sync_git_commit`
Stages all changes and commits with a message.

### `sync_git_pull`
Runs `git pull --rebase` in the sync directory.

### `sync_git_push`
Runs `git push` in the sync directory.

### `run_test_suite`
Runs a single test suite (sends request, evaluates assertions).

```typescript
const result = await invoke<TestSuiteResult>('run_test_suite', {
  suite: { id: '...', name: '...', request: {...}, assertions: [...] },
  environmentVariables: [...]
})
```

### `run_test_suites`
Runs multiple test suites and returns aggregate results.

```typescript
const result = await invoke<TestRunResult>('run_test_suites', {
  suites: [...],
  environmentVariables: [...]
})
```

### `generate_collection_docs`
Generates API documentation from a saved collection.

```typescript
const doc = await invoke<ApiDoc>('generate_collection_docs', { collectionId: '...' })
```

### `generate_all_docs`
Generates API documentation from all saved collections.

```typescript
const docs = await invoke<ApiDoc[]>('generate_all_docs')
```

### `docs_to_markdown`
Converts an ApiDoc to Markdown string.

### `docs_to_html`
Converts an ApiDoc to styled HTML string.

### `write_text_file`
Writes text content to a file path. Restricted to the user's home directory.

```typescript
await invoke('write_text_file', { path: '/path/to/file.md', content: '...' })
```

**Parameters:**
- `path: string` — file path (must be within the user's home directory)
- `content: string` — text content to write

**Returns:** `void`

### `plugin_list`
Lists all installed plugins.

```typescript
const plugins = await invoke<Plugin[]>('plugin_list')
```

**Returns:** `Plugin[]`

### `plugin_get`
Gets a single plugin by ID.

```typescript
const plugin = await invoke<Plugin | null>('plugin_get', { id: 'my-plugin' })
```

**Parameters:**
- `id: string` — plugin ID

**Returns:** `Plugin | null`

### `plugin_install`
Installs a plugin from a manifest.

```typescript
const plugin = await invoke<Plugin>('plugin_install', {
  manifest: { id: 'my-plugin', name: 'My Plugin', version: '1.0.0', author: 'Me', description: '', permissions: [], hooks: [] }
})
```

**Parameters:**
- `manifest: PluginManifest` — plugin manifest

**Returns:** `Plugin`

### `plugin_uninstall`
Uninstalls a plugin by ID.

```typescript
await invoke('plugin_uninstall', { id: 'my-plugin' })
```

### `plugin_enable`
Enables a plugin by ID.

```typescript
await invoke('plugin_enable', { id: 'my-plugin' })
```

### `plugin_disable`
Disables a plugin by ID.

```typescript
await invoke('plugin_disable', { id: 'my-plugin' })
```

### `plugin_update_config`
Updates a plugin's configuration.

```typescript
await invoke('plugin_update_config', { id: 'my-plugin', config: { key: 'value' } })
```

**Parameters:**
- `id: string` — plugin ID
- `config: Record<string, string>` — configuration key-value pairs

### `team_list_workspaces`
Lists all team workspaces.

```typescript
const workspaces = await invoke<Workspace[]>('team_list_workspaces')
```

**Returns:** `Workspace[]`

### `team_get_workspace`
Gets a single workspace by ID.

```typescript
const workspace = await invoke<Workspace | null>('team_get_workspace', { id: 'ws-1' })
```

**Returns:** `Workspace | null`

### `team_create_workspace`
Creates a new workspace.

```typescript
const workspace = await invoke<Workspace>('team_create_workspace', {
  name: 'My Team',
  description: 'Team description',
  owner: { id: 'user-1', name: 'Alice', email: 'alice@example.com', role: 'owner' }
})
```

**Parameters:**
- `name: string` — workspace name
- `description: string | null` — optional description
- `owner: TeamMember` — workspace owner

**Returns:** `Workspace`

### `team_delete_workspace`
Deletes a workspace by ID.

```typescript
await invoke('team_delete_workspace', { id: 'ws-1' })
```

### `team_add_member`
Adds a member to a workspace.

```typescript
await invoke('team_add_member', { workspaceId: 'ws-1', member: { id: 'u2', name: 'Bob', email: 'bob@example.com', role: 'editor' } })
```

### `team_remove_member`
Removes a member from a workspace.

```typescript
await invoke('team_remove_member', { workspaceId: 'ws-1', memberId: 'u2' })
```

### `team_update_member_role`
Updates a member's role in a workspace.

```typescript
await invoke('team_update_member_role', { workspaceId: 'ws-1', memberId: 'u2', role: 'viewer' })
```

### `team_get_activity`
Gets recent activity events for a workspace.

```typescript
const events = await invoke<ActivityEvent[]>('team_get_activity', { workspaceId: 'ws-1', limit: 50 })
```

**Returns:** `ActivityEvent[]`

### `team_share_collection`
Shares a collection with a workspace.

```typescript
await invoke('team_share_collection', { workspaceId: 'ws-1', collectionId: 'col-1' })
```

### `team_unshare_collection`
Unshares a collection from a workspace.

```typescript
await invoke('team_unshare_collection', { workspaceId: 'ws-1', collectionId: 'col-1' })
```

### `import_postman`
Imports a Postman collection from a JSON file.

```typescript
const collection = await invoke<Collection>('import_postman', { path: '/path/to/postman.json' })
```

**Parameters:**
- `path: string` — path to the Postman collection JSON file

**Returns:** `Collection`

## Types

```typescript
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS'

interface KeyValue {
  key: string
  value: string
  enabled: boolean
}

interface RequestConfig {
  method: HttpMethod
  url: string
  headers: KeyValue[]
  params: KeyValue[]
  body_type: 'none' | 'json' | 'form_data' | 'x_www_form_urlencoded' | 'raw' | 'binary'
  body: string
  auth: AuthConfig
}

interface ResponseData {
  status: number
  status_text: string
  headers: Record<string, string>
  body: string
  time_ms: number
  size_bytes: number
}

interface Collection {
  id: string
  name: string
  description: string | null
  created_at: string
  updated_at: string
}

interface Environment {
  id: string
  name: string
  variables: string  // JSON array of KeyValue
  created_at: string
  updated_at: string
}

interface HistoryEntry {
  id: string
  method: string
  url: string
  status: number
  time_ms: number
  created_at: string
}

interface SavedRequest {
  id: string
  collection_id: string
  name: string
  method: string
  url: string
  headers: string  // JSON array of KeyValue
  params: string   // JSON array of KeyValue
  body_type: string
  body: string
  auth_type: string
  auth_config: string  // JSON AuthConfig
  sort_order: number
  created_at: string
  updated_at: string
}

interface EnvironmentVariable {
  key: string
  value: string
  enabled: boolean
}

interface TestResult {
  name: string
  passed: boolean
  message: string
}

interface ScriptOutput {
  logs: string[]
  test_results: TestResult[]
  error: string | null
}

interface WsMessage {
  id: string
  direction: 'sent' | 'received'
  content: string
  message_type: 'text' | 'binary' | 'ping' | 'pong' | 'close'
  timestamp: number
}

interface WsConnectionState {
  id: string
  url: string
  status: 'connecting' | 'connected' | 'disconnected' | 'error'
  messages: WsMessage[]
}

interface SseEvent {
  id: string
  event_type: string
  data: string
  retry: number | null
  timestamp: number
}

interface SseConnectionState {
  id: string
  url: string
  status: 'connecting' | 'connected' | 'disconnected' | 'error'
  event_count: number
}

interface GrpcResponse {
  status: number
  grpc_status: number
  grpc_message: string
  body_hex: string
  body_size: number
  time_ms: number
  headers: Record<string, string>
  trailers: Record<string, string>
}

interface MockRoute {
  id: string
  method: string
  path: string
  status: number
  headers: KeyValue[]
  body: string
  delay_ms: number
}

interface MockServerConfig {
  port: number
  routes: MockRoute[]
}

interface MockServerState {
  port: number
  running: boolean
  request_count: number
}

interface SyncConfig {
  directory: string
  auto_sync: boolean
  author_name: string
  author_email: string
}

interface ExportRequest {
  name: string
  method: string
  url: string
  headers: KeyValue[]
  params: KeyValue[]
  body_type: string
  body: string
  auth_type: string
  auth_config: string
}

interface ExportCollection {
  name: string
  description: string | null
  requests: ExportRequest[]
  exported_at: string
  version: string
}

interface GitStatus {
  is_repo: boolean
  changed_files: string[]
}

type AssertionType = 'status' | 'header' | 'body' | 'body_json_path' | 'response_time' | 'body_contains'
type AssertionOperator = 'equals' | 'not_equals' | 'contains' | 'not_contains' | 'greater_than' | 'less_than' | 'exists' | 'not_exists'

interface Assertion {
  id: string
  assertion_type: AssertionType
  target: string
  operator: AssertionOperator
  expected: string
}

interface TestRequest {
  method: string
  url: string
  headers: KeyValue[]
  params: KeyValue[]
  body_type: string
  body: string
  auth: AuthConfig
}

interface TestSuite {
  id: string
  name: string
  request: TestRequest
  assertions: Assertion[]
  pre_request_script: string
  test_script: string
}

interface AssertionResult {
  assertion_id: string
  passed: boolean
  actual: string
  message: string
}

interface TestSuiteResult {
  suite_id: string
  suite_name: string
  passed: boolean
  assertions: AssertionResult[]
  response_status: number
  response_time_ms: number
  error: string | null
}

interface TestRunResult {
  total: number
  passed: number
  failed: number
  duration_ms: number
  results: TestSuiteResult[]
}

interface EndpointDoc {
  name: string
  method: string
  url: string
  headers: KeyValue[]
  params: KeyValue[]
  body_type: string
  body: string
  auth_type: string
  description: string
}

interface ApiDoc {
  collection_name: string
  collection_description: string | null
  endpoints: EndpointDoc[]
}

type AuthType = 'none' | 'basic' | 'bearer' | 'api_key' | 'o_auth2' | 'o_auth1' | 'aws_sig_v4' | 'hawk'

interface AuthConfig {
  auth_type: AuthType
  username: string
  password: string
  token: string
  api_key: string
  api_key_name: string
  api_key_in: string
  oauth2_access_token: string
  oauth2_token_type: string
  oauth2_refresh_token: string
  oauth1_consumer_key: string
  oauth1_consumer_secret: string
  oauth1_token: string
  oauth1_token_secret: string
  aws_access_key_id: string
  aws_secret_access_key: string
  aws_region: string
  aws_service: string
  hawk_id: string
  hawk_key: string
  hawk_algorithm: string
}

interface PluginManifest {
  id: string
  name: string
  version: string
  author: string
  description: string
  permissions: string[]
  hooks: string[]
}

interface Plugin {
  id: string
  manifest: PluginManifest
  enabled: boolean
  installed_at: string
  config: Record<string, string>
}

type TeamRole = 'owner' | 'admin' | 'editor' | 'viewer'

interface TeamMember {
  id: string
  name: string
  email: string
  role: TeamRole
}

interface Workspace {
  id: string
  name: string
  description: string | null
  owner_id: string
  members: TeamMember[]
  collection_ids: string[]
  environment_ids: string[]
  created_at: string
}

interface ActivityEvent {
  id: string
  workspace_id: string
  event_type: string
  actor_id: string
  description: string
  timestamp: string
}
```
