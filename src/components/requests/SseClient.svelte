<script lang="ts">
  import { invoke, listen } from '../../lib/tauri'
  import { Plug, Unplug, Trash2 } from '@lucide/svelte'

  type SseEvent = {
    id: string
    event_type: string
    data: string
    retry: number | null
    timestamp: number
  }

  type SseConnectionState = {
    id: string
    url: string
    status: 'connecting' | 'connected' | 'disconnected' | 'error'
    event_count: number
  }

  type KeyValue = { key: string; value: string; enabled: boolean }

  let url = $state('https://httpbin.org/stream')
  let headers = $state<KeyValue[]>([])
  let events = $state<SseEvent[]>([])
  let status = $state<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
  let connectionId = $state<string | null>(null)
  let unlistenState: (() => void) | null = null
  let unlistenEvent: (() => void) | null = null
  let showHeaders = $state(false)

  const statusColors: Record<string, string> = {
    disconnected: 'text-text-muted',
    connecting: 'text-warning',
    connected: 'text-success',
    error: 'text-error',
  }

  async function connect() {
    if (!url.trim()) return
    connectionId = crypto.randomUUID()
    events = []
    status = 'connecting'

    unlistenState = await listen<SseConnectionState>(`sse-${connectionId}-state`, (event) => {
      status = event.payload.status as typeof status
    })

    unlistenEvent = await listen<SseEvent>(`sse-${connectionId}-event`, (event) => {
      events = [...events, event.payload]
    })

    try {
      await invoke('sse_connect', {
        id: connectionId,
        url,
        headers: headers.filter((h) => h.key.trim() !== ''),
      })
    } catch (e) {
      status = 'error'
      console.error('SSE connect failed:', e)
    }
  }

  async function disconnect() {
    if (!connectionId) return
    try {
      await invoke('sse_disconnect', { id: connectionId })
    } catch (e) {
      console.error('SSE disconnect failed:', e)
    }
    status = 'disconnected'
    if (unlistenState) { unlistenState(); unlistenState = null }
    if (unlistenEvent) { unlistenEvent(); unlistenEvent = null }
    connectionId = null
  }

  function clearEvents() {
    events = []
  }

  function addHeader() {
    headers = [...headers, { key: '', value: '', enabled: true }]
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index)
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString()
  }

  $effect(() => {
    return () => {
      if (unlistenState) unlistenState()
      if (unlistenEvent) unlistenEvent()
    }
  })
</script>

<div class="flex h-full flex-col">
  <!-- Connection Bar -->
  <div class="flex items-center gap-2 border-b border-border p-3">
    <input
      type="text"
      class="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="https://api.example.com/events"
      bind:value={url}
      disabled={status === 'connected' || status === 'connecting'}
      onkeydown={(e) => e.key === 'Enter' && status === 'disconnected' && connect()}
    />
    <span class="flex items-center gap-1.5 text-xs font-medium {statusColors[status]}">
      <span class="h-2 w-2 rounded-full {status === 'connected' ? 'bg-success' : status === 'connecting' ? 'bg-warning' : status === 'error' ? 'bg-error' : 'bg-text-muted'}"></span>
      {status}
    </span>
    {#if status === 'connected' || status === 'connecting'}
      <button
        class="flex items-center gap-2 rounded-md border border-error/30 bg-error/10 px-4 py-2 text-sm font-medium text-error transition-colors hover:bg-error/20"
        onclick={disconnect}
      >
        <Unplug class="h-4 w-4" />
        Disconnect
      </button>
    {:else}
      <button
        class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
        onclick={connect}
        disabled={!url.trim()}
      >
        <Plug class="h-4 w-4" />
        Connect
      </button>
    {/if}
  </div>

  <!-- Headers toggle -->
  <div class="border-b border-border px-3 py-1.5">
    <button
      class="text-xs text-text-muted hover:text-text"
      onclick={() => (showHeaders = !showHeaders)}
    >
      {showHeaders ? '▼' : '▶'} Headers {#if headers.filter((h) => h.key.trim()).length > 0}({headers.filter((h) => h.key.trim()).length}){/if}
    </button>
    {#if showHeaders}
      <div class="mt-2 space-y-2">
        {#each headers as header, i (i)}
          <div class="flex items-center gap-2">
            <input type="checkbox" bind:checked={header.enabled} class="accent-accent" />
            <input
              type="text"
              class="flex-1 rounded border border-border bg-surface px-2 py-1 text-sm font-mono outline-none focus:border-accent"
              placeholder="Header name"
              bind:value={header.key}
            />
            <input
              type="text"
              class="flex-1 rounded border border-border bg-surface px-2 py-1 text-sm font-mono outline-none focus:border-accent"
              placeholder="Value"
              bind:value={header.value}
            />
            <button class="text-text-muted hover:text-error" onclick={() => removeHeader(i)}>✕</button>
          </div>
        {/each}
        <button class="text-sm text-accent hover:text-accent-hover" onclick={addHeader}>+ Add Header</button>
      </div>
    {/if}
  </div>

  <!-- Events -->
  <div class="flex-1 overflow-y-auto p-3">
    {#if events.length === 0}
      <div class="flex h-full items-center justify-center text-text-muted">
        <p>{status === 'connected' ? 'Listening for events...' : 'Connect to an SSE endpoint to see events.'}</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each events as ev (ev.id)}
          <div class="rounded-md bg-surface p-3 text-sm">
            <div class="mb-1 flex items-center gap-2 text-xs">
              <span class="rounded bg-accent/10 px-1.5 py-0.5 font-medium text-accent">{ev.event_type}</span>
              <span class="text-text-muted">{formatTime(ev.timestamp)}</span>
              {#if ev.retry}
                <span class="text-text-muted">retry: {ev.retry}ms</span>
              {/if}
            </div>
            <pre class="whitespace-pre-wrap font-mono text-sm">{ev.data}</pre>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <div class="flex items-center justify-between border-t border-border p-3">
    <span class="text-xs text-text-muted">
      {events.length} event{events.length !== 1 ? 's' : ''} received
    </span>
    <button
      class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-text-muted transition-colors hover:text-text"
      onclick={clearEvents}
    >
      <Trash2 class="h-4 w-4" />
      Clear
    </button>
  </div>
</div>
