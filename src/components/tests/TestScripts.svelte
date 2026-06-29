<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { Play, CheckCircle, XCircle, AlertCircle } from '@lucide/svelte'

  type TestResult = {
    name: string
    passed: boolean
    message: string
  }

  type ScriptOutput = {
    logs: string[]
    test_results: TestResult[]
    error: string | null
  }

  let script = $state('// Access response via api900.response\n// Example:\n// var status = api900.response.status;\n// if (status === 200) {\n//   api900.expect.pass("Status is 200");\n// } else {\n//   api900.expect.fail("Status is not 200");\n// }')
  let output = $state<ScriptOutput | null>(null)
  let running = $state(false)

  let responseBody = $state('{"message": "hello world"}')
  let responseStatus = $state(200)
  let responseHeaders = $state('{"content-type": "application/json"}')

  async function runTests() {
    running = true
    output = null
    try {
      output = await invoke<ScriptOutput>('run_test_script', {
        script,
        responseBody,
        responseStatus,
        responseHeaders,
      })
    } catch (e) {
      output = {
        logs: [],
        test_results: [],
        error: String(e),
      }
    } finally {
      running = false
    }
  }

  function loadExample() {
    script = `// Check status code
var status = api900.response.status;
if (status === 200) {
  // Pass
} else {
  throw new Error("Expected 200, got " + status);
}

// Check response body contains expected text
var body = api900.response.body;
if (body.indexOf("hello") !== -1) {
  // Body contains "hello"
} else {
  throw new Error("Response body does not contain 'hello'");
}`
  }
</script>

<div class="flex h-full flex-col">
  <div class="flex items-center justify-between border-b border-border p-3">
    <h2 class="text-sm font-medium">Test Scripts</h2>
    <div class="flex gap-2">
      <button
        class="rounded-md border border-border bg-surface px-3 py-1.5 text-xs text-text-muted hover:text-text"
        onclick={loadExample}
      >
        Load Example
      </button>
      <button
        class="flex items-center gap-2 rounded-md bg-accent px-4 py-1.5 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
        onclick={runTests}
        disabled={running || !script.trim()}
      >
        <Play class="h-4 w-4" />
        Run Tests
      </button>
    </div>
  </div>

  <div class="flex flex-1 overflow-hidden">
    <!-- Script Editor -->
    <div class="flex-1 overflow-y-auto p-3">
      <textarea
        class="h-full w-full rounded border border-border bg-surface p-3 font-mono text-sm outline-none focus:border-accent"
        bind:value={script}
      ></textarea>
    </div>

    <!-- Results Panel -->
    <div class="w-96 border-l border-border overflow-y-auto p-3">
      <h3 class="mb-3 text-xs font-medium uppercase text-text-muted">Results</h3>

      {#if running}
        <p class="text-sm text-text-muted">Running...</p>
      {:else if output}
        {#if output.error}
          <div class="mb-3 flex items-start gap-2 rounded-md bg-error/10 p-3 text-sm text-error">
            <AlertCircle class="mt-0.5 h-4 w-4 shrink-0" />
            <span class="font-mono">{output.error}</span>
          </div>
        {/if}

        {#if output.test_results.length === 0 && !output.error}
          <p class="text-sm text-text-muted">No test results. Script ran without errors.</p>
        {/if}

        {#each output.test_results as result, i (i)}
          <div class="mb-2 flex items-start gap-2 rounded-md p-3 text-sm {result.passed ? 'bg-success/10' : 'bg-error/10'}">
            {#if result.passed}
              <CheckCircle class="mt-0.5 h-4 w-4 shrink-0 text-success" />
            {:else}
              <XCircle class="mt-0.5 h-4 w-4 shrink-0 text-error" />
            {/if}
            <div>
              <p class="font-medium">{result.name}</p>
              <p class="text-text-muted">{result.message}</p>
            </div>
          </div>
        {/each}

        {#if output.logs.length > 0}
          <div class="mt-4">
            <h4 class="mb-2 text-xs font-medium uppercase text-text-muted">Logs</h4>
            {#each output.logs as log, i (i)}
              <pre class="mb-1 rounded bg-surface p-2 text-xs font-mono">{log}</pre>
            {/each}
          </div>
        {/if}
      {:else}
        <p class="text-sm text-text-muted">Run a test script to see results.</p>
      {/if}
    </div>
  </div>

  <!-- Mock Response Config (for testing) -->
  <div class="border-t border-border p-3">
    <details>
      <summary class="cursor-pointer text-xs font-medium text-text-muted">Mock Response (for testing)</summary>
      <div class="mt-2 space-y-2">
        <div class="flex gap-2">
          <input
            type="number"
            class="w-24 rounded border border-border bg-surface px-2 py-1 text-sm outline-none focus:border-accent"
            bind:value={responseStatus}
            placeholder="Status"
          />
          <input
            type="text"
            class="flex-1 rounded border border-border bg-surface px-2 py-1 text-sm font-mono outline-none focus:border-accent"
            bind:value={responseHeaders}
            placeholder="Headers JSON"
          />
        </div>
        <textarea
          class="h-20 w-full rounded border border-border bg-surface p-2 text-sm font-mono outline-none focus:border-accent"
          bind:value={responseBody}
          placeholder="Response body"
        ></textarea>
      </div>
    </details>
  </div>
</div>
