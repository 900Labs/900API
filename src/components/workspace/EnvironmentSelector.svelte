<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { Plus, RefreshCw } from '@lucide/svelte'
  import { activeEnvironmentStore } from '../../lib/stores'

  type Environment = {
    id: string
    name: string
    variables: string
    created_at: string
    updated_at: string
  }

  type EnvVariable = {
    key: string
    value: string
    enabled: boolean
  }

  let environments = $state<Environment[]>([])
  let selectedId = $state('')
  let newEnvName = $state('')
  let showNew = $state(false)
  let loading = $state(false)
  let error = $state<string | null>(null)

  function parseVariables(env: Environment | undefined): EnvVariable[] {
    if (!env) return []
    try {
      return JSON.parse(env.variables || '[]')
    } catch {
      return []
    }
  }

  function selectEnvironment(id: string) {
    selectedId = id
    const env = environments.find((item) => item.id === id)
    if (!env) {
      activeEnvironmentStore.set(null)
      return
    }
    activeEnvironmentStore.set({
      id: env.id,
      name: env.name,
      variables: parseVariables(env),
    })
  }

  async function loadEnvironments() {
    loading = true
    error = null
    try {
      environments = await invoke<Environment[]>('list_environments')
      if (selectedId && environments.some((env) => env.id === selectedId)) {
        selectEnvironment(selectedId)
      } else if (environments.length > 0) {
        selectEnvironment(environments[0].id)
      } else {
        selectedId = ''
        activeEnvironmentStore.set(null)
      }
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function createEnvironment() {
    if (!newEnvName.trim()) return
    try {
      const env = await invoke<Environment>('create_environment', { name: newEnvName.trim() })
      environments = [...environments, env]
      newEnvName = ''
      showNew = false
      selectEnvironment(env.id)
    } catch (e) {
      error = String(e)
    }
  }

  $effect(() => {
    loadEnvironments()

    const refresh = () => loadEnvironments()
    const selectHandler = (event: Event) => {
      const id = (event as CustomEvent<{ id?: string }>).detail?.id
      if (id) selectEnvironment(id)
    }
    window.addEventListener('900api:environments-changed', refresh)
    window.addEventListener('900api:select-environment', selectHandler)
    return () => {
      window.removeEventListener('900api:environments-changed', refresh)
      window.removeEventListener('900api:select-environment', selectHandler)
    }
  })
</script>

<div class="flex items-center gap-2">
  <label for="workspace-environment-select" class="text-xs font-medium text-text-muted">Environment</label>
  <select
    id="workspace-environment-select"
    class="h-8 max-w-48 rounded border border-border bg-surface px-2 text-xs outline-none focus:border-accent"
    bind:value={selectedId}
    onchange={() => selectEnvironment(selectedId)}
    disabled={loading}
  >
    <option value="">No environment</option>
    {#each environments as env (env.id)}
      <option value={env.id}>{env.name}</option>
    {/each}
  </select>
  <button
    class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text"
    title="Refresh environments"
    onclick={loadEnvironments}
  >
    <RefreshCw class="h-3.5 w-3.5" />
  </button>
  <button
    class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text"
    title="New environment"
    onclick={() => (showNew = !showNew)}
  >
    <Plus class="h-3.5 w-3.5" />
  </button>
</div>

{#if showNew}
  <div class="absolute right-3 top-12 z-30 w-72 rounded-md border border-border bg-surface p-3 shadow-xl">
    <label for="quick-env-name" class="mb-1 block text-xs font-medium text-text-muted">Environment name</label>
    <div class="flex gap-2">
      <input
        id="quick-env-name"
        class="min-w-0 flex-1 rounded border border-border bg-bg px-2 py-1.5 text-sm outline-none focus:border-accent"
        placeholder="Development"
        bind:value={newEnvName}
        onkeydown={(event) => event.key === 'Enter' && createEnvironment()}
      />
      <button
        class="rounded bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
        onclick={createEnvironment}
        disabled={!newEnvName.trim()}
      >
        Add
      </button>
    </div>
    {#if error}
      <p class="mt-2 text-xs text-error">{error}</p>
    {/if}
  </div>
{/if}
