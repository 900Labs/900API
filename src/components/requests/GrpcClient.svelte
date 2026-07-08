<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { Braces, Check, Copy, FileCode2, Play, RefreshCcw, Wand2 } from '@lucide/svelte'

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
  type RequestBodyMode = 'hex' | 'text' | 'builder'
  type ResponseTab = 'hex' | 'text' | 'headers' | 'trailers'

  type ProtoField = {
    name: string
    number: number
    type: string
    repeated: boolean
    enabled: boolean
    value: string
  }

  type ProtoMessage = {
    name: string
    fields: ProtoField[]
  }

  type ProtoRpc = {
    service: string
    method: string
    requestType: string
    responseType: string
    requestStream: boolean
    responseStream: boolean
  }

  type ProtoService = {
    name: string
    rpcs: ProtoRpc[]
  }

  let address = $state('localhost:50051')
  let serviceMethod = $state('/package.Service/Method')
  let bodyHex = $state('')
  let bodyText = $state('')
  let bodyMode = $state<RequestBodyMode>('hex')
  let useTls = $state(false)
  let headers = $state<KeyValue[]>([
    { key: 'grpc-encoding', value: 'identity', enabled: false },
  ])
  let response = $state<GrpcResponse | null>(null)
  let sending = $state(false)
  let error = $state<string | null>(null)
  let copied = $state(false)
  let activeTab = $state<ResponseTab>('hex')

  let protoText = $state('')
  let protoPackage = $state('')
  let protoServices = $state<ProtoService[]>([])
  let protoMessages = $state<ProtoMessage[]>([])
  let selectedRpcKey = $state('')
  let selectedMessageName = $state('')
  let builderFields = $state<ProtoField[]>([])
  let protoParseMessage = $state<string | null>(null)
  let builderError = $state<string | null>(null)

  let rpcOptions = $derived(
    protoServices.flatMap((service) =>
      service.rpcs.map((rpc) => ({
        key: `${service.name}.${rpc.method}`,
        label: `${service.name}/${rpc.method}`,
        rpc,
      })),
    ),
  )

  let selectedMessage = $derived(protoMessages.find((message) => message.name === selectedMessageName) ?? null)

  function enabledHeaders(): KeyValue[] {
    return headers.filter((h) => h.enabled && h.key.trim() !== '')
  }

  function compactHex(hex: string): string {
    return hex.trim().replace(/[\s:,_-]/g, '')
  }

  function formatHex(hex: string): string {
    return compactHex(hex).replace(/(.{2})/g, '$1 ').replace(/(.{48})/g, '$1\n').trim()
  }

  function utf8ToHex(value: string): string {
    return Array.from(new TextEncoder().encode(value))
      .map((byte) => byte.toString(16).padStart(2, '0'))
      .join('')
  }

  function hexToBytes(hex: string): number[] {
    const clean = compactHex(hex)
    if (!clean) return []
    if (clean.length % 2 !== 0 || /[^0-9a-f]/i.test(clean)) {
      throw new Error('Hex values must contain an even number of hexadecimal characters.')
    }
    const bytes: number[] = []
    for (let index = 0; index < clean.length; index += 2) {
      bytes.push(Number.parseInt(clean.slice(index, index + 2), 16))
    }
    return bytes
  }

  function hexToUtf8(hex: string): string {
    try {
      const bytes = new Uint8Array(hexToBytes(hex))
      return new TextDecoder('utf-8', { fatal: false }).decode(bytes)
    } catch {
      return ''
    }
  }

  function stripProtoComments(input: string): string {
    return input
      .replace(/\/\/.*$/gm, '')
      .replace(/\/\*[\s\S]*?\*\//g, '')
  }

  function cleanTypeName(value: string): string {
    const clean = value.trim().replace(/^stream\s+/, '').replace(/^\./, '')
    return clean.split('.').pop() || clean
  }

  function parseProtoSchema() {
    protoParseMessage = null
    selectedRpcKey = ''
    selectedMessageName = ''
    builderFields = []

    const source = stripProtoComments(protoText)
    const packageMatch = source.match(/\bpackage\s+([A-Za-z_][\w.]*)\s*;/)
    protoPackage = packageMatch?.[1] ?? ''

    const services: ProtoService[] = []
    const serviceRegex = /\bservice\s+([A-Za-z_]\w*)\s*\{([\s\S]*?)\}/g
    let serviceMatch: RegExpExecArray | null
    while ((serviceMatch = serviceRegex.exec(source)) !== null) {
      const [, serviceName, body] = serviceMatch
      const rpcs: ProtoRpc[] = []
      const rpcRegex =
        /\brpc\s+([A-Za-z_]\w*)\s*\(\s*(stream\s+)?([.\w]+)\s*\)\s+returns\s+\(\s*(stream\s+)?([.\w]+)\s*\)/g
      let rpcMatch: RegExpExecArray | null
      while ((rpcMatch = rpcRegex.exec(body)) !== null) {
        rpcs.push({
          service: serviceName,
          method: rpcMatch[1],
          requestStream: Boolean(rpcMatch[2]),
          requestType: cleanTypeName(rpcMatch[3]),
          responseStream: Boolean(rpcMatch[4]),
          responseType: cleanTypeName(rpcMatch[5]),
        })
      }
      services.push({ name: serviceName, rpcs })
    }

    const messages: ProtoMessage[] = []
    const messageRegex = /\bmessage\s+([A-Za-z_]\w*)\s*\{([\s\S]*?)\}/g
    let messageMatch: RegExpExecArray | null
    while ((messageMatch = messageRegex.exec(source)) !== null) {
      const [, messageName, body] = messageMatch
      const fields: ProtoField[] = []
      const fieldRegex = /^(optional|required|repeated)?\s*([.\w]+)\s+([A-Za-z_]\w*)\s*=\s*(\d+)(?:\s*\[[^\]]*\])?$/
      for (const statement of body.split(';')) {
        const fieldMatch = statement.trim().match(fieldRegex)
        if (!fieldMatch) continue
        fields.push({
          name: fieldMatch[3],
          number: Number(fieldMatch[4]),
          type: cleanTypeName(fieldMatch[2]),
          repeated: fieldMatch[1] === 'repeated',
          enabled: true,
          value: '',
        })
      }
      messages.push({ name: messageName, fields })
    }

    protoServices = services
    protoMessages = messages
    protoParseMessage = `Parsed ${services.length} service${services.length === 1 ? '' : 's'} and ${messages.length} message${messages.length === 1 ? '' : 's'}.`

    if (rpcOptions.length > 0) {
      selectedRpcKey = rpcOptions[0].key
      applySelectedRpc()
    } else if (messages.length > 0) {
      selectedMessageName = messages[0].name
      syncBuilderFields()
    }
  }

  function applySelectedRpc() {
    const option = rpcOptions.find((item) => item.key === selectedRpcKey)
    if (!option) return
    const packagePrefix = protoPackage ? `${protoPackage}.` : ''
    serviceMethod = `/${packagePrefix}${option.rpc.service}/${option.rpc.method}`
    selectedMessageName = option.rpc.requestType
    syncBuilderFields()
  }

  function syncBuilderFields() {
    const previous = new Map(builderFields.map((field) => [`${field.number}:${field.name}`, field.value]))
    builderFields = (selectedMessage?.fields ?? []).map((field) => ({
      ...field,
      value: previous.get(`${field.number}:${field.name}`) ?? field.value,
    }))
    builderError = null
  }

  function wireTypeFor(field: ProtoField): number {
    const type = field.type.toLowerCase()
    if (['double', 'fixed64', 'sfixed64'].includes(type)) return 1
    if (['string', 'bytes'].includes(type) || protoMessages.some((message) => message.name === field.type)) return 2
    if (['float', 'fixed32', 'sfixed32'].includes(type)) return 5
    return 0
  }

  function encodeVarint(value: bigint): number[] {
    let remaining = value < 0n ? BigInt.asUintN(64, value) : value
    const bytes: number[] = []
    do {
      let byte = Number(remaining & 0x7fn)
      remaining >>= 7n
      if (remaining !== 0n) byte |= 0x80
      bytes.push(byte)
    } while (remaining !== 0n)
    return bytes
  }

  function encodeLengthDelimited(value: number[]): number[] {
    return [...encodeVarint(BigInt(value.length)), ...value]
  }

  function encodeFixed32(value: string, type: string): number[] {
    const buffer = new ArrayBuffer(4)
    const view = new DataView(buffer)
    if (type === 'float') {
      view.setFloat32(0, Number(value || '0'), true)
    } else if (type === 'sfixed32') {
      view.setInt32(0, Number(BigInt(value || '0')), true)
    } else {
      view.setUint32(0, Number(BigInt.asUintN(32, BigInt(value || '0'))), true)
    }
    return Array.from(new Uint8Array(buffer))
  }

  function encodeFixed64(value: string, type: string): number[] {
    const buffer = new ArrayBuffer(8)
    const view = new DataView(buffer)
    if (type === 'double') {
      view.setFloat64(0, Number(value || '0'), true)
    } else if (type === 'sfixed64') {
      view.setBigInt64(0, BigInt(value || '0'), true)
    } else {
      view.setBigUint64(0, BigInt.asUintN(64, BigInt(value || '0')), true)
    }
    return Array.from(new Uint8Array(buffer))
  }

  function encodeFieldValue(field: ProtoField): number[] {
    const value = field.value.trim()
    const type = field.type.toLowerCase()

    if (type === 'string') return encodeLengthDelimited(Array.from(new TextEncoder().encode(value)))
    if (type === 'bytes') return encodeLengthDelimited(hexToBytes(value))
    if (type === 'bool') return encodeVarint(value === 'true' || value === '1' ? 1n : 0n)
    if (type === 'sint32' || type === 'sint64') {
      const parsed = BigInt(value || '0')
      const shift = type === 'sint32' ? 31n : 63n
      return encodeVarint((parsed << 1n) ^ (parsed >> shift))
    }
    if (['double', 'fixed64', 'sfixed64'].includes(type)) return encodeFixed64(value, type)
    if (['float', 'fixed32', 'sfixed32'].includes(type)) return encodeFixed32(value, type)
    if (protoMessages.some((message) => message.name === field.type)) return encodeLengthDelimited(hexToBytes(value))
    return encodeVarint(BigInt(value || '0'))
  }

  function buildHexFromFields(): string {
    const bytes: number[] = []
    for (const field of builderFields) {
      if (!field.enabled || !field.value.trim()) continue
      const wireType = wireTypeFor(field)
      bytes.push(...encodeVarint((BigInt(field.number) << 3n) | BigInt(wireType)))
      bytes.push(...encodeFieldValue(field))
    }
    return bytes.map((byte) => byte.toString(16).padStart(2, '0')).join('')
  }

  function applyBuilderToHex() {
    try {
      builderError = null
      bodyHex = formatHex(buildHexFromFields())
      bodyMode = 'hex'
    } catch (e) {
      builderError = e instanceof Error ? e.message : String(e)
    }
  }

  function effectiveBodyHex(): string {
    if (bodyMode === 'text') return utf8ToHex(bodyText)
    if (bodyMode === 'builder') return buildHexFromFields()
    return compactHex(bodyHex)
  }

  async function sendGrpc() {
    sending = true
    error = null
    builderError = null
    response = null
    try {
      const requestBodyHex = effectiveBodyHex()
      if (bodyMode !== 'hex') bodyHex = formatHex(requestBodyHex)
      response = await invoke<GrpcResponse>('send_grpc', {
        address,
        serviceMethod,
        bodyHex: requestBodyHex,
        headers: enabledHeaders(),
        useTls,
      })
    } catch (e) {
      error = String(e)
      if (bodyMode === 'builder') builderError = String(e)
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
      const value = activeTab === 'text' ? hexToUtf8(response.body_hex) : response.body_hex
      await navigator.clipboard.writeText(value)
      copied = true
      setTimeout(() => (copied = false), 2000)
    }
  }

  function responseText(): string {
    if (!response) return ''
    return hexToUtf8(response.body_hex)
  }
</script>

<div class="flex h-full flex-col">
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

  <div class="grid min-h-0 flex-1 grid-cols-1 overflow-hidden lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]">
    <div class="min-h-0 overflow-y-auto p-3">
      <section class="mb-4 rounded border border-border bg-surface p-3">
        <div class="mb-3 flex items-center gap-2">
          <FileCode2 class="h-4 w-4 text-text-muted" />
          <h3 class="text-sm font-medium">Proto helper</h3>
          <button class="ml-auto flex items-center gap-1 rounded border border-border bg-bg px-2 py-1 text-xs text-text-muted hover:text-text" onclick={parseProtoSchema}>
            <RefreshCcw class="h-3.5 w-3.5" />
            Parse
          </button>
        </div>
        <textarea
          class="h-28 w-full rounded border border-border bg-bg p-2 font-mono text-xs outline-none focus:border-accent"
          placeholder={'syntax = "proto3";\npackage demo;\nservice Greeter { rpc SayHello (HelloRequest) returns (HelloReply); }\nmessage HelloRequest { string name = 1; }'}
          bind:value={protoText}
        ></textarea>
        {#if protoParseMessage}
          <p class="mt-2 text-xs text-success">{protoParseMessage}</p>
        {/if}
        {#if rpcOptions.length > 0}
          <div class="mt-3 grid gap-2 md:grid-cols-2">
            <label class="space-y-1 text-xs text-text-muted">
              <span>RPC</span>
              <select class="w-full rounded border border-border bg-bg px-2 py-1.5 text-sm text-text outline-none focus:border-accent" bind:value={selectedRpcKey} onchange={applySelectedRpc}>
                {#each rpcOptions as option (option.key)}
                  <option value={option.key}>{option.label} ({option.rpc.requestType} → {option.rpc.responseType})</option>
                {/each}
              </select>
            </label>
            <label class="space-y-1 text-xs text-text-muted">
              <span>Request message</span>
              <select class="w-full rounded border border-border bg-bg px-2 py-1.5 text-sm text-text outline-none focus:border-accent" bind:value={selectedMessageName} onchange={syncBuilderFields}>
                {#each protoMessages as message (message.name)}
                  <option value={message.name}>{message.name}</option>
                {/each}
              </select>
            </label>
          </div>
        {/if}
      </section>

      <section class="rounded border border-border bg-surface p-3">
        <div class="mb-3 flex flex-wrap items-center gap-2">
          <h3 class="text-sm font-medium">Request body</h3>
          <div class="ml-auto flex rounded border border-border bg-bg p-0.5">
            <button class="rounded px-2 py-1 text-xs {bodyMode === 'hex' ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}" onclick={() => (bodyMode = 'hex')}>Hex</button>
            <button class="rounded px-2 py-1 text-xs {bodyMode === 'text' ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}" onclick={() => (bodyMode = 'text')}>Text</button>
            <button class="flex items-center gap-1 rounded px-2 py-1 text-xs {bodyMode === 'builder' ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}" onclick={() => (bodyMode = 'builder')}>
              <Wand2 class="h-3.5 w-3.5" />
              Fields
            </button>
          </div>
        </div>

        {#if bodyMode === 'hex'}
          <textarea
            class="h-48 w-full rounded border border-border bg-bg p-3 font-mono text-sm outline-none focus:border-accent"
            placeholder={'0a05 4865 6c6c 6f'}
            bind:value={bodyHex}
          ></textarea>
        {:else if bodyMode === 'text'}
          <textarea
            class="h-48 w-full rounded border border-border bg-bg p-3 font-mono text-sm outline-none focus:border-accent"
            placeholder="UTF-8 text body converted to protobuf bytes as-is"
            bind:value={bodyText}
          ></textarea>
          <p class="mt-2 text-xs text-text-muted">Text mode sends raw UTF-8 bytes. Use field mode for simple protobuf scalar encoding.</p>
        {:else}
          {#if protoMessages.length === 0}
            <p class="rounded border border-border bg-bg p-3 text-sm text-text-muted">Paste and parse a `.proto` file to build a scalar protobuf request body.</p>
          {:else}
            <div class="mb-3 grid gap-2 md:grid-cols-[minmax(0,1fr)_auto]">
              <select class="rounded border border-border bg-bg px-2 py-1.5 text-sm text-text outline-none focus:border-accent" bind:value={selectedMessageName} onchange={syncBuilderFields}>
                {#each protoMessages as message (message.name)}
                  <option value={message.name}>{message.name}</option>
                {/each}
              </select>
              <button class="flex items-center justify-center gap-2 rounded bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-hover" onclick={applyBuilderToHex}>
                <Braces class="h-4 w-4" />
                Build Hex
              </button>
            </div>
            <div class="space-y-2">
              {#each builderFields as field, index (`${field.number}:${field.name}`)}
                <div class="grid gap-2 rounded border border-border bg-bg p-2 md:grid-cols-[auto_7rem_minmax(0,1fr)_minmax(0,1.4fr)]">
                  <label class="flex items-center gap-2 text-xs text-text-muted">
                    <input type="checkbox" bind:checked={field.enabled} class="accent-accent" />
                    {field.number}
                  </label>
                  <span class="rounded bg-surface px-2 py-1 font-mono text-xs text-text-muted">{field.type}{field.repeated ? '[]' : ''}</span>
                  <span class="min-w-0 truncate py-1 text-sm">{field.name}</span>
                  <input
                    class="rounded border border-border bg-surface px-2 py-1 text-sm font-mono outline-none focus:border-accent"
                    placeholder={field.type === 'string' ? 'value' : field.type === 'bytes' ? 'hex bytes' : '0'}
                    bind:value={builderFields[index].value}
                  />
                </div>
              {/each}
            </div>
            {#if builderError}
              <p class="mt-2 rounded border border-error/30 bg-error/10 p-2 text-sm text-error">{builderError}</p>
            {/if}
            <p class="mt-2 text-xs text-text-muted">Field mode supports scalar protobuf values, strings, bytes, enums as numeric values, and nested messages as hex payloads.</p>
          {/if}
        {/if}
      </section>

      <section class="mt-4 rounded border border-border bg-surface p-3">
        <h3 class="mb-3 text-sm font-medium">Metadata</h3>
        <div class="space-y-2">
          {#each headers as header, i (i)}
            <div class="flex items-center gap-2">
              <input type="checkbox" bind:checked={header.enabled} class="accent-accent" />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-bg px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="Key"
                bind:value={header.key}
              />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-bg px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="Value"
                bind:value={header.value}
              />
              <button class="text-text-muted hover:text-error" onclick={() => removeHeader(i)}>x</button>
            </div>
          {/each}
          <button class="text-sm text-accent hover:text-accent-hover" onclick={addHeader}>+ Add Metadata</button>
        </div>
      </section>
    </div>

    <div class="min-h-0 overflow-y-auto border-l border-border p-3">
      {#if sending}
        <p class="text-sm text-text-muted">Sending gRPC request...</p>
      {:else if error}
        <div class="rounded-md bg-error/10 p-3 text-sm text-error">
          <p class="font-medium">Error</p>
          <p class="mt-1 font-mono">{error}</p>
        </div>
      {:else if response}
        <div class="mb-3 flex flex-wrap items-center gap-4">
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

        <div class="mb-3 flex gap-1 border-b border-border">
          {#each [
            { value: 'hex', label: 'Hex' },
            { value: 'text', label: 'Text' },
            { value: 'headers', label: `Headers (${Object.keys(response.headers).length})` },
            { value: 'trailers', label: `Trailers (${Object.keys(response.trailers).length})` },
          ] as tab (tab.value)}
            <button
              class="px-3 py-1.5 text-sm font-medium transition-colors {activeTab === tab.value ? 'border-b-2 border-accent text-accent' : 'text-text-muted hover:text-text'}"
              onclick={() => (activeTab = tab.value as ResponseTab)}
            >
              {tab.label}
            </button>
          {/each}
        </div>

        {#if activeTab === 'hex'}
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
        {:else if activeTab === 'text'}
          <div class="relative">
            <button
              class="absolute right-2 top-2 rounded p-1 text-text-muted hover:text-text"
              onclick={copyResponse}
              title="Copy text"
            >
              {#if copied}<Check class="h-4 w-4 text-success" />{:else}<Copy class="h-4 w-4" />{/if}
            </button>
            <pre class="min-h-40 whitespace-pre-wrap rounded-md bg-surface p-3 text-sm font-mono">{responseText()}</pre>
          </div>
        {:else if activeTab === 'headers'}
          <div class="space-y-1">
            {#each Object.entries(response.headers) as [key, value] (key)}
              <div class="flex gap-2 text-sm">
                <span class="font-mono font-medium text-text">{key}:</span>
                <span class="font-mono text-text-muted">{value}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="space-y-1">
            {#each Object.entries(response.trailers) as [key, value] (key)}
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
