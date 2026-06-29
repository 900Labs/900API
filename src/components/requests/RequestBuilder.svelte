<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { Send, LoaderCircle, Save } from '@lucide/svelte'
  import { loadRequestStore, activeEnvironmentStore } from '../../lib/stores'

  type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS'

  type KeyValue = { key: string; value: string; enabled: boolean }

  type AuthType = 'none' | 'basic' | 'bearer' | 'api_key' | 'o_auth2' | 'o_auth1' | 'aws_sig_v4' | 'hawk'

  type AuthConfig = {
    auth_type: AuthType
    username: string
    password: string
    token: string
    api_key: string
    api_key_name: string
    api_key_in: string
    oauth2_access_token: string
    oauth2_token_type: string
    oauth2_refresh_token: string
    oauth1_consumer_key: string
    oauth1_consumer_secret: string
    oauth1_token: string
    oauth1_token_secret: string
    aws_access_key_id: string
    aws_secret_access_key: string
    aws_region: string
    aws_service: string
    hawk_id: string
    hawk_key: string
    hawk_algorithm: string
  }

  type RequestConfig = {
    method: HttpMethod
    url: string
    headers: KeyValue[]
    params: KeyValue[]
    body_type: 'none' | 'json' | 'form_data' | 'x_www_form_urlencoded' | 'raw' | 'binary'
    body: string
    auth: AuthConfig
  }

  type ResponseData = {
    status: number
    status_text: string
    headers: Record<string, string>
    body: string
    time_ms: number
    size_bytes: number
  } | null

  type Collection = { id: string; name: string }

  let method = $state<HttpMethod>('GET')
  let url = $state('https://jsonplaceholder.typicode.com/posts/1')
  let headers = $state<KeyValue[]>([])
  let params = $state<KeyValue[]>([])
  let bodyType = $state<RequestConfig['body_type']>('none')
  let body = $state('')
  let authType = $state<AuthType>('none')
  let authUsername = $state('')
  let authPassword = $state('')
  let authToken = $state('')
  let authApiKey = $state('')
  let authKeyName = $state('')
  let authKeyIn = $state<'header' | 'query'>('header')
  // OAuth 2.0
  let oauth2AccessToken = $state('')
  let oauth2TokenType = $state('Bearer')
  let oauth2RefreshToken = $state('')
  // OAuth 1.0a
  let oauth1ConsumerKey = $state('')
  let oauth1ConsumerSecret = $state('')
  let oauth1Token = $state('')
  let oauth1TokenSecret = $state('')
  // AWS Sig v4
  let awsAccessKeyId = $state('')
  let awsSecretAccessKey = $state('')
  let awsRegion = $state('us-east-1')
  let awsService = $state('execute-api')
  // Hawk
  let hawkId = $state('')
  let hawkKey = $state('')
  let hawkAlgorithm = $state('sha256')
  let activeTab = $state<'params' | 'headers' | 'body' | 'auth'>('params')
  let responseTab = $state<'body' | 'headers'>('body')
  let response = $state<ResponseData>(null)
  let loading = $state(false)
  let error = $state<string | null>(null)
  let currentRequestId = $state<string | null>(null)
  let currentCollectionId = $state<string | null>(null)
  let showSaveDialog = $state(false)
  let saveName = $state('')
  let saveCollectionId = $state('')
  let collections = $state<Collection[]>([])
  let activeEnvVars = $state<{ key: string; value: string; enabled: boolean }[]>([])

  const methods: HttpMethod[] = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']

  const methodColors: Record<HttpMethod, string> = {
    GET: 'text-success',
    POST: 'text-warning',
    PUT: 'text-accent-hover',
    PATCH: 'text-accent-hover',
    DELETE: 'text-error',
    HEAD: 'text-text-muted',
    OPTIONS: 'text-text-muted',
  }

  function addRow(list: KeyValue[]) {
    list.push({ key: '', value: '', enabled: true })
    list = list
  }

  function removeRow(list: KeyValue[], index: number) {
    list.splice(index, 1)
    list = list
  }

  async function sendRequest() {
    loading = true
    error = null
    response = null

    try {
      const config: RequestConfig = {
        method,
        url,
        headers: headers.filter((h) => h.key.trim() !== ''),
        params: params.filter((p) => p.key.trim() !== ''),
        body_type: bodyType,
        body,
        auth: {
          auth_type: authType,
          username: authUsername,
          password: authPassword,
          token: authToken,
          api_key: authApiKey,
          api_key_name: authKeyName,
          api_key_in: authKeyIn,
          oauth2_access_token: oauth2AccessToken,
          oauth2_token_type: oauth2TokenType,
          oauth2_refresh_token: oauth2RefreshToken,
          oauth1_consumer_key: oauth1ConsumerKey,
          oauth1_consumer_secret: oauth1ConsumerSecret,
          oauth1_token: oauth1Token,
          oauth1_token_secret: oauth1TokenSecret,
          aws_access_key_id: awsAccessKeyId,
          aws_secret_access_key: awsSecretAccessKey,
          aws_region: awsRegion,
          aws_service: awsService,
          hawk_id: hawkId,
          hawk_key: hawkKey,
          hawk_algorithm: hawkAlgorithm,
        },
      }

      const result = await invoke<ResponseData>('send_request', {
        config,
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

  function getContentType(): string {
    if (!response) return 'text'
    const ct = response.headers['content-type'] || response.headers['Content-Type'] || ''
    if (ct.includes('json')) return 'json'
    if (ct.includes('xml')) return 'xml'
    if (ct.includes('html')) return 'html'
    if (ct.includes('text')) return 'text'
    return 'text'
  }

  function getFormattedBody(): string {
    if (!response) return ''
    const ct = getContentType()
    if (ct === 'json') return formatJson(response.body)
    if (ct === 'xml' || ct === 'html') {
      // Basic indentation for XML/HTML
      return response.body.replace(/></g, '>\n<')
    }
    return response.body
  }

  function copyResponse() {
    if (!response) return
    navigator.clipboard.writeText(response.body)
  }

  function loadFromStore(data: typeof loadRequestStore extends import('svelte/store').Writable<infer T> ? NonNullable<T> : never) {
    method = data.method as HttpMethod
    url = data.url
    try { headers = JSON.parse(data.headers || '[]') } catch { headers = [] }
    try { params = JSON.parse(data.params || '[]') } catch { params = [] }
    bodyType = data.bodyType as RequestConfig['body_type']
    body = data.body
    authType = data.authType as AuthType
    try {
      const ac = JSON.parse(data.authConfig || '{}')
      authUsername = ac.username || ''
      authPassword = ac.password || ''
      authToken = ac.token || ''
      authApiKey = ac.api_key || ''
      authKeyName = ac.api_key_name || ''
      authKeyIn = (ac.api_key_in || 'header') as 'header' | 'query'
      oauth2AccessToken = ac.oauth2_access_token || ''
      oauth2TokenType = ac.oauth2_token_type || 'Bearer'
      oauth2RefreshToken = ac.oauth2_refresh_token || ''
      oauth1ConsumerKey = ac.oauth1_consumer_key || ''
      oauth1ConsumerSecret = ac.oauth1_consumer_secret || ''
      oauth1Token = ac.oauth1_token || ''
      oauth1TokenSecret = ac.oauth1_token_secret || ''
      awsAccessKeyId = ac.aws_access_key_id || ''
      awsSecretAccessKey = ac.aws_secret_access_key || ''
      awsRegion = ac.aws_region || 'us-east-1'
      awsService = ac.aws_service || 'execute-api'
      hawkId = ac.hawk_id || ''
      hawkKey = ac.hawk_key || ''
      hawkAlgorithm = ac.hawk_algorithm || 'sha256'
    } catch {
      authUsername = ''
      authPassword = ''
      authToken = ''
      authApiKey = ''
      authKeyName = ''
      authKeyIn = 'header'
      oauth2AccessToken = ''
      oauth2TokenType = 'Bearer'
      oauth2RefreshToken = ''
      oauth1ConsumerKey = ''
      oauth1ConsumerSecret = ''
      oauth1Token = ''
      oauth1TokenSecret = ''
      awsAccessKeyId = ''
      awsSecretAccessKey = ''
      awsRegion = 'us-east-1'
      awsService = 'execute-api'
      hawkId = ''
      hawkKey = ''
      hawkAlgorithm = 'sha256'
    }
    currentRequestId = data.id
    currentCollectionId = data.collectionId
    saveName = data.name
    response = null
    error = null
  }

  async function openSaveDialog() {
    try {
      collections = await invoke<Collection[]>('list_collections')
    } catch {
      collections = []
    }
    if (currentCollectionId) {
      saveCollectionId = currentCollectionId
    } else if (collections.length > 0) {
      saveCollectionId = collections[0].id
    }
    if (!saveName && url) {
      saveName = `${method} ${url}`
    }
    showSaveDialog = true
  }

  async function saveRequest() {
    if (!saveCollectionId || !saveName.trim()) return
    const authConfigStr = JSON.stringify({
      auth_type: authType,
      username: authUsername,
      password: authPassword,
      token: authToken,
      api_key: authApiKey,
      api_key_name: authKeyName,
      api_key_in: authKeyIn,
      oauth2_access_token: oauth2AccessToken,
      oauth2_token_type: oauth2TokenType,
      oauth2_refresh_token: oauth2RefreshToken,
      oauth1_consumer_key: oauth1ConsumerKey,
      oauth1_consumer_secret: oauth1ConsumerSecret,
      oauth1_token: oauth1Token,
      oauth1_token_secret: oauth1TokenSecret,
      aws_access_key_id: awsAccessKeyId,
      aws_secret_access_key: awsSecretAccessKey,
      aws_region: awsRegion,
      aws_service: awsService,
      hawk_id: hawkId,
      hawk_key: hawkKey,
      hawk_algorithm: hawkAlgorithm,
    })
    const headersStr = JSON.stringify(headers)
    const paramsStr = JSON.stringify(params)

    try {
      if (currentRequestId) {
        await invoke('update_request', {
          id: currentRequestId,
          name: saveName.trim(),
          method,
          url,
          headers: headersStr,
          params: paramsStr,
          bodyType: bodyType,
          body,
          authType: authType,
          authConfig: authConfigStr,
        })
      } else {
        const req = await invoke<{ id: string }>('create_request', {
          collectionId: saveCollectionId,
          name: saveName.trim(),
          method,
          url,
          headers: headersStr,
          params: paramsStr,
          bodyType: bodyType,
          body,
          authType: authType,
          authConfig: authConfigStr,
        })
        currentRequestId = req.id
        currentCollectionId = saveCollectionId
      }
      showSaveDialog = false
    } catch (e) {
      error = String(e)
    }
  }

  // Subscribe to stores
  const unsubLoad = loadRequestStore.subscribe((data) => {
    if (data) loadFromStore(data)
  })
  const unsubEnv = activeEnvironmentStore.subscribe((env) => {
    activeEnvVars = env?.variables || []
  })

  $effect(() => {
    return () => {
      unsubLoad()
      unsubEnv()
    }
  })
</script>

<div class="flex h-full flex-col">
  <!-- URL Bar -->
  <div class="flex items-center gap-2 border-b border-border p-3">
    <select
      class="rounded-md border border-border bg-surface px-3 py-2 text-sm font-medium {methodColors[method]}"
      bind:value={method}
    >
      {#each methods as m (m)}
        <option value={m}>{m}</option>
      {/each}
    </select>
    <input
      type="text"
      class="flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="https://api.example.com/endpoint"
      bind:value={url}
      onkeydown={(e) => e.key === 'Enter' && sendRequest()}
    />
    <button
      class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
      onclick={sendRequest}
      disabled={loading || !url.trim()}
    >
      {#if loading}
        <LoaderCircle class="h-4 w-4 animate-spin" />
      {:else}
        <Send class="h-4 w-4" />
      {/if}
      Send
    </button>
    <button
      class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-2 text-sm font-medium text-text-muted transition-colors hover:text-text"
      onclick={openSaveDialog}
      title="Save to collection"
    >
      <Save class="h-4 w-4" />
    </button>
  </div>

  <!-- Request Tabs -->
  <div class="flex border-b border-border">
    {#each ['params', 'headers', 'body', 'auth'] as tab (tab)}
      <button
        class="px-4 py-2 text-sm transition-colors {activeTab === tab ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
        onclick={() => (activeTab = tab as typeof activeTab)}
      >
        {tab.charAt(0).toUpperCase() + tab.slice(1)}
        {#if tab === 'params' && params.filter((p) => p.key.trim()).length > 0}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{params.filter((p) => p.key.trim()).length}</span>
        {/if}
        {#if tab === 'headers' && headers.filter((h) => h.key.trim()).length > 0}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{headers.filter((h) => h.key.trim()).length}</span>
        {/if}
        {#if tab === 'auth' && authType !== 'none'}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{authType}</span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Request Config Area -->
  <div class="flex-1 overflow-y-auto p-3">
    {#if activeTab === 'params'}
      <div class="space-y-2">
        {#each params as param, i (i)}
          <div class="flex items-center gap-2">
            <input type="checkbox" bind:checked={param.enabled} class="accent-accent" />
            <input
              type="text"
              class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
              placeholder="key"
              bind:value={param.key}
            />
            <input
              type="text"
              class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
              placeholder="value"
              bind:value={param.value}
            />
            <button class="text-text-muted hover:text-error" onclick={() => removeRow(params, i)}>✕</button>
          </div>
        {/each}
        <button class="text-sm text-accent hover:text-accent-hover" onclick={() => addRow(params)}>+ Add Param</button>
      </div>
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
            <button class="text-text-muted hover:text-error" onclick={() => removeRow(headers, i)}>✕</button>
          </div>
        {/each}
        <button class="text-sm text-accent hover:text-accent-hover" onclick={() => addRow(headers)}>+ Add Header</button>
      </div>
    {:else if activeTab === 'body'}
      <div class="space-y-3">
        <div class="flex gap-2">
          {#each ['none', 'json', 'form_data', 'x_www_form_urlencoded', 'raw', 'binary'] as bt (bt)}
            <button
              class="rounded px-3 py-1 text-xs transition-colors {bodyType === bt ? 'bg-accent text-white' : 'bg-surface text-text-muted hover:text-text'}"
              onclick={() => (bodyType = bt as typeof bodyType)}
            >
              {bt === 'x_www_form_urlencoded' ? 'x-www-form-urlencoded' : bt === 'form_data' ? 'form-data' : bt}
            </button>
          {/each}
        </div>
        {#if bodyType !== 'none' && bodyType !== 'binary'}
          <textarea
            class="h-64 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
            placeholder={bodyType === 'json' ? '{\n  "key": "value"\n}' : 'Enter body content...'}
            bind:value={body}
          ></textarea>
        {/if}
      </div>
    {:else if activeTab === 'auth'}
      <div class="space-y-3">
        <div class="flex flex-wrap gap-2">
          {#each ['none', 'basic', 'bearer', 'api_key', 'o_auth2', 'o_auth1', 'aws_sig_v4', 'hawk'] as at (at)}
            <button
              class="rounded px-3 py-1 text-xs transition-colors {authType === at ? 'bg-accent text-white' : 'bg-surface text-text-muted hover:text-text'}"
              onclick={() => (authType = at as AuthType)}
            >
              {at === 'api_key' ? 'API Key' : at === 'o_auth2' ? 'OAuth 2.0' : at === 'o_auth1' ? 'OAuth 1.0a' : at === 'aws_sig_v4' ? 'AWS Sig v4' : at.charAt(0).toUpperCase() + at.slice(1)}
            </button>
          {/each}
        </div>
        {#if authType === 'basic'}
          <div class="space-y-2">
            <input
              type="text"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
              placeholder="Username"
              bind:value={authUsername}
            />
            <input
              type="password"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
              placeholder="Password"
              bind:value={authPassword}
            />
          </div>
        {:else if authType === 'bearer'}
          <input
            type="password"
            class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
            placeholder="Token"
            bind:value={authToken}
          />
        {:else if authType === 'api_key'}
          <div class="space-y-2">
            <div class="flex gap-2">
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
                placeholder="Key name (e.g. X-API-Key)"
                bind:value={authKeyName}
              />
              <select
                class="rounded border border-border bg-surface px-3 py-2 text-sm"
                bind:value={authKeyIn}
              >
                <option value="header">Header</option>
                <option value="query">Query Param</option>
              </select>
            </div>
            <input
              type="password"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="API Key value"
              bind:value={authApiKey}
            />
          </div>
        {:else if authType === 'o_auth2'}
          <div class="space-y-2">
            <input
              type="password"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Access Token"
              bind:value={oauth2AccessToken}
            />
            <div class="flex gap-2">
              <select
                class="rounded border border-border bg-surface px-3 py-2 text-sm"
                bind:value={oauth2TokenType}
              >
                <option value="Bearer">Bearer</option>
                <option value="token">Token</option>
              </select>
              <input
                type="password"
                class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
                placeholder="Refresh Token (optional)"
                bind:value={oauth2RefreshToken}
              />
            </div>
          </div>
        {:else if authType === 'o_auth1'}
          <div class="space-y-2">
            <input
              type="text"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Consumer Key"
              bind:value={oauth1ConsumerKey}
            />
            <input
              type="password"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Consumer Secret"
              bind:value={oauth1ConsumerSecret}
            />
            <input
              type="text"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Token"
              bind:value={oauth1Token}
            />
            <input
              type="password"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Token Secret"
              bind:value={oauth1TokenSecret}
            />
          </div>
        {:else if authType === 'aws_sig_v4'}
          <div class="space-y-2">
            <input
              type="text"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Access Key ID"
              bind:value={awsAccessKeyId}
            />
            <input
              type="password"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Secret Access Key"
              bind:value={awsSecretAccessKey}
            />
            <div class="flex gap-2">
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
                placeholder="Region (e.g. us-east-1)"
                bind:value={awsRegion}
              />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
                placeholder="Service (e.g. execute-api)"
                bind:value={awsService}
              />
            </div>
          </div>
        {:else if authType === 'hawk'}
          <div class="space-y-2">
            <input
              type="text"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Hawk ID"
              bind:value={hawkId}
            />
            <input
              type="password"
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
              placeholder="Hawk Key (secret)"
              bind:value={hawkKey}
            />
            <select
              class="w-full rounded border border-border bg-surface px-3 py-2 text-sm"
              bind:value={hawkAlgorithm}
            >
              <option value="sha256">SHA-256</option>
              <option value="sha1">SHA-1</option>
            </select>
          </div>
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
        <pre class="text-sm font-mono whitespace-pre-wrap">{getFormattedBody()}</pre>
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

{#if showSaveDialog}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="button" tabindex="0" onclick={() => (showSaveDialog = false)} onkeydown={(e) => e.key === 'Escape' && (showSaveDialog = false)}>
    <div class="w-96 rounded-lg border border-border bg-bg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <h3 class="mb-4 text-lg font-medium">Save Request</h3>
      <div class="space-y-3">
        <div>
          <label for="save-name-input" class="mb-1 block text-xs font-medium text-text-muted">Name</label>
          <input
            id="save-name-input"
            type="text"
            class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
            bind:value={saveName}
            placeholder="Request name"
          />
        </div>
        <div>
          <label for="save-collection-select" class="mb-1 block text-xs font-medium text-text-muted">Collection</label>
          <select
            id="save-collection-select"
            class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
            bind:value={saveCollectionId}
          >
            {#each collections as col (col.id)}
              <option value={col.id}>{col.name}</option>
            {/each}
          </select>
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <button
            class="rounded-md border border-border bg-surface px-4 py-2 text-sm text-text-muted hover:text-text"
            onclick={() => (showSaveDialog = false)}
          >
            Cancel
          </button>
          <button
            class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
            onclick={saveRequest}
            disabled={!saveName.trim() || !saveCollectionId}
          >
            {currentRequestId ? 'Update' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
