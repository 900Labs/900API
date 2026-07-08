<script lang="ts">
  import { ChevronDown, Command } from '@lucide/svelte'
  import type { CommandAction } from './CommandPalette.svelte'

  type MenuGroup = {
    label: string
    actions: CommandAction[]
  }

  let {
    groups = [],
    onOpenPalette,
  }: {
    groups: MenuGroup[]
    onOpenPalette: () => void
  } = $props()

  let openMenu = $state<string | null>(null)

  function run(action: CommandAction) {
    action.run()
    openMenu = null
  }
</script>

<div class="flex h-10 items-center gap-1 border-b border-border bg-bg px-2">
  {#each groups as group (group.label)}
    <div class="relative">
      <button
        class="flex items-center gap-1 rounded px-2 py-1.5 text-sm text-text-muted hover:bg-surface-hover hover:text-text"
        onclick={() => (openMenu = openMenu === group.label ? null : group.label)}
      >
        <span>{group.label}</span>
        <ChevronDown class="h-3.5 w-3.5" />
      </button>

      {#if openMenu === group.label}
        <div class="absolute left-0 top-full z-40 mt-1 min-w-56 rounded-md border border-border bg-surface p-1 shadow-xl">
          {#each group.actions as action (action.id)}
            <button
              class="flex w-full items-center gap-3 rounded px-2 py-1.5 text-left text-sm text-text-muted hover:bg-surface-hover hover:text-text"
              onclick={() => run(action)}
            >
              <span class="flex-1 truncate">{action.label}</span>
              {#if action.shortcut}
                <span class="text-xs text-text-muted">{action.shortcut}</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}

  <button
    class="ml-auto flex items-center gap-2 rounded border border-border bg-surface px-2.5 py-1.5 text-xs text-text-muted hover:bg-surface-hover hover:text-text"
    onclick={onOpenPalette}
    title="Open command palette"
  >
    <Command class="h-3.5 w-3.5" />
    <span>Command</span>
    <span class="rounded bg-bg px-1.5 py-0.5">Ctrl K</span>
  </button>
</div>

{#if openMenu}
  <button
    class="fixed inset-0 z-30 cursor-default"
    aria-label="Close menu"
    onclick={() => (openMenu = null)}
  ></button>
{/if}
