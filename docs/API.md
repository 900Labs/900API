# API Documentation — Tauri Command Reference

All commands are invoked from the frontend via `@tauri-apps/api/core` `invoke()`.

## Commands

### `get_app_version`
Returns the application version string.

```typescript
const version = await invoke<string>('get_app_version')
// "0.1.1"
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
    body: '',
    auth: { auth_type: 'none' },
    settings: {
      timeout_ms: 120000,
      connect_timeout_ms: 30000,
      follow_redirects: true,
      verify_ssl: true,
      proxy_url: '',
      use_cookie_jar: false
    }
  },
  environmentVariables: undefined
})
```

**Parameters:**
- `config: RequestConfig` — the request configuration
- `environmentVariables?: EnvironmentVariable[]` — optional env vars for `{{var}}` resolution in URL, headers, params, body, and auth fields

**Returns:** `ResponseData` with `status`, `status_text`, `headers`, `body`, `time_ms`, `size_bytes`

`config.settings` controls request runtime behavior. Defaults are a 120000 ms request timeout, 30000 ms connect timeout, redirects enabled, SSL verification enabled, no proxy, and no shared local cookie jar. Setting `use_cookie_jar: true` opts into an in-memory app-local cookie jar shared by cookie-enabled REST requests during the current app session.

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

### `introspect_graphql_schema`
Runs the standard GraphQL introspection query and returns a compact schema summary for the GraphQL schema explorer.

```typescript
const schema = await invoke<GraphQLSchema>('introspect_graphql_schema', {
  url: 'https://api.example.com/graphql',
  headers: [{ key: 'Authorization', value: 'Bearer token', enabled: true }],
  auth: { auth_type: 'none', username: '', password: '', token: '', api_key: '', api_key_name: '', api_key_in: 'header' },
  environmentVariables: undefined
})
```

**Parameters:**
- `url: string` — the GraphQL endpoint URL
- `headers: KeyValue[]` — custom headers
- `auth: AuthConfig` — auth configuration
- `environmentVariables?: EnvironmentVariable[]` — optional env vars for `{{var}}` resolution

**Returns:** `GraphQLSchema` with root operation names and non-executable schema metadata for types, fields, arguments, input fields, enum values, and possible types.

If the server disables introspection or returns GraphQL `errors`, the command returns an error with the server-provided message when available.

### `list_collections`
Returns all collections, including nested folder metadata.

```typescript
const collections = await invoke<Collection[]>('list_collections')
```

**Returns:** `Collection[]` with `id`, `name`, `description`, `parent_id`, `sort_order`, `created_at`, `updated_at`

### `create_collection`
Creates a new collection or subcollection.

```typescript
const collection = await invoke<Collection>('create_collection', {
  name: 'My API',
  description: 'Production API tests',
  parentId: null
})
```

**Parameters:**
- `name: string` — collection name
- `description?: string` — optional description
- `parentId?: string | null` — optional parent collection/folder ID

### `update_collection`
Renames or updates a collection description.

```typescript
await invoke('update_collection', {
  id: 'uuid-here',
  name: 'Renamed API',
  description: 'Updated description'
})
```

**Parameters:**
- `id: string` — collection ID
- `name: string` — collection name
- `description?: string | null` — optional description

### `move_collection`
Moves a collection into another collection/folder or back to the root. The backend rejects moves that would create a parent/child cycle.

```typescript
await invoke('move_collection', {
  id: 'uuid-here',
  parentId: 'parent-uuid'
})

await invoke('move_collection', {
  id: 'uuid-here',
  parentId: null
})
```

**Parameters:**
- `id: string` — collection ID to move
- `parentId?: string | null` — destination parent collection/folder ID

### `delete_collection`
Deletes a collection, its child collections, and its requests.

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
Returns recent request history, including response size and the original request snapshot used for replay.

```typescript
const history = await invoke<HistoryEntry[]>('list_history', { limit: 100 })
```

**Parameters:**
- `limit?: number` — max entries (default 100, max 500)

**Returns:** `HistoryEntry[]` with `id`, `method`, `url`, `status`, `time_ms`, `size_bytes`, `request_snapshot`, and `created_at`.

`request_snapshot` is a JSON-serialized `RequestConfig` captured before active environment variables are resolved. The REST workbench uses it to reopen history entries with headers, params, body, and auth intact while preserving `{{variable}}` placeholders.

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

**Returns:** `SavedRequest[]` with `id`, `collection_id`, `name`, `method`, `url`, `headers`, `params`, `body_type`, `body`, `auth_type`, `auth_config`, `settings`, `sort_order`, `created_at`, `updated_at`

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
  authConfig: '{}',
  settings: '{}'
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
  authConfig: '{"token":"abc"}',
  settings: '{"timeout_ms":120000,"connect_timeout_ms":30000,"follow_redirects":true,"verify_ssl":true,"proxy_url":"","use_cookie_jar":false}'
})
```

### `delete_request`
Deletes a request.

```typescript
await invoke('delete_request', { id: 'uuid' })
```

Deleting a request also deletes response examples attached to that request.

### `list_response_examples`
Returns saved response examples for a request.

```typescript
const examples = await invoke<ResponseExample[]>('list_response_examples', {
  requestId: 'request-uuid'
})
```

**Parameters:**
- `requestId: string` — saved request ID

**Returns:** `ResponseExample[]` ordered newest first.

### `create_response_example`
Saves the current response body, headers, status, timing, and size as an example attached to a saved request.

```typescript
const example = await invoke<ResponseExample>('create_response_example', {
  requestId: 'request-uuid',
  name: '200 OK',
  response
})
```

**Parameters:**
- `requestId: string` — saved request ID
- `name: string` — user-facing example name
- `response: ResponseData` — response payload to persist

**Returns:** `ResponseExample`

### `delete_response_example`
Deletes a saved response example.

The desktop response panel uses saved response examples for local current-vs-expected comparison. The Compare tab loads the same request-owned examples returned by `list_response_examples` and compares the current response status, formatted body, headers, and size without sending data to any external service.

```typescript
await invoke('delete_response_example', { id: 'example-uuid' })
```

### `move_request`
Moves a request to another collection/folder.

```typescript
await invoke('move_request', {
  id: 'request-uuid',
  collectionId: 'destination-collection-uuid'
})
```

**Parameters:**
- `id: string` — request ID to move
- `collectionId: string` — destination collection/folder ID

### `update_environment`
Updates an environment's variables.

```typescript
await invoke('update_environment', {
  id: 'uuid',
  variables: '[{"key":"baseUrl","value":"https://api.example.com","enabled":true}]'
})
```

### `export_collection`
Exports a collection, all requests, saved request scripts, request settings, and saved response examples to a 900API JSON file.

```typescript
await invoke('export_collection', { collectionId: 'uuid', path: '/path/to/export.json' })
```

### `export_openapi`
Exports a collection and all its requests to an OpenAPI 3.0.3 JSON file.

```typescript
await invoke('export_openapi', { collectionId: 'uuid', path: '/path/to/openapi.json' })
```

Exported OpenAPI docs include collection title/description, request paths and methods, query/header parameters, JSON/form/raw request bodies, and supported auth schemes as OpenAPI security schemes. 900API path variables such as `{{id}}` are exported as OpenAPI path variables such as `{id}`.

### `import_collection_file`
Imports a collection from a 900API JSON file, creating a new collection with all requests and saved response examples.

```typescript
const collection = await invoke<Collection>('import_collection_file', { path: '/path/to/import.json' })
```

### `run_test_script`
Runs a JavaScript test script in a sandboxed JS engine (boa) with access to `api900.response`. Scripts run without filesystem, network, DOM, `require`, `import`, or `process` access, and are bounded by loop, recursion, and stack limits.

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

### `import_openapi`
Imports an OpenAPI 3.x or Swagger 2.0 document, creating a new collection with all operations as requests.

```typescript
const collection = await invoke<Collection>('import_openapi', { path: '/path/to/openapi.yaml' })
```

**Parameters:**
- `path: string` — path to an OpenAPI/Swagger JSON, YAML, or YML file

**Returns:** `Collection` with all imported operations

The importer resolves local `$ref` references for common parameters, request bodies, schemas, and security schemes. It imports servers, path variables, query/header parameters, sample JSON/form/raw request bodies, and API key/basic/bearer/OAuth2 auth placeholders. Unsupported OpenAPI extensions are ignored rather than executed.

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

The frontend gRPC client includes a `.proto` helper that parses package, service, RPC, message, and scalar field declarations from pasted proto text. It can populate the `serviceMethod` path and generate `bodyHex` for common scalar protobuf fields. Full server reflection and descriptor-driven nested message editing are not part of this command yet.

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
Starts a local mock server with defined routes. By default, servers bind to `127.0.0.1` and do not enable permissive CORS. LAN binding and permissive CORS must be explicitly requested.

```typescript
await invoke('mock_start', {
  config: {
    port: 3001,
    bind_host: '127.0.0.1',
    cors_permissive: false,
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

**Parameters:**
- `port: number` — local port to listen on
- `bind_host?: string` — optional IP address; defaults to `127.0.0.1`, use `0.0.0.0` only when LAN access is intentional
- `cors_permissive?: boolean` — defaults to `false`
- `routes: MockRoute[]` — mock response routes

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

**Returns:** `MockServerState` with `port`, `running`, `request_count`, `bind_host`, and `cors_permissive`

### `mock_list_servers`
Returns a list of running mock server ports.

```typescript
const ports = await invoke<number[]>('mock_list_servers')
```

### `sync_set_config`
Configures the git sync directory and author info. The config is persisted in local app data and reloaded on desktop startup.

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
Runs a single test suite. The runner resolves environment variables, runs the pre-request script, sends the request, evaluates assertions, then runs the test script.

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

The desktop Test Runner can import saved requests from a collection as suites, including the saved pre-request and test scripts. Manually created and imported suites are persisted in the local WebView storage for the desktop user profile.

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

Generated HTML escapes collection names, descriptions, endpoint names, URLs, headers, params, body content, and auth labels before rendering.

### `write_text_file`
Writes text content to a file path. Restricted to an existing parent directory inside the user's home directory. The command rejects path traversal and refuses to write through symbolic links.

```typescript
await invoke('write_text_file', { path: '/path/to/file.md', content: '...' })
```

**Parameters:**
- `path: string` — file path; parent directory must exist inside the user's home directory
- `content: string` — text content to write

**Returns:** `void`

### `plugin_list`
Lists all installed plugin manifests. Plugin manifests, enabled state, and config values are persisted in local app data. Hook and permission fields are metadata only in this release; 900API does not execute plugin hook code at runtime.

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
Installs a plugin manifest into the local registry.

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
Lists all local team workspaces. Workspaces, members, roles, shared collection IDs, and shared environment IDs are persisted in local app data. Activity events are an in-session activity feed.

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

### `import_openapi`
Imports an OpenAPI 3.x or Swagger 2.0 JSON/YAML file.

```typescript
const collection = await invoke<Collection>('import_openapi', { path: '/path/to/openapi.yaml' })
```

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
  settings: RequestSettings
}

interface RequestSettings {
  timeout_ms: number
  connect_timeout_ms: number
  follow_redirects: boolean
  verify_ssl: boolean
  proxy_url: string
  use_cookie_jar: boolean
}

interface ResponseData {
  status: number
  status_text: string
  headers: Record<string, string>
  body: string
  time_ms: number
  size_bytes: number
}

interface ResponseExample {
  id: string
  request_id: string
  name: string
  status: number
  status_text: string
  headers: string
  body: string
  time_ms: number
  size_bytes: number
  created_at: string
}

interface GraphQLSchema {
  query_type: string | null
  mutation_type: string | null
  subscription_type: string | null
  types: GraphQLSchemaType[]
}

interface GraphQLSchemaType {
  kind: string
  name: string
  description: string | null
  fields: GraphQLField[]
  input_fields: GraphQLInputValue[]
  enum_values: GraphQLEnumValue[]
  possible_types: GraphQLTypeRef[]
}

interface GraphQLField {
  name: string
  description: string | null
  args: GraphQLInputValue[]
  field_type: GraphQLTypeRef
  is_deprecated: boolean
  deprecation_reason: string | null
}

interface GraphQLInputValue {
  name: string
  description: string | null
  value_type: GraphQLTypeRef
  default_value: string | null
}

interface GraphQLEnumValue {
  name: string
  description: string | null
  is_deprecated: boolean
  deprecation_reason: string | null
}

interface GraphQLTypeRef {
  kind: string
  name: string | null
  of_type: GraphQLTypeRef | null
}

interface Collection {
  id: string
  name: string
  description: string | null
  parent_id: string | null
  sort_order: number
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
  size_bytes: number
  request_snapshot: string  // JSON RequestConfig captured before environment variable resolution
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
  settings: string     // JSON RequestSettings
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
  bind_host?: string | null
  cors_permissive?: boolean
  routes: MockRoute[]
}

interface MockServerState {
  port: number
  running: boolean
  request_count: number
  bind_host: string
  cors_permissive: boolean
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
