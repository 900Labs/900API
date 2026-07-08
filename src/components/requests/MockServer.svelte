<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { Play, Square, Plus, Trash2, Server } from '@lucide/svelte'

  type KeyValue = { key: string; value: string; enabled: boolean }

  type MockRoute = {
    id: string
    method: string
    path: string
    status: number
    headers: KeyValue[]
    body: string
    delay_ms: number
  }

  type MockServerState = {
    port: number
    running: boolean
    request_count: number
    bind_host: string
    cors_permissive: boolean
  }

  let port = $state(3001)
  let allowLan = $state(false)
  let allowCors = $state(false)
  let runningBindHost = $state('127.0.0.1')
  let routes = $state<MockRoute[]>([
    {
      id: crypto.randomUUID(),
      method: 'GET',
      path: '/api/hello',
      status: 200,
      headers: [{ key: 'Content-Type', value: 'application/json', enabled: true }],
      body: '{"message": "Hello from mock server!"}',
      delay_ms: 0,
    },
  ])
  let running = $state(false)
  let requestCount = $state(0)
  let error = $state<string | null>(null)
  let selectedRouteId = $state<string | null>(null)

  const bodyPlaceholder = '{"key": "value"}'

  const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS', '*']

  $effect(() => {
    if (selectedRouteId === null && routes.length > 0) {
      selectedRouteId = routes[0].id
    }
  })

  let selectedRoute = $derived(routes.find((r) => r.id === selectedRouteId))

  async function startServer() {
    error = null
    try {
      await invoke('mock_start', {
        config: {
          port,
          routes,
          bind_host: allowLan ? '0.0.0.0' : '127.0.0.1',
          cors_permissive: allowCors,
        },
      })
      running = true
      runningBindHost = allowLan ? '0.0.0.0' : '127.0.0.1'
      pollState()
    } catch (e) {
      error = String(e)
    }
  }

  async function stopServer() {
    try {
      await invoke('mock_stop', { port })
      running = false
    } catch (e) {
      error = String(e)
    }
  }

  async function pollState() {
    if (!running) return
    try {
      const state = await invoke<MockServerState>('mock_get_state', { port })
      requestCount = state.request_count
      runningBindHost = state.bind_host
      allowCors = state.cors_permissive
      setTimeout(pollState, 2000)
    } catch {
      running = false
    }
  }

  function addRoute() {
    const newRoute: MockRoute = {
      id: crypto.randomUUID(),
      method: 'GET',
      path: '/api/new',
      status: 200,
      headers: [{ key: 'Content-Type', value: 'application/json', enabled: true }],
      body: '{}',
      delay_ms: 0,
    }
    routes = [...routes, newRoute]
    selectedRouteId = newRoute.id
  }

  function removeRoute(id: string) {
    routes = routes.filter((r) => r.id !== id)
    if (selectedRouteId === id) {
      selectedRouteId = routes.length > 0 ? routes[0].id : null
    }
  }

  function addHeader() {
    if (selectedRoute) {
      selectedRoute.headers = [...selectedRoute.headers, { key: '', value: '', enabled: true }]
    }
  }

  function removeHeader(index: number) {
    if (selectedRoute) {
      selectedRoute.headers = selectedRoute.headers.filter((_, i) => i !== index)
    }
  }
</script>

<div class="flex h-full flex-col">
  <!-- Server Controls -->
  <div class="flex items-center gap-3 border-b border-border p-3">
    <Server class="h-5 w-5 text-text-muted" />
    <div class="flex items-center gap-2">
      <span class="text-sm text-text-muted">Port:</span>
      <input
        type="number"
        class="w-20 rounded-md border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
        bind:value={port}
        disabled={running}
      />
    </div>
    <label class="flex items-center gap-1.5 text-xs text-text-muted">
      <input type="checkbox" bind:checked={allowLan} class="accent-accent" disabled={running} />
      LAN
    </label>
    <label class="flex items-center gap-1.5 text-xs text-text-muted">
      <input type="checkbox" bind:checked={allowCors} class="accent-accent" disabled={running} />
      CORS
    </label>
    <span class="flex items-center gap-1.5 text-xs font-medium {running ? 'text-success' : 'text-text-muted'}">
      <span class="h-2 w-2 rounded-full {running ? 'bg-success' : 'bg-text-muted'}"></span>
      {running ? 'Running' : 'Stopped'}
    </span>
    {#if running}
      <span class="text-xs text-text-muted">{requestCount} requests served</span>
    {/if}
    <div class="flex-1"></div>
    {#if running}
      <button
        class="flex items-center gap-2 rounded-md border border-error/30 bg-error/10 px-4 py-2 text-sm font-medium text-error transition-colors hover:bg-error/20"
        onclick={stopServer}
      >
        <Square class="h-4 w-4" />
        Stop
      </button>
    {:else}
      <button
        class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
        onclick={startServer}
      >
        <Play class="h-4 w-4" />
        Start
      </button>
    {/if}
  </div>

  {#if error}
    <div class="border-b border-border bg-error/10 p-3 text-sm text-error">{error}</div>
  {/if}

  {#if running}
    <div class="border-b border-border bg-success/5 px-3 py-2 text-xs text-success">
      Mock server running at http://localhost:{port}{runningBindHost === '0.0.0.0' ? ' (LAN enabled)' : ''}
    </div>
  {/if}

  <div class="flex flex-1 overflow-hidden">
    <!-- Route List -->
    <div class="w-64 border-r border-border overflow-y-auto">
      <div class="flex items-center justify-between p-2">
        <span class="text-xs font-medium uppercase text-text-muted">Routes</span>
        <button class="text-accent hover:text-accent-hover" onclick={addRoute} title="Add route">
          <Plus class="h-4 w-4" />
        </button>
      </div>
      {#each routes as route (route.id)}
        <button
          class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors {selectedRouteId === route.id ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => (selectedRouteId = route.id)}
        >
          <span class="rounded bg-accent/10 px-1.5 py-0.5 text-xs font-medium text-accent">{route.method}</span>
          <span class="flex-1 truncate font-mono text-xs">{route.path}</span>
          <span
            class="text-text-muted hover:text-error"
            role="button"
            tabindex="0"
            onclick={(e) => { e.stopPropagation(); removeRoute(route.id) }}
            onkeydown={(e) => e.key === 'Enter' && (e.stopPropagation(), removeRoute(route.id))}
          >
            <Trash2 class="h-3.5 w-3.5" />
          </span>
        </button>
      {/each}
    </div>

    <!-- Route Editor -->
    <div class="flex-1 overflow-y-auto p-4">
      {#if selectedRoute}
        <div class="space-y-4">
          <!-- Method & Path -->
          <div class="flex gap-2">
            <select
              class="rounded-md border border-border bg-surface px-2 py-2 text-sm font-mono outline-none focus:border-accent"
              bind:value={selectedRoute.method}
              disabled={running}
            >
              {#each methods as m}
                <option value={m}>{m}</option>
              {/each}
            </select>
            <input
              type="text"
              class="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="/api/users/:id"
              bind:value={selectedRoute.path}
              disabled={running}
            />
          </div>

          <!-- Status & Delay -->
          <div class="flex gap-4">
            <div class="flex items-center gap-2">
              <span class="text-xs text-text-muted">Status:</span>
              <input
                type="number"
                class="w-20 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                bind:value={selectedRoute.status}
                disabled={running}
              />
            </div>
            <div class="flex items-center gap-2">
              <span class="text-xs text-text-muted">Delay (ms):</span>
              <input
                type="number"
                class="w-24 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                bind:value={selectedRoute.delay_ms}
                disabled={running}
              />
            </div>
          </div>

          <!-- Headers -->
          <div>
            <div class="mb-2 flex items-center justify-between">
              <span class="text-xs font-medium uppercase text-text-muted">Response Headers</span>
              <button class="text-accent hover:text-accent-hover" onclick={addHeader} disabled={running}>+ Add</button>
            </div>
            <div class="space-y-2">
              {#each selectedRoute.headers as header, i (i)}
                <div class="flex items-center gap-2">
                  <input type="checkbox" bind:checked={header.enabled} class="accent-accent" disabled={running} />
                  <input
                    type="text"
                    class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                    placeholder="Header name"
                    bind:value={header.key}
                    disabled={running}
                  />
                  <input
                    type="text"
                    class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                    placeholder="Value"
                    bind:value={header.value}
                    disabled={running}
                  />
                  <button class="text-text-muted hover:text-error" onclick={() => removeHeader(i)} disabled={running}>✕</button>
                </div>
              {/each}
            </div>
          </div>

          <!-- Response Body -->
          <div>
            <span class="mb-2 block text-xs font-medium uppercase text-text-muted">Response Body</span>
            <textarea
              class="h-48 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
              placeholder={bodyPlaceholder}
              bind:value={selectedRoute.body}
              disabled={running}
            ></textarea>
          </div>
        </div>
      {:else}
        <div class="flex h-full items-center justify-center text-text-muted">
          <p>Select a route or create a new one.</p>
        </div>
      {/if}
    </div>
  </div>
</div>
