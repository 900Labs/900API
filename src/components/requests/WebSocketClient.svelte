<script lang="ts">
  import { invoke, listen } from '../../lib/tauri'
  import { createConnectionLifecycle } from '../../lib/connectionCleanup'
  import { Plug, Unplug, Send, Trash2 } from '@lucide/svelte'

  type WsMessage = {
    id: string
    direction: 'sent' | 'received'
    content: string
    message_type: 'text' | 'binary' | 'ping' | 'pong' | 'close'
    timestamp: number
  }

  type WsConnectionState = {
    id: string
    url: string
    status: 'connecting' | 'connected' | 'disconnected' | 'error'
    error: string | null
    messages: WsMessage[]
  }

  let url = $state('wss://echo.websocket.org')
  let message = $state('')
  let messages = $state<WsMessage[]>([])
  let status = $state<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
  let connectionId = $state<string | null>(null)
  let error = $state<string | null>(null)
  let unlistenState: (() => void) | null = null
  let unlistenMessage: (() => void) | null = null
  const lifecycle = createConnectionLifecycle('ws_disconnect')

  const statusColors: Record<string, string> = {
    disconnected: 'text-text-muted',
    connecting: 'text-warning',
    connected: 'text-success',
    error: 'text-error',
  }

  async function connect() {
    if (!url.trim()) return
    const id = crypto.randomUUID()
    connectionId = id
    messages = []
    error = null
    status = 'connecting'

    const stateListener = await listen<WsConnectionState>(`ws-${id}-state`, (event) => {
      if (lifecycle.isDestroyed() || connectionId !== id) return
      status = event.payload.status as typeof status
      error = event.payload.error
    })
    if (lifecycle.isDestroyed() || connectionId !== id) {
      stateListener()
      return
    }
    unlistenState = stateListener

    const messageListener = await listen<WsMessage>(`ws-${id}-message`, (event) => {
      if (lifecycle.isDestroyed() || connectionId !== id) return
      messages = [...messages, event.payload].slice(-500)
    })
    if (lifecycle.isDestroyed() || connectionId !== id) {
      messageListener()
      stateListener()
      if (unlistenState === stateListener) unlistenState = null
      return
    }
    unlistenMessage = messageListener

    const result = await lifecycle.establish(id, () => invoke('ws_connect', { id, url }))
    if (!result.active && result.error && !lifecycle.isDestroyed() && connectionId === id) {
      status = 'error'
      error = String(result.error)
    }
  }

  async function disconnect() {
    const id = connectionId
    if (!id) return
    connectionId = null
    try {
      await lifecycle.cancel(id)
    } catch (e) {
      if (!lifecycle.isDestroyed()) error = String(e)
    }
    if (lifecycle.isDestroyed()) return
    status = 'disconnected'
    if (unlistenState) { unlistenState(); unlistenState = null }
    if (unlistenMessage) { unlistenMessage(); unlistenMessage = null }
  }

  async function sendMessage() {
    if (!connectionId || !message.trim()) return
    try {
      await invoke('ws_send', { id: connectionId, message })
      message = ''
    } catch (e) {
      error = String(e)
    }
  }

  function clearMessages() {
    messages = []
  }

  function formatTime(ts: number): string {
    const d = new Date(ts)
    return d.toLocaleTimeString()
  }

  $effect(() => {
    return () => {
      if (unlistenState) unlistenState()
      if (unlistenMessage) unlistenMessage()
      const id = connectionId
      connectionId = null
      lifecycle.destroy(id)
    }
  })
</script>

<div class="flex h-full flex-col">
  <!-- Connection Bar -->
  <div class="flex items-center gap-2 border-b border-border p-3">
    <input
      type="text"
      class="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="wss://echo.websocket.org"
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

  {#if error}
    <div class="border-b border-border bg-error/10 px-3 py-2 text-sm text-error" role="alert">{error}</div>
  {/if}

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-3">
    {#if messages.length === 0}
      <div class="flex h-full items-center justify-center text-text-muted">
        <p>{status === 'connected' ? 'No messages yet. Send a message below.' : 'Connect to a WebSocket server to see messages.'}</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each messages as msg (msg.id)}
          <div class="flex gap-2 rounded-md p-2 text-sm {msg.direction === 'sent' ? 'bg-accent/5' : 'bg-surface'}">
            <span class="shrink-0 text-xs font-medium {msg.direction === 'sent' ? 'text-accent' : 'text-success'}">
              {msg.direction === 'sent' ? '→' : '←'} {msg.message_type}
            </span>
            <span class="shrink-0 text-xs text-text-muted">{formatTime(msg.timestamp)}</span>
            <span class="flex-1 font-mono break-all">{msg.content}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Message Input -->
  <div class="flex items-center gap-2 border-t border-border p-3">
    <input
      type="text"
      class="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="Type a message to send..."
      bind:value={message}
      disabled={status !== 'connected'}
      onkeydown={(e) => e.key === 'Enter' && sendMessage()}
    />
    <button
      class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
      onclick={sendMessage}
      disabled={status !== 'connected' || !message.trim()}
    >
      <Send class="h-4 w-4" />
      Send
    </button>
    <button
      class="rounded-md border border-border bg-surface px-3 py-2 text-sm text-text-muted transition-colors hover:text-text"
      onclick={clearMessages}
      title="Clear messages"
    >
      <Trash2 class="h-4 w-4" />
    </button>
  </div>
</div>
