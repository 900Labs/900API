<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { Play, Plus, Trash2, CheckCircle, XCircle, LoaderCircle, FlaskConical } from '@lucide/svelte'

  type KeyValue = { key: string; value: string; enabled: boolean }

  type AssertionType = 'status' | 'header' | 'body' | 'body_json_path' | 'response_time' | 'body_contains'
  type AssertionOperator = 'equals' | 'not_equals' | 'contains' | 'not_contains' | 'greater_than' | 'less_than' | 'exists' | 'not_exists'

  type Assertion = {
    id: string
    assertion_type: AssertionType
    target: string
    operator: AssertionOperator
    expected: string
  }

  type TestRequest = {
    method: string
    url: string
    headers: KeyValue[]
    params: KeyValue[]
    body_type: string
    body: string
    auth: Record<string, unknown>
  }

  type TestSuite = {
    id: string
    name: string
    request: TestRequest
    assertions: Assertion[]
    pre_request_script: string
    test_script: string
  }

  type AssertionResult = {
    assertion_id: string
    passed: boolean
    actual: string
    message: string
  }

  type TestSuiteResult = {
    suite_id: string
    suite_name: string
    passed: boolean
    assertions: AssertionResult[]
    response_status: number
    response_time_ms: number
    error: string | null
  }

  type TestRunResult = {
    total: number
    passed: number
    failed: number
    duration_ms: number
    results: TestSuiteResult[]
  }

  let suites = $state<TestSuite[]>([
    {
      id: crypto.randomUUID(),
      name: 'GET /posts/1',
      request: {
        method: 'GET',
        url: 'https://jsonplaceholder.typicode.com/posts/1',
        headers: [],
        params: [],
        body_type: 'none',
        body: '',
        auth: { auth_type: 'none' },
      },
      assertions: [
        {
          id: crypto.randomUUID(),
          assertion_type: 'status',
          target: 'status',
          operator: 'equals',
          expected: '200',
        },
        {
          id: crypto.randomUUID(),
          assertion_type: 'body_json_path',
          target: 'userId',
          operator: 'exists',
          expected: '',
        },
      ],
      pre_request_script: '',
      test_script: '',
    },
  ])
  let selectedSuiteId = $state<string | null>(null)
  let running = $state(false)
  let runResult = $state<TestRunResult | null>(null)
  let error = $state<string | null>(null)

  $effect(() => {
    if (selectedSuiteId === null && suites.length > 0) {
      selectedSuiteId = suites[0].id
    }
  })

  let selectedSuite = $derived(suites.find((s) => s.id === selectedSuiteId))

  const assertionTypes: { value: AssertionType; label: string }[] = [
    { value: 'status', label: 'Status Code' },
    { value: 'header', label: 'Header' },
    { value: 'body', label: 'Body (raw)' },
    { value: 'body_contains', label: 'Body contains' },
    { value: 'body_json_path', label: 'Body JSON path' },
    { value: 'response_time', label: 'Response time (ms)' },
  ]

  const operators: { value: AssertionOperator; label: string }[] = [
    { value: 'equals', label: 'equals' },
    { value: 'not_equals', label: 'not equals' },
    { value: 'contains', label: 'contains' },
    { value: 'not_contains', label: 'not contains' },
    { value: 'greater_than', label: 'greater than' },
    { value: 'less_than', label: 'less than' },
    { value: 'exists', label: 'exists' },
    { value: 'not_exists', label: 'not exists' },
  ]

  function addSuite() {
    const suite: TestSuite = {
      id: crypto.randomUUID(),
      name: 'New Test Suite',
      request: {
        method: 'GET',
        url: 'https://api.example.com/endpoint',
        headers: [],
        params: [],
        body_type: 'none',
        body: '',
        auth: { auth_type: 'none' },
      },
      assertions: [],
      pre_request_script: '',
      test_script: '',
    }
    suites = [...suites, suite]
    selectedSuiteId = suite.id
  }

  function removeSuite(id: string) {
    suites = suites.filter((s) => s.id !== id)
    if (selectedSuiteId === id) {
      selectedSuiteId = suites.length > 0 ? suites[0].id : null
    }
  }

  function addAssertion() {
    if (!selectedSuite) return
    const assertion: Assertion = {
      id: crypto.randomUUID(),
      assertion_type: 'status',
      target: 'status',
      operator: 'equals',
      expected: '200',
    }
    selectedSuite.assertions = [...selectedSuite.assertions, assertion]
  }

  function removeAssertion(id: string) {
    if (!selectedSuite) return
    selectedSuite.assertions = selectedSuite.assertions.filter((a) => a.id !== id)
  }

  async function runAllTests() {
    running = true
    error = null
    runResult = null
    try {
      runResult = await invoke<TestRunResult>('run_test_suites', { suites })
    } catch (e) {
      error = String(e)
    } finally {
      running = false
    }
  }

  function getResultForSuite(suiteId: string): TestSuiteResult | null {
    if (!runResult) return null
    return runResult.results.find((r) => r.suite_id === suiteId) ?? null
  }
</script>

<div class="flex h-full flex-col">
  <!-- Toolbar -->
  <div class="flex items-center gap-3 border-b border-border p-3">
    <FlaskConical class="h-5 w-5 text-text-muted" />
    <span class="text-sm font-medium">Test Runner</span>
    <span class="text-xs text-text-muted">{suites.length} suite(s)</span>
    <div class="flex-1"></div>
    {#if runResult}
      <span class="text-xs {runResult.failed === 0 ? 'text-success' : 'text-error'}">
        {runResult.passed}/{runResult.total} passed in {runResult.duration_ms}ms
      </span>
    {/if}
    <button
      class="flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
      onclick={runAllTests}
      disabled={running || suites.length === 0}
    >
      {#if running}
        <LoaderCircle class="h-4 w-4 animate-spin" />
        Running...
      {:else}
        <Play class="h-4 w-4" />
        Run All
      {/if}
    </button>
  </div>

  {#if error}
    <div class="border-b border-border bg-error/10 p-3 text-sm text-error">{error}</div>
  {/if}

  <div class="flex flex-1 overflow-hidden">
    <!-- Suite List -->
    <div class="w-56 border-r border-border overflow-y-auto">
      <div class="flex items-center justify-between p-2">
        <span class="text-xs font-medium uppercase text-text-muted">Suites</span>
        <button class="text-accent hover:text-accent-hover" onclick={addSuite} title="Add suite">
          <Plus class="h-4 w-4" />
        </button>
      </div>
      {#each suites as suite (suite.id)}
        {@const result = getResultForSuite(suite.id)}
        <button
          class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors {selectedSuiteId === suite.id ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => (selectedSuiteId = suite.id)}
        >
          {#if result}
            {#if result.passed}
              <CheckCircle class="h-4 w-4 text-success" />
            {:else}
              <XCircle class="h-4 w-4 text-error" />
            {/if}
          {:else}
            <span class="h-4 w-4 rounded-full border border-border"></span>
          {/if}
          <span class="flex-1 truncate">{suite.name}</span>
          <span
            class="text-text-muted hover:text-error"
            role="button"
            tabindex="0"
            onclick={(e) => { e.stopPropagation(); removeSuite(suite.id) }}
            onkeydown={(e) => e.key === 'Enter' && (e.stopPropagation(), removeSuite(suite.id))}
          >
            <Trash2 class="h-3.5 w-3.5" />
          </span>
        </button>
      {/each}
    </div>

    <!-- Suite Editor -->
    <div class="flex-1 overflow-y-auto p-4">
      {#if selectedSuite}
        <div class="space-y-4">
          <!-- Suite Name -->
          <input
            type="text"
            class="w-full rounded-md border border-border bg-surface px-3 py-2 text-sm font-medium outline-none focus:border-accent"
            bind:value={selectedSuite.name}
          />

          <!-- Request Config -->
          <div class="rounded-lg border border-border bg-surface p-3">
            <h3 class="mb-2 text-xs font-medium uppercase text-text-muted">Request</h3>
            <div class="flex gap-2">
              <select
                class="rounded border border-border bg-bg px-2 py-2 text-sm font-mono outline-none focus:border-accent"
                bind:value={selectedSuite.request.method}
              >
                {#each ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'] as m}
                  <option value={m}>{m}</option>
                {/each}
              </select>
              <input
                type="text"
                class="flex-1 rounded border border-border bg-bg px-3 py-2 text-sm font-mono outline-none focus:border-accent"
                placeholder="https://api.example.com/endpoint"
                bind:value={selectedSuite.request.url}
              />
            </div>
          </div>

          <!-- Assertions -->
          <div class="rounded-lg border border-border bg-surface p-3">
            <div class="mb-2 flex items-center justify-between">
              <h3 class="text-xs font-medium uppercase text-text-muted">Assertions</h3>
              <button class="text-accent hover:text-accent-hover" onclick={addAssertion}>+ Add</button>
            </div>
            <div class="space-y-2">
              {#each selectedSuite.assertions as assertion, i (assertion.id)}
                {@const result = getResultForSuite(selectedSuite.id)}
                {@const assertionResult = result?.assertions.find((a) => a.assertion_id === assertion.id)}
                <div class="flex items-center gap-2 rounded border border-border bg-bg p-2">
                  {#if assertionResult}
                    {#if assertionResult.passed}
                      <CheckCircle class="h-4 w-4 flex-shrink-0 text-success" />
                    {:else}
                      <XCircle class="h-4 w-4 flex-shrink-0 text-error" />
                    {/if}
                  {/if}
                  <select
                    class="rounded border border-border bg-bg px-2 py-1.5 text-xs outline-none focus:border-accent"
                    bind:value={assertion.assertion_type}
                  >
                    {#each assertionTypes as at}
                      <option value={at.value}>{at.label}</option>
                    {/each}
                  </select>
                  <input
                    type="text"
                    class="w-32 rounded border border-border bg-bg px-2 py-1.5 text-xs font-mono outline-none focus:border-accent"
                    placeholder="target"
                    bind:value={assertion.target}
                  />
                  <select
                    class="rounded border border-border bg-bg px-2 py-1.5 text-xs outline-none focus:border-accent"
                    bind:value={assertion.operator}
                  >
                    {#each operators as op}
                      <option value={op.value}>{op.label}</option>
                    {/each}
                  </select>
                  <input
                    type="text"
                    class="flex-1 rounded border border-border bg-bg px-2 py-1.5 text-xs font-mono outline-none focus:border-accent"
                    placeholder="expected value"
                    bind:value={assertion.expected}
                  />
                  <button class="text-text-muted hover:text-error" onclick={() => removeAssertion(assertion.id)}>
                    <Trash2 class="h-3.5 w-3.5" />
                  </button>
                </div>
                {#if assertionResult && !assertionResult.passed}
                  <div class="ml-6 mb-1 text-xs text-error">{assertionResult.message}</div>
                {/if}
              {/each}
              {#if selectedSuite.assertions.length === 0}
                <p class="text-xs text-text-muted">No assertions. Add one to start testing.</p>
              {/if}
            </div>
          </div>

          <!-- Suite Result -->
          {#if getResultForSuite(selectedSuite.id) !== null}
            {@const result = getResultForSuite(selectedSuite.id)!}
            <div class="rounded-lg border border-border bg-surface p-3">
              <h3 class="mb-2 text-xs font-medium uppercase text-text-muted">Result</h3>
              <div class="flex items-center gap-4 text-sm">
                <span class={result.passed ? 'text-success' : 'text-error'}>
                  {result.passed ? 'PASSED' : 'FAILED'}
                </span>
                <span class="text-text-muted">HTTP {result.response_status}</span>
                <span class="text-text-muted">{result.response_time_ms}ms</span>
                {#if result.error}
                  <span class="text-error">{result.error}</span>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      {:else}
        <div class="flex h-full items-center justify-center text-text-muted">
          <p>Select a test suite or create a new one.</p>
        </div>
      {/if}
    </div>
  </div>
</div>
