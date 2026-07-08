<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { Plus, Trash2, Save } from '@lucide/svelte'
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
  let selectedId = $state<string | null>(null)
  let loading = $state(true)
  let variables = $state<EnvVariable[]>([])
  let newEnvName = $state('')
  let showNewEnv = $state(false)
  let saving = $state(false)

  async function loadEnvironments() {
    try {
      environments = await invoke<Environment[]>('list_environments')
      if (environments.length > 0 && !selectedId) {
        selectEnvironment(environments[0].id)
      }
    } catch {
      // Backend not available
    } finally {
      loading = false
    }
  }

  function selectEnvironment(id: string) {
    selectedId = id
    const env = environments.find((e) => e.id === id)
    if (env) {
      try {
        variables = JSON.parse(env.variables || '[]')
      } catch {
        variables = []
      }
      activeEnvironmentStore.set({
        id: env.id,
        name: env.name,
        variables,
      })
    }
  }

  async function createEnvironment() {
    if (!newEnvName.trim()) return
    try {
      const env = await invoke<Environment>('create_environment', { name: newEnvName.trim() })
      environments = [...environments, env]
      newEnvName = ''
      showNewEnv = false
      selectEnvironment(env.id)
    } catch (e) {
      console.error('Failed to create environment:', e)
    }
  }

  async function deleteEnvironment(id: string) {
    try {
      await invoke('delete_environment', { id })
      environments = environments.filter((e) => e.id !== id)
      if (selectedId === id) {
        selectedId = null
        variables = []
      }
    } catch (e) {
      console.error('Failed to delete environment:', e)
    }
  }

  async function saveVariables() {
    if (!selectedId) return
    saving = true
    try {
      await invoke('update_environment', {
        id: selectedId,
        variables: JSON.stringify(variables),
      })
      // Update local state
      const env = environments.find((e) => e.id === selectedId)
      if (env) {
        env.variables = JSON.stringify(variables)
        activeEnvironmentStore.set({
          id: env.id,
          name: env.name,
          variables,
        })
      }
    } catch (e) {
      console.error('Failed to save environment:', e)
    } finally {
      saving = false
    }
  }

  function addVariable() {
    variables = [...variables, { key: '', value: '', enabled: true }]
  }

  function removeVariable(index: number) {
    variables = variables.filter((_, i) => i !== index)
  }

  $effect(() => {
    loadEnvironments()
  })
</script>

<div class="flex h-full flex-col">
  <div class="flex items-center justify-between border-b border-border p-3">
    <h2 class="text-sm font-medium">Environments</h2>
    <div class="flex gap-1">
      <button
        class="rounded p-1.5 text-text-muted hover:bg-surface-hover hover:text-text"
        title="New Environment"
        onclick={() => (showNewEnv = !showNewEnv)}
      >
        <Plus class="h-4 w-4" />
      </button>
    </div>
  </div>

  {#if showNewEnv}
    <div class="border-b border-border p-2">
      <input
        type="text"
        class="w-full rounded border border-border bg-surface px-2 py-1.5 text-sm outline-none focus:border-accent"
        placeholder="Environment name (e.g. Production)"
        bind:value={newEnvName}
        onkeydown={(e) => e.key === 'Enter' && createEnvironment()}
      />
    </div>
  {/if}

  {#if loading}
    <p class="p-4 text-sm text-text-muted">Loading...</p>
  {:else if environments.length === 0}
    <p class="p-4 text-sm text-text-muted">No environments yet. Create one to manage variables.</p>
  {:else}
    <div class="flex border-b border-border">
      {#each environments as env (env.id)}
        <button
          class="px-4 py-2 text-sm transition-colors {selectedId === env.id ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => selectEnvironment(env.id)}
        >
          {env.name}
        </button>
      {/each}
    </div>

    {#if selectedId}
      <div class="flex items-center justify-between p-3">
        <h3 class="text-xs font-medium uppercase text-text-muted">Variables</h3>
        <div class="flex gap-2">
          <button
            class="flex items-center gap-1 rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
            onclick={saveVariables}
            disabled={saving}
          >
            <Save class="h-3.5 w-3.5" />
            {saving ? 'Saving...' : 'Save'}
          </button>
          <button
            class="rounded p-1.5 text-text-muted hover:text-error"
            onclick={() => deleteEnvironment(selectedId!)}
            title="Delete environment"
          >
            <Trash2 class="h-4 w-4" />
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto px-3 pb-3">
        <div class="space-y-2">
          {#each variables as variable, i (i)}
            <div class="flex items-center gap-2">
              <input type="checkbox" bind:checked={variable.enabled} class="accent-accent" />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="variable_name"
                bind:value={variable.key}
              />
              <input
                type="text"
                class="flex-1 rounded border border-border bg-surface px-2 py-1.5 text-sm font-mono outline-none focus:border-accent"
                placeholder="value"
                bind:value={variable.value}
              />
              <button class="text-text-muted hover:text-error" onclick={() => removeVariable(i)}>
                <Trash2 class="h-3.5 w-3.5" />
              </button>
            </div>
          {/each}
          <button class="text-sm text-accent hover:text-accent-hover" onclick={addVariable}>
            + Add Variable
          </button>
        </div>
      </div>
    {/if}
  {/if}
</div>
