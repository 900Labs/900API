<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { Play, Copy, Check } from '@lucide/svelte'

  type GrpcResponse = {
    status: number
    grpc_status: number
    grpc_message: string
    body_hex: string
    body_size: number
    time_ms: number
    headers: Record<string, string>
    trailers: Record<string, string>
  }

  type KeyValue = { key: string; value: string; enabled: boolean }

  let address = $state('localhost:50051')
  let serviceMethod = $state('/package.Service/Method')
  let bodyHex = $state('')
  let useTls = $state(false)
  let headers = $state<KeyValue[]>([
    { key: 'grpc-encoding', value: 'identity', enabled: false },
  ])
  let response = $state<GrpcResponse | null>(null)
  let sending = $state(false)
  let error = $state<string | null>(null)
  let copied = $state(false)
  let activeTab = $state<'response' | 'headers' | 'trailers'>('response')

  async function sendGrpc() {
    sending = true
    error = null
    response = null
    try {
      response = await invoke<GrpcResponse>('send_grpc', {
        address,
        serviceMethod,
        bodyHex,
        headers: headers.filter((h) => h.key.trim() !== ''),
        useTls,
      })
    } catch (e) {
      error = String(e)
    } finally {
      sending = false
    }
  }

  function addHeader() {
    headers = [...headers, { key: '', value: '', enabled: true }]
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index)
  }

  async function copyResponse() {
    if (response) {
      await navigator.clipboard.writeText(response.body_hex)
      copied = true
      setTimeout(() => (copied = false), 2000)
    }
  }

  function formatHex(hex: string): string {
    return hex.replace(/(.{2})/g, '$1 ').replace(/(.{48})/g, '$1\n')
  }
</script>

<div class="flex h-full flex-col">
  <!-- Address Bar -->
  <div class="flex items-center gap-2 border-b border-border p-3">
    <select
      class="rounded-md border border-border bg-surface px-2 py-2 text-sm outline-none focus:border-accent"
      bind:value={useTls}
    >
      <option value={false}>h2c</option>
      <option value={true}>TLS</option>
    </select>
    <input
      type="text"
      class="w-64 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="localhost:50051"
      bind:value={address}
    />
    <input
      type="text"
      class="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="/package.Service/Method"
      bind:value={serviceMethod}
    />
    <button
      class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
      onclick={sendGrpc}
      disabled={sending || !address.trim() || !serviceMethod.trim()}
    >
      <Play class="h-4 w-4" />
      Send
    </button>
  </div>

  <div class="flex flex-1 overflow-hidden">
    <!-- Left: Request -->
    <div class="flex-1 overflow-y-auto p-3">
      <h3 class="mb-2 text-xs font-medium uppercase text-text-muted">Request Body (hex)</h3>
      <textarea
        class="h-48 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
        placeholder={'0a05 4865 6c6c 6f'}
        bind:value={bodyHex}
      ></textarea>

      <h3 class="mb-2 mt-4 text-xs font-medium uppercase text-text-muted">Metadata</h3>
      <div class="space-y-2">
        {#each headers as header, i (i)}
          <div class="flex items-center gap-2">
            <input type="checkbox" bind:checked={header.enabled} class="accent-accent" />
            <input
              type="text"
              class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
              placeholder="Key"
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
        <button class="text-sm text-accent hover:text-accent-hover" onclick={addHeader}>+ Add Metadata</button>
      </div>
    </div>

    <!-- Right: Response -->
    <div class="flex-1 border-l border-border overflow-y-auto p-3">
      {#if sending}
        <p class="text-sm text-text-muted">Sending gRPC request...</p>
      {:else if error}
        <div class="rounded-md bg-error/10 p-3 text-sm text-error">
          <p class="font-medium">Error</p>
          <p class="mt-1 font-mono">{error}</p>
        </div>
      {:else if response}
        <div class="mb-3 flex items-center gap-4">
          <span class="text-sm font-medium {response.status === 200 ? 'text-success' : 'text-error'}">
            HTTP {response.status}
          </span>
          <span class="text-sm {response.grpc_status === 0 ? 'text-success' : 'text-error'}">
            gRPC {response.grpc_status === 0 ? 'OK' : response.grpc_status}
          </span>
          <span class="text-sm text-text-muted">{response.time_ms}ms</span>
          <span class="text-sm text-text-muted">{response.body_size} bytes</span>
        </div>

        {#if response.grpc_message}
          <p class="mb-3 text-sm text-error">{response.grpc_message}</p>
        {/if}

        <!-- Tabs -->
        <div class="mb-3 flex gap-1 border-b border-border">
          <button
            class="px-3 py-1.5 text-sm font-medium transition-colors {activeTab === 'response' ? 'border-b-2 border-accent text-accent' : 'text-text-muted hover:text-text'}"
            onclick={() => (activeTab = 'response')}
          >
            Response
          </button>
          <button
            class="px-3 py-1.5 text-sm font-medium transition-colors {activeTab === 'headers' ? 'border-b-2 border-accent text-accent' : 'text-text-muted hover:text-text'}"
            onclick={() => (activeTab = 'headers')}
          >
            Headers ({Object.keys(response.headers).length})
          </button>
          <button
            class="px-3 py-1.5 text-sm font-medium transition-colors {activeTab === 'trailers' ? 'border-b-2 border-accent text-accent' : 'text-text-muted hover:text-text'}"
            onclick={() => (activeTab = 'trailers')}
          >
            Trailers ({Object.keys(response.trailers).length})
          </button>
        </div>

        {#if activeTab === 'response'}
          <div class="relative">
            <button
              class="absolute right-2 top-2 rounded p-1 text-text-muted hover:text-text"
              onclick={copyResponse}
              title="Copy hex"
            >
              {#if copied}<Check class="h-4 w-4 text-success" />{:else}<Copy class="h-4 w-4" />{/if}
            </button>
            <pre class="overflow-x-auto rounded-md bg-surface p-3 text-xs font-mono">{formatHex(response.body_hex)}</pre>
          </div>
        {:else if activeTab === 'headers'}
          <div class="space-y-1">
            {#each Object.entries(response.headers) as [key, value]}
              <div class="flex gap-2 text-sm">
                <span class="font-mono font-medium text-text">{key}:</span>
                <span class="font-mono text-text-muted">{value}</span>
              </div>
            {/each}
          </div>
        {:else if activeTab === 'trailers'}
          <div class="space-y-1">
            {#each Object.entries(response.trailers) as [key, value]}
              <div class="flex gap-2 text-sm">
                <span class="font-mono font-medium text-text">{key}:</span>
                <span class="font-mono text-text-muted">{value}</span>
              </div>
            {/each}
          </div>
        {/if}
      {:else}
        <div class="flex h-full items-center justify-center text-text-muted">
          <p>Send a gRPC request to see the response.</p>
        </div>
      {/if}
    </div>
  </div>
</div>
