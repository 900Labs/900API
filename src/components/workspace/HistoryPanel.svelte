<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { RefreshCw, Trash2 } from '@lucide/svelte'

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

  let entries = $state<HistoryEntry[]>([])
  let query = $state('')
  let loading = $state(false)
  let error = $state<string | null>(null)

  const methodColors: Record<string, string> = {
    GET: 'text-success',
    POST: 'text-warning',
    PUT: 'text-accent-hover',
    PATCH: 'text-accent-hover',
    DELETE: 'text-error',
    HEAD: 'text-text-muted',
    OPTIONS: 'text-text-muted',
  }

  let filteredEntries = $derived(
    entries.filter((entry) => `${entry.method} ${entry.url} ${entry.status}`.toLowerCase().includes(query.toLowerCase())),
  )

  async function loadHistory() {
    loading = true
    error = null
    try {
      entries = await invoke<HistoryEntry[]>('list_history', { limit: 100 })
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function clearHistory() {
    try {
      await invoke('clear_history')
      entries = []
    } catch (e) {
      error = String(e)
    }
  }

  function openEntry(entry: HistoryEntry) {
    window.dispatchEvent(new CustomEvent('900api:open-history', { detail: entry }))
  }

  function formatDate(value: string): string {
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value
    return date.toLocaleString()
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  $effect(() => {
    loadHistory()

    const refresh = () => loadHistory()
    window.addEventListener('900api:history-changed', refresh)
    return () => window.removeEventListener('900api:history-changed', refresh)
  })
</script>

<div class="flex h-full flex-col">
  <div class="flex items-center justify-between border-b border-border p-2">
    <h2 class="text-sm font-medium">History</h2>
    <div class="flex gap-1">
      <button class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text" title="Refresh" onclick={loadHistory}>
        <RefreshCw class="h-3.5 w-3.5" />
      </button>
      <button class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-error" title="Clear history" onclick={clearHistory}>
        <Trash2 class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>

  <div class="border-b border-border p-2">
    <input
      class="w-full rounded border border-border bg-surface px-2 py-1.5 text-sm outline-none focus:border-accent"
      placeholder="Search history"
      bind:value={query}
    />
  </div>

  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <p class="p-3 text-sm text-text-muted">Loading...</p>
    {:else if error}
      <p class="p-3 text-sm text-error">{error}</p>
    {:else if filteredEntries.length === 0}
      <p class="p-3 text-sm text-text-muted">No history entries.</p>
    {:else}
      {#each filteredEntries as entry (entry.id)}
        <button
          class="mb-1 flex w-full flex-col rounded-md px-2 py-1.5 text-left text-sm hover:bg-surface-hover"
          onclick={() => openEntry(entry)}
        >
          <span class="flex w-full items-center gap-2">
            <span class="text-xs font-semibold {methodColors[entry.method] ?? 'text-text-muted'}">{entry.method}</span>
            <span class="truncate font-mono text-xs text-text">{entry.url}</span>
          </span>
          <span class="mt-1 flex w-full items-center gap-2 text-xs text-text-muted">
            <span class={entry.status < 300 ? 'text-success' : entry.status < 400 ? 'text-warning' : 'text-error'}>{entry.status}</span>
            <span>{entry.time_ms} ms</span>
            <span>{formatSize(entry.size_bytes)}</span>
            <span class="ml-auto truncate">{formatDate(entry.created_at)}</span>
          </span>
        </button>
      {/each}
    {/if}
  </div>
</div>
