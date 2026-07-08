<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { Puzzle, Plus, Trash2, Power, PowerOff, Shield, Code } from '@lucide/svelte'

  type PluginPermission = 'network' | 'file_system' | 'environment' | 'clipboard' | 'notifications'
  type PluginHook = 'before_request' | 'after_request' | 'before_test' | 'after_test' | 'on_collection_load' | 'on_environment_change'

  type PluginManifest = {
    id: string
    name: string
    version: string
    description: string
    author: string
    homepage: string | null
    permissions: PluginPermission[]
    hooks: PluginHook[]
  }

  type Plugin = {
    manifest: PluginManifest
    enabled: boolean
    installed_at: string
    config: Record<string, string>
  }

  let plugins = $state<Plugin[]>([])
  let loading = $state(false)
  let error = $state<string | null>(null)
  let success = $state<string | null>(null)
  let showInstallForm = $state(false)

  let newManifest = $state('')

  async function loadPlugins() {
    loading = true
    error = null
    try {
      plugins = await invoke<Plugin[]>('plugin_list')
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function installFromManifest() {
    error = null
    success = null
    try {
      const manifest = JSON.parse(newManifest) as PluginManifest
      await invoke('plugin_install', { manifest })
      newManifest = ''
      showInstallForm = false
      await loadPlugins()
      success = `Plugin "${manifest.name}" installed`
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    }
  }

  async function uninstall(id: string) {
    try {
      await invoke('plugin_uninstall', { id })
      await loadPlugins()
      success = 'Plugin removed'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    }
  }

  async function togglePlugin(plugin: Plugin) {
    try {
      if (plugin.enabled) {
        await invoke('plugin_disable', { id: plugin.manifest.id })
      } else {
        await invoke('plugin_enable', { id: plugin.manifest.id })
      }
      await loadPlugins()
    } catch (e) {
      error = String(e)
    }
  }

  const hookLabels: Record<PluginHook, string> = {
    before_request: 'Before Request',
    after_request: 'After Request',
    before_test: 'Before Test',
    after_test: 'After Test',
    on_collection_load: 'On Collection Load',
    on_environment_change: 'On Environment Change',
  }

  const permissionLabels: Record<PluginPermission, string> = {
    network: 'Network',
    file_system: 'File System',
    environment: 'Environment',
    clipboard: 'Clipboard',
    notifications: 'Notifications',
  }

  const sampleManifest = JSON.stringify({
    id: "my-plugin",
    name: "My Plugin",
    version: "1.0.0",
    description: "A custom plugin for 900API",
    author: "Your Name",
    homepage: null,
    permissions: ["network"],
    hooks: ["before_request", "after_request"]
  }, null, 2)

  loadPlugins()
</script>

<div class="flex h-full flex-col overflow-y-auto p-4">
  <div class="mb-4 flex items-center gap-3">
    <Puzzle class="h-5 w-5 text-text-muted" />
    <h2 class="text-lg font-semibold">Plugins</h2>
    <span class="text-xs text-text-muted">{plugins.length} installed</span>
    <div class="flex-1"></div>
    <button
      class="flex items-center gap-2 rounded-md bg-accent px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
      onclick={() => (showInstallForm = !showInstallForm)}
    >
      <Plus class="h-4 w-4" />
      Install
    </button>
  </div>

  {#if error}
    <div class="mb-4 rounded-md bg-error/10 p-3 text-sm text-error">{error}</div>
  {/if}
  {#if success}
    <div class="mb-4 rounded-md bg-success/10 p-3 text-sm text-success">{success}</div>
  {/if}
  <div class="mb-4 rounded-md border border-border bg-surface p-3 text-xs text-text-muted">
    Plugins are a local, persisted manifest registry. Hook and permission labels are stored metadata for review and planning; this release does not execute plugin hook code in request or test paths.
  </div>

  {#if showInstallForm}
    <div class="mb-4 rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-2 text-sm font-medium">Install from Manifest JSON</h3>
      <textarea
        class="h-48 w-full rounded border border-border bg-bg p-3 font-mono text-sm outline-none focus:border-accent"
        placeholder={sampleManifest}
        bind:value={newManifest}
      ></textarea>
      <div class="mt-2 flex gap-2">
        <button
          class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
          onclick={installFromManifest}
        >
          Install Plugin
        </button>
        <button
          class="rounded-md border border-border bg-bg px-4 py-2 text-sm transition-colors hover:bg-surface-hover"
          onclick={() => (showInstallForm = false)}
        >
          Cancel
        </button>
      </div>
    </div>
  {/if}

  {#if plugins.length === 0 && !loading}
    <div class="flex flex-1 items-center justify-center text-text-muted">
      <div class="text-center">
        <Puzzle class="mx-auto mb-3 h-12 w-12 opacity-30" />
        <p class="text-sm">No plugins installed.</p>
        <p class="mt-1 text-xs">Click "Install" to add a plugin from a manifest JSON.</p>
      </div>
    </div>
  {:else}
    <div class="space-y-3">
      {#each plugins as plugin (plugin.manifest.id)}
        <div class="rounded-lg border border-border bg-surface p-4">
          <div class="flex items-start gap-3">
            <div class="flex-1">
              <div class="flex items-center gap-2">
                <h3 class="text-sm font-medium">{plugin.manifest.name}</h3>
                <span class="rounded bg-accent/10 px-1.5 py-0.5 text-xs text-accent">v{plugin.manifest.version}</span>
                {#if plugin.enabled}
                  <span class="flex items-center gap-1 text-xs text-success">
                    <span class="h-2 w-2 rounded-full bg-success"></span>
                    Enabled
                  </span>
                {:else}
                  <span class="flex items-center gap-1 text-xs text-text-muted">
                    <span class="h-2 w-2 rounded-full bg-text-muted"></span>
                    Disabled
                  </span>
                {/if}
              </div>
              <p class="mt-1 text-xs text-text-muted">{plugin.manifest.description}</p>
              <p class="mt-1 text-xs text-text-muted">by {plugin.manifest.author}</p>

              {#if plugin.manifest.hooks.length > 0}
                <div class="mt-2 flex flex-wrap gap-1">
                  {#each plugin.manifest.hooks as hook}
                    <span class="flex items-center gap-1 rounded bg-bg px-1.5 py-0.5 text-xs text-text-muted">
                      <Code class="h-3 w-3" />
                      {hookLabels[hook]} metadata
                    </span>
                  {/each}
                </div>
              {/if}

              {#if plugin.manifest.permissions.length > 0}
                <div class="mt-2 flex flex-wrap gap-1">
                  {#each plugin.manifest.permissions as perm}
                    <span class="flex items-center gap-1 rounded bg-warning/10 px-1.5 py-0.5 text-xs text-warning">
                      <Shield class="h-3 w-3" />
                      {permissionLabels[perm]}
                    </span>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="flex flex-col gap-2">
              <button
                class="flex items-center gap-1.5 rounded-md border border-border bg-bg px-3 py-1.5 text-xs transition-colors hover:bg-surface-hover"
                onclick={() => togglePlugin(plugin)}
              >
                {#if plugin.enabled}
                  <PowerOff class="h-3.5 w-3.5" />
                  Disable
                {:else}
                  <Power class="h-3.5 w-3.5" />
                  Enable
                {/if}
              </button>
              <button
                class="flex items-center gap-1.5 rounded-md border border-error/30 bg-error/10 px-3 py-1.5 text-xs text-error transition-colors hover:bg-error/20"
                onclick={() => uninstall(plugin.manifest.id)}
              >
                <Trash2 class="h-3.5 w-3.5" />
                Remove
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
