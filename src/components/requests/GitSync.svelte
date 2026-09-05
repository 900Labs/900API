<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { open } from '@tauri-apps/plugin-dialog'
  import { GitBranch, GitCommit, GitPullRequest, Upload, Download, FolderOpen, RefreshCw, FileJson } from '@lucide/svelte'

  type SyncConfig = {
    directory: string
    auto_sync: boolean
    author_name: string
    author_email: string
  }

  type GitStatus = {
    is_repo: boolean
    changed_files: string[]
  }

  type LocalCollection = {
    id: string
    name: string
    description: string | null
  }

  let config = $state<SyncConfig | null>(null)
  let gitStatus = $state<GitStatus | null>(null)
  let collections = $state<string[]>([])
  let localCollections = $state<LocalCollection[]>([])
  let commitMessage = $state('')
  let loading = $state(false)
  let error = $state<string | null>(null)
  let success = $state<string | null>(null)
  let gitOutput = $state<string | null>(null)

  async function loadConfig() {
    try {
      config = await invoke<SyncConfig | null>('sync_get_config')
      localCollections = await invoke<LocalCollection[]>('list_collections')
      if (config?.directory) await refreshStatus()
    } catch (e) {
      error = String(e)
    }
  }

  async function saveConfig() {
    if (!config) return
    loading = true
    error = null
    try {
      await invoke('sync_set_config', { config })
      await refreshStatus()
      success = 'Sync directory configured'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function pickDirectory() {
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === 'string') {
      if (!config) {
        config = { directory: selected, auto_sync: false, author_name: '', author_email: '' }
      } else {
        config.directory = selected
      }
    }
  }

  async function refreshStatus() {
    loading = true
    error = null
    try {
      localCollections = await invoke<LocalCollection[]>('list_collections')
      gitStatus = await invoke<GitStatus>('sync_git_status')
      collections = await invoke<string[]>('sync_list_collections')
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function gitInit() {
    loading = true
    error = null
    try {
      await invoke('sync_git_init')
      await refreshStatus()
      success = 'Git repository initialized'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function gitCommit() {
    if (!commitMessage.trim()) return
    loading = true
    error = null
    try {
      await invoke('sync_git_commit', { message: commitMessage })
      commitMessage = ''
      await refreshStatus()
      success = 'Changes committed'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function gitPull() {
    loading = true
    error = null
    try {
      gitOutput = await invoke<string>('sync_git_pull')
      await refreshStatus()
      success = 'Pull complete'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function gitPush() {
    loading = true
    error = null
    try {
      gitOutput = await invoke<string>('sync_git_push')
      success = 'Push complete'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function importCollection(name: string) {
    if (!config) return
    loading = true
    error = null
    try {
      const collection = await invoke<LocalCollection>('sync_import_collection', { name })
      window.dispatchEvent(new CustomEvent('900api:collections-changed'))
      success = `Imported "${collection.name}" into the local workspace`
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function exportCollection(collection: LocalCollection) {
    loading = true
    error = null
    try {
      await invoke<string>('sync_export_collection', { collectionId: collection.id })
      await refreshStatus()
      success = `Exported "${collection.name}" to the sync directory`
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  loadConfig()
</script>

<div class="flex h-full flex-col overflow-y-auto p-4">
  <h2 class="mb-4 flex items-center gap-2 text-lg font-semibold">
    <GitBranch class="h-5 w-5" />
    Git-Native Sync
  </h2>

  {#if error}
    <div class="mb-4 rounded-md bg-error/10 p-3 text-sm text-error">{error}</div>
  {/if}
  {#if success}
    <div class="mb-4 rounded-md bg-success/10 p-3 text-sm text-success">{success}</div>
  {/if}

  <!-- Configuration -->
  <div class="mb-6 rounded-lg border border-border bg-surface p-4">
    <h3 class="mb-3 text-sm font-medium">Sync Directory</h3>
    <div class="flex items-center gap-2">
      <input
        type="text"
        class="flex-1 rounded border border-border bg-bg px-3 py-2 text-sm font-mono outline-none focus:border-accent"
        placeholder="/path/to/git/repo"
        value={config?.directory ?? ''}
        oninput={(e) => { if (!config) config = { directory: '', auto_sync: false, author_name: '', author_email: '' }; config.directory = e.currentTarget.value }}
      />
      <button
        class="flex items-center gap-2 rounded-md border border-border bg-bg px-3 py-2 text-sm transition-colors hover:bg-surface-hover"
        onclick={pickDirectory}
      >
        <FolderOpen class="h-4 w-4" />
        Browse
      </button>
      <button
        class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
        onclick={saveConfig}
      >
        Save
      </button>
    </div>

    {#if config}
      <div class="mt-3 grid grid-cols-2 gap-3">
        <div>
          <span class="mb-1 block text-xs text-text-muted">Author Name</span>
          <input
            type="text"
            class="w-full rounded border border-border bg-bg px-3 py-1.5 text-sm outline-none focus:border-accent"
            bind:value={config.author_name}
          />
        </div>
        <div>
          <span class="mb-1 block text-xs text-text-muted">Author Email</span>
          <input
            type="text"
            class="w-full rounded border border-border bg-bg px-3 py-1.5 text-sm outline-none focus:border-accent"
            bind:value={config.author_email}
          />
        </div>
      </div>
    {/if}
  </div>

  {#if config?.directory}
    <div class="mb-6 rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 text-sm font-medium">Local Collections</h3>
      {#if localCollections.length === 0}
        <p class="text-sm text-text-muted">Create a collection before exporting it for Git.</p>
      {:else}
        <div class="space-y-2">
          {#each localCollections as collection (collection.id)}
            <div class="flex items-center gap-2 rounded border border-border bg-bg p-2">
              <FileJson class="h-4 w-4 text-accent" />
              <span class="flex-1 text-sm">{collection.name}</span>
              <button
                class="rounded px-2 py-1 text-xs text-accent hover:text-accent-hover disabled:opacity-50"
                onclick={() => exportCollection(collection)}
                disabled={loading}
              >
                Export
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Git Actions -->
    <div class="mb-6 rounded-lg border border-border bg-surface p-4">
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-sm font-medium">Git Actions</h3>
        <button
          class="flex items-center gap-1 text-xs text-text-muted hover:text-text"
          onclick={refreshStatus}
          disabled={loading}
        >
          <RefreshCw class="h-3.5 w-3.5 {loading ? 'animate-spin' : ''}" />
          Refresh
        </button>
      </div>

      {#if gitStatus}
        <div class="mb-3 flex items-center gap-2 text-sm">
          <span class="rounded px-2 py-0.5 text-xs {gitStatus.is_repo ? 'bg-success/10 text-success' : 'bg-warning/10 text-warning'}">
            {gitStatus.is_repo ? 'Git repo' : 'Not a git repo'}
          </span>
          {#if gitStatus.changed_files.length > 0}
            <span class="text-xs text-text-muted">{gitStatus.changed_files.length} changed file(s)</span>
          {/if}
        </div>

        {#if gitStatus.changed_files.length > 0}
          <div class="mb-3 rounded border border-border bg-bg p-2">
            {#each gitStatus.changed_files as file}
              <div class="py-0.5 font-mono text-xs text-text-muted">{file}</div>
            {/each}
          </div>
        {/if}
      {/if}

      <div class="flex flex-wrap gap-2">
        {#if gitStatus && !gitStatus.is_repo}
          <button
            class="flex items-center gap-2 rounded-md border border-border bg-bg px-3 py-2 text-sm transition-colors hover:bg-surface-hover"
            onclick={gitInit}
            disabled={loading}
          >
            <GitBranch class="h-4 w-4" />
            Init Repo
          </button>
        {/if}
        <div class="flex flex-1 items-center gap-2">
          <input
            type="text"
            class="flex-1 rounded border border-border bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
            placeholder="Commit message..."
            bind:value={commitMessage}
            disabled={loading}
          />
          <button
            class="flex items-center gap-2 rounded-md bg-accent px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
            onclick={gitCommit}
            disabled={loading || !commitMessage.trim()}
          >
            <GitCommit class="h-4 w-4" />
            Commit
          </button>
        </div>
        <button
          class="flex items-center gap-2 rounded-md border border-border bg-bg px-3 py-2 text-sm transition-colors hover:bg-surface-hover disabled:opacity-50"
          onclick={gitPull}
          disabled={loading}
        >
          <Download class="h-4 w-4" />
          Pull
        </button>
        <button
          class="flex items-center gap-2 rounded-md border border-border bg-bg px-3 py-2 text-sm transition-colors hover:bg-surface-hover disabled:opacity-50"
          onclick={gitPush}
          disabled={loading}
        >
          <Upload class="h-4 w-4" />
          Push
        </button>
      </div>

      {#if gitOutput}
        <div class="mt-3 rounded border border-border bg-bg p-2">
          <pre class="text-xs font-mono text-text-muted">{gitOutput}</pre>
        </div>
      {/if}
    </div>

    <!-- Synced Collections -->
    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 text-sm font-medium">Synced Collections</h3>
      {#if collections.length === 0}
        <p class="text-sm text-text-muted">No synced collections found. Export a collection to get started.</p>
      {:else}
        <div class="space-y-2">
          {#each collections as name}
            <div class="flex items-center gap-2 rounded border border-border bg-bg p-2">
              <FileJson class="h-4 w-4 text-accent" />
              <span class="flex-1 font-mono text-sm">{name}.json</span>
              <button
                class="rounded px-2 py-1 text-xs text-accent hover:text-accent-hover"
                onclick={() => importCollection(name)}
                disabled={loading}
              >
                Import
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center text-text-muted">
      <p>Configure a sync directory to get started.</p>
    </div>
  {/if}
</div>
