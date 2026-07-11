<script lang="ts">
  import Sidebar from './components/Sidebar.svelte'
  import AppMenuBar from './components/workspace/AppMenuBar.svelte'
  import CommandPalette, { type CommandAction } from './components/workspace/CommandPalette.svelte'
  import WorkspaceShell from './components/workspace/WorkspaceShell.svelte'
  import GraphQLBuilder from './components/requests/GraphQLBuilder.svelte'
  import WebSocketClient from './components/requests/WebSocketClient.svelte'
  import SseClient from './components/requests/SseClient.svelte'
  import GrpcClient from './components/requests/GrpcClient.svelte'
  import MockServer from './components/requests/MockServer.svelte'
  import GitSync from './components/requests/GitSync.svelte'
  import CollectionTree from './components/collections/CollectionTree.svelte'
  import EnvironmentManager from './components/environments/EnvironmentManager.svelte'
  import TestRunner from './components/requests/TestRunner.svelte'
  import ApiDocs from './components/requests/ApiDocs.svelte'
  import SettingsView from './components/requests/Settings.svelte'
  import PluginManager from './components/requests/PluginManager.svelte'
  import TeamWorkflows from './components/requests/TeamWorkflows.svelte'
  import { activeEnvironmentStore, loadRequestStore } from './lib/stores'
  import { invoke } from './lib/tauri'

  type ViewName = 'requests' | 'graphql' | 'websocket' | 'sse' | 'grpc' | 'mock' | 'sync' | 'collections' | 'environments' | 'tests' | 'docs' | 'plugins' | 'team' | 'settings'

  type Collection = {
    id: string
    name: string
    description?: string | null
    parent_id?: string | null
    sort_order?: number
  }

  type SavedRequest = {
    id: string
    collection_id: string
    name: string
    method: string
    url: string
    headers: string
    params: string
    body_type: string
    body: string
    auth_type: string
    auth_config: string
    settings: string
    pre_request_script: string
    test_script: string
  }

  type Environment = {
    id: string
    name: string
    variables: string
  }

  type HistoryEntry = {
    id: string
    method: string
    url: string
    status: number
    time_ms: number
    size_bytes: number
    request_snapshot: string
    created_at: string
  }

  let activeView = $state<ViewName>('requests')
  let appVersion = $state('0.2.0')
  let showCommandPalette = $state(false)
  let collections = $state<Collection[]>([])
  let savedRequests = $state<SavedRequest[]>([])
  let environments = $state<Environment[]>([])
  let historyEntries = $state<HistoryEntry[]>([])
  let searchIndexError = $state<string | null>(null)

  function dispatchClientAction(name: string) {
    window.dispatchEvent(new CustomEvent(`900api:${name}`))
  }

  function openRestAndDispatch(name: string) {
    activeView = 'requests'
    setTimeout(() => dispatchClientAction(name), 0)
  }

  function shortUrl(value: string): string {
    if (!value.trim()) return ''
    try {
      const parsed = new URL(value)
      return `${parsed.host}${parsed.pathname === '/' ? '' : parsed.pathname}`
    } catch {
      return value
    }
  }

  function formatDate(value: string): string {
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value
    return date.toLocaleString()
  }

  function parseEnvironmentVariables(env: Environment): { key: string; value: string; enabled: boolean }[] {
    try {
      const parsed = JSON.parse(env.variables || '[]')
      return Array.isArray(parsed) ? parsed : []
    } catch {
      return []
    }
  }

  function collectionPath(collectionId: string | null | undefined): string {
    if (!collectionId) return 'No collection'
    const byId = new Map(collections.map((collection) => [collection.id, collection]))
    const parts: string[] = []
    let current = byId.get(collectionId)
    const seen = new Set<string>()
    while (current && !seen.has(current.id)) {
      seen.add(current.id)
      parts.unshift(current.name)
      current = current.parent_id ? byId.get(current.parent_id) : undefined
    }
    return parts.join(' / ') || 'Collection'
  }

  function openSavedRequest(request: SavedRequest) {
    activeView = 'requests'
    setTimeout(() => {
      loadRequestStore.set({
        id: request.id,
        collectionId: request.collection_id,
        name: request.name,
        method: request.method,
        url: request.url,
        headers: request.headers,
        params: request.params,
        bodyType: request.body_type,
        body: request.body,
        authType: request.auth_type,
        authConfig: request.auth_config,
        settings: request.settings || '{}',
        preRequestScript: request.pre_request_script,
        testScript: request.test_script,
      })
    }, 0)
  }

  function openHistoryEntry(entry: HistoryEntry) {
    activeView = 'requests'
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent('900api:open-history', { detail: entry }))
    }, 0)
  }

  function activateEnvironment(env: Environment) {
    activeView = 'requests'
    activeEnvironmentStore.set({
      id: env.id,
      name: env.name,
      variables: parseEnvironmentVariables(env),
    })
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent('900api:select-environment', { detail: { id: env.id } }))
    }, 0)
  }

  async function loadSearchIndex() {
    searchIndexError = null
    try {
      const [loadedCollections, loadedEnvironments, loadedHistory] = await Promise.all([
        invoke<Collection[]>('list_collections'),
        invoke<Environment[]>('list_environments'),
        invoke<HistoryEntry[]>('list_history', { limit: 100 }),
      ])
      collections = loadedCollections
      environments = loadedEnvironments
      historyEntries = loadedHistory
      const requestGroups = await Promise.all(
        loadedCollections.map(async (collection) => {
          try {
            return await invoke<SavedRequest[]>('list_requests', { collectionId: collection.id })
          } catch {
            return []
          }
        }),
      )
      savedRequests = requestGroups.flat()
    } catch (e) {
      searchIndexError = String(e)
    }
  }

  let baseCommandActions = $derived<CommandAction[]>([
    { id: 'new-request', group: 'File', label: 'New REST request', shortcut: 'Ctrl N', run: () => openRestAndDispatch('new-request') },
    { id: 'save-request', group: 'File', label: 'Save active request', shortcut: 'Ctrl S', run: () => openRestAndDispatch('save-request') },
    { id: 'import-curl', group: 'File', label: 'Import cURL', run: () => openRestAndDispatch('import-curl') },
    { id: 'import-collection', group: 'File', label: 'Import collection', run: () => openRestAndDispatch('import-collection') },
    { id: 'new-collection', group: 'File', label: 'New collection', run: () => openRestAndDispatch('new-collection') },
    { id: 'send-request', group: 'Request', label: 'Send active request', shortcut: 'Ctrl Enter', run: () => openRestAndDispatch('send-request') },
    { id: 'open-code-snippets', group: 'Request', label: 'Generate code snippet', run: () => openRestAndDispatch('open-code-snippets') },
    { id: 'duplicate-request-tab', group: 'Request', label: 'Duplicate request tab', run: () => openRestAndDispatch('duplicate-request-tab') },
    { id: 'close-request-tab', group: 'Request', label: 'Close request tab', run: () => openRestAndDispatch('close-request-tab') },
    { id: 'view-rest', group: 'View', label: 'REST workbench', run: () => (activeView = 'requests') },
    { id: 'view-graphql', group: 'View', label: 'GraphQL client', run: () => (activeView = 'graphql') },
    { id: 'view-websocket', group: 'View', label: 'WebSocket client', run: () => (activeView = 'websocket') },
    { id: 'view-sse', group: 'View', label: 'SSE client', run: () => (activeView = 'sse') },
    { id: 'view-grpc', group: 'View', label: 'gRPC client', run: () => (activeView = 'grpc') },
    { id: 'view-mock', group: 'Tools', label: 'Mock server', run: () => (activeView = 'mock') },
    { id: 'view-sync', group: 'Tools', label: 'Git sync', run: () => (activeView = 'sync') },
    { id: 'view-tests', group: 'Tools', label: 'Collection runner', run: () => (activeView = 'tests') },
    { id: 'view-docs', group: 'Tools', label: 'API docs generator', run: () => (activeView = 'docs') },
    { id: 'view-settings', group: 'Help', label: 'Settings', run: () => (activeView = 'settings') },
    { id: 'refresh-search-index', group: 'Search', label: 'Refresh local search index', detail: searchIndexError ?? `${savedRequests.length} requests, ${collections.length} collections, ${historyEntries.length} history items`, run: loadSearchIndex },
  ])

  let searchResultActions = $derived<CommandAction[]>([
    ...savedRequests.map((request) => ({
      id: `request-${request.id}`,
      group: 'Requests',
      label: `${request.method} ${request.name}`,
      detail: `${collectionPath(request.collection_id)} · ${request.url}`,
      run: () => openSavedRequest(request),
    })),
    ...historyEntries.map((entry) => ({
      id: `history-${entry.id}`,
      group: 'History',
      label: `${entry.method} ${shortUrl(entry.url) || entry.url}`,
      detail: `${entry.status} · ${entry.time_ms} ms · ${formatDate(entry.created_at)}`,
      run: () => openHistoryEntry(entry),
    })),
    ...collections.map((collection) => ({
      id: `collection-${collection.id}`,
      group: 'Collections',
      label: collectionPath(collection.id),
      detail: collection.description || 'Open collection manager',
      run: () => (activeView = 'collections'),
    })),
    ...environments.map((env) => ({
      id: `environment-${env.id}`,
      group: 'Environments',
      label: `Use ${env.name}`,
      detail: `${parseEnvironmentVariables(env).filter((variable) => variable.enabled).length} enabled variables`,
      run: () => activateEnvironment(env),
    })),
  ])

  let commandActions = $derived<CommandAction[]>([...baseCommandActions, ...searchResultActions])

  let menuGroups = $derived([
    {
      label: 'File',
      actions: baseCommandActions.filter((action) => ['new-request', 'save-request', 'import-curl', 'new-collection', 'import-collection'].includes(action.id)),
    },
    {
      label: 'Request',
      actions: baseCommandActions.filter((action) => ['send-request', 'open-code-snippets', 'duplicate-request-tab', 'close-request-tab'].includes(action.id)),
    },
    {
      label: 'View',
      actions: baseCommandActions.filter((action) => ['view-rest', 'view-graphql', 'view-websocket', 'view-sse', 'view-grpc'].includes(action.id)),
    },
    {
      label: 'Tools',
      actions: baseCommandActions.filter((action) => ['view-mock', 'view-sync', 'view-tests', 'view-docs'].includes(action.id)),
    },
    {
      label: 'Help',
      actions: baseCommandActions.filter((action) => ['view-settings'].includes(action.id)),
    },
  ])

  async function checkBackend() {
    try {
      const version = await invoke<string>('get_app_version')
      appVersion = version
    } catch {
      // Backend not available in pure browser dev mode
    }
  }

  $effect(() => {
    checkBackend()
    loadSearchIndex()
  })

  $effect(() => {
    if (showCommandPalette) loadSearchIndex()
  })

  $effect(() => {
    function handleKeydown(event: KeyboardEvent) {
      const modifier = event.metaKey || event.ctrlKey
      if (modifier && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        showCommandPalette = true
      } else if (modifier && event.key.toLowerCase() === 'n') {
        event.preventDefault()
        openRestAndDispatch('new-request')
      } else if (modifier && event.key.toLowerCase() === 's') {
        event.preventDefault()
        openRestAndDispatch('save-request')
      } else if (modifier && event.key === 'Enter') {
        event.preventDefault()
        openRestAndDispatch('send-request')
      }
    }

    window.addEventListener('keydown', handleKeydown)
    window.addEventListener('900api:collections-changed', loadSearchIndex)
    window.addEventListener('900api:history-changed', loadSearchIndex)
    window.addEventListener('900api:environments-changed', loadSearchIndex)
    return () => {
      window.removeEventListener('keydown', handleKeydown)
      window.removeEventListener('900api:collections-changed', loadSearchIndex)
      window.removeEventListener('900api:history-changed', loadSearchIndex)
      window.removeEventListener('900api:environments-changed', loadSearchIndex)
    }
  })
</script>

<div class="flex h-screen w-screen overflow-hidden bg-bg text-text">
  <Sidebar bind:activeView />
  <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <AppMenuBar groups={menuGroups} onOpenPalette={() => (showCommandPalette = true)} />
    {#if activeView === 'requests'}
      <WorkspaceShell />
    {:else if activeView === 'graphql'}
      <GraphQLBuilder />
    {:else if activeView === 'websocket'}
      <WebSocketClient />
    {:else if activeView === 'sse'}
      <SseClient />
    {:else if activeView === 'grpc'}
      <GrpcClient />
    {:else if activeView === 'mock'}
      <MockServer />
    {:else if activeView === 'sync'}
      <GitSync />
    {:else if activeView === 'collections'}
      <CollectionTree />
    {:else if activeView === 'environments'}
      <EnvironmentManager />
    {:else if activeView === 'tests'}
      <TestRunner />
    {:else if activeView === 'docs'}
      <ApiDocs />
    {:else if activeView === 'plugins'}
      <PluginManager />
    {:else if activeView === 'team'}
      <TeamWorkflows />
    {:else if activeView === 'settings'}
      <SettingsView />
    {/if}
  </main>
  <CommandPalette bind:open={showCommandPalette} actions={commandActions} />
</div>
