<script lang="ts">
  import { Search, X } from '@lucide/svelte'

  export type CommandAction = {
    id: string
    label: string
    group: string
    detail?: string
    shortcut?: string
    run: () => void
  }

  let {
    open = $bindable(false),
    actions = [],
  }: {
    open: boolean
    actions: CommandAction[]
  } = $props()

  let query = $state('')
  let selectedIndex = $state(0)
  let searchInput = $state<HTMLInputElement | null>(null)

  let filteredActions = $derived(
    actions.filter((action) => {
      const haystack = `${action.group} ${action.label} ${action.detail ?? ''} ${action.shortcut ?? ''}`.toLowerCase()
      return haystack.includes(query.trim().toLowerCase())
    }),
  )

  function close() {
    open = false
    query = ''
    selectedIndex = 0
  }

  function runAction(action: CommandAction) {
    action.run()
    close()
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault()
      close()
      return
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      selectedIndex = Math.min(selectedIndex + 1, Math.max(filteredActions.length - 1, 0))
      return
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault()
      selectedIndex = Math.max(selectedIndex - 1, 0)
      return
    }
    if (event.key === 'Enter' && filteredActions[selectedIndex]) {
      event.preventDefault()
      runAction(filteredActions[selectedIndex])
    }
  }

  $effect(() => {
    if (selectedIndex >= filteredActions.length) {
      selectedIndex = Math.max(filteredActions.length - 1, 0)
    }
  })

  $effect(() => {
    if (open) {
      setTimeout(() => searchInput?.focus(), 0)
    }
  })
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-start justify-center bg-black/55 px-4 pt-[12vh]"
    role="button"
    tabindex="0"
    onclick={close}
    onkeydown={(event) => event.key === 'Escape' && close()}
  >
    <div
      class="w-full max-w-2xl overflow-hidden rounded-lg border border-border bg-bg shadow-2xl"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(event) => event.stopPropagation()}
      onkeydown={handleKeydown}
    >
      <div class="flex items-center gap-3 border-b border-border px-4 py-3">
        <Search class="h-4 w-4 text-text-muted" />
        <input
          class="flex-1 bg-transparent text-sm outline-none placeholder:text-text-muted"
          placeholder="Search commands, requests, collections, history"
          bind:value={query}
          bind:this={searchInput}
        />
        <button class="rounded p-1 text-text-muted hover:bg-surface-hover hover:text-text" onclick={close} title="Close">
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="max-h-[55vh] overflow-y-auto p-2">
        {#if filteredActions.length === 0}
          <p class="px-3 py-6 text-center text-sm text-text-muted">No commands found.</p>
        {:else}
          {#each filteredActions as action, index (action.id)}
            <button
              class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-left text-sm transition-colors {index === selectedIndex ? 'bg-surface-hover text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'}"
              onmouseenter={() => (selectedIndex = index)}
              onclick={() => runAction(action)}
            >
              <span class="w-24 shrink-0 text-xs uppercase text-text-muted">{action.group}</span>
              <span class="min-w-0 flex-1">
                <span class="block truncate">{action.label}</span>
                {#if action.detail}
                  <span class="block truncate text-xs text-text-muted">{action.detail}</span>
                {/if}
              </span>
              {#if action.shortcut}
                <span class="rounded border border-border px-1.5 py-0.5 text-xs text-text-muted">{action.shortcut}</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
