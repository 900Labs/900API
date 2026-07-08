<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { Database, LoaderCircle, Search, Send, Wand2 } from '@lucide/svelte'
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

  type GraphQLTypeRef = {
    kind: string
    name: string | null
    of_type: GraphQLTypeRef | null
  }

  type GraphQLInputValue = {
    name: string
    description: string | null
    value_type: GraphQLTypeRef
    default_value: string | null
  }

  type GraphQLField = {
    name: string
    description: string | null
    args: GraphQLInputValue[]
    field_type: GraphQLTypeRef
    is_deprecated: boolean
    deprecation_reason: string | null
  }

  type GraphQLEnumValue = {
    name: string
    description: string | null
    is_deprecated: boolean
    deprecation_reason: string | null
  }

  type GraphQLSchemaType = {
    kind: string
    name: string
    description: string | null
    fields: GraphQLField[]
    input_fields: GraphQLInputValue[]
    enum_values: GraphQLEnumValue[]
    possible_types: GraphQLTypeRef[]
  }

  type GraphQLSchema = {
    query_type: string | null
    mutation_type: string | null
    subscription_type: string | null
    types: GraphQLSchemaType[]
  }

  type RequestTab = 'query' | 'variables' | 'headers' | 'auth' | 'schema'
  type ResponseTab = 'body' | 'headers'
  type OperationKind = 'query' | 'mutation' | 'subscription'

  const requestTabs: RequestTab[] = ['query', 'variables', 'headers', 'auth', 'schema']
  const leafKinds = new Set(['SCALAR', 'ENUM'])

  let url = $state('https://countries.trevorblades.com')
  let query = $state('query {\n  countries {\n    code\n    name\n  }\n}')
  let variables = $state('{}')
  let operationName = $state('')
  let headers = $state<KeyValue[]>([])
  let authType = $state<AuthType>('none')
  let authToken = $state('')
  let activeTab = $state<RequestTab>('query')
  let responseTab = $state<ResponseTab>('body')
  let response = $state<ResponseData>(null)
  let loading = $state(false)
  let schemaLoading = $state(false)
  let error = $state<string | null>(null)
  let schemaError = $state<string | null>(null)
  let schema = $state<GraphQLSchema | null>(null)
  let schemaSearch = $state('')
  let selectedTypeName = $state('')
  let queryAssistSearch = $state('')
  let queryAssistOperation = $state<OperationKind>('query')
  let activeEnvVars = $state<{ key: string; value: string; enabled: boolean }[]>([])

  function addHeader() {
    headers = [...headers, { key: '', value: '', enabled: true }]
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index)
  }

  function buildAuth(): AuthConfig {
    return {
      auth_type: authType,
      username: '',
      password: '',
      token: authToken,
      api_key: '',
      api_key_name: '',
      api_key_in: 'header',
    }
  }

  async function sendGraphQL() {
    loading = true
    error = null
    response = null

    try {
      const result = await invoke<ResponseData>('send_graphql', {
        url,
        query,
        variables,
        operationName: operationName || null,
        headers,
        auth: buildAuth(),
        environmentVariables: activeEnvVars.length > 0 ? activeEnvVars : undefined,
      })
      response = result
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function introspectSchema() {
    schemaLoading = true
    schemaError = null

    try {
      const result = await invoke<GraphQLSchema>('introspect_graphql_schema', {
        url,
        headers,
        auth: buildAuth(),
        environmentVariables: activeEnvVars.length > 0 ? activeEnvVars : undefined,
      })
      schema = result
      selectedTypeName = result.query_type || visibleSchemaTypes(result)[0]?.name || ''
      queryAssistOperation = result.query_type ? 'query' : result.mutation_type ? 'mutation' : result.subscription_type ? 'subscription' : 'query'
      activeTab = 'schema'
    } catch (e) {
      schemaError = String(e)
    } finally {
      schemaLoading = false
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

  function typeRefLabel(ref: GraphQLTypeRef | null): string {
    if (!ref) return ''
    if (ref.kind === 'NON_NULL') return `${typeRefLabel(ref.of_type)}!`
    if (ref.kind === 'LIST') return `[${typeRefLabel(ref.of_type)}]`
    return ref.name || ref.kind
  }

  function namedTypeName(ref: GraphQLTypeRef | null): string {
    if (!ref) return ''
    if (ref.name) return ref.name
    return namedTypeName(ref.of_type)
  }

  function visibleSchemaTypes(source = schema): GraphQLSchemaType[] {
    if (!source) return []
    const needle = schemaSearch.trim().toLowerCase()
    return source.types
      .filter((type) => !type.name.startsWith('__'))
      .filter((type) => {
        if (!needle) return true
        return (
          type.name.toLowerCase().includes(needle) ||
          type.kind.toLowerCase().includes(needle) ||
          type.fields.some((field) => field.name.toLowerCase().includes(needle))
        )
      })
      .sort((a, b) => rootTypeRank(a.name) - rootTypeRank(b.name) || a.name.localeCompare(b.name))
  }

  function rootTypeRank(typeName: string): number {
    if (typeName === schema?.query_type) return 0
    if (typeName === schema?.mutation_type) return 1
    if (typeName === schema?.subscription_type) return 2
    return 3
  }

  function selectedType(): GraphQLSchemaType | null {
    if (!schema) return null
    return schema.types.find((type) => type.name === selectedTypeName) || visibleSchemaTypes()[0] || null
  }

  function typeByName(typeName: string): GraphQLSchemaType | null {
    return schema?.types.find((type) => type.name === typeName) || null
  }

  function isLeafType(ref: GraphQLTypeRef | null): boolean {
    const typeName = namedTypeName(ref)
    const type = typeByName(typeName)
    return !type || leafKinds.has(type.kind)
  }

  function argPlaceholder(arg: GraphQLInputValue): string {
    const typeName = namedTypeName(arg.value_type)
    if (arg.default_value) return arg.default_value
    if (typeName === 'Int' || typeName === 'Float') return '0'
    if (typeName === 'Boolean') return 'false'
    if (typeByName(typeName)?.kind === 'ENUM') return typeByName(typeName)?.enum_values[0]?.name || 'VALUE'
    return `"{{${arg.name}}}"`
  }

  function fieldCall(field: GraphQLField): string {
    if (field.args.length === 0) return field.name
    const args = field.args.map((arg) => `${arg.name}: ${argPlaceholder(arg)}`).join(', ')
    return `${field.name}(${args})`
  }

  function fieldSelection(field: GraphQLField, depth = 1): string {
    const call = fieldCall(field)
    if (isLeafType(field.field_type) || depth > 2) return call

    const type = typeByName(namedTypeName(field.field_type))
    const childFields = (type?.fields || []).filter((child) => isLeafType(child.field_type)).slice(0, 4)
    const indent = '  '.repeat(depth)
    const childSelection = childFields.length > 0 ? childFields.map((child) => child.name).join(`\n${indent}  `) : '__typename'
    return `${call} {\n${indent}  ${childSelection}\n${indent}}`
  }

  function indentSelection(value: string, depth = 1): string {
    const prefix = '  '.repeat(depth)
    return value
      .split('\n')
      .map((line) => `${prefix}${line}`)
      .join('\n')
  }

  function insertField(field: GraphQLField) {
    const type = selectedType()
    const selection = fieldSelection(field)
    if (!type || type.name === schema?.query_type || type.name === schema?.mutation_type || type.name === schema?.subscription_type) {
      const operation = type?.name === schema?.mutation_type ? 'mutation' : type?.name === schema?.subscription_type ? 'subscription' : 'query'
      insertOperationField(field, operation)
    } else {
      const insertion = `${indentSelection(selection)}\n`
      const lastBrace = query.lastIndexOf('}')
      query = lastBrace >= 0 ? `${query.slice(0, lastBrace)}${insertion}${query.slice(lastBrace)}` : `query {\n${insertion}}`
      activeTab = 'query'
    }
  }

  function schemaSummary(): string {
    if (!schema) return ''
    const types = schema.types.filter((type) => !type.name.startsWith('__')).length
    const operations = [schema.query_type, schema.mutation_type, schema.subscription_type].filter(Boolean).join(' / ')
    return `${types} types${operations ? ` - ${operations}` : ''}`
  }

  function operationTypeName(kind: OperationKind): string | null {
    if (!schema) return null
    if (kind === 'mutation') return schema.mutation_type
    if (kind === 'subscription') return schema.subscription_type
    return schema.query_type
  }

  function availableOperations(): { kind: OperationKind; label: string; typeName: string }[] {
    if (!schema) return []
    return [
      { kind: 'query' as OperationKind, label: 'Query', typeName: schema.query_type || '' },
      { kind: 'mutation' as OperationKind, label: 'Mutation', typeName: schema.mutation_type || '' },
      { kind: 'subscription' as OperationKind, label: 'Subscription', typeName: schema.subscription_type || '' },
    ].filter((operation) => operation.typeName)
  }

  function queryAssistType(): GraphQLSchemaType | null {
    return typeByName(operationTypeName(queryAssistOperation) || '')
  }

  function queryAssistFields(): GraphQLField[] {
    const type = queryAssistType()
    if (!type) return []
    const needle = queryAssistSearch.trim().toLowerCase()
    return type.fields
      .filter((field) => {
        if (!needle) return true
        return (
          field.name.toLowerCase().includes(needle) ||
          (field.description || '').toLowerCase().includes(needle) ||
          field.args.some((arg) => arg.name.toLowerCase().includes(needle))
        )
      })
      .slice(0, 40)
  }

  function insertOperationField(field: GraphQLField, kind = queryAssistOperation) {
    const selection = fieldSelection(field)
    query = `${kind} {\n${indentSelection(selection)}\n}`
    operationName = ''
    activeTab = 'query'
  }

  const unsubEnv = activeEnvironmentStore.subscribe((env) => {
    activeEnvVars = env?.variables || []
  })

  $effect(() => {
    return () => unsubEnv()
  })
</script>

<div class="flex h-full flex-col">
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
      class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-2 text-sm font-medium text-text-muted transition-colors hover:text-text disabled:opacity-50"
      onclick={introspectSchema}
      disabled={schemaLoading || !url.trim()}
      title="Fetch GraphQL schema"
    >
      {#if schemaLoading}
        <LoaderCircle class="h-4 w-4 animate-spin" />
      {:else}
        <Database class="h-4 w-4" />
      {/if}
      Schema
    </button>
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

  <div class="flex border-b border-border">
    {#each requestTabs as tab (tab)}
      <button
        class="px-4 py-2 text-sm transition-colors {activeTab === tab ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
        onclick={() => (activeTab = tab)}
      >
        {tab.charAt(0).toUpperCase() + tab.slice(1)}
        {#if tab === 'headers' && headers.filter((h) => h.key.trim()).length > 0}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{headers.filter((h) => h.key.trim()).length}</span>
        {:else if tab === 'schema' && schema}
          <span class="ml-1 rounded bg-surface-hover px-1.5 text-xs">{schema.types.filter((type) => !type.name.startsWith('__')).length}</span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="flex-1 overflow-y-auto p-3">
    {#if activeTab === 'query'}
      <div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_22rem]">
        <div class="min-w-0">
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
        </div>

        <aside class="rounded border border-border bg-surface">
          <div class="flex items-center gap-2 border-b border-border px-3 py-2">
            <Wand2 class="h-4 w-4 text-text-muted" />
            <h3 class="text-sm font-medium">Query Assist</h3>
          </div>
          {#if schema}
            <div class="space-y-3 p-3">
              <div class="flex flex-wrap gap-1">
                {#each availableOperations() as operation (operation.kind)}
                  <button
                    class="rounded px-2 py-1 text-xs {queryAssistOperation === operation.kind ? 'bg-accent text-white' : 'border border-border bg-bg text-text-muted hover:text-text'}"
                    onclick={() => (queryAssistOperation = operation.kind)}
                  >
                    {operation.label}
                  </button>
                {/each}
              </div>
              <div class="flex items-center gap-2 rounded border border-border bg-bg px-2 py-1.5">
                <Search class="h-3.5 w-3.5 text-text-muted" />
                <input
                  class="min-w-0 flex-1 bg-transparent text-sm outline-none"
                  placeholder="Find fields or args"
                  bind:value={queryAssistSearch}
                />
              </div>
              <div class="max-h-72 overflow-y-auto space-y-2">
                {#if queryAssistFields().length === 0}
                  <p class="rounded border border-border bg-bg p-3 text-sm text-text-muted">No matching fields.</p>
                {:else}
                  {#each queryAssistFields() as field (field.name)}
                    <div class="rounded border border-border bg-bg p-2">
                      <div class="flex items-start gap-2">
                        <div class="min-w-0 flex-1">
                          <div class="truncate font-mono text-sm">
                            <span class="text-accent-hover">{field.name}</span>
                            {#if field.args.length > 0}
                              <span class="text-text-muted">({field.args.map((arg) => `${arg.name}: ${typeRefLabel(arg.value_type)}`).join(', ')})</span>
                            {/if}
                          </div>
                          <div class="mt-0.5 truncate font-mono text-xs text-text-muted">{typeRefLabel(field.field_type)}</div>
                          {#if field.description}
                            <p class="mt-1 line-clamp-2 text-xs text-text-muted">{field.description}</p>
                          {/if}
                        </div>
                        <button class="rounded border border-border px-2 py-1 text-xs text-text-muted hover:text-text" onclick={() => insertOperationField(field)}>Insert</button>
                      </div>
                    </div>
                  {/each}
                {/if}
              </div>
            </div>
          {:else}
            <div class="space-y-3 p-3">
              <p class="text-sm text-text-muted">Fetch a schema to search root fields and insert starter operations from the query editor.</p>
              <button
                class="flex w-full items-center justify-center gap-2 rounded border border-border bg-bg px-3 py-2 text-sm text-text-muted hover:text-text disabled:opacity-50"
                onclick={introspectSchema}
                disabled={schemaLoading || !url.trim()}
              >
                {#if schemaLoading}
                  <LoaderCircle class="h-4 w-4 animate-spin" />
                {:else}
                  <Database class="h-4 w-4" />
                {/if}
                Fetch schema
              </button>
              {#if schemaError}
                <p class="rounded border border-error/30 bg-error/10 p-2 text-sm text-error">{schemaError}</p>
              {/if}
            </div>
          {/if}
        </aside>
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
            <button class="text-text-muted hover:text-error" onclick={() => removeHeader(i)}>X</button>
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
    {:else if activeTab === 'schema'}
      <div class="flex min-h-[24rem] flex-col gap-3">
        <div class="flex flex-wrap items-center gap-2">
          <button
            class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-2 text-sm font-medium text-text-muted hover:text-text disabled:opacity-50"
            onclick={introspectSchema}
            disabled={schemaLoading || !url.trim()}
          >
            {#if schemaLoading}
              <LoaderCircle class="h-4 w-4 animate-spin" />
            {:else}
              <Database class="h-4 w-4" />
            {/if}
            Fetch schema
          </button>
          {#if schema}
            <span class="text-sm text-text-muted">{schemaSummary()}</span>
          {/if}
          {#if schemaError}
            <span class="text-sm text-error">{schemaError}</span>
          {/if}
        </div>

        {#if schema}
          <div class="grid min-h-[22rem] gap-3 lg:grid-cols-[18rem_minmax(0,1fr)]">
            <div class="min-h-0 overflow-hidden rounded border border-border bg-surface">
              <div class="flex items-center gap-2 border-b border-border px-3 py-2">
                <Search class="h-4 w-4 text-text-muted" />
                <input
                  class="min-w-0 flex-1 bg-transparent text-sm outline-none"
                  placeholder="Search schema"
                  bind:value={schemaSearch}
                />
              </div>
              <div class="max-h-[26rem] overflow-y-auto p-1">
                {#each visibleSchemaTypes() as type (type.name)}
                  <button
                    class="flex w-full items-center justify-between gap-2 rounded px-2 py-1.5 text-left text-sm {selectedType()?.name === type.name ? 'bg-surface-hover text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'}"
                    onclick={() => (selectedTypeName = type.name)}
                  >
                    <span class="min-w-0 truncate">{type.name}</span>
                    <span class="shrink-0 text-[10px] uppercase text-text-muted">{type.kind}</span>
                  </button>
                {/each}
              </div>
            </div>

            <div class="min-h-0 overflow-y-auto rounded border border-border bg-surface p-4">
              {#if selectedType()}
                <div class="mb-4">
                  <div class="flex flex-wrap items-center gap-2">
                    <h2 class="text-base font-semibold">{selectedType()?.name}</h2>
                    <span class="rounded bg-surface-hover px-2 py-0.5 text-xs uppercase text-text-muted">{selectedType()?.kind}</span>
                    {#if selectedType()?.name === schema.query_type}
                      <span class="rounded bg-accent/10 px-2 py-0.5 text-xs text-accent-hover">Query root</span>
                    {:else if selectedType()?.name === schema.mutation_type}
                      <span class="rounded bg-warning/10 px-2 py-0.5 text-xs text-warning">Mutation root</span>
                    {:else if selectedType()?.name === schema.subscription_type}
                      <span class="rounded bg-success/10 px-2 py-0.5 text-xs text-success">Subscription root</span>
                    {/if}
                  </div>
                  {#if selectedType()?.description}
                    <p class="mt-2 text-sm text-text-muted">{selectedType()?.description}</p>
                  {/if}
                </div>

                {#if (selectedType()?.fields || []).length > 0}
                  <div class="space-y-2">
                    <h3 class="text-xs font-semibold uppercase text-text-muted">Fields</h3>
                    {#each selectedType()?.fields || [] as field (field.name)}
                      <div class="rounded border border-border bg-bg p-3">
                        <div class="flex flex-wrap items-start justify-between gap-2">
                          <div class="min-w-0">
                            <div class="flex flex-wrap items-center gap-1 font-mono text-sm">
                              <span class="text-accent-hover">{field.name}</span>
                              {#if field.args.length > 0}
                                <span class="text-text-muted">({field.args.map((arg) => `${arg.name}: ${typeRefLabel(arg.value_type)}`).join(', ')})</span>
                              {/if}
                              <span class="text-text-muted">: {typeRefLabel(field.field_type)}</span>
                            </div>
                            {#if field.description}
                              <p class="mt-1 text-sm text-text-muted">{field.description}</p>
                            {/if}
                            {#if field.is_deprecated}
                              <p class="mt-1 text-xs text-warning">Deprecated{field.deprecation_reason ? `: ${field.deprecation_reason}` : ''}</p>
                            {/if}
                          </div>
                          <button class="rounded border border-border px-2 py-1 text-xs text-text-muted hover:text-text" onclick={() => insertField(field)}>Insert</button>
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}

                {#if (selectedType()?.input_fields || []).length > 0}
                  <div class="mt-4 space-y-2">
                    <h3 class="text-xs font-semibold uppercase text-text-muted">Input fields</h3>
                    {#each selectedType()?.input_fields || [] as field (field.name)}
                      <div class="font-mono text-sm">
                        <span class="text-accent-hover">{field.name}</span>
                        <span class="text-text-muted">: {typeRefLabel(field.value_type)}</span>
                        {#if field.default_value}
                          <span class="text-text-muted"> = {field.default_value}</span>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {/if}

                {#if (selectedType()?.enum_values || []).length > 0}
                  <div class="mt-4 space-y-2">
                    <h3 class="text-xs font-semibold uppercase text-text-muted">Enum values</h3>
                    <div class="flex flex-wrap gap-2">
                      {#each selectedType()?.enum_values || [] as value (value.name)}
                        <span class="rounded border border-border px-2 py-1 font-mono text-xs text-text-muted">{value.name}</span>
                      {/each}
                    </div>
                  </div>
                {/if}
              {:else}
                <p class="text-sm text-text-muted">No schema type selected.</p>
              {/if}
            </div>
          </div>
        {:else}
          <div class="rounded border border-border bg-surface p-4 text-sm text-text-muted">
            Fetch a schema to inspect available GraphQL types, fields, arguments, and enums.
          </div>
        {/if}
      </div>
    {/if}
  </div>

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
