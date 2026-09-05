<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { save } from '@tauri-apps/plugin-dialog'
  import { BookOpen, Download, FileCode, ChevronDown, ChevronRight } from '@lucide/svelte'

  type KeyValue = { key: string; value: string; enabled: boolean }

  type EndpointDoc = {
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

  type ApiDoc = {
    collection_name: string
    collection_description: string | null
    endpoints: EndpointDoc[]
  }

  let docs = $state<ApiDoc[]>([])
  let selectedDocIndex = $state(0)
  let expandedEndpoints = $state<Set<string>>(new Set())
  let loading = $state(false)
  let error = $state<string | null>(null)

  async function loadDocs() {
    loading = true
    error = null
    try {
      docs = await invoke<ApiDoc[]>('generate_all_docs')
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function toggleEndpoint(name: string) {
    const next = new Set(expandedEndpoints)
    if (next.has(name)) {
      next.delete(name)
    } else {
      next.add(name)
    }
    expandedEndpoints = next
  }

  async function exportMarkdown() {
    if (docs.length === 0) return
    error = null
    try {
      const doc = docs[selectedDocIndex]
      const markdown = await invoke<string>('docs_to_markdown', { doc })
      const filePath = await save({
        defaultPath: `${doc.collection_name.replace(/\s+/g, '_')}_api.md`,
        filters: [{ name: 'Markdown', extensions: ['md'] }],
      })
      if (filePath) {
        await invoke('write_text_file', { path: filePath, content: markdown })
      }
    } catch (e) {
      error = String(e)
    }
  }

  async function exportHtml() {
    if (docs.length === 0) return
    error = null
    try {
      const doc = docs[selectedDocIndex]
      const html = await invoke<string>('docs_to_html', { doc })
      const filePath = await save({
        defaultPath: `${doc.collection_name.replace(/\s+/g, '_')}_api.html`,
        filters: [{ name: 'HTML', extensions: ['html'] }],
      })
      if (filePath) {
        await invoke('write_text_file', { path: filePath, content: html })
      }
    } catch (e) {
      error = String(e)
    }
  }

  const methodColors: Record<string, string> = {
    GET: 'bg-blue-500/10 text-blue-400',
    POST: 'bg-green-500/10 text-green-400',
    PUT: 'bg-orange-500/10 text-orange-400',
    PATCH: 'bg-teal-500/10 text-teal-400',
    DELETE: 'bg-red-500/10 text-red-400',
    HEAD: 'bg-purple-500/10 text-purple-400',
    OPTIONS: 'bg-indigo-500/10 text-indigo-400',
  }

  loadDocs()
</script>

<div class="flex h-full flex-col">
  <!-- Toolbar -->
  <div class="flex items-center gap-3 border-b border-border p-3">
    <BookOpen class="h-5 w-5 text-text-muted" />
    <span class="text-sm font-medium">API Documentation</span>
    <span class="text-xs text-text-muted">{docs.length} collection(s)</span>
    <div class="flex-1"></div>
    {#if docs.length > 0}
      <button
        class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-2 text-sm transition-colors hover:bg-surface-hover"
        onclick={exportMarkdown}
      >
        <Download class="h-4 w-4" />
        Export MD
      </button>
      <button
        class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-2 text-sm transition-colors hover:bg-surface-hover"
        onclick={exportHtml}
      >
        <Download class="h-4 w-4" />
        Export HTML
      </button>
    {/if}
    <button
      class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-2 text-sm transition-colors hover:bg-surface-hover"
      onclick={loadDocs}
      disabled={loading}
    >
      <FileCode class="h-4 w-4" />
      Refresh
    </button>
  </div>

  {#if error}
    <div class="border-b border-border bg-error/10 p-3 text-sm text-error">{error}</div>
  {/if}

  {#if docs.length === 0 && !loading}
    <div class="flex flex-1 items-center justify-center text-text-muted">
      <div class="text-center">
        <BookOpen class="mx-auto mb-3 h-12 w-12 opacity-30" />
        <p class="text-sm">No collections found.</p>
        <p class="mt-1 text-xs">Create collections and save requests to generate API documentation.</p>
      </div>
    </div>
  {:else if docs.length > 0}
    <div class="flex flex-1 overflow-hidden">
      <!-- Collection List -->
      {#if docs.length > 1}
        <div class="w-56 border-r border-border overflow-y-auto">
          {#each docs as doc, i (doc.collection_name)}
            <button
              class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors {selectedDocIndex === i ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}"
              onclick={() => (selectedDocIndex = i)}
            >
              <FileCode class="h-4 w-4 flex-shrink-0" />
              <span class="flex-1 truncate">{doc.collection_name}</span>
              <span class="text-xs text-text-muted">{doc.endpoints.length}</span>
            </button>
          {/each}
        </div>
      {/if}

      <!-- Documentation View -->
      <div class="flex-1 overflow-y-auto p-6">
        {#if docs.length > 0}
          {@const doc = docs[selectedDocIndex]}
          <h1 class="mb-2 text-2xl font-bold">{doc.collection_name}</h1>
          {#if doc.collection_description}
            <p class="mb-6 text-sm text-text-muted">{doc.collection_description}</p>
          {/if}

          <h2 class="mb-4 border-b border-border pb-2 text-lg font-semibold">Endpoints</h2>

          <div class="space-y-3">
            {#each doc.endpoints as endpoint (endpoint.name)}
            <div class="rounded-lg border border-border bg-surface">
              <button
                class="flex w-full items-center gap-3 p-3 text-left"
                onclick={() => toggleEndpoint(endpoint.name)}
              >
                {#if expandedEndpoints.has(endpoint.name)}
                  <ChevronDown class="h-4 w-4 text-text-muted" />
                {:else}
                  <ChevronRight class="h-4 w-4 text-text-muted" />
                {/if}
                <span class="rounded px-2 py-0.5 text-xs font-bold {methodColors[endpoint.method] ?? 'bg-gray-500/10 text-gray-400'}">
                  {endpoint.method}
                </span>
                <span class="flex-1 truncate font-mono text-sm">{endpoint.url}</span>
                <span class="text-xs text-text-muted">{endpoint.name}</span>
              </button>

              {#if expandedEndpoints.has(endpoint.name)}
                <div class="border-t border-border p-4">
                  <!-- Headers -->
                  {#if endpoint.headers.length > 0}
                    <h3 class="mb-2 text-xs font-medium uppercase text-text-muted">Headers</h3>
                    <div class="mb-4 overflow-hidden rounded border border-border">
                      <table class="w-full text-xs">
                        <thead class="bg-bg">
                          <tr>
                            <th class="px-3 py-1.5 text-left font-medium text-text-muted">Key</th>
                            <th class="px-3 py-1.5 text-left font-medium text-text-muted">Value</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each endpoint.headers as h}
                            <tr class="border-t border-border">
                              <td class="px-3 py-1.5 font-mono">{h.key}</td>
                              <td class="px-3 py-1.5 font-mono">{h.value}</td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  {/if}

                  <!-- Params -->
                  {#if endpoint.params.length > 0}
                    <h3 class="mb-2 text-xs font-medium uppercase text-text-muted">Query Parameters</h3>
                    <div class="mb-4 overflow-hidden rounded border border-border">
                      <table class="w-full text-xs">
                        <thead class="bg-bg">
                          <tr>
                            <th class="px-3 py-1.5 text-left font-medium text-text-muted">Key</th>
                            <th class="px-3 py-1.5 text-left font-medium text-text-muted">Value</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each endpoint.params as p}
                            <tr class="border-t border-border">
                              <td class="px-3 py-1.5 font-mono">{p.key}</td>
                              <td class="px-3 py-1.5 font-mono">{p.value}</td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  {/if}

                  <!-- Body -->
                  {#if endpoint.body_type !== 'none' && endpoint.body}
                    <h3 class="mb-2 text-xs font-medium uppercase text-text-muted">Body ({endpoint.body_type})</h3>
                    <pre class="mb-4 overflow-x-auto rounded border border-border bg-bg p-3 text-xs font-mono">{endpoint.body}</pre>
                  {/if}

                  <!-- Auth -->
                  <h3 class="mb-1 text-xs font-medium uppercase text-text-muted">Authentication</h3>
                  <p class="text-sm">{endpoint.auth_type}</p>
                </div>
              {/if}
            </div>
          {/each}
        </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
