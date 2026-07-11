<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { open, save } from '@tauri-apps/plugin-dialog'
  import {
    ChevronDown,
    ChevronRight,
    Download,
    FilePlus,
    FolderPlus,
    MoreHorizontal,
    Pencil,
    Trash2,
    Upload,
  } from '@lucide/svelte'
  import { loadRequestStore } from '../../lib/stores'

  type Collection = {
    id: string
    name: string
    description: string | null
    parent_id: string | null
    sort_order: number
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
    settings: string
  }

  type VisibleCollection = Collection & { depth: number }
  type DragItem =
    | { type: 'collection'; id: string }
    | { type: 'request'; id: string; collectionId: string }
    | null
  type ContextMenu =
    | { type: 'collection'; x: number; y: number; collection: Collection }
    | { type: 'request'; x: number; y: number; collectionId: string; request: SavedRequest }
    | null

  let { embedded = false }: { embedded?: boolean } = $props()

  let collections = $state<Collection[]>([])
  let requests = $state<Record<string, SavedRequest[]>>({})
  let expanded = $state<Set<string>>(new Set())
  let loading = $state(true)
  let newCollectionName = $state('')
  let newCollectionParentId = $state<string | null>(null)
  let showNewCollection = $state(false)
  let contextMenu = $state<ContextMenu>(null)
  let dragItem = $state<DragItem>(null)
  let renameTarget = $state<ContextMenu>(null)
  let renameValue = $state('')
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

  function childrenOf(parentId: string | null): Collection[] {
    return collections
      .filter((collection) => (collection.parent_id ?? null) === parentId)
      .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name))
  }

  function flattenCollections(parentId: string | null = null, depth = 0): VisibleCollection[] {
    const visible: VisibleCollection[] = []
    for (const collection of childrenOf(parentId)) {
      visible.push({ ...collection, depth })
      if (expanded.has(collection.id)) {
        visible.push(...flattenCollections(collection.id, depth + 1))
      }
    }
    return visible
  }

  let visibleCollections = $derived(flattenCollections())

  function hasChildren(collectionId: string): boolean {
    return collections.some((collection) => collection.parent_id === collectionId)
  }

  async function loadCollections() {
    loading = true
    error = null
    try {
      collections = await invoke<Collection[]>('list_collections')
      const nextRequests: Record<string, SavedRequest[]> = {}
      for (const col of collections) {
        nextRequests[col.id] = await invoke<SavedRequest[]>('list_requests', { collectionId: col.id })
      }
      requests = nextRequests
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function startNewCollection(parentId: string | null = null) {
    newCollectionParentId = parentId
    newCollectionName = ''
    showNewCollection = true
    contextMenu = null
    if (parentId) {
      expanded.add(parentId)
      expanded = expanded
    }
  }

  async function createCollection() {
    if (!newCollectionName.trim()) return
    try {
      await invoke<Collection>('create_collection', {
        name: newCollectionName.trim(),
        parentId: newCollectionParentId,
      })
      newCollectionName = ''
      showNewCollection = false
      await loadCollections()
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
    }
  }

  async function deleteCollection(id: string) {
    try {
      await invoke('delete_collection', { id })
      collections = collections.filter((collection) => collection.id !== id)
      delete requests[id]
      requests = requests
      contextMenu = null
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
    }
  }

  async function deleteRequest(collectionId: string, requestId: string) {
    try {
      await invoke('delete_request', { id: requestId })
      await loadRequests(collectionId)
      contextMenu = null
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
    }
  }

  async function loadRequests(collectionId: string) {
    try {
      requests[collectionId] = await invoke<SavedRequest[]>('list_requests', { collectionId })
      requests = requests
    } catch (e) {
      error = String(e)
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
      settings: req.settings || '{}',
      preRequestScript: req.pre_request_script,
      testScript: req.test_script,
    })
  }

  function newRequest(collectionId: string) {
    window.dispatchEvent(new CustomEvent('900api:new-request', { detail: { collectionId } }))
    contextMenu = null
  }

  async function duplicateRequest(collectionId: string, req: SavedRequest) {
    try {
      await invoke('create_request', {
        collectionId,
        name: `${req.name} Copy`,
        method: req.method,
        url: req.url,
        headers: req.headers,
        params: req.params,
        bodyType: req.body_type,
        body: req.body,
        authType: req.auth_type,
        authConfig: req.auth_config,
        settings: req.settings || '{}',
        preRequestScript: req.pre_request_script,
        testScript: req.test_script,
      })
      await loadRequests(collectionId)
      contextMenu = null
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
    }
  }

  function beginRename(target: ContextMenu) {
    if (!target) return
    renameTarget = target
    renameValue = target.type === 'collection' ? target.collection.name : target.request.name
    contextMenu = null
  }

  function newRequestFromContext() {
    if (contextMenu?.type !== 'collection') return
    newRequest(contextMenu.collection.id)
  }

  function newFolderFromContext() {
    if (contextMenu?.type !== 'collection') return
    startNewCollection(contextMenu.collection.id)
  }

  function exportCollectionFromContext() {
    if (contextMenu?.type !== 'collection') return
    exportCollection(contextMenu.collection)
  }

  function exportOpenApiFromContext() {
    if (contextMenu?.type !== 'collection') return
    exportOpenApi(contextMenu.collection)
  }

  function deleteCollectionFromContext() {
    if (contextMenu?.type !== 'collection') return
    deleteCollection(contextMenu.collection.id)
  }

  function openRequestFromContext() {
    if (contextMenu?.type !== 'request') return
    loadRequest(contextMenu.request)
    contextMenu = null
  }

  function duplicateRequestFromContext() {
    if (contextMenu?.type !== 'request') return
    duplicateRequest(contextMenu.collectionId, contextMenu.request)
  }

  function deleteRequestFromContext() {
    if (contextMenu?.type !== 'request') return
    deleteRequest(contextMenu.collectionId, contextMenu.request.id)
  }

  async function commitRename() {
    if (!renameTarget || !renameValue.trim()) return
    try {
      if (renameTarget.type === 'collection') {
        await invoke('update_collection', {
          id: renameTarget.collection.id,
          name: renameValue.trim(),
          description: renameTarget.collection.description,
        })
      } else {
        const req = renameTarget.request
        await invoke('update_request', {
          id: req.id,
          name: renameValue.trim(),
          method: req.method,
          url: req.url,
          headers: req.headers,
          params: req.params,
          bodyType: req.body_type,
          body: req.body,
          authType: req.auth_type,
          authConfig: req.auth_config,
          settings: req.settings || '{}',
          preRequestScript: req.pre_request_script,
          testScript: req.test_script,
        })
      }
      renameTarget = null
      renameValue = ''
      await loadCollections()
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
    }
  }

  async function exportCollection(col: Collection) {
    try {
      const path = await save({
        defaultPath: `${col.name.replace(/\s+/g, '_')}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (path) {
        await invoke('export_collection', { collectionId: col.id, path })
      }
      contextMenu = null
    } catch (e) {
      error = String(e)
    }
  }

  async function exportOpenApi(col: Collection) {
    try {
      const path = await save({
        defaultPath: `${col.name.replace(/\s+/g, '_')}.openapi.json`,
        filters: [{ name: 'OpenAPI JSON', extensions: ['json'] }],
      })
      if (path) {
        await invoke('export_openapi', { collectionId: col.id, path })
      }
      contextMenu = null
    } catch (e) {
      error = String(e)
    }
  }

  async function importCollection() {
    try {
      const path = await open({
        filters: [{ name: 'API collections', extensions: ['json', 'yaml', 'yml'] }],
        multiple: false,
      })
      if (path && typeof path === 'string') {
        try {
          await invoke('import_collection_file', { path })
        } catch (nativeError) {
          try {
            await invoke('import_postman', { path })
          } catch (postmanError) {
            try {
              await invoke('import_openapi', { path })
            } catch (openApiError) {
              throw new Error(
                `Could not import file as 900API JSON, Postman, or OpenAPI. Native: ${nativeError}; Postman: ${postmanError}; OpenAPI: ${openApiError}`
              )
            }
          }
        }
        await loadCollections()
        window.dispatchEvent(new CustomEvent('900api:collections-changed'))
      }
    } catch (e) {
      error = String(e)
    }
  }

  function openCollectionMenu(event: MouseEvent, collection: Collection) {
    event.preventDefault()
    contextMenu = { type: 'collection', x: event.clientX, y: event.clientY, collection }
  }

  function openRequestMenu(event: MouseEvent, collectionId: string, request: SavedRequest) {
    event.preventDefault()
    contextMenu = { type: 'request', x: event.clientX, y: event.clientY, collectionId, request }
  }

  function startDragCollection(collection: Collection) {
    dragItem = { type: 'collection', id: collection.id }
  }

  function startDragRequest(collectionId: string, request: SavedRequest) {
    dragItem = { type: 'request', id: request.id, collectionId }
  }

  async function dropOnCollection(event: DragEvent, targetId: string) {
    event.preventDefault()
    event.stopPropagation()
    if (!dragItem) return
    try {
      if (dragItem.type === 'collection') {
        await invoke('move_collection', { id: dragItem.id, parentId: targetId })
      } else if (dragItem.collectionId !== targetId) {
        await invoke('move_request', { id: dragItem.id, collectionId: targetId })
      }
      dragItem = null
      expanded.add(targetId)
      expanded = expanded
      await loadCollections()
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
      dragItem = null
    }
  }

  async function dropOnRoot(event: DragEvent) {
    event.preventDefault()
    if (!dragItem || dragItem.type !== 'collection') return
    try {
      await invoke('move_collection', { id: dragItem.id, parentId: null })
      dragItem = null
      await loadCollections()
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
      dragItem = null
    }
  }

  $effect(() => {
    loadCollections()

    const refresh = () => loadCollections()
    const importHandler = () => importCollection()
    const newCollectionHandler = () => startNewCollection(null)
    const closeMenu = () => (contextMenu = null)
    window.addEventListener('900api:collections-changed', refresh)
    window.addEventListener('900api:import-collection', importHandler)
    window.addEventListener('900api:new-collection', newCollectionHandler)
    window.addEventListener('click', closeMenu)
    return () => {
      window.removeEventListener('900api:collections-changed', refresh)
      window.removeEventListener('900api:import-collection', importHandler)
      window.removeEventListener('900api:new-collection', newCollectionHandler)
      window.removeEventListener('click', closeMenu)
    }
  })
</script>

<div class="flex h-full flex-col {embedded ? '' : 'bg-bg'}">
  <div class="flex items-center justify-between border-b border-border p-3">
    <h2 class="text-sm font-medium">Collections</h2>
    <div class="flex gap-1">
      <button
        class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text"
        title="Import collection"
        onclick={importCollection}
      >
        <Upload class="h-4 w-4" />
      </button>
      <button
        class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text"
        title="New collection"
        onclick={() => startNewCollection(null)}
      >
        <FolderPlus class="h-4 w-4" />
      </button>
    </div>
  </div>

  {#if showNewCollection}
    <div class="border-b border-border p-2">
      <div class="mb-1 text-xs text-text-muted">
        {newCollectionParentId ? `New folder in ${collections.find((collection) => collection.id === newCollectionParentId)?.name ?? 'collection'}` : 'New collection'}
      </div>
      <input
        type="text"
        class="w-full rounded border border-border bg-surface px-2 py-1.5 text-sm outline-none focus:border-accent"
        placeholder="Name"
        bind:value={newCollectionName}
        onkeydown={(e) => e.key === 'Enter' && createCollection()}
      />
    </div>
  {/if}

  {#if error}
    <div class="border-b border-border bg-error/10 px-3 py-2 text-xs text-error">{error}</div>
  {/if}

  <div
    class="flex-1 overflow-y-auto p-2"
    role="list"
    ondragover={(event) => dragItem?.type === 'collection' && event.preventDefault()}
    ondrop={dropOnRoot}
  >
    {#if loading}
      <p class="p-4 text-sm text-text-muted">Loading...</p>
    {:else if collections.length === 0}
      <p class="p-4 text-sm text-text-muted">No collections yet. Create one to get started.</p>
    {:else}
      {#each visibleCollections as col (col.id)}
        <div class="mb-1">
          <div
            class="group flex items-center gap-1 rounded-md py-1.5 pr-1 text-sm hover:bg-surface-hover"
            style={`padding-left: ${8 + col.depth * 16}px`}
            role="button"
            tabindex="0"
            draggable="true"
            ondragstart={() => startDragCollection(col)}
            ondragend={() => (dragItem = null)}
            ondragover={(event) => event.preventDefault()}
            ondrop={(event) => dropOnCollection(event, col.id)}
            oncontextmenu={(event) => openCollectionMenu(event, col)}
          >
            <button onclick={() => toggleExpand(col.id)} class="text-text-muted" title="Expand collection">
              {#if expanded.has(col.id)}
                <ChevronDown class="h-3.5 w-3.5" />
              {:else}
                <ChevronRight class="h-3.5 w-3.5 {hasChildren(col.id) || (requests[col.id] || []).length > 0 ? '' : 'opacity-30'}" />
              {/if}
            </button>
            <span class="flex-1 truncate">{col.name}</span>
            <button
              class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-accent"
              onclick={(event) => { event.stopPropagation(); newRequest(col.id) }}
              title="New request"
            >
              <FilePlus class="h-3.5 w-3.5" />
            </button>
            <button
              class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-text"
              onclick={(event) => { event.stopPropagation(); openCollectionMenu(event, col) }}
              title="Collection actions"
            >
              <MoreHorizontal class="h-3.5 w-3.5" />
            </button>
          </div>

          {#if expanded.has(col.id)}
            {#each requests[col.id] || [] as req (req.id)}
              <div
                class="group flex cursor-pointer items-center gap-2 rounded-md py-1 pr-1 text-sm hover:bg-surface-hover"
                style={`padding-left: ${32 + col.depth * 16}px`}
                role="button"
                tabindex="0"
                draggable="true"
                ondragstart={() => startDragRequest(col.id, req)}
                ondragend={() => (dragItem = null)}
                onclick={() => loadRequest(req)}
                onkeydown={(e) => e.key === 'Enter' && loadRequest(req)}
                oncontextmenu={(event) => openRequestMenu(event, col.id, req)}
              >
                <span class="w-12 shrink-0 text-xs font-medium {methodColors[req.method] || 'text-text-muted'}">{req.method}</span>
                <span class="min-w-0 flex-1 truncate">{req.name}</span>
                <button
                  class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-text"
                  onclick={(event) => { event.stopPropagation(); openRequestMenu(event, col.id, req) }}
                  title="Request actions"
                >
                  <MoreHorizontal class="h-3 w-3" />
                </button>
              </div>
            {/each}
            {#if (requests[col.id] || []).length === 0 && !hasChildren(col.id)}
              <p class="py-1 pr-2 text-xs text-text-muted" style={`padding-left: ${32 + col.depth * 16}px`}>No requests</p>
            {/if}
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

{#if contextMenu}
  <div
    class="fixed z-50 min-w-48 rounded-md border border-border bg-surface p-1 shadow-xl"
    style={`left: ${contextMenu.x}px; top: ${contextMenu.y}px`}
    role="menu"
  >
    {#if contextMenu.type === 'collection'}
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={newRequestFromContext}>
        <FilePlus class="h-3.5 w-3.5" />
        New request
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={newFolderFromContext}>
        <FolderPlus class="h-3.5 w-3.5" />
        New folder
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={() => beginRename(contextMenu)}>
        <Pencil class="h-3.5 w-3.5" />
        Rename
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={exportCollectionFromContext}>
        <Download class="h-3.5 w-3.5" />
        Export 900API JSON
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={exportOpenApiFromContext}>
        <Download class="h-3.5 w-3.5" />
        Export OpenAPI
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-error hover:bg-surface-hover" onclick={deleteCollectionFromContext}>
        <Trash2 class="h-3.5 w-3.5" />
        Delete
      </button>
    {:else}
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={openRequestFromContext}>
        <FilePlus class="h-3.5 w-3.5" />
        Open
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={duplicateRequestFromContext}>
        <FilePlus class="h-3.5 w-3.5" />
        Duplicate
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text" onclick={() => beginRename(contextMenu)}>
        <Pencil class="h-3.5 w-3.5" />
        Rename
      </button>
      <button class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-error hover:bg-surface-hover" onclick={deleteRequestFromContext}>
        <Trash2 class="h-3.5 w-3.5" />
        Delete
      </button>
    {/if}
  </div>
{/if}

{#if renameTarget}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="button" tabindex="0" onclick={() => (renameTarget = null)} onkeydown={(e) => e.key === 'Escape' && (renameTarget = null)}>
    <div class="w-96 rounded-lg border border-border bg-bg p-5 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <h3 class="mb-3 text-base font-medium">Rename {renameTarget.type === 'collection' ? 'collection' : 'request'}</h3>
      <input
        class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
        bind:value={renameValue}
        onkeydown={(event) => event.key === 'Enter' && commitRename()}
      />
      <div class="mt-4 flex justify-end gap-2">
        <button class="rounded border border-border bg-surface px-3 py-1.5 text-sm text-text-muted hover:text-text" onclick={() => (renameTarget = null)}>Cancel</button>
        <button class="rounded bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50" onclick={commitRename} disabled={!renameValue.trim()}>Rename</button>
      </div>
    </div>
  </div>
{/if}
