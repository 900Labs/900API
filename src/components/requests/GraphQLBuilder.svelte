<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { Send, LoaderCircle } from '@lucide/svelte'
  import { activeEnvironmentStore } from '../../lib/stores'

  type KeyValue = { key: string; value: string; enabled: boolean }

  type AuthType = 'none' | 'basic' | 'bearer' | 'api_key'

  type AuthConfig = {
    auth_type: AuthType
    username: string
    password: string
    token: string
    api_key: string
    api_key_name: string
    api_key_in: string
  }

  type ResponseData = {
    status: number
    status_text: string
    headers: Record<string, string>
    body: string
    time_ms: number
    size_bytes: number
  } | null

  let url = $state('https://countries.trevorblades.com')
  let query = $state('query {\n  countries {\n    code\n    name\n  }\n}')
  let variables = $state('{}')
  let operationName = $state('')
  let headers = $state<KeyValue[]>([])
  let authType = $state<AuthType>('none')
  let authToken = $state('')
  let activeTab = $state<'query' | 'variables' | 'headers' | 'auth'>('query')
  let responseTab = $state<'body' | 'headers'>('body')
  let response = $state<ResponseData>(null)
  let loading = $state(false)
  let error = $state<string | null>(null)
  let activeEnvVars = $state<{ key: string; value: string; enabled: boolean }[]>([])

  function addHeader() {
    headers = [...headers, { key: '', value: '', enabled: true }]
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index)
  }

  async function sendGraphQL() {
    loading = true
    error = null
    response = null

    try {
      const auth: AuthConfig = {
        auth_type: authType,
        username: '',
        password: '',
        token: authToken,
        api_key: '',
        api_key_name: '',
        api_key_in: 'header',
      }

      const result = await invoke<ResponseData>('send_graphql', {
        url,
        query,
        variables,
        operationName: operationName || null,
        headers,
        auth,
        environmentVariables: activeEnvVars.length > 0 ? activeEnvVars : undefined,
      })
      response = result
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function formatJson(str: string): string {
    try {
      return JSON.stringify(JSON.parse(str), null, 2)
    } catch {
      return str
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  function copyResponse() {
    if (!response) return
    navigator.clipboard.writeText(response.body)
  }

  const unsubEnv = activeEnvironmentStore.subscribe((env) => {
    activeEnvVars = env?.variables || []
  })

  $effect(() => {
    return () => unsubEnv()
  })
</script>

<div class="flex h-full flex-col">
  <!-- URL Bar -->
  <div class="flex items-center gap-2 border-b border-border p-3">
    <span class="rounded-md bg-surface px-3 py-2 text-sm font-medium text-accent-hover">POST</span>
    <input
      type="text"
      class="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="https://api.example.com/graphql"
      bind:value={url}
      onkeydown={(e) => e.key === 'Enter' && sendGraphQL()}
    />
    <button
      class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
      onclick={sendGraphQL}
      disabled={loading || !url.trim() || !query.trim()}
    >
      {#if loading}
        <LoaderCircle class="h-4 w-4 animate-spin" />
      {:else}
        <Send class="h-4 w-4" />
      {/if}
      Send
    </button>
  </div>

  <!-- Tabs -->
  <div class="flex border-b border-border">
    {#each ['query', 'variables', 'headers', 'auth'] as tab (tab)}
      <button
        class="px-4 py-2 text-sm transition-colors {activeTab === tab ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
        onclick={() => (activeTab = tab as typeof activeTab)}
      >
        {tab.charAt(0).toUpperCase() + tab.slice(1)}
        {#if tab === 'headers' && headers.filter((h) => h.key.trim()).length > 0}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{headers.filter((h) => h.key.trim()).length}</span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Tab Content -->
  <div class="flex-1 overflow-y-auto p-3">
    {#if activeTab === 'query'}
      <textarea
        class="h-96 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
        placeholder={'query {\n  field {\n    subField\n  }\n}'}
        bind:value={query}
      ></textarea>
      <div class="mt-2">
        <input
          type="text"
          class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
          placeholder="Operation name (optional)"
          bind:value={operationName}
        />
      </div>
    {:else if activeTab === 'variables'}
      <textarea
        class="h-96 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
        placeholder={'{\n  "key": "value"\n}'}
        bind:value={variables}
      ></textarea>
    {:else if activeTab === 'headers'}
      <div class="space-y-2">
        {#each headers as header, i (i)}
          <div class="flex items-center gap-2">
            <input type="checkbox" bind:checked={header.enabled} class="accent-accent" />
            <input
              type="text"
              class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
              placeholder="Header name"
              bind:value={header.key}
            />
            <input
              type="text"
              class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
              placeholder="Value"
              bind:value={header.value}
            />
            <button class="text-text-muted hover:text-error" onclick={() => removeHeader(i)}>✕</button>
          </div>
        {/each}
        <button class="text-sm text-accent hover:text-accent-hover" onclick={addHeader}>+ Add Header</button>
      </div>
    {:else if activeTab === 'auth'}
      <div class="space-y-3">
        <div class="flex gap-2">
          {#each ['none', 'bearer'] as at (at)}
            <button
              class="rounded px-3 py-1 text-xs transition-colors {authType === at ? 'bg-accent text-white' : 'bg-surface text-text-muted hover:text-text'}"
              onclick={() => (authType = at as AuthType)}
            >
              {at.charAt(0).toUpperCase() + at.slice(1)}
            </button>
          {/each}
        </div>
        {#if authType === 'bearer'}
          <input
            type="password"
            class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
            placeholder="Bearer token"
            bind:value={authToken}
          />
        {/if}
      </div>
    {/if}
  </div>

  <!-- Response Area -->
  {#if error}
    <div class="border-t border-border bg-error/10 p-3 text-sm text-error">
      {error}
    </div>
  {:else if response}
    <div class="flex border-t border-border">
      <div class="flex items-center gap-3 px-4 py-2 text-sm">
        <span class="font-medium {response.status < 300 ? 'text-success' : response.status < 400 ? 'text-warning' : 'text-error'}">
          {response.status} {response.status_text}
        </span>
        <span class="text-text-muted">{response.time_ms} ms</span>
        <span class="text-text-muted">{formatSize(response.size_bytes)}</span>
      </div>
      <div class="ml-auto flex items-center gap-2">
        <button
          class="px-3 py-2 text-xs text-text-muted transition-colors hover:text-text"
          onclick={copyResponse}
          title="Copy response body"
        >
          Copy
        </button>
        <button
          class="px-4 py-2 text-sm transition-colors {responseTab === 'body' ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => (responseTab = 'body')}
        >
          Body
        </button>
        <button
          class="px-4 py-2 text-sm transition-colors {responseTab === 'headers' ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => (responseTab = 'headers')}
        >
          Headers
        </button>
      </div>
    </div>
    <div class="h-64 overflow-y-auto border-t border-border bg-surface p-3">
      {#if responseTab === 'body'}
        <pre class="text-sm font-mono whitespace-pre-wrap">{formatJson(response.body)}</pre>
      {:else}
        <div class="space-y-1">
          {#each Object.entries(response.headers) as [key, value] (key)}
            <div class="flex gap-2 text-sm">
              <span class="font-medium text-text-muted">{key}:</span>
              <span class="font-mono">{value}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
