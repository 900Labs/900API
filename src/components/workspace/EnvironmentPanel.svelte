<script lang="ts">
  import { activeEnvironmentStore } from '../../lib/stores'

  let activeName = $state<string | null>(null)
  let variables = $state<{ key: string; value: string; enabled: boolean }[]>([])

  const unsubscribe = activeEnvironmentStore.subscribe((env) => {
    activeName = env?.name ?? null
    variables = env?.variables ?? []
  })

  function copyToken(key: string) {
    navigator.clipboard.writeText(`{{${key}}}`)
  }

  $effect(() => {
    return () => unsubscribe()
  })
</script>

<div class="flex h-full flex-col">
  <div class="border-b border-border p-3">
    <h2 class="text-sm font-medium">Variables</h2>
    <p class="mt-1 truncate text-xs text-text-muted">{activeName ?? 'No active environment'}</p>
  </div>

  <div class="flex-1 overflow-y-auto p-2">
    {#if variables.filter((variable) => variable.enabled && variable.key.trim()).length === 0}
      <p class="p-3 text-sm text-text-muted">No enabled variables.</p>
    {:else}
      {#each variables.filter((variable) => variable.enabled && variable.key.trim()) as variable (variable.key)}
        <button
          class="mb-1 flex w-full flex-col rounded-md px-2 py-1.5 text-left hover:bg-surface-hover"
          onclick={() => copyToken(variable.key)}
          title="Copy variable token"
        >
          <span class="font-mono text-sm text-text">{'{{'}{variable.key}{'}}'}</span>
          <span class="truncate font-mono text-xs text-text-muted">{variable.value}</span>
        </button>
      {/each}
    {/if}
  </div>
</div>
