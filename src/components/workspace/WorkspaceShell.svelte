<script lang="ts">
  import { Clock, Code2, FolderTree, Plus, Send, SlidersHorizontal, Upload } from '@lucide/svelte'
  import CollectionTree from '../collections/CollectionTree.svelte'
  import RequestBuilder from '../requests/RequestBuilder.svelte'
  import EnvironmentSelector from './EnvironmentSelector.svelte'
  import HistoryPanel from './HistoryPanel.svelte'
  import EnvironmentPanel from './EnvironmentPanel.svelte'

  type RailTab = 'collections' | 'history' | 'variables'

  let railTab = $state<RailTab>('collections')

  function dispatch(name: string) {
    window.dispatchEvent(new CustomEvent(`900api:${name}`))
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="relative flex h-12 items-center gap-2 border-b border-border bg-bg px-3">
    <div class="flex items-center gap-1">
      <button
        class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-text-muted hover:bg-surface-hover hover:text-text"
        onclick={() => dispatch('new-request')}
      >
        <Plus class="h-4 w-4" />
        <span>New</span>
      </button>
      <button
        class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-text-muted hover:bg-surface-hover hover:text-text"
        onclick={() => dispatch('import-curl')}
      >
        <Upload class="h-4 w-4" />
        <span>Import</span>
      </button>
      <button
        class="flex items-center gap-2 rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-text-muted hover:bg-surface-hover hover:text-text"
        onclick={() => dispatch('open-code-snippets')}
      >
        <Code2 class="h-4 w-4" />
        <span>Code</span>
      </button>
      <button
        class="flex items-center gap-2 rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-hover"
        onclick={() => dispatch('send-request')}
      >
        <Send class="h-4 w-4" />
        <span>Send</span>
      </button>
    </div>

    <div class="ml-auto flex items-center gap-3">
      <EnvironmentSelector />
    </div>
  </div>

  <div class="flex min-h-0 flex-1">
    <aside class="flex w-80 min-w-64 max-w-96 flex-col border-r border-border bg-bg">
      <div class="grid grid-cols-3 border-b border-border">
        <button
          class="flex items-center justify-center gap-1.5 px-2 py-2 text-xs {railTab === 'collections' ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => (railTab = 'collections')}
        >
          <FolderTree class="h-3.5 w-3.5" />
          <span>Collections</span>
        </button>
        <button
          class="flex items-center justify-center gap-1.5 px-2 py-2 text-xs {railTab === 'history' ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => (railTab = 'history')}
        >
          <Clock class="h-3.5 w-3.5" />
          <span>History</span>
        </button>
        <button
          class="flex items-center justify-center gap-1.5 px-2 py-2 text-xs {railTab === 'variables' ? 'border-b-2 border-accent text-text' : 'text-text-muted hover:text-text'}"
          onclick={() => (railTab = 'variables')}
        >
          <SlidersHorizontal class="h-3.5 w-3.5" />
          <span>Vars</span>
        </button>
      </div>

      <div class="min-h-0 flex-1">
        {#if railTab === 'collections'}
          <CollectionTree embedded />
        {:else if railTab === 'history'}
          <HistoryPanel />
        {:else}
          <EnvironmentPanel />
        {/if}
      </div>
    </aside>

    <section class="min-w-0 flex-1">
      <RequestBuilder embedded />
    </section>
  </div>
</div>
