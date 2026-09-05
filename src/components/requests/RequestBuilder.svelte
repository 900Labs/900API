<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { save } from '@tauri-apps/plugin-dialog'
  import {
    Code2,
    Copy,
    Download,
    Eye,
    FileCode2,
    FileText,
    ListChecks,
    LoaderCircle,
    Plus,
    Save,
    Search,
    Send,
    Trash2,
    Upload,
    X,
  } from '@lucide/svelte'
  import { activeEnvironmentStore, loadRequestStore, type SavedRequestData } from '../../lib/stores'

  type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS'
  type KeyValue = { key: string; value: string; enabled: boolean }
  type AuthType = 'none' | 'basic' | 'bearer' | 'api_key' | 'o_auth2' | 'o_auth1' | 'aws_sig_v4' | 'hawk'
  type CodeLanguage = 'curl' | 'fetch' | 'python' | 'go'
  type ResponseTab = 'body' | 'headers' | 'examples' | 'compare'
  type ResponseBodyMode = 'formatted' | 'raw' | 'preview'

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
    body_type: 'none' | 'json' | 'form_data' | 'x_www_form_urlencoded' | 'raw'
    body: string
    auth: AuthConfig
    settings: RequestSettings
  }

  type RequestSettings = {
    timeout_ms: number
    connect_timeout_ms: number
    follow_redirects: boolean
    verify_ssl: boolean
    proxy_url: string
    use_cookie_jar: boolean
  }

  type ResponseData = {
    status: number
    status_text: string
    headers: Record<string, string>
    body: string
    time_ms: number
    size_bytes: number
  } | null

  type ResponseExample = {
    id: string
    request_id: string
    name: string
    status: number
    status_text: string
    headers: string
    body: string
    time_ms: number
    size_bytes: number
    created_at: string
  }

  type ScriptOutput = {
    logs: string[]
    test_results: unknown[]
    error: string | null
  }

  type Collection = {
    id: string
    name: string
    parent_id?: string | null
    sort_order?: number
  }

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

  type RequestDraft = {
    tabId: string
    requestId: string | null
    collectionId: string | null
    name: string
    method: HttpMethod
    url: string
    headers: KeyValue[]
    params: KeyValue[]
    bodyType: RequestConfig['body_type']
    body: string
    authType: AuthType
    authUsername: string
    authPassword: string
    authToken: string
    authApiKey: string
    authKeyName: string
    authKeyIn: 'header' | 'query'
    oauth2AccessToken: string
    oauth2TokenType: string
    oauth2RefreshToken: string
    oauth1ConsumerKey: string
    oauth1ConsumerSecret: string
    oauth1Token: string
    oauth1TokenSecret: string
    awsAccessKeyId: string
    awsSecretAccessKey: string
    awsRegion: string
    awsService: string
    hawkId: string
    hawkKey: string
    hawkAlgorithm: string
    settingsTimeoutMs: number
    settingsConnectTimeoutMs: number
    settingsFollowRedirects: boolean
    settingsVerifySsl: boolean
    settingsProxyUrl: string
    settingsUseCookieJar: boolean
    preRequestScript: string
    testScript: string
    configTab: 'params' | 'headers' | 'body' | 'auth' | 'scripts' | 'settings'
    responseTab: ResponseTab
    responseBodyMode: ResponseBodyMode
    responseSearch: string
    responseCompareExampleId: string
    response: ResponseData
    error: string | null
    dirty: boolean
  }

  let { embedded = false }: { embedded?: boolean } = $props()

  const methods: HttpMethod[] = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']
  const bodyTypes: RequestConfig['body_type'][] = ['none', 'json', 'form_data', 'x_www_form_urlencoded', 'raw']
  const authTypes: AuthType[] = ['none', 'basic', 'bearer', 'api_key', 'o_auth2', 'o_auth1', 'aws_sig_v4', 'hawk']
  const configTabs: RequestDraft['configTab'][] = ['params', 'headers', 'body', 'auth', 'scripts', 'settings']
  const responseTabs: { value: ResponseTab; label: string }[] = [
    { value: 'body', label: 'Body' },
    { value: 'headers', label: 'Headers' },
    { value: 'examples', label: 'Examples' },
    { value: 'compare', label: 'Compare' },
  ]
  const responseBodyModes: { value: ResponseBodyMode; label: string }[] = [
    { value: 'formatted', label: 'Formatted' },
    { value: 'raw', label: 'Raw' },
    { value: 'preview', label: 'Preview' },
  ]
  const codeLanguages: { value: CodeLanguage; label: string }[] = [
    { value: 'curl', label: 'cURL' },
    { value: 'fetch', label: 'JavaScript fetch' },
    { value: 'python', label: 'Python requests' },
    { value: 'go', label: 'Go net/http' },
  ]
  const preRequestScriptPlaceholder = 'var target = api900.response.status;'
  const testScriptPlaceholder = "if (api900.response.status !== 200) { throw new Error('Expected 200'); }"

  const methodColors: Record<HttpMethod, string> = {
    GET: 'text-success',
    POST: 'text-warning',
    PUT: 'text-accent-hover',
    PATCH: 'text-accent-hover',
    DELETE: 'text-error',
    HEAD: 'text-text-muted',
    OPTIONS: 'text-text-muted',
  }

  function newId(): string {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) return crypto.randomUUID()
    return `${Date.now()}-${Math.random().toString(16).slice(2)}`
  }

  function defaultAuthConfig(): AuthConfig {
    return {
      auth_type: 'none',
      username: '',
      password: '',
      token: '',
      api_key: '',
      api_key_name: '',
      api_key_in: 'header',
      oauth2_access_token: '',
      oauth2_token_type: 'Bearer',
      oauth2_refresh_token: '',
      oauth1_consumer_key: '',
      oauth1_consumer_secret: '',
      oauth1_token: '',
      oauth1_token_secret: '',
      aws_access_key_id: '',
      aws_secret_access_key: '',
      aws_region: 'us-east-1',
      aws_service: 'execute-api',
      hawk_id: '',
      hawk_key: '',
      hawk_algorithm: 'sha256',
    }
  }

  function defaultRequestSettings(): RequestSettings {
    return {
      timeout_ms: 120000,
      connect_timeout_ms: 30000,
      follow_redirects: true,
      verify_ssl: true,
      proxy_url: '',
      use_cookie_jar: false,
    }
  }

  function createBlankDraft(collectionId: string | null = null): RequestDraft {
    return {
      tabId: newId(),
      requestId: null,
      collectionId,
      name: 'Untitled Request',
      method: 'GET',
      url: 'https://jsonplaceholder.typicode.com/posts/1',
      headers: [],
      params: [],
      bodyType: 'none',
      body: '',
      authType: 'none',
      authUsername: '',
      authPassword: '',
      authToken: '',
      authApiKey: '',
      authKeyName: '',
      authKeyIn: 'header',
      oauth2AccessToken: '',
      oauth2TokenType: 'Bearer',
      oauth2RefreshToken: '',
      oauth1ConsumerKey: '',
      oauth1ConsumerSecret: '',
      oauth1Token: '',
      oauth1TokenSecret: '',
      awsAccessKeyId: '',
      awsSecretAccessKey: '',
      awsRegion: 'us-east-1',
      awsService: 'execute-api',
      hawkId: '',
      hawkKey: '',
      hawkAlgorithm: 'sha256',
      settingsTimeoutMs: 120000,
      settingsConnectTimeoutMs: 30000,
      settingsFollowRedirects: true,
      settingsVerifySsl: true,
      settingsProxyUrl: '',
      settingsUseCookieJar: false,
      preRequestScript: '',
      testScript: '',
      configTab: 'params',
      responseTab: 'body',
      responseBodyMode: 'formatted',
      responseSearch: '',
      responseCompareExampleId: '',
      response: null,
      error: null,
      dirty: false,
    }
  }

  const initialDraft = createBlankDraft()
  let requestTabs = $state<RequestDraft[]>([initialDraft])
  let activeRequestTabId = $state(initialDraft.tabId)

  let method = $state<HttpMethod>(initialDraft.method)
  let url = $state(initialDraft.url)
  let headers = $state<KeyValue[]>([])
  let params = $state<KeyValue[]>([])
  let bodyType = $state<RequestConfig['body_type']>('none')
  let body = $state('')
  let bodyFields = $state<KeyValue[]>([])
  let authType = $state<AuthType>('none')
  let authUsername = $state('')
  let authPassword = $state('')
  let authToken = $state('')
  let authApiKey = $state('')
  let authKeyName = $state('')
  let authKeyIn = $state<'header' | 'query'>('header')
  let oauth2AccessToken = $state('')
  let oauth2TokenType = $state('Bearer')
  let oauth2RefreshToken = $state('')
  let oauth1ConsumerKey = $state('')
  let oauth1ConsumerSecret = $state('')
  let oauth1Token = $state('')
  let oauth1TokenSecret = $state('')
  let awsAccessKeyId = $state('')
  let awsSecretAccessKey = $state('')
  let awsRegion = $state('us-east-1')
  let awsService = $state('execute-api')
  let hawkId = $state('')
  let hawkKey = $state('')
  let hawkAlgorithm = $state('sha256')
  let settingsTimeoutMs = $state(120000)
  let settingsConnectTimeoutMs = $state(30000)
  let settingsFollowRedirects = $state(true)
  let settingsVerifySsl = $state(true)
  let settingsProxyUrl = $state('')
  let settingsUseCookieJar = $state(false)
  let preRequestScript = $state('')
  let testScript = $state('')
  let configTab = $state<RequestDraft['configTab']>('params')
  let responseTab = $state<ResponseTab>('body')
  let responseBodyMode = $state<ResponseBodyMode>('formatted')
  let responseSearch = $state('')
  let responseCompareExampleId = $state('')
  let response = $state<ResponseData>(null)
  let loading = $state(false)
  let error = $state<string | null>(null)
  let currentRequestId = $state<string | null>(null)
  let currentCollectionId = $state<string | null>(null)
  let currentDirty = $state(false)
  let showSaveDialog = $state(false)
  let saveName = $state('Untitled Request')
  let saveCollectionId = $state('')
  let collections = $state<Collection[]>([])
  let activeEnvVars = $state<{ key: string; value: string; enabled: boolean }[]>([])
  let showImportCurlDialog = $state(false)
  let curlInput = $state('')
  let importCurlError = $state<string | null>(null)
  let showCodeDialog = $state(false)
  let codeLanguage = $state<CodeLanguage>('curl')
  let responseExamples = $state<ResponseExample[]>([])
  let responseExamplesLoading = $state(false)
  let responseExamplesError = $state<string | null>(null)
  let showSaveExampleDialog = $state(false)
  let responseExampleName = $state('')
  let responseActionMessage = $state<string | null>(null)
  let scriptExecutionMessage = $state<string | null>(null)

  let envSuggestions = $derived(
    activeEnvVars
      .filter((variable) => variable.enabled && variable.key.trim())
      .map((variable) => `{{${variable.key.trim()}}}`),
  )

  function markDirty() {
    currentDirty = true
  }

  function asHttpMethod(value: string): HttpMethod {
    return methods.includes(value as HttpMethod) ? (value as HttpMethod) : 'GET'
  }

  function asBodyType(value: string | undefined): RequestConfig['body_type'] {
    return bodyTypes.includes(value as RequestConfig['body_type']) ? (value as RequestConfig['body_type']) : 'none'
  }

  function asAuthType(value: string | undefined): AuthType {
    return authTypes.includes(value as AuthType) ? (value as AuthType) : 'none'
  }

  function cloneRows(rows: KeyValue[]): KeyValue[] {
    return rows.map((row) => ({ ...row }))
  }

  function normalizeRows(value: unknown): KeyValue[] {
    if (!Array.isArray(value)) return []
    return value
      .filter((row): row is Partial<KeyValue> => typeof row === 'object' && row !== null)
      .map((row) => ({
        key: typeof row.key === 'string' ? row.key : '',
        value: typeof row.value === 'string' ? row.value : '',
        enabled: typeof row.enabled === 'boolean' ? row.enabled : true,
      }))
  }

  function parseBodyFields(value: string): KeyValue[] {
    if (!value.trim()) return []
    try {
      return normalizeRows(JSON.parse(value))
    } catch {
      return []
    }
  }

  function selectBodyType(nextType: RequestConfig['body_type']) {
    bodyType = nextType
    if (nextType === 'form_data' || nextType === 'x_www_form_urlencoded') {
      bodyFields = parseBodyFields(body)
      body = JSON.stringify(bodyFields)
    }
    markDirty()
  }

  function syncBodyFields() {
    body = JSON.stringify(bodyFields)
    markDirty()
  }

  function addBodyField() {
    bodyFields = [...bodyFields, { key: '', value: '', enabled: true }]
    syncBodyFields()
  }

  function removeBodyField(index: number) {
    bodyFields = bodyFields.filter((_, rowIndex) => rowIndex !== index)
    syncBodyFields()
  }

  function normalizeAuthConfig(value: unknown): AuthConfig {
    const source = typeof value === 'object' && value !== null ? (value as Partial<AuthConfig>) : {}
    const defaults = defaultAuthConfig()
    return {
      ...defaults,
      ...source,
      auth_type: asAuthType(source.auth_type),
      api_key_in: source.api_key_in === 'query' ? 'query' : 'header',
      oauth2_token_type: source.oauth2_token_type || defaults.oauth2_token_type,
      aws_region: source.aws_region || defaults.aws_region,
      aws_service: source.aws_service || defaults.aws_service,
      hawk_algorithm: source.hawk_algorithm || defaults.hawk_algorithm,
    }
  }

  function normalizeRequestSettings(value: unknown): RequestSettings {
    const source = typeof value === 'object' && value !== null ? (value as Partial<RequestSettings>) : {}
    const defaults = defaultRequestSettings()
    const timeout = Number(source.timeout_ms ?? defaults.timeout_ms)
    const connectTimeout = Number(source.connect_timeout_ms ?? defaults.connect_timeout_ms)
    return {
      timeout_ms: Number.isFinite(timeout) && timeout > 0 ? timeout : defaults.timeout_ms,
      connect_timeout_ms: Number.isFinite(connectTimeout) && connectTimeout > 0 ? connectTimeout : defaults.connect_timeout_ms,
      follow_redirects: typeof source.follow_redirects === 'boolean' ? source.follow_redirects : defaults.follow_redirects,
      verify_ssl: typeof source.verify_ssl === 'boolean' ? source.verify_ssl : defaults.verify_ssl,
      proxy_url: typeof source.proxy_url === 'string' ? source.proxy_url : defaults.proxy_url,
      use_cookie_jar: typeof source.use_cookie_jar === 'boolean' ? source.use_cookie_jar : defaults.use_cookie_jar,
    }
  }

  function snapshotCurrentDraft(): RequestDraft {
    const existing = requestTabs.find((tab) => tab.tabId === activeRequestTabId) ?? initialDraft
    return {
      ...existing,
      requestId: currentRequestId,
      collectionId: currentCollectionId,
      name: saveName.trim() || existing.name,
      method,
      url,
      headers: cloneRows(headers),
      params: cloneRows(params),
      bodyType,
      body,
      authType,
      authUsername,
      authPassword,
      authToken,
      authApiKey,
      authKeyName,
      authKeyIn,
      oauth2AccessToken,
      oauth2TokenType,
      oauth2RefreshToken,
      oauth1ConsumerKey,
      oauth1ConsumerSecret,
      oauth1Token,
      oauth1TokenSecret,
      awsAccessKeyId,
      awsSecretAccessKey,
      awsRegion,
      awsService,
      hawkId,
      hawkKey,
      hawkAlgorithm,
      settingsTimeoutMs,
      settingsConnectTimeoutMs,
      settingsFollowRedirects,
      settingsVerifySsl,
      settingsProxyUrl,
      settingsUseCookieJar,
      preRequestScript,
      testScript,
      configTab,
      responseTab,
      responseBodyMode,
      responseSearch,
      responseCompareExampleId,
      response,
      error,
      dirty: currentDirty,
    }
  }

  function persistActiveDraft() {
    const index = requestTabs.findIndex((tab) => tab.tabId === activeRequestTabId)
    if (index === -1) return
    requestTabs[index] = snapshotCurrentDraft()
    requestTabs = requestTabs
  }

  function applyDraft(draft: RequestDraft) {
    currentRequestId = draft.requestId
    currentCollectionId = draft.collectionId
    saveName = draft.name
    method = draft.method
    url = draft.url
    headers = cloneRows(draft.headers)
    params = cloneRows(draft.params)
    bodyType = draft.bodyType
    body = draft.body
    bodyFields = parseBodyFields(draft.body)
    authType = draft.authType
    authUsername = draft.authUsername
    authPassword = draft.authPassword
    authToken = draft.authToken
    authApiKey = draft.authApiKey
    authKeyName = draft.authKeyName
    authKeyIn = draft.authKeyIn
    oauth2AccessToken = draft.oauth2AccessToken
    oauth2TokenType = draft.oauth2TokenType
    oauth2RefreshToken = draft.oauth2RefreshToken
    oauth1ConsumerKey = draft.oauth1ConsumerKey
    oauth1ConsumerSecret = draft.oauth1ConsumerSecret
    oauth1Token = draft.oauth1Token
    oauth1TokenSecret = draft.oauth1TokenSecret
    awsAccessKeyId = draft.awsAccessKeyId
    awsSecretAccessKey = draft.awsSecretAccessKey
    awsRegion = draft.awsRegion
    awsService = draft.awsService
    hawkId = draft.hawkId
    hawkKey = draft.hawkKey
    hawkAlgorithm = draft.hawkAlgorithm
    settingsTimeoutMs = draft.settingsTimeoutMs
    settingsConnectTimeoutMs = draft.settingsConnectTimeoutMs
    settingsFollowRedirects = draft.settingsFollowRedirects
    settingsVerifySsl = draft.settingsVerifySsl
    settingsProxyUrl = draft.settingsProxyUrl
    settingsUseCookieJar = draft.settingsUseCookieJar
    preRequestScript = draft.preRequestScript
    testScript = draft.testScript
    configTab = draft.configTab
    responseTab = draft.responseTab
    responseBodyMode = draft.responseBodyMode
    responseSearch = draft.responseSearch
    responseCompareExampleId = draft.responseCompareExampleId
    response = draft.response
    error = draft.error
    currentDirty = draft.dirty
    responseExamples = []
    responseExamplesError = null
    responseActionMessage = null
    scriptExecutionMessage = null
  }

  function switchRequestTab(tabId: string) {
    if (tabId === activeRequestTabId) return
    persistActiveDraft()
    const next = requestTabs.find((tab) => tab.tabId === tabId)
    if (!next) return
    activeRequestTabId = tabId
    applyDraft(next)
  }

  function newRequestTab(collectionId: string | null = null) {
    persistActiveDraft()
    const draft = createBlankDraft(collectionId)
    requestTabs = [...requestTabs, draft]
    activeRequestTabId = draft.tabId
    applyDraft(draft)
  }

  function duplicateActiveTab() {
    persistActiveDraft()
    const clone = {
      ...snapshotCurrentDraft(),
      tabId: newId(),
      requestId: null,
      name: `${saveName || 'Untitled Request'} Copy`,
      dirty: true,
      response: null,
      responseBodyMode,
      responseSearch: '',
      responseCompareExampleId: '',
      error: null,
    }
    requestTabs = [...requestTabs, clone]
    activeRequestTabId = clone.tabId
    applyDraft(clone)
  }

  function closeRequestTab(tabId: string = activeRequestTabId) {
    if (requestTabs.length === 1) {
      const draft = createBlankDraft()
      requestTabs = [draft]
      activeRequestTabId = draft.tabId
      applyDraft(draft)
      return
    }

    const index = requestTabs.findIndex((tab) => tab.tabId === tabId)
    if (index === -1) return
    requestTabs = requestTabs.filter((tab) => tab.tabId !== tabId)
    if (tabId === activeRequestTabId) {
      const next = requestTabs[Math.max(0, index - 1)]
      activeRequestTabId = next.tabId
      applyDraft(next)
    }
  }

  function tabTitle(tab: RequestDraft): string {
    if (tab.tabId === activeRequestTabId) {
      return saveName.trim() || shortUrl(url) || 'Untitled Request'
    }
    return tab.name.trim() || shortUrl(tab.url) || 'Untitled Request'
  }

  function tabMethod(tab: RequestDraft): HttpMethod {
    return tab.tabId === activeRequestTabId ? method : tab.method
  }

  function tabDirty(tab: RequestDraft): boolean {
    return tab.tabId === activeRequestTabId ? currentDirty : tab.dirty
  }

  function shortUrl(value: string): string {
    if (!value.trim()) return ''
    try {
      const parsed = new URL(value)
      return `${parsed.host}${parsed.pathname === '/' ? '' : parsed.pathname}`
    } catch {
      return value
    }
  }

  function addParam() {
    params = [...params, { key: '', value: '', enabled: true }]
    markDirty()
  }

  function addHeader() {
    headers = [...headers, { key: '', value: '', enabled: true }]
    markDirty()
  }

  function removeParam(index: number) {
    params = params.filter((_, i) => i !== index)
    markDirty()
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index)
    markDirty()
  }

  function buildAuthConfig(): AuthConfig {
    return {
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
    }
  }

  function buildRequestSettings(): RequestSettings {
    return {
      timeout_ms: Math.max(1, Math.min(600000, Number(settingsTimeoutMs) || 120000)),
      connect_timeout_ms: Math.max(1, Math.min(Number(settingsTimeoutMs) || 120000, Number(settingsConnectTimeoutMs) || 30000)),
      follow_redirects: settingsFollowRedirects,
      verify_ssl: settingsVerifySsl,
      proxy_url: settingsProxyUrl.trim(),
      use_cookie_jar: settingsUseCookieJar,
    }
  }

  function buildCurrentRequestConfig(): RequestConfig {
    return {
      method,
      url,
      headers: headers.filter((h) => h.enabled && h.key.trim() !== ''),
      params: params.filter((p) => p.enabled && p.key.trim() !== ''),
      body_type: bodyType,
      body,
      auth: buildAuthConfig(),
      settings: buildRequestSettings(),
    }
  }

  async function runWorkbenchScript(
    label: string,
    script: string,
    responseBody = '',
    responseStatus = 0,
    responseHeaders = '{}',
  ) {
    if (!script.trim()) return
    const output = await invoke<ScriptOutput>('run_test_script', {
      script,
      responseBody,
      responseStatus,
      responseHeaders,
    })
    if (output.error) {
      throw new Error(`${label} script failed: ${output.error || 'unknown script error'}`)
    }
  }

  async function sendRequest() {
    loading = true
    error = null
    response = null
    responseActionMessage = null
    scriptExecutionMessage = null

    try {
      const config = buildCurrentRequestConfig()
      await runWorkbenchScript('Pre-request', preRequestScript)

      response = await invoke<ResponseData>('send_request', {
        config,
        environmentVariables: activeEnvVars.length > 0 ? activeEnvVars : undefined,
      })
      if (response) {
        await runWorkbenchScript(
          'Test',
          testScript,
          response.body,
          response.status,
          JSON.stringify(response.headers),
        )
        if (preRequestScript.trim() || testScript.trim()) {
          scriptExecutionMessage = 'Scripts completed in the Rust sandbox.'
        }
      }
      if (responseBodyMode === 'preview' && getContentType() !== 'html') responseBodyMode = 'formatted'
      persistActiveDraft()
      window.dispatchEvent(new CustomEvent('900api:history-changed'))
    } catch (e) {
      error = String(e)
      persistActiveDraft()
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
    const ct = contentTypeFromHeaders(response.headers)
    if (ct.includes('json')) return 'json'
    if (ct.includes('xml')) return 'xml'
    if (ct.includes('html')) return 'html'
    if (ct.includes('text')) return 'text'
    return 'text'
  }

  function contentTypeFromHeaders(headersRecord: Record<string, string>): string {
    const match = Object.entries(headersRecord).find(([key]) => key.toLowerCase() === 'content-type')
    return match?.[1] ?? ''
  }

  function contentTypeHeader(): string {
    if (!response) return ''
    return contentTypeFromHeaders(response.headers)
  }

  function getFormattedBody(): string {
    if (!response) return ''
    return formatResponseBody(response.body, response.headers)
  }

  function formatResponseBody(responseBody: string, headersRecord: Record<string, string>): string {
    const ct = contentTypeFromHeaders(headersRecord)
    if (ct.includes('json')) return formatJson(responseBody)
    if (ct.includes('xml') || ct.includes('html')) return responseBody.replace(/></g, '>\n<')
    return responseBody
  }

  function getVisibleResponseBody(): string {
    if (!response) return ''
    if (responseBodyMode === 'raw') return response.body
    return getFormattedBody()
  }

  function responseCanPreview(): boolean {
    return getContentType() === 'html'
  }

  function responseBodyParts(): { text: string; match: boolean }[] {
    const bodyText = getVisibleResponseBody()
    const needle = responseSearch.trim()
    if (!needle) return [{ text: bodyText, match: false }]

    const lowerBody = bodyText.toLowerCase()
    const lowerNeedle = needle.toLowerCase()
    const parts: { text: string; match: boolean }[] = []
    let cursor = 0
    let next = lowerBody.indexOf(lowerNeedle)

    while (next !== -1) {
      if (next > cursor) parts.push({ text: bodyText.slice(cursor, next), match: false })
      parts.push({ text: bodyText.slice(next, next + needle.length), match: true })
      cursor = next + needle.length
      next = lowerBody.indexOf(lowerNeedle, cursor)
    }

    if (cursor < bodyText.length) parts.push({ text: bodyText.slice(cursor), match: false })
    return parts
  }

  function responseMatchCount(): number {
    const needle = responseSearch.trim()
    if (!needle) return 0
    const lowerBody = getVisibleResponseBody().toLowerCase()
    const lowerNeedle = needle.toLowerCase()
    let count = 0
    let cursor = lowerBody.indexOf(lowerNeedle)
    while (cursor !== -1) {
      count += 1
      cursor = lowerBody.indexOf(lowerNeedle, cursor + lowerNeedle.length)
    }
    return count
  }

  function copyResponse() {
    if (!response) return
    navigator.clipboard.writeText(response.body).catch((e) => console.error('[900api] clipboard write failed:', e))
    responseActionMessage = 'Response body copied.'
  }

  function responseFileExtension(): string {
    const ct = getContentType()
    if (ct === 'json') return 'json'
    if (ct === 'html') return 'html'
    if (ct === 'xml') return 'xml'
    return 'txt'
  }

  async function downloadResponse() {
    if (!response) return
    responseActionMessage = null
    try {
      const path = await save({
        defaultPath: `response-${response.status}.${responseFileExtension()}`,
        filters: [
          { name: 'Response body', extensions: [responseFileExtension()] },
          { name: 'All files', extensions: ['*'] },
        ],
      })
      if (!path) return
      await invoke('write_text_file', { path, content: response.body })
      responseActionMessage = 'Response body exported.'
    } catch (e) {
      responseActionMessage = String(e)
    }
  }

  function openSaveExampleDialog() {
    responseActionMessage = null
    if (!response) return
    if (!currentRequestId) {
      responseActionMessage = 'Save the request to a collection before saving response examples.'
      return
    }
    responseExampleName = `${response.status} ${response.status_text || 'Response'}`
    showSaveExampleDialog = true
  }

  async function loadResponseExamples() {
    if (!currentRequestId) {
      responseExamples = []
      responseCompareExampleId = ''
      return
    }
    responseExamplesLoading = true
    responseExamplesError = null
    try {
      responseExamples = await invoke<ResponseExample[]>('list_response_examples', { requestId: currentRequestId })
      if (!responseExamples.some((example) => example.id === responseCompareExampleId)) {
        responseCompareExampleId = responseExamples[0]?.id ?? ''
      }
    } catch (e) {
      responseExamplesError = String(e)
    } finally {
      responseExamplesLoading = false
    }
  }

  function showResponseTab(tab: ResponseTab) {
    responseTab = tab
    if (tab === 'examples' || tab === 'compare') loadResponseExamples()
    persistActiveDraft()
  }

  async function saveResponseExample() {
    if (!response || !currentRequestId || !responseExampleName.trim()) return
    responseExamplesError = null
    try {
      const saved = await invoke<ResponseExample>('create_response_example', {
        requestId: currentRequestId,
        name: responseExampleName.trim(),
        response,
      })
      responseExamples = [saved, ...responseExamples.filter((example) => example.id !== saved.id)]
      responseCompareExampleId = saved.id
      responseTab = 'examples'
      showSaveExampleDialog = false
      responseActionMessage = 'Response example saved.'
    } catch (e) {
      responseExamplesError = String(e)
    }
  }

  async function deleteResponseExample(id: string) {
    responseExamplesError = null
    try {
      await invoke('delete_response_example', { id })
      responseExamples = responseExamples.filter((example) => example.id !== id)
      if (responseCompareExampleId === id) responseCompareExampleId = responseExamples[0]?.id ?? ''
    } catch (e) {
      responseExamplesError = String(e)
    }
  }

  function parseExampleHeaders(example: ResponseExample): Record<string, string> {
    try {
      const parsed = JSON.parse(example.headers || '{}')
      return typeof parsed === 'object' && parsed !== null ? parsed : {}
    } catch {
      return {}
    }
  }

  function loadExampleAsResponse(example: ResponseExample) {
    response = {
      status: example.status,
      status_text: example.status_text,
      headers: parseExampleHeaders(example),
      body: example.body,
      time_ms: example.time_ms,
      size_bytes: example.size_bytes,
    }
    responseTab = 'body'
    responseSearch = ''
    persistActiveDraft()
  }

  function exampleSummary(example: ResponseExample): string {
    const statusDelta = response ? example.status - response.status : 0
    const sizeDelta = response ? example.size_bytes - response.size_bytes : 0
    if (!response) return `${example.time_ms} ms · ${formatSize(example.size_bytes)}`
    const statusText = statusDelta === 0 ? 'same status' : `${statusDelta > 0 ? '+' : ''}${statusDelta} status`
    const sizeText = sizeDelta === 0 ? 'same size' : `${sizeDelta > 0 ? '+' : '-'}${formatSize(Math.abs(sizeDelta))}`
    return `${statusText} · ${sizeText}`
  }

  function selectedCompareExample(): ResponseExample | null {
    return responseExamples.find((example) => example.id === responseCompareExampleId) ?? responseExamples[0] ?? null
  }

  function normalizedHeaderEntries(headersRecord: Record<string, string>): Map<string, { key: string; value: string }> {
    return new Map(Object.entries(headersRecord).map(([key, value]) => [key.toLowerCase(), { key, value }]))
  }

  function compareHeaderRows(example: ResponseExample): { key: string; expected: string; actual: string; kind: 'added' | 'removed' | 'changed' }[] {
    if (!response) return []
    const expected = normalizedHeaderEntries(parseExampleHeaders(example))
    const actual = normalizedHeaderEntries(response.headers)
    const keys = Array.from(new Set([...expected.keys(), ...actual.keys()])).sort()
    return keys
      .map((key) => {
        const expectedHeader = expected.get(key)
        const actualHeader = actual.get(key)
        if (!expectedHeader && actualHeader) {
          return { key: actualHeader.key, expected: '', actual: actualHeader.value, kind: 'added' as const }
        }
        if (expectedHeader && !actualHeader) {
          return { key: expectedHeader.key, expected: expectedHeader.value, actual: '', kind: 'removed' as const }
        }
        if (expectedHeader && actualHeader && expectedHeader.value !== actualHeader.value) {
          return { key: expectedHeader.key, expected: expectedHeader.value, actual: actualHeader.value, kind: 'changed' as const }
        }
        return null
      })
      .filter((row): row is { key: string; expected: string; actual: string; kind: 'added' | 'removed' | 'changed' } => row !== null)
  }

  function compareBodyRows(example: ResponseExample): { line: number; expected: string; actual: string; kind: 'added' | 'removed' | 'changed' }[] {
    if (!response) return []
    const expectedLines = formatResponseBody(example.body, parseExampleHeaders(example)).split(/\r?\n/)
    const actualLines = formatResponseBody(response.body, response.headers).split(/\r?\n/)
    const rowCount = Math.max(expectedLines.length, actualLines.length)
    const rows: { line: number; expected: string; actual: string; kind: 'added' | 'removed' | 'changed' }[] = []
    for (let index = 0; index < rowCount; index += 1) {
      const expected = expectedLines[index]
      const actual = actualLines[index]
      if (expected === actual) continue
      rows.push({
        line: index + 1,
        expected: expected ?? '',
        actual: actual ?? '',
        kind: expected === undefined ? 'added' : actual === undefined ? 'removed' : 'changed',
      })
      if (rows.length >= 200) break
    }
    return rows
  }

  function compareSummary(example: ResponseExample): { statusMatches: boolean; bodyMatches: boolean; headerDiffs: number; bodyDiffs: number; sizeDelta: number } {
    const headerDiffs = compareHeaderRows(example).length
    const bodyDiffs = compareBodyRows(example).length
    return {
      statusMatches: response?.status === example.status,
      bodyMatches: bodyDiffs === 0,
      headerDiffs,
      bodyDiffs,
      sizeDelta: response ? response.size_bytes - example.size_bytes : 0,
    }
  }

  function shellQuote(value: string): string {
    return "'" + value.replace(/'/g, "'\\''") + "'"
  }

  function hasHeader(headersList: KeyValue[], name: string): boolean {
    return headersList.some((header) => header.key.toLowerCase() === name.toLowerCase())
  }

  function requestParams(config: RequestConfig): KeyValue[] {
    const rows = cloneRows(config.params)
    if (config.auth.auth_type === 'api_key' && config.auth.api_key_in === 'query' && config.auth.api_key_name) {
      rows.push({ key: config.auth.api_key_name, value: config.auth.api_key, enabled: true })
    }
    return rows
  }

  function requestHeaders(config: RequestConfig): KeyValue[] {
    const rows = cloneRows(config.headers)
    if (config.auth.auth_type === 'bearer' && config.auth.token && !hasHeader(rows, 'Authorization')) {
      rows.push({ key: 'Authorization', value: `Bearer ${config.auth.token}`, enabled: true })
    }
    if (config.auth.auth_type === 'o_auth2' && config.auth.oauth2_access_token && !hasHeader(rows, 'Authorization')) {
      rows.push({
        key: 'Authorization',
        value: `${config.auth.oauth2_token_type || 'Bearer'} ${config.auth.oauth2_access_token}`,
        enabled: true,
      })
    }
    if (config.auth.auth_type === 'api_key' && config.auth.api_key_in === 'header' && config.auth.api_key_name) {
      rows.push({ key: config.auth.api_key_name, value: config.auth.api_key, enabled: true })
    }
    if (config.body_type === 'json' && config.body.trim() && !hasHeader(rows, 'Content-Type')) {
      rows.push({ key: 'Content-Type', value: 'application/json', enabled: true })
    }
    return rows.filter((row) => row.enabled && row.key.trim())
  }

  function withQueryParams(baseUrl: string, rows: KeyValue[]): string {
    const enabledRows = rows.filter((row) => row.enabled && row.key.trim())
    if (enabledRows.length === 0) return baseUrl
    try {
      const parsed = new URL(baseUrl)
      for (const row of enabledRows) parsed.searchParams.append(row.key, row.value)
      return parsed.toString()
    } catch {
      const queryString = enabledRows
        .map((row) => `${encodeURIComponent(row.key)}=${encodeURIComponent(row.value)}`)
        .join('&')
      return `${baseUrl}${baseUrl.includes('?') ? '&' : '?'}${queryString}`
    }
  }

  function jsonLiteral(value: unknown): string {
    return JSON.stringify(value, null, 2)
  }

  function rowRecord(rows: KeyValue[]): Record<string, string> {
    return rows.reduce<Record<string, string>>((record, row) => {
      record[row.key] = row.value
      return record
    }, {})
  }

  function bodyIsSendable(config: RequestConfig): boolean {
    return config.body_type !== 'none' && config.body.trim().length > 0
  }

  function requestBodyFields(config: RequestConfig): KeyValue[] {
    if (config.body_type !== 'form_data' && config.body_type !== 'x_www_form_urlencoded') return []
    return parseBodyFields(config.body).filter((field) => field.enabled && field.key.trim())
  }

  function jsonBodyExpression(config: RequestConfig): string {
    if (config.body_type === 'json') {
      try {
        return `JSON.stringify(${JSON.stringify(JSON.parse(config.body), null, 2)})`
      } catch {
        return JSON.stringify(config.body)
      }
    }
    return JSON.stringify(config.body)
  }

  function pythonBodyArgument(config: RequestConfig): string {
    if (!bodyIsSendable(config)) return ''
    if (config.body_type === 'form_data') return 'files=files'
    if (config.body_type === 'x_www_form_urlencoded') return 'data=form_data'
    if (config.body_type === 'json') {
      try {
        return `json=${JSON.stringify(JSON.parse(config.body), null, 2)}`
      } catch {
        return `data=${JSON.stringify(config.body)}`
      }
    }
    return `data=${JSON.stringify(config.body)}`
  }

  function buildCurlCommand(config: RequestConfig = buildCurrentRequestConfig()): string {
    const lines = [`curl -X ${config.method} ${shellQuote(withQueryParams(config.url, requestParams(config)))}`]
    lines.push(`  --max-time ${Math.ceil(config.settings.timeout_ms / 1000)}`)
    lines.push(`  --connect-timeout ${Math.ceil(config.settings.connect_timeout_ms / 1000)}`)
    if (config.settings.follow_redirects) lines.push('  -L')
    if (!config.settings.verify_ssl) lines.push('  --insecure')
    if (config.settings.proxy_url.trim()) lines.push(`  --proxy ${shellQuote(config.settings.proxy_url.trim())}`)
    const configHeaders = requestHeaders(config)
    if (config.auth.auth_type === 'basic' && (config.auth.username || config.auth.password)) {
      lines.push(`  -u ${shellQuote(`${config.auth.username}:${config.auth.password}`)}`)
    }
    for (const header of configHeaders) {
      lines.push(`  -H ${shellQuote(`${header.key}: ${header.value}`)}`)
    }
    if (bodyIsSendable(config)) {
      const fields = requestBodyFields(config)
      if (config.body_type === 'form_data') {
        for (const field of fields) lines.push(`  --form ${shellQuote(`${field.key}=${field.value}`)}`)
      } else if (config.body_type === 'x_www_form_urlencoded') {
        for (const field of fields) lines.push(`  --data-urlencode ${shellQuote(`${field.key}=${field.value}`)}`)
      } else {
        lines.push(`  --data ${shellQuote(config.body)}`)
      }
    }
    return lines.join(' \\\n')
  }

  function buildFetchSnippet(config: RequestConfig = buildCurrentRequestConfig()): string {
    const configHeaders = requestHeaders(config)
    const fields = requestBodyFields(config)
    const lines: string[] = []
    if (config.body_type === 'form_data') {
      lines.push('const formData = new FormData()')
      for (const field of fields) {
        lines.push(`formData.append(${JSON.stringify(field.key)}, ${JSON.stringify(field.value)})`)
      }
      lines.push('')
    }
    lines.push(
      `const response = await fetch(${JSON.stringify(withQueryParams(config.url, requestParams(config)))}, {`,
      `  method: ${JSON.stringify(config.method)},`,
    )

    if (configHeaders.length > 0) {
      lines.push(`  headers: ${jsonLiteral(rowRecord(configHeaders)).replace(/\n/g, '\n  ')},`)
    }
    if (bodyIsSendable(config)) {
      if (config.body_type === 'form_data') {
        lines.push('  body: formData,')
      } else if (config.body_type === 'x_www_form_urlencoded') {
        lines.push(`  body: new URLSearchParams(${jsonLiteral(rowRecord(fields))}),`)
      } else {
        lines.push(`  body: ${jsonBodyExpression(config)},`)
      }
    }
    lines.push('})')
    lines.push('')
    lines.push('console.log(response.status)')
    lines.push('console.log(await response.text())')
    return lines.join('\n')
  }

  function buildPythonSnippet(config: RequestConfig = buildCurrentRequestConfig()): string {
    const configHeaders = requestHeaders(config)
    const configParams = requestParams(config)
    const args = [
      JSON.stringify(config.method),
      'url',
      configHeaders.length > 0 ? 'headers=headers' : '',
      configParams.length > 0 ? 'params=params' : '',
      pythonBodyArgument(config),
      config.auth.auth_type === 'basic' && (config.auth.username || config.auth.password)
        ? `auth=(${JSON.stringify(config.auth.username)}, ${JSON.stringify(config.auth.password)})`
        : '',
    ].filter(Boolean)

    const fields = requestBodyFields(config)
    const lines = [
      'import requests',
      '',
      `url = ${JSON.stringify(config.url)}`,
      configHeaders.length > 0 ? `headers = ${jsonLiteral(rowRecord(configHeaders))}` : 'headers = {}',
      configParams.length > 0 ? `params = ${jsonLiteral(rowRecord(configParams))}` : 'params = {}',
    ]
    if (config.body_type === 'form_data') {
      lines.push(`files = ${jsonLiteral(Object.fromEntries(fields.map((field) => [field.key, [null, field.value]])))}`)
    } else if (config.body_type === 'x_www_form_urlencoded') {
      lines.push(`form_data = ${jsonLiteral(rowRecord(fields))}`)
    }
    lines.push(
      '',
      `response = requests.request(${args.join(', ')})`,
      'print(response.status_code)',
      'print(response.text)',
    )
    return lines.join('\n')
  }

  function buildGoSnippet(config: RequestConfig = buildCurrentRequestConfig()): string {
    const configHeaders = requestHeaders(config)
    const hasBody = bodyIsSendable(config)
    const imports = ['"fmt"', '"io"', '"net/http"']
    if (hasBody && config.body_type !== 'form_data') imports.push('"strings"')
    if (config.body_type === 'form_data') imports.push('"bytes"', '"mime/multipart"')
    if (config.body_type === 'x_www_form_urlencoded') imports.push('"net/url"')

    const lines = [
      'package main',
      '',
      'import (',
      ...imports.map((item) => `  ${item}`),
      ')',
      '',
      'func main() {',
      '  var requestBody io.Reader',
    ]
    const fields = requestBodyFields(config)
    if (config.body_type === 'form_data') {
      lines.push('  var multipartBody bytes.Buffer')
      lines.push('  multipartWriter := multipart.NewWriter(&multipartBody)')
      for (const field of fields) {
        lines.push(`  if err := multipartWriter.WriteField(${JSON.stringify(field.key)}, ${JSON.stringify(field.value)}); err != nil { panic(err) }`)
      }
      lines.push('  if err := multipartWriter.Close(); err != nil { panic(err) }')
      lines.push('  requestBody = &multipartBody')
    } else if (config.body_type === 'x_www_form_urlencoded') {
      lines.push('  formData := url.Values{}')
      for (const field of fields) {
        lines.push(`  formData.Add(${JSON.stringify(field.key)}, ${JSON.stringify(field.value)})`)
      }
      lines.push('  requestBody = strings.NewReader(formData.Encode())')
    } else if (hasBody) {
      lines.push(`  requestBody = strings.NewReader(${JSON.stringify(config.body)})`)
    }
    lines.push(`  req, err := http.NewRequest(${JSON.stringify(config.method)}, ${JSON.stringify(withQueryParams(config.url, requestParams(config)))}, requestBody)`)
    lines.push('  if err != nil {')
    lines.push('    panic(err)')
    lines.push('  }')
    for (const header of configHeaders) {
      lines.push(`  req.Header.Set(${JSON.stringify(header.key)}, ${JSON.stringify(header.value)})`)
    }
    if (config.body_type === 'form_data') {
      lines.push('  req.Header.Set("Content-Type", multipartWriter.FormDataContentType())')
    } else if (config.body_type === 'x_www_form_urlencoded') {
      lines.push('  req.Header.Set("Content-Type", "application/x-www-form-urlencoded")')
    }
    if (config.auth.auth_type === 'basic' && (config.auth.username || config.auth.password)) {
      lines.push(`  req.SetBasicAuth(${JSON.stringify(config.auth.username)}, ${JSON.stringify(config.auth.password)})`)
    }
    lines.push('')
    lines.push('  resp, err := http.DefaultClient.Do(req)')
    lines.push('  if err != nil {')
    lines.push('    panic(err)')
    lines.push('  }')
    lines.push('  defer resp.Body.Close()')
    lines.push('')
    lines.push('  body, err := io.ReadAll(resp.Body)')
    lines.push('  if err != nil {')
    lines.push('    panic(err)')
    lines.push('  }')
    lines.push('  fmt.Println(resp.StatusCode)')
    lines.push('  fmt.Println(string(body))')
    lines.push('}')
    return lines.join('\n')
  }

  function generatedCode(): string {
    const config = buildCurrentRequestConfig()
    if (codeLanguage === 'fetch') return buildFetchSnippet(config)
    if (codeLanguage === 'python') return buildPythonSnippet(config)
    if (codeLanguage === 'go') return buildGoSnippet(config)
    return buildCurlCommand(config)
  }

  function copyGeneratedCode() {
    navigator.clipboard.writeText(generatedCode()).catch((e) => console.error('[900api] clipboard write failed:', e))
  }

  function openCodeDialog() {
    codeLanguage = 'curl'
    showCodeDialog = true
  }

  function copyCurl() {
    navigator.clipboard.writeText(buildCurlCommand()).catch((e) => console.error('[900api] clipboard write failed:', e))
  }

  function draftFromRequestConfig(config: RequestConfig, name: string, dirty = true): RequestDraft {
    const normalizedAuth = normalizeAuthConfig(config.auth)
    const normalizedSettings = normalizeRequestSettings(config.settings)
    return {
      ...createBlankDraft(),
      name,
      method: asHttpMethod(config.method),
      url: config.url,
      headers: normalizeRows(config.headers),
      params: normalizeRows(config.params),
      bodyType: asBodyType(config.body_type),
      body: config.body || '',
      authType: normalizedAuth.auth_type,
      authUsername: normalizedAuth.username,
      authPassword: normalizedAuth.password,
      authToken: normalizedAuth.token,
      authApiKey: normalizedAuth.api_key,
      authKeyName: normalizedAuth.api_key_name,
      authKeyIn: normalizedAuth.api_key_in === 'query' ? 'query' : 'header',
      oauth2AccessToken: normalizedAuth.oauth2_access_token,
      oauth2TokenType: normalizedAuth.oauth2_token_type || 'Bearer',
      oauth2RefreshToken: normalizedAuth.oauth2_refresh_token,
      oauth1ConsumerKey: normalizedAuth.oauth1_consumer_key,
      oauth1ConsumerSecret: normalizedAuth.oauth1_consumer_secret,
      oauth1Token: normalizedAuth.oauth1_token,
      oauth1TokenSecret: normalizedAuth.oauth1_token_secret,
      awsAccessKeyId: normalizedAuth.aws_access_key_id,
      awsSecretAccessKey: normalizedAuth.aws_secret_access_key,
      awsRegion: normalizedAuth.aws_region || 'us-east-1',
      awsService: normalizedAuth.aws_service || 'execute-api',
      hawkId: normalizedAuth.hawk_id,
      hawkKey: normalizedAuth.hawk_key,
      hawkAlgorithm: normalizedAuth.hawk_algorithm || 'sha256',
      settingsTimeoutMs: normalizedSettings.timeout_ms,
      settingsConnectTimeoutMs: normalizedSettings.connect_timeout_ms,
      settingsFollowRedirects: normalizedSettings.follow_redirects,
      settingsVerifySsl: normalizedSettings.verify_ssl,
      settingsProxyUrl: normalizedSettings.proxy_url,
      settingsUseCookieJar: normalizedSettings.use_cookie_jar,
      preRequestScript: '',
      testScript: '',
      dirty,
    }
  }

  function requestConfigFromUnknown(value: unknown): RequestConfig | null {
    if (typeof value !== 'object' || value === null) return null
    const source = value as Partial<RequestConfig>
    if (typeof source.url !== 'string' || typeof source.method !== 'string') return null
    return {
      method: asHttpMethod(source.method),
      url: source.url,
      headers: normalizeRows(source.headers),
      params: normalizeRows(source.params),
      body_type: asBodyType(source.body_type),
      body: typeof source.body === 'string' ? source.body : '',
      auth: normalizeAuthConfig(source.auth),
      settings: normalizeRequestSettings(source.settings),
    }
  }

  function tokenizeCurl(input: string): string[] {
    const normalized = input.replace(/\\\r?\n/g, ' ')
    const tokens: string[] = []
    let current = ''
    let quote: '"' | "'" | null = null

    for (let index = 0; index < normalized.length; index += 1) {
      const char = normalized[index]
      if (quote) {
        if (char === quote) {
          quote = null
        } else if (quote === '"' && char === '\\' && index + 1 < normalized.length) {
          index += 1
          current += normalized[index]
        } else {
          current += char
        }
        continue
      }

      if (char === '"' || char === "'") {
        quote = char
      } else if (/\s/.test(char)) {
        if (current) {
          tokens.push(current)
          current = ''
        }
      } else if (char === '\\' && index + 1 < normalized.length) {
        index += 1
        current += normalized[index]
      } else {
        current += char
      }
    }

    if (quote) throw new Error('Unclosed quote in cURL command.')
    if (current) tokens.push(current)
    return tokens
  }

  function splitInlineOption(token: string, option: string): string | null {
    if (token.startsWith(`${option}=`)) return token.slice(option.length + 1)
    return null
  }

  function parseHeaderLine(value: string): KeyValue | null {
    const separator = value.indexOf(':')
    if (separator === -1) return null
    const key = value.slice(0, separator).trim()
    if (!key) return null
    return { key, value: value.slice(separator + 1).trim(), enabled: true }
  }

  function splitUrlParams(rawUrl: string): { url: string; params: KeyValue[] } {
    try {
      const parsed = new URL(rawUrl)
      const paramsFromUrl: KeyValue[] = []
      for (const [key, value] of parsed.searchParams.entries()) {
        paramsFromUrl.push({ key, value, enabled: true })
      }
      parsed.search = ''
      return { url: parsed.toString(), params: paramsFromUrl }
    } catch {
      return { url: rawUrl, params: [] }
    }
  }

  function applyAuthFromHeaders(headersList: KeyValue[], auth: AuthConfig): KeyValue[] {
    const nextHeaders = [...headersList]
    const authorizationIndex = nextHeaders.findIndex((header) => header.key.toLowerCase() === 'authorization')
    if (authorizationIndex === -1) return nextHeaders

    const value = nextHeaders[authorizationIndex].value.trim()
    const bearerMatch = value.match(/^Bearer\s+(.+)$/i)
    if (bearerMatch) {
      auth.auth_type = 'bearer'
      auth.token = bearerMatch[1]
      nextHeaders.splice(authorizationIndex, 1)
    }
    return nextHeaders
  }

  function parseCurlCommand(input: string): RequestConfig {
    const tokens = tokenizeCurl(input)
    if (tokens.length === 0) throw new Error('Paste a cURL command to import.')
    let index = tokens[0] === 'curl' ? 1 : 0
    let parsedMethod: HttpMethod = 'GET'
    let parsedUrl = ''
    let parsedHeaders: KeyValue[] = []
    const dataParts: string[] = []
    const parsedAuth = defaultAuthConfig()
    let forceHead = false

    const readValue = (token: string): string => {
      index += 1
      if (index >= tokens.length) throw new Error(`${token} expects a value.`)
      return tokens[index]
    }

    while (index < tokens.length) {
      const token = tokens[index]
      const requestValue = splitInlineOption(token, '--request')
      const headerValue = splitInlineOption(token, '--header')
      const dataValue = splitInlineOption(token, '--data')
        ?? splitInlineOption(token, '--data-raw')
        ?? splitInlineOption(token, '--data-binary')
        ?? splitInlineOption(token, '--data-urlencode')
      const urlValue = splitInlineOption(token, '--url')
      const userValue = splitInlineOption(token, '--user')

      if (requestValue !== null) {
        parsedMethod = asHttpMethod(requestValue)
      } else if (headerValue !== null) {
        const header = parseHeaderLine(headerValue)
        if (header) parsedHeaders.push(header)
      } else if (dataValue !== null) {
        dataParts.push(dataValue)
      } else if (urlValue !== null) {
        parsedUrl = urlValue
      } else if (userValue !== null) {
        parsedAuth.auth_type = 'basic'
        const separator = userValue.indexOf(':')
        parsedAuth.username = separator === -1 ? userValue : userValue.slice(0, separator)
        parsedAuth.password = separator === -1 ? '' : userValue.slice(separator + 1)
      } else if (token === '-X' || token === '--request') {
        parsedMethod = asHttpMethod(readValue(token))
      } else if (token.startsWith('-X') && token.length > 2) {
        parsedMethod = asHttpMethod(token.slice(2))
      } else if (token === '-H' || token === '--header') {
        const header = parseHeaderLine(readValue(token))
        if (header) parsedHeaders.push(header)
      } else if (token === '-d' || token === '--data' || token === '--data-raw' || token === '--data-binary' || token === '--data-urlencode') {
        dataParts.push(readValue(token))
      } else if (token.startsWith('-d') && token.length > 2) {
        dataParts.push(token.slice(2))
      } else if (token === '-u' || token === '--user') {
        const user = readValue(token)
        parsedAuth.auth_type = 'basic'
        const separator = user.indexOf(':')
        parsedAuth.username = separator === -1 ? user : user.slice(0, separator)
        parsedAuth.password = separator === -1 ? '' : user.slice(separator + 1)
      } else if (token === '-A' || token === '--user-agent') {
        parsedHeaders.push({ key: 'User-Agent', value: readValue(token), enabled: true })
      } else if (token === '-I' || token === '--head') {
        forceHead = true
      } else if (token === '--url') {
        parsedUrl = readValue(token)
      } else if (!token.startsWith('-') && !parsedUrl) {
        parsedUrl = token
      }
      index += 1
    }

    if (!parsedUrl) throw new Error('Could not find a URL in the cURL command.')
    if (forceHead) parsedMethod = 'HEAD'
    const { url: importedUrl, params: importedParams } = splitUrlParams(parsedUrl)
    parsedHeaders = applyAuthFromHeaders(parsedHeaders, parsedAuth)
    const importedBody = dataParts.join('&')
    const contentType = parsedHeaders.find((header) => header.key.toLowerCase() === 'content-type')?.value.toLowerCase() ?? ''
    const importedBodyType: RequestConfig['body_type'] = importedBody
      ? contentType.includes('json') || (() => {
          try {
            JSON.parse(importedBody)
            return true
          } catch {
            return false
          }
        })()
        ? 'json'
        : contentType.includes('x-www-form-urlencoded')
          ? 'x_www_form_urlencoded'
          : 'raw'
      : 'none'

    return {
      method: importedBody && parsedMethod === 'GET' ? 'POST' : parsedMethod,
      url: importedUrl,
      headers: parsedHeaders,
      params: importedParams,
      body_type: importedBodyType,
      body: importedBody,
      auth: parsedAuth,
      settings: defaultRequestSettings(),
    }
  }

  function importCurlCommand() {
    importCurlError = null
    try {
      const config = parseCurlCommand(curlInput)
      persistActiveDraft()
      const draft = draftFromRequestConfig(config, `${config.method} ${shortUrl(config.url) || 'Imported cURL'}`, true)
      requestTabs = [...requestTabs, draft]
      activeRequestTabId = draft.tabId
      applyDraft(draft)
      showImportCurlDialog = false
      curlInput = ''
    } catch (e) {
      importCurlError = e instanceof Error ? e.message : String(e)
    }
  }

  function openImportCurlDialog() {
    importCurlError = null
    curlInput = ''
    showImportCurlDialog = true
  }

  function parseSavedDraft(data: SavedRequestData): RequestDraft {
    let parsedHeaders: KeyValue[] = []
    let parsedParams: KeyValue[] = []
    let parsedAuth: Partial<AuthConfig> = {}
    let parsedSettings: Partial<RequestSettings> = {}
    try { parsedHeaders = JSON.parse(data.headers || '[]') } catch { parsedHeaders = [] }
    try { parsedParams = JSON.parse(data.params || '[]') } catch { parsedParams = [] }
    try { parsedAuth = JSON.parse(data.authConfig || '{}') } catch { parsedAuth = {} }
    try { parsedSettings = JSON.parse(data.settings || '{}') } catch { parsedSettings = {} }
    const normalizedAuth = normalizeAuthConfig({ ...parsedAuth, auth_type: data.authType as AuthType })
    const normalizedSettings = normalizeRequestSettings(parsedSettings)

    return {
      ...createBlankDraft(data.collectionId),
      requestId: data.id,
      collectionId: data.collectionId,
      name: data.name,
      method: asHttpMethod(data.method),
      url: data.url,
      headers: normalizeRows(parsedHeaders),
      params: normalizeRows(parsedParams),
      bodyType: asBodyType(data.bodyType),
      body: data.body,
      authType: normalizedAuth.auth_type,
      authUsername: normalizedAuth.username,
      authPassword: normalizedAuth.password,
      authToken: normalizedAuth.token,
      authApiKey: normalizedAuth.api_key,
      authKeyName: normalizedAuth.api_key_name,
      authKeyIn: normalizedAuth.api_key_in === 'query' ? 'query' : 'header',
      oauth2AccessToken: normalizedAuth.oauth2_access_token,
      oauth2TokenType: normalizedAuth.oauth2_token_type || 'Bearer',
      oauth2RefreshToken: normalizedAuth.oauth2_refresh_token,
      oauth1ConsumerKey: normalizedAuth.oauth1_consumer_key,
      oauth1ConsumerSecret: normalizedAuth.oauth1_consumer_secret,
      oauth1Token: normalizedAuth.oauth1_token,
      oauth1TokenSecret: normalizedAuth.oauth1_token_secret,
      awsAccessKeyId: normalizedAuth.aws_access_key_id,
      awsSecretAccessKey: normalizedAuth.aws_secret_access_key,
      awsRegion: normalizedAuth.aws_region || 'us-east-1',
      awsService: normalizedAuth.aws_service || 'execute-api',
      hawkId: normalizedAuth.hawk_id,
      hawkKey: normalizedAuth.hawk_key,
      hawkAlgorithm: normalizedAuth.hawk_algorithm || 'sha256',
      settingsTimeoutMs: normalizedSettings.timeout_ms,
      settingsConnectTimeoutMs: normalizedSettings.connect_timeout_ms,
      settingsFollowRedirects: normalizedSettings.follow_redirects,
      settingsVerifySsl: normalizedSettings.verify_ssl,
      settingsProxyUrl: normalizedSettings.proxy_url,
      settingsUseCookieJar: normalizedSettings.use_cookie_jar,
      preRequestScript: data.preRequestScript || '',
      testScript: data.testScript || '',
      dirty: false,
    }
  }

  function loadFromStore(data: SavedRequestData) {
    persistActiveDraft()
    const existing = requestTabs.find((tab) => tab.requestId === data.id)
    if (existing) {
      switchRequestTab(existing.tabId)
      return
    }

    const draft = parseSavedDraft(data)
    requestTabs = [...requestTabs, draft]
    activeRequestTabId = draft.tabId
    applyDraft(draft)
  }

  async function openSaveDialog() {
    try {
      collections = await invoke<Collection[]>('list_collections')
    } catch {
      collections = []
    }
    saveCollectionId = currentCollectionId || collections[0]?.id || ''
    if (!saveName.trim()) saveName = `${method} ${url}`
    showSaveDialog = true
  }

  function saveOrPrompt() {
    if (currentRequestId && currentCollectionId && saveName.trim()) {
      saveRequest()
    } else {
      openSaveDialog()
    }
  }

  async function saveRequest() {
    if (!saveCollectionId || !saveName.trim()) return

    const authConfigStr = JSON.stringify(buildAuthConfig())
    const headersStr = JSON.stringify(headers)
    const paramsStr = JSON.stringify(params)
    const settingsStr = JSON.stringify(buildRequestSettings())

    try {
      if (currentRequestId) {
        await invoke('update_request', {
          id: currentRequestId,
          name: saveName.trim(),
          method,
          url,
          headers: headersStr,
          params: paramsStr,
          bodyType,
          body,
          authType,
          authConfig: authConfigStr,
          preRequestScript,
          testScript,
          settings: settingsStr,
        })
      } else {
        const req = await invoke<{ id: string }>('create_request', {
          collectionId: saveCollectionId,
          name: saveName.trim(),
          method,
          url,
          headers: headersStr,
          params: paramsStr,
          bodyType,
          body,
          authType,
          authConfig: authConfigStr,
          preRequestScript,
          testScript,
          settings: settingsStr,
        })
        currentRequestId = req.id
      }
      currentCollectionId = saveCollectionId
      currentDirty = false
      showSaveDialog = false
      persistActiveDraft()
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
    } catch (e) {
      error = String(e)
      persistActiveDraft()
    }
  }

  function collectionChildren(parentId: string | null): Collection[] {
    return collections
      .filter((collection) => (collection.parent_id ?? null) === parentId)
      .sort((a, b) => (a.sort_order ?? 0) - (b.sort_order ?? 0) || a.name.localeCompare(b.name))
  }

  function flattenCollectionOptions(parentId: string | null = null, depth = 0): { id: string; label: string }[] {
    const options: { id: string; label: string }[] = []
    for (const collection of collectionChildren(parentId)) {
      options.push({ id: collection.id, label: `${'  '.repeat(depth)}${collection.name}` })
      options.push(...flattenCollectionOptions(collection.id, depth + 1))
    }
    return options
  }

  let collectionOptions = $derived(flattenCollectionOptions())

  function openHistoryEntry(entry: HistoryEntry) {
    persistActiveDraft()
    let draft: RequestDraft | null = null
    try {
      const config = requestConfigFromUnknown(JSON.parse(entry.request_snapshot || '{}'))
      if (config) {
        draft = draftFromRequestConfig(config, `${config.method} ${shortUrl(config.url) || 'History request'}`, true)
      }
    } catch {
      draft = null
    }

    if (!draft) {
      draft = createBlankDraft()
      draft.method = asHttpMethod(entry.method)
      draft.url = entry.url
      draft.name = `${draft.method} ${shortUrl(entry.url)}`
      draft.dirty = true
    }
    requestTabs = [...requestTabs, draft]
    activeRequestTabId = draft.tabId
    applyDraft(draft)
  }

  const unsubLoad = loadRequestStore.subscribe((data) => {
    if (data) loadFromStore(data)
  })
  const unsubEnv = activeEnvironmentStore.subscribe((env) => {
    activeEnvVars = env?.variables || []
  })

  $effect(() => {
    const newRequestHandler = (event: Event) => {
      const detail = (event as CustomEvent<{ collectionId?: string }>).detail
      newRequestTab(detail?.collectionId ?? null)
    }
    const sendHandler = () => sendRequest()
    const saveHandler = () => saveOrPrompt()
    const duplicateHandler = () => duplicateActiveTab()
    const closeHandler = () => closeRequestTab()
    const historyHandler = (event: Event) => openHistoryEntry((event as CustomEvent<HistoryEntry>).detail)
    const importCurlHandler = () => openImportCurlDialog()
    const codeSnippetsHandler = () => openCodeDialog()

    window.addEventListener('900api:new-request', newRequestHandler)
    window.addEventListener('900api:send-request', sendHandler)
    window.addEventListener('900api:save-request', saveHandler)
    window.addEventListener('900api:duplicate-request-tab', duplicateHandler)
    window.addEventListener('900api:close-request-tab', closeHandler)
    window.addEventListener('900api:open-history', historyHandler)
    window.addEventListener('900api:import-curl', importCurlHandler)
    window.addEventListener('900api:open-code-snippets', codeSnippetsHandler)
    return () => {
      unsubLoad()
      unsubEnv()
      window.removeEventListener('900api:new-request', newRequestHandler)
      window.removeEventListener('900api:send-request', sendHandler)
      window.removeEventListener('900api:save-request', saveHandler)
      window.removeEventListener('900api:duplicate-request-tab', duplicateHandler)
      window.removeEventListener('900api:close-request-tab', closeHandler)
      window.removeEventListener('900api:open-history', historyHandler)
      window.removeEventListener('900api:import-curl', importCurlHandler)
      window.removeEventListener('900api:open-code-snippets', codeSnippetsHandler)
    }
  })
</script>

<div class="flex h-full min-h-0 flex-col {embedded ? '' : 'bg-bg'}">
  <datalist id="env-variable-suggestions">
    {#each envSuggestions as suggestion (suggestion)}
      <option value={suggestion}>{suggestion}</option>
    {/each}
  </datalist>

  <div class="flex h-10 items-center gap-1 overflow-x-auto border-b border-border bg-surface/40 px-2">
    {#each requestTabs as tab (tab.tabId)}
      <div class="group flex max-w-64 shrink-0 items-center rounded-md border border-border bg-surface text-sm {tab.tabId === activeRequestTabId ? 'border-accent' : ''}">
        <button class="flex min-w-0 items-center gap-2 px-2 py-1.5" onclick={() => switchRequestTab(tab.tabId)}>
          <span class="text-xs font-semibold {methodColors[tabMethod(tab)]}">{tabMethod(tab)}</span>
          <span class="truncate">{tabTitle(tab)}</span>
          {#if tabDirty(tab)}
            <span class="text-accent">*</span>
          {/if}
        </button>
        <button class="rounded p-1 text-text-muted hover:bg-surface-hover hover:text-text" onclick={() => closeRequestTab(tab.tabId)} title="Close tab">
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    {/each}
    <button class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text" title="New request" onclick={() => newRequestTab()}>
      <Plus class="h-4 w-4" />
    </button>
  </div>

  <div class="flex items-center gap-2 border-b border-border p-3">
    <input
      type="text"
      class="w-52 rounded-md border border-border bg-surface px-3 py-2 text-sm font-medium outline-none focus:border-accent"
      bind:value={saveName}
      oninput={markDirty}
      placeholder="Request name"
    />
    <select
      class="rounded-md border border-border bg-surface px-3 py-2 text-sm font-medium {methodColors[method]}"
      bind:value={method}
      onchange={markDirty}
    >
      {#each methods as m (m)}
        <option value={m}>{m}</option>
      {/each}
    </select>
    <input
      type="text"
      class="min-w-0 flex-1 rounded-md border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent"
      placeholder="https://api.example.com/endpoint"
      bind:value={url}
      list="env-variable-suggestions"
      oninput={markDirty}
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
      onclick={saveOrPrompt}
      title="Save to collection"
    >
      <Save class="h-4 w-4" />
    </button>
  </div>

  <div class="flex border-b border-border">
    {#each configTabs as tab (tab)}
      <button
        class="px-4 py-2 text-sm transition-colors {configTab === tab ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
        onclick={() => (configTab = tab as typeof configTab)}
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
        {#if tab === 'scripts' && (preRequestScript.trim() || testScript.trim())}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">on</span>
        {/if}
        {#if tab === 'settings' && (settingsUseCookieJar || !settingsFollowRedirects || !settingsVerifySsl || settingsProxyUrl.trim() || settingsTimeoutMs !== 120000)}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">custom</span>
        {/if}
      </button>
    {/each}
    <button class="ml-auto flex items-center gap-1 px-3 py-2 text-xs text-text-muted hover:text-text" onclick={openImportCurlDialog} title="Import cURL">
      <Upload class="h-3.5 w-3.5" />
      Import
    </button>
    <button class="flex items-center gap-1 px-3 py-2 text-xs text-text-muted hover:text-text" onclick={openCodeDialog} title="Generate code snippets">
      <Code2 class="h-3.5 w-3.5" />
      Code
    </button>
    <button class="flex items-center gap-1 px-3 py-2 text-xs text-text-muted hover:text-text" onclick={copyCurl} title="Copy cURL">
      <FileCode2 class="h-3.5 w-3.5" />
      cURL
    </button>
  </div>

  {#if envSuggestions.length > 0}
    <div class="flex flex-wrap items-center gap-1 border-b border-border px-3 py-2">
      {#each envSuggestions.slice(0, 8) as suggestion (suggestion)}
        <button class="rounded border border-border bg-surface px-2 py-1 font-mono text-xs text-text-muted hover:bg-surface-hover hover:text-text" onclick={() => navigator.clipboard.writeText(suggestion).catch((e) => console.error('[900api] clipboard write failed:', e))}>
          {suggestion}
        </button>
      {/each}
    </div>
  {/if}

  <div class="flex min-h-0 flex-1 flex-col">
    <div class="min-h-56 flex-1 overflow-y-auto p-3">
      {#if configTab === 'params'}
        <div class="space-y-2">
          {#each params as param, i (i)}
            <div class="flex items-center gap-2">
              <input type="checkbox" bind:checked={param.enabled} class="accent-accent" onchange={markDirty} />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="key"
                bind:value={param.key}
                oninput={markDirty}
              />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="value"
                bind:value={param.value}
                list="env-variable-suggestions"
                oninput={markDirty}
              />
              <button class="text-text-muted hover:text-error" onclick={() => removeParam(i)}>x</button>
            </div>
          {/each}
          <button class="text-sm text-accent hover:text-accent-hover" onclick={addParam}>+ Add Param</button>
        </div>
      {:else if configTab === 'headers'}
        <div class="space-y-2">
          {#each headers as header, i (i)}
            <div class="flex items-center gap-2">
              <input type="checkbox" bind:checked={header.enabled} class="accent-accent" onchange={markDirty} />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="Header name"
                bind:value={header.key}
                oninput={markDirty}
              />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="Value"
                bind:value={header.value}
                list="env-variable-suggestions"
                oninput={markDirty}
              />
              <button class="text-text-muted hover:text-error" onclick={() => removeHeader(i)}>x</button>
            </div>
          {/each}
          <button class="text-sm text-accent hover:text-accent-hover" onclick={addHeader}>+ Add Header</button>
        </div>
      {:else if configTab === 'body'}
        <div class="space-y-3">
          <div class="flex flex-wrap gap-2">
            {#each bodyTypes as bt (bt)}
              <button
                class="rounded px-3 py-1 text-xs transition-colors {bodyType === bt ? 'bg-accent text-white' : 'bg-surface text-text-muted hover:text-text'}"
                onclick={() => selectBodyType(bt)}
              >
                {bt === 'x_www_form_urlencoded' ? 'x-www-form-urlencoded' : bt === 'form_data' ? 'form-data' : bt}
              </button>
            {/each}
          </div>
          {#if bodyType === 'form_data' || bodyType === 'x_www_form_urlencoded'}
            <div class="space-y-2">
              {#each bodyFields as field, i (i)}
                <div class="flex items-center gap-2">
                  <input type="checkbox" bind:checked={field.enabled} class="accent-accent" onchange={syncBodyFields} />
                  <input
                    type="text"
                    class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                    placeholder="Field name"
                    bind:value={field.key}
                    oninput={syncBodyFields}
                  />
                  <input
                    type="text"
                    class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                    placeholder="Value"
                    bind:value={field.value}
                    list="env-variable-suggestions"
                    oninput={syncBodyFields}
                  />
                  <button class="text-text-muted hover:text-error" aria-label="Remove body field" title="Remove body field" onclick={() => removeBodyField(i)}>x</button>
                </div>
              {/each}
              <button class="text-sm text-accent hover:text-accent-hover" onclick={addBodyField}>+ Add Field</button>
              {#if bodyType === 'form_data'}
                <p class="text-xs text-text-muted">Multipart fields are text only. File parts are not stored in portable collections.</p>
              {/if}
            </div>
          {:else if bodyType !== 'none'}
            <textarea
              class="h-64 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
              placeholder={bodyType === 'json' ? '{\n  "key": "value"\n}' : 'Enter body content...'}
              bind:value={body}
              oninput={markDirty}
            ></textarea>
          {/if}
        </div>
      {:else if configTab === 'auth'}
        <div class="space-y-3">
          <div class="flex flex-wrap gap-2">
            {#each authTypes as at (at)}
              <button
                class="rounded px-3 py-1 text-xs transition-colors {authType === at ? 'bg-accent text-white' : 'bg-surface text-text-muted hover:text-text'}"
                onclick={() => { authType = at as AuthType; markDirty() }}
              >
                {at === 'api_key' ? 'API Key' : at === 'o_auth2' ? 'OAuth 2.0' : at === 'o_auth1' ? 'OAuth 1.0a' : at === 'aws_sig_v4' ? 'AWS Sig v4' : at.charAt(0).toUpperCase() + at.slice(1)}
              </button>
            {/each}
          </div>
          {#if authType === 'basic'}
            <div class="space-y-2">
              <input type="text" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent" placeholder="Username" bind:value={authUsername} list="env-variable-suggestions" oninput={markDirty} />
              <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent" placeholder="Password" bind:value={authPassword} list="env-variable-suggestions" oninput={markDirty} />
            </div>
          {:else if authType === 'bearer'}
            <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Token" bind:value={authToken} list="env-variable-suggestions" oninput={markDirty} />
          {:else if authType === 'api_key'}
            <div class="space-y-2">
              <div class="flex gap-2">
                <input type="text" class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent" placeholder="Key name" bind:value={authKeyName} oninput={markDirty} />
                <select class="rounded border border-border bg-surface px-3 py-2 text-sm" bind:value={authKeyIn} onchange={markDirty}>
                  <option value="header">Header</option>
                  <option value="query">Query Param</option>
                </select>
              </div>
              <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="API Key value" bind:value={authApiKey} list="env-variable-suggestions" oninput={markDirty} />
            </div>
          {:else if authType === 'o_auth2'}
            <div class="space-y-2">
              <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Access Token" bind:value={oauth2AccessToken} list="env-variable-suggestions" oninput={markDirty} />
              <div class="flex gap-2">
                <select class="rounded border border-border bg-surface px-3 py-2 text-sm" bind:value={oauth2TokenType} onchange={markDirty}>
                  <option value="Bearer">Bearer</option>
                  <option value="token">Token</option>
                </select>
                <input type="password" class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Refresh Token" bind:value={oauth2RefreshToken} list="env-variable-suggestions" oninput={markDirty} />
              </div>
            </div>
          {:else if authType === 'o_auth1'}
            <div class="space-y-2">
              <input type="text" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Consumer Key" bind:value={oauth1ConsumerKey} oninput={markDirty} />
              <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Consumer Secret" bind:value={oauth1ConsumerSecret} list="env-variable-suggestions" oninput={markDirty} />
              <input type="text" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Token" bind:value={oauth1Token} list="env-variable-suggestions" oninput={markDirty} />
              <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Token Secret" bind:value={oauth1TokenSecret} list="env-variable-suggestions" oninput={markDirty} />
            </div>
          {:else if authType === 'aws_sig_v4'}
            <div class="space-y-2">
              <input type="text" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Access Key ID" bind:value={awsAccessKeyId} list="env-variable-suggestions" oninput={markDirty} />
              <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Secret Access Key" bind:value={awsSecretAccessKey} list="env-variable-suggestions" oninput={markDirty} />
              <div class="flex gap-2">
                <input type="text" class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Region" bind:value={awsRegion} oninput={markDirty} />
                <input type="text" class="flex-1 rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Service" bind:value={awsService} oninput={markDirty} />
              </div>
            </div>
          {:else if authType === 'hawk'}
            <div class="space-y-2">
              <input type="text" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Hawk ID" bind:value={hawkId} oninput={markDirty} />
              <input type="password" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm font-mono outline-none focus:border-accent" placeholder="Hawk Key" bind:value={hawkKey} list="env-variable-suggestions" oninput={markDirty} />
              <select class="w-full rounded border border-border bg-surface px-3 py-2 text-sm" bind:value={hawkAlgorithm} onchange={markDirty}>
                <option value="sha256">SHA-256</option>
                <option value="sha1">SHA-1</option>
              </select>
            </div>
          {/if}
        </div>
      {:else if configTab === 'scripts'}
        <div class="grid gap-4 lg:grid-cols-2">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase text-text-muted">Pre-request script</span>
            <textarea
              class="h-64 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
              placeholder={preRequestScriptPlaceholder}
              bind:value={preRequestScript}
              oninput={markDirty}
            ></textarea>
          </label>
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase text-text-muted">Test script</span>
            <textarea
              class="h-64 w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
              placeholder={testScriptPlaceholder}
              bind:value={testScript}
              oninput={markDirty}
            ></textarea>
          </label>
          <p class="lg:col-span-2 text-xs text-text-muted">
            Scripts run through the Rust scripting sandbox before and after REST sends. They are saved with the request and are not evaluated by browser JavaScript.
          </p>
        </div>
      {:else if configTab === 'settings'}
        <div class="grid gap-4 lg:grid-cols-2">
          <div class="rounded border border-border bg-surface p-3">
            <h3 class="mb-3 text-sm font-medium">Network</h3>
            <div class="grid gap-3 sm:grid-cols-2">
              <label class="space-y-1 text-xs text-text-muted">
                <span>Request timeout (ms)</span>
                <input
                  type="number"
                  min="1"
                  max="600000"
                  step="1000"
                  class="w-full rounded border border-border bg-bg px-3 py-2 text-sm font-mono text-text outline-none focus:border-accent"
                  bind:value={settingsTimeoutMs}
                  oninput={markDirty}
                />
              </label>
              <label class="space-y-1 text-xs text-text-muted">
                <span>Connect timeout (ms)</span>
                <input
                  type="number"
                  min="1"
                  max={settingsTimeoutMs}
                  step="1000"
                  class="w-full rounded border border-border bg-bg px-3 py-2 text-sm font-mono text-text outline-none focus:border-accent"
                  bind:value={settingsConnectTimeoutMs}
                  oninput={markDirty}
                />
              </label>
            </div>
            <label class="mt-3 block space-y-1 text-xs text-text-muted">
              <span>Proxy URL</span>
              <input
                type="text"
                class="w-full rounded border border-border bg-bg px-3 py-2 text-sm font-mono text-text outline-none focus:border-accent"
                placeholder="http://127.0.0.1:8080"
                bind:value={settingsProxyUrl}
                oninput={markDirty}
              />
            </label>
          </div>

          <div class="rounded border border-border bg-surface p-3">
            <h3 class="mb-3 text-sm font-medium">Runtime</h3>
            <div class="space-y-3">
              <label class="flex items-center justify-between gap-4 rounded border border-border bg-bg px-3 py-2">
                <span class="text-sm">Follow redirects</span>
                <input type="checkbox" class="accent-accent" bind:checked={settingsFollowRedirects} onchange={markDirty} />
              </label>
              <label class="flex items-center justify-between gap-4 rounded border border-border bg-bg px-3 py-2">
                <span class="text-sm">Verify SSL certificates</span>
                <input type="checkbox" class="accent-accent" bind:checked={settingsVerifySsl} onchange={markDirty} />
              </label>
              <label class="flex items-center justify-between gap-4 rounded border border-border bg-bg px-3 py-2">
                <span class="text-sm">Use local cookie jar</span>
                <input type="checkbox" class="accent-accent" bind:checked={settingsUseCookieJar} onchange={markDirty} />
              </label>
            </div>
          </div>
        </div>
      {/if}
    </div>

    {#if error}
      <div class="border-t border-border bg-error/10 p-3 text-sm text-error">{error}</div>
    {/if}
    {#if response}
      <div class="border-t border-border bg-bg">
        <div class="flex flex-wrap items-center gap-2 border-b border-border px-4 py-2 text-sm">
          <span class="font-semibold {response.status < 300 ? 'text-success' : response.status < 400 ? 'text-warning' : 'text-error'}">
            {response.status} {response.status_text}
          </span>
          <span class="text-text-muted">{response.time_ms} ms</span>
          <span class="text-text-muted">{formatSize(response.size_bytes)}</span>
          {#if contentTypeHeader()}
            <span class="max-w-72 truncate rounded border border-border bg-surface px-2 py-1 font-mono text-xs text-text-muted">{contentTypeHeader()}</span>
          {/if}
          <div class="ml-auto flex flex-wrap items-center gap-1">
            <button class="flex items-center gap-1 rounded px-2 py-1.5 text-xs text-text-muted transition-colors hover:bg-surface-hover hover:text-text" onclick={copyResponse} title="Copy raw response body">
              <Copy class="h-3.5 w-3.5" />
              Copy
            </button>
            <button class="flex items-center gap-1 rounded px-2 py-1.5 text-xs text-text-muted transition-colors hover:bg-surface-hover hover:text-text" onclick={downloadResponse} title="Export response body">
              <Download class="h-3.5 w-3.5" />
              Export
            </button>
            <button class="flex items-center gap-1 rounded px-2 py-1.5 text-xs text-text-muted transition-colors hover:bg-surface-hover hover:text-text" onclick={openSaveExampleDialog} title="Save response example">
              <ListChecks class="h-3.5 w-3.5" />
              Example
            </button>
          </div>
        </div>
        {#if responseActionMessage}
          <div class="border-b border-border bg-surface px-4 py-2 text-xs {responseActionMessage.startsWith('Response') ? 'text-success' : 'text-warning'}">{responseActionMessage}</div>
        {/if}
        {#if scriptExecutionMessage}
          <div class="border-b border-border bg-success/10 px-4 py-2 text-xs text-success">{scriptExecutionMessage}</div>
        {/if}
        <div class="flex border-b border-border px-3">
          {#each responseTabs as tab (tab.value)}
            <button
              class="px-4 py-2 text-sm transition-colors {responseTab === tab.value ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
              onclick={() => showResponseTab(tab.value)}
            >
              {tab.label}
              {#if tab.value === 'examples' && responseExamples.length > 0}
                <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{responseExamples.length}</span>
              {/if}
              {#if tab.value === 'compare' && responseExamples.length > 0}
                <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{responseExamples.length}</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>
      <div class="h-80 overflow-y-auto border-t border-border bg-surface p-3">
        {#if responseTab === 'body'}
          <div class="mb-3 flex flex-wrap items-center gap-2">
            <div class="flex rounded border border-border bg-bg p-0.5">
              {#each responseBodyModes as mode (mode.value)}
                <button
                  class="flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors {responseBodyMode === mode.value ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}"
                  onclick={() => { responseBodyMode = mode.value; persistActiveDraft() }}
                  disabled={mode.value === 'preview' && !responseCanPreview()}
                  title={mode.value === 'preview' && !responseCanPreview() ? 'Preview is available for HTML responses' : mode.label}
                >
                  {#if mode.value === 'preview'}
                    <Eye class="h-3.5 w-3.5" />
                  {:else}
                    <FileText class="h-3.5 w-3.5" />
                  {/if}
                  {mode.label}
                </button>
              {/each}
            </div>
            {#if responseBodyMode !== 'preview'}
              <div class="ml-auto flex min-w-72 flex-1 items-center gap-2">
                <Search class="h-4 w-4 text-text-muted" />
                <input
                  class="min-w-0 flex-1 rounded border border-border bg-bg px-2 py-1.5 text-sm outline-none focus:border-accent"
                  placeholder="Search response body"
                  bind:value={responseSearch}
                />
                <span class="w-20 text-right text-xs text-text-muted">{responseMatchCount()} matches</span>
              </div>
            {/if}
          </div>
          {#if responseBodyMode === 'preview' && responseCanPreview()}
            <iframe class="h-64 w-full rounded border border-border bg-white" title="Response preview" sandbox="" srcdoc={response.body}></iframe>
          {:else}
            <pre class="whitespace-pre-wrap font-mono text-sm">{#each responseBodyParts() as part, index (index)}<span class={part.match ? 'rounded bg-warning/30 text-warning' : ''}>{part.text}</span>{/each}</pre>
          {/if}
        {:else if responseTab === 'headers'}
          <div class="space-y-1">
            {#each Object.entries(response.headers) as [key, value] (key)}
              <div class="flex gap-2 text-sm">
                <span class="font-medium text-text-muted">{key}:</span>
                <span class="font-mono">{value}</span>
              </div>
            {/each}
          </div>
        {:else if responseTab === 'examples'}
          <div class="space-y-3">
            {#if !currentRequestId}
              <p class="rounded border border-border bg-bg p-3 text-sm text-text-muted">Save this request to a collection before attaching response examples.</p>
            {:else if responseExamplesLoading}
              <div class="flex items-center gap-2 text-sm text-text-muted">
                <LoaderCircle class="h-4 w-4 animate-spin" />
                Loading examples
              </div>
            {:else}
              <div class="flex items-center justify-between gap-3">
                <div class="text-sm text-text-muted">{responseExamples.length} saved example{responseExamples.length === 1 ? '' : 's'}</div>
                <button class="rounded border border-border bg-bg px-3 py-1.5 text-xs text-text-muted hover:text-text" onclick={loadResponseExamples}>Refresh</button>
              </div>
              {#if responseExamplesError}
                <p class="rounded border border-error/30 bg-error/10 p-2 text-sm text-error">{responseExamplesError}</p>
              {/if}
              {#if responseExamples.length === 0}
                <p class="rounded border border-border bg-bg p-3 text-sm text-text-muted">Save a response example to document expected payloads or compare later responses.</p>
              {:else}
                <div class="grid gap-2">
                  {#each responseExamples as example (example.id)}
                    <div class="rounded border border-border bg-bg p-3">
                      <div class="flex items-start gap-3">
                        <div class="min-w-0 flex-1">
                          <div class="flex flex-wrap items-center gap-2">
                            <span class="font-medium">{example.name}</span>
                            <span class="{example.status < 300 ? 'text-success' : example.status < 400 ? 'text-warning' : 'text-error'}">{example.status} {example.status_text}</span>
                            <span class="text-xs text-text-muted">{example.time_ms} ms · {formatSize(example.size_bytes)}</span>
                          </div>
                          <div class="mt-1 text-xs text-text-muted">{exampleSummary(example)}</div>
                        </div>
                        <button class="rounded px-2 py-1 text-xs text-text-muted hover:bg-surface-hover hover:text-text" onclick={() => loadExampleAsResponse(example)}>Load</button>
                        <button class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-error" onclick={() => deleteResponseExample(example.id)} title="Delete example">
                          <Trash2 class="h-3.5 w-3.5" />
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            {/if}
          </div>
        {:else}
          <div class="space-y-3">
            {#if !currentRequestId}
              <p class="rounded border border-border bg-bg p-3 text-sm text-text-muted">Save this request to a collection before comparing against saved response examples.</p>
            {:else if responseExamplesLoading}
              <div class="flex items-center gap-2 text-sm text-text-muted">
                <LoaderCircle class="h-4 w-4 animate-spin" />
                Loading examples
              </div>
            {:else if responseExamples.length === 0}
              <p class="rounded border border-border bg-bg p-3 text-sm text-text-muted">Save a response example first, then compare later responses against it.</p>
            {:else}
              {@const compareExample = selectedCompareExample()}
              {#if compareExample}
              {@const summary = compareSummary(compareExample)}
              {@const headerRows = compareHeaderRows(compareExample)}
              {@const bodyRows = compareBodyRows(compareExample)}
              <div class="flex flex-wrap items-center gap-2">
                <label class="flex min-w-72 flex-1 items-center gap-2 text-sm text-text-muted">
                  <span>Expected</span>
                  <select class="min-w-0 flex-1 rounded border border-border bg-bg px-2 py-1.5 text-sm text-text outline-none focus:border-accent" bind:value={responseCompareExampleId} onchange={persistActiveDraft}>
                    {#each responseExamples as example (example.id)}
                      <option value={example.id}>{example.name} · {example.status} {example.status_text}</option>
                    {/each}
                  </select>
                </label>
                <button class="rounded border border-border bg-bg px-3 py-1.5 text-xs text-text-muted hover:text-text" onclick={loadResponseExamples}>Refresh</button>
              </div>

              <div class="grid gap-2 md:grid-cols-4">
                <div class="rounded border border-border bg-bg p-3">
                  <div class="text-xs text-text-muted">Status</div>
                  <div class="mt-1 text-sm font-medium {summary.statusMatches ? 'text-success' : 'text-error'}">
                    {summary.statusMatches ? 'Matches' : `${compareExample.status} expected, ${response.status} current`}
                  </div>
                </div>
                <div class="rounded border border-border bg-bg p-3">
                  <div class="text-xs text-text-muted">Body</div>
                  <div class="mt-1 text-sm font-medium {summary.bodyMatches ? 'text-success' : 'text-warning'}">
                    {summary.bodyMatches ? 'Matches' : `${summary.bodyDiffs}${summary.bodyDiffs >= 200 ? '+' : ''} changed line${summary.bodyDiffs === 1 ? '' : 's'}`}
                  </div>
                </div>
                <div class="rounded border border-border bg-bg p-3">
                  <div class="text-xs text-text-muted">Headers</div>
                  <div class="mt-1 text-sm font-medium {summary.headerDiffs === 0 ? 'text-success' : 'text-warning'}">
                    {summary.headerDiffs === 0 ? 'Matches' : `${summary.headerDiffs} difference${summary.headerDiffs === 1 ? '' : 's'}`}
                  </div>
                </div>
                <div class="rounded border border-border bg-bg p-3">
                  <div class="text-xs text-text-muted">Size</div>
                  <div class="mt-1 text-sm font-medium {summary.sizeDelta === 0 ? 'text-success' : 'text-text'}">
                    {summary.sizeDelta === 0 ? 'Same size' : `${summary.sizeDelta > 0 ? '+' : '-'}${formatSize(Math.abs(summary.sizeDelta))}`}
                  </div>
                </div>
              </div>

              {#if headerRows.length > 0}
                <div class="rounded border border-border bg-bg">
                  <div class="border-b border-border px-3 py-2 text-sm font-medium">Header Differences</div>
                  <div class="divide-y divide-border">
                    {#each headerRows as row (`${row.kind}:${row.key}`)}
                      <div class="grid gap-2 px-3 py-2 text-xs md:grid-cols-[10rem_minmax(0,1fr)_minmax(0,1fr)]">
                        <span class="font-mono text-text-muted">{row.key}</span>
                        <span class="min-w-0 break-all font-mono text-error">{row.expected || 'missing from expected'}</span>
                        <span class="min-w-0 break-all font-mono text-success">{row.actual || 'missing from current'}</span>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <div class="rounded border border-border bg-bg">
                <div class="border-b border-border px-3 py-2 text-sm font-medium">Body Differences</div>
                {#if bodyRows.length === 0}
                  <p class="p-3 text-sm text-success">Body matches the saved example after response formatting.</p>
                {:else}
                  <div class="divide-y divide-border">
                    {#each bodyRows as row (`${row.line}:${row.kind}`)}
                      <div class="grid gap-2 px-3 py-2 text-xs md:grid-cols-[4rem_minmax(0,1fr)_minmax(0,1fr)]">
                        <span class="font-mono text-text-muted">L{row.line}</span>
                        <pre class="min-w-0 whitespace-pre-wrap break-words rounded bg-error/10 p-2 font-mono text-error">{row.expected || 'missing from expected'}</pre>
                        <pre class="min-w-0 whitespace-pre-wrap break-words rounded bg-success/10 p-2 font-mono text-success">{row.actual || 'missing from current'}</pre>
                      </div>
                    {/each}
                  </div>
                  {#if bodyRows.length >= 200}
                    <p class="border-t border-border px-3 py-2 text-xs text-text-muted">Showing the first 200 changed lines.</p>
                  {/if}
                {/if}
              </div>
            {/if}
            {/if}
          </div>
        {/if}
      </div>
    {:else}
      <div class="border-t border-border p-4 text-sm text-text-muted">Send a request to view the response.</div>
    {/if}
  </div>
</div>

{#if showSaveDialog}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="button" tabindex="0" onclick={() => (showSaveDialog = false)} onkeydown={(e) => e.key === 'Escape' && (showSaveDialog = false)}>
    <div class="w-96 rounded-lg border border-border bg-bg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <h3 class="mb-4 text-lg font-medium">Save Request</h3>
      <div class="space-y-3">
        <div>
          <label for="save-name-input" class="mb-1 block text-xs font-medium text-text-muted">Name</label>
          <input id="save-name-input" type="text" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent" bind:value={saveName} placeholder="Request name" />
        </div>
        <div>
          <label for="save-collection-select" class="mb-1 block text-xs font-medium text-text-muted">Collection</label>
          <select id="save-collection-select" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent" bind:value={saveCollectionId}>
            {#each collectionOptions as option (option.id)}
              <option value={option.id}>{option.label}</option>
            {/each}
          </select>
        </div>
        {#if collectionOptions.length === 0}
          <p class="text-xs text-warning">Create a collection before saving requests.</p>
        {/if}
        <div class="flex justify-end gap-2 pt-2">
          <button class="rounded-md border border-border bg-surface px-4 py-2 text-sm text-text-muted hover:text-text" onclick={() => (showSaveDialog = false)}>
            Cancel
          </button>
          <button class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50" onclick={saveRequest} disabled={!saveName.trim() || !saveCollectionId}>
            {currentRequestId ? 'Update' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if showSaveExampleDialog}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="button" tabindex="0" onclick={() => (showSaveExampleDialog = false)} onkeydown={(e) => e.key === 'Escape' && (showSaveExampleDialog = false)}>
    <div class="w-96 rounded-lg border border-border bg-bg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <h3 class="mb-4 text-lg font-medium">Save Response Example</h3>
      <div class="space-y-3">
        <div>
          <label for="response-example-name-input" class="mb-1 block text-xs font-medium text-text-muted">Name</label>
          <input id="response-example-name-input" type="text" class="w-full rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent" bind:value={responseExampleName} placeholder="200 OK" />
        </div>
        {#if response}
          <div class="rounded border border-border bg-surface p-3 text-sm">
            <div class="{response.status < 300 ? 'text-success' : response.status < 400 ? 'text-warning' : 'text-error'}">{response.status} {response.status_text}</div>
            <div class="mt-1 text-xs text-text-muted">{response.time_ms} ms · {formatSize(response.size_bytes)}</div>
          </div>
        {/if}
        {#if responseExamplesError}
          <p class="rounded border border-error/30 bg-error/10 p-2 text-sm text-error">{responseExamplesError}</p>
        {/if}
        <div class="flex justify-end gap-2 pt-2">
          <button class="rounded-md border border-border bg-surface px-4 py-2 text-sm text-text-muted hover:text-text" onclick={() => (showSaveExampleDialog = false)}>
            Cancel
          </button>
          <button class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50" onclick={saveResponseExample} disabled={!responseExampleName.trim() || !response}>
            Save
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if showImportCurlDialog}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="button" tabindex="0" onclick={() => (showImportCurlDialog = false)} onkeydown={(e) => e.key === 'Escape' && (showImportCurlDialog = false)}>
    <div class="flex max-h-[80vh] w-[42rem] flex-col rounded-lg border border-border bg-bg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <h3 class="mb-4 text-lg font-medium">Import cURL</h3>
      <textarea
        class="min-h-64 rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
        placeholder="Paste a curl command from browser devtools, docs, Postman, or Insomnia"
        bind:value={curlInput}
      ></textarea>
      {#if importCurlError}
        <p class="mt-3 rounded border border-error/30 bg-error/10 p-2 text-sm text-error">{importCurlError}</p>
      {/if}
      <div class="mt-4 flex justify-end gap-2">
        <button class="rounded-md border border-border bg-surface px-4 py-2 text-sm text-text-muted hover:text-text" onclick={() => (showImportCurlDialog = false)}>
          Cancel
        </button>
        <button class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50" onclick={importCurlCommand} disabled={!curlInput.trim()}>
          Import
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showCodeDialog}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="button" tabindex="0" onclick={() => (showCodeDialog = false)} onkeydown={(e) => e.key === 'Escape' && (showCodeDialog = false)}>
    <div class="flex max-h-[82vh] w-[48rem] flex-col rounded-lg border border-border bg-bg p-6 shadow-xl" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
      <div class="mb-4 flex items-center gap-3">
        <h3 class="text-lg font-medium">Code Snippet</h3>
        <select class="ml-auto rounded border border-border bg-surface px-3 py-2 text-sm outline-none focus:border-accent" bind:value={codeLanguage}>
          {#each codeLanguages as item (item.value)}
            <option value={item.value}>{item.label}</option>
          {/each}
        </select>
      </div>
      <pre class="min-h-80 overflow-auto rounded border border-border bg-surface p-3 font-mono text-sm">{generatedCode()}</pre>
      <div class="mt-4 flex justify-end gap-2">
        <button class="rounded-md border border-border bg-surface px-4 py-2 text-sm text-text-muted hover:text-text" onclick={() => (showCodeDialog = false)}>
          Close
        </button>
        <button class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover" onclick={copyGeneratedCode}>
          <Copy class="h-4 w-4" />
          Copy
        </button>
      </div>
    </div>
  </div>
{/if}
