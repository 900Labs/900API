<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { FolderPlus, Trash2, ChevronRight, ChevronDown, Download, Upload, FilePlus } from '@lucide/svelte'
  import { loadRequestStore } from '../../lib/stores'

  type Collection = {
    id: string
    name: string
    description: string | null
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
    pre_request_script: string
    test_script: string
  }

  let collections = $state<Collection[]>([])
  let requests = $state<Record<string, SavedRequest[]>>({})
  let expanded = $state<Set<string>>(new Set())
  let loading = $state(true)
  let newCollectionName = $state('')
  let showNewCollection = $state(false)

  const methodColors: Record<string, string> = {
    GET: 'text-success',
    POST: 'text-warning',
    PUT: 'text-accent-hover',
    PATCH: 'text-accent-hover',
    DELETE: 'text-error',
    HEAD: 'text-text-muted',
    OPTIONS: 'text-text-muted',
  }

  async function loadCollections() {
    try {
      collections = await invoke<Collection[]>('list_collections')
      for (const col of collections) {
        await loadRequests(col.id)
      }
    } catch {
      // Backend not available
    } finally {
      loading = false
    }
  }

  async function loadRequests(collectionId: string) {
    try {
      const reqs = await invoke<SavedRequest[]>('list_requests', { collectionId })
      requests[collectionId] = reqs
      requests = requests
    } catch {
      // ignore
    }
  }

  async function createCollection() {
    if (!newCollectionName.trim()) return
    try {
      const col = await invoke<Collection>('create_collection', {
        name: newCollectionName.trim(),
      })
      collections = [...collections, col]
      requests[col.id] = []
      newCollectionName = ''
      showNewCollection = false
    } catch (e) {
      console.error('Failed to create collection:', e)
    }
  }

  async function deleteCollection(id: string) {
    try {
      await invoke('delete_collection', { id })
      collections = collections.filter((c) => c.id !== id)
      delete requests[id]
      requests = requests
    } catch (e) {
      console.error('Failed to delete collection:', e)
    }
  }

  async function deleteRequest(collectionId: string, requestId: string) {
    try {
      await invoke('delete_request', { id: requestId })
      await loadRequests(collectionId)
    } catch (e) {
      console.error('Failed to delete request:', e)
    }
  }

  function toggleExpand(id: string) {
    if (expanded.has(id)) {
      expanded.delete(id)
    } else {
      expanded.add(id)
    }
    expanded = expanded
  }

  function loadRequest(req: SavedRequest) {
    loadRequestStore.set({
      id: req.id,
      collectionId: req.collection_id,
      name: req.name,
      method: req.method,
      url: req.url,
      headers: req.headers,
      params: req.params,
      bodyType: req.body_type,
      body: req.body,
      authType: req.auth_type,
      authConfig: req.auth_config,
      preRequestScript: req.pre_request_script,
      testScript: req.test_script,
    })
  }

  async function exportCollection(col: Collection) {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog')
      const path = await save({
        defaultPath: `${col.name.replace(/\s+/g, '_')}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (path) {
        await invoke('export_collection', { collectionId: col.id, path })
      }
    } catch (e) {
      console.error('Export failed:', e)
    }
  }

  async function importCollection() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const path = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      })
      if (path && typeof path === 'string') {
        // Try 900API format first, then Postman
        try {
          await invoke('import_collection_file', { path })
        } catch {
          await invoke('import_postman', { path })
        }
        await loadCollections()
      }
    } catch (e) {
      console.error('Import failed:', e)
    }
  }

  $effect(() => {
    loadCollections()
  })
</script>

<div class="flex h-full flex-col">
  <div class="flex items-center justify-between border-b border-border p-3">
    <h2 class="text-sm font-medium">Collections</h2>
    <div class="flex gap-1">
      <button
        class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text"
        title="Import Collection"
        onclick={importCollection}
      >
        <Upload class="h-4 w-4" />
      </button>
      <button
        class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text"
        title="New Collection"
        onclick={() => (showNewCollection = !showNewCollection)}
      >
        <FolderPlus class="h-4 w-4" />
      </button>
    </div>
  </div>

  {#if showNewCollection}
    <div class="border-b border-border p-2">
      <input
        type="text"
        class="w-full rounded border border-border bg-surface px-2 py-1.5 text-sm outline-none focus:border-accent"
        placeholder="Collection name"
        bind:value={newCollectionName}
        onkeydown={(e) => e.key === 'Enter' && createCollection()}
      />
    </div>
  {/if}

  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <p class="p-4 text-sm text-text-muted">Loading...</p>
    {:else if collections.length === 0}
      <p class="p-4 text-sm text-text-muted">No collections yet. Create one to get started.</p>
    {:else}
      {#each collections as col (col.id)}
        <div class="mb-1">
          <div class="group flex items-center gap-1 rounded-md px-2 py-1.5 text-sm hover:bg-surface-hover">
            <button onclick={() => toggleExpand(col.id)} class="text-text-muted">
              {#if expanded.has(col.id)}
                <ChevronDown class="h-3.5 w-3.5" />
              {:else}
                <ChevronRight class="h-3.5 w-3.5" />
              {/if}
            </button>
            <span class="flex-1 truncate">{col.name}</span>
            <button
              class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-accent"
              onclick={() => exportCollection(col)}
              title="Export collection"
            >
              <Download class="h-3.5 w-3.5" />
            </button>
            <button
              class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-error"
              onclick={() => deleteCollection(col.id)}
              title="Delete collection"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </button>
          </div>
          {#if expanded.has(col.id)}
            <div class="ml-5 border-l border-border pl-2">
              {#each requests[col.id] || [] as req (req.id)}
                <div
                  class="group flex items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-surface-hover cursor-pointer"
                  role="button"
                  tabindex="0"
                  onclick={() => loadRequest(req)}
                  onkeydown={(e) => e.key === 'Enter' && loadRequest(req)}
                >
                  <span class="text-xs font-medium {methodColors[req.method] || 'text-text-muted'}">{req.method}</span>
                  <span class="flex-1 truncate">{req.name}</span>
                  <button
                    class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-error"
                    onclick={(e) => { e.stopPropagation(); deleteRequest(col.id, req.id) }}
                    title="Delete request"
                  >
                    <Trash2 class="h-3 w-3" />
                  </button>
                </div>
              {/each}
              {#if (requests[col.id] || []).length === 0}
                <p class="px-2 py-1 text-xs text-text-muted">No requests</p>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
