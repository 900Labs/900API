<script lang="ts">
  import { invoke } from '../../lib/tauri'
  import { Settings, Info, Zap, Database, Code, Globe } from '@lucide/svelte'
  import { t, locale, locales, type Locale } from '../../lib/i18n'

  let appVersion = $state('0.1.1')

  async function loadVersion() {
    try {
      appVersion = await invoke<string>('get_app_version')
    } catch {
      // ignore
    }
  }

  loadVersion()
</script>

<div class="flex h-full flex-col overflow-y-auto p-6">
  <h2 class="mb-6 flex items-center gap-2 text-lg font-semibold">
    <Settings class="h-5 w-5" />
    {$t('nav.settings')}
  </h2>

  <div class="space-y-6">
    <!-- Language -->
    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 flex items-center gap-2 text-sm font-medium">
        <Globe class="h-4 w-4 text-text-muted" />
        Language / Idioma / 言語
      </h3>
      <div class="grid grid-cols-3 gap-2">
        {#each locales as l}
          <button
            class="flex items-center gap-2 rounded-md border px-3 py-2 text-sm transition-colors {$locale === l.value ? 'border-accent bg-accent/10 text-accent' : 'border-border bg-bg text-text-muted hover:bg-surface-hover'}"
            onclick={() => locale.set(l.value)}
          >
            <span>{l.flag}</span>
            <span>{l.label}</span>
          </button>
        {/each}
      </div>
    </div>
    <!-- App Info -->
    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 flex items-center gap-2 text-sm font-medium">
        <Info class="h-4 w-4 text-text-muted" />
        {$t('settings.app')}
      </h3>
      <div class="space-y-2 text-sm">
        <div class="flex justify-between">
          <span class="text-text-muted">{$t('settings.version')}</span>
          <span class="font-mono">{appVersion}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">Platform</span>
          <span class="font-mono">Tauri + Svelte</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">Engine</span>
          <span class="font-mono">Rust + Boa JS</span>
        </div>
      </div>
    </div>

    <!-- HTTP Engine -->
    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 flex items-center gap-2 text-sm font-medium">
        <Zap class="h-4 w-4 text-text-muted" />
        {$t('settings.http')}
      </h3>
      <div class="space-y-2 text-sm">
        <div class="flex justify-between">
          <span class="text-text-muted">Connection Pooling</span>
          <span class="font-mono text-success">Enabled</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">Max Idle Connections</span>
          <span class="font-mono">20 per host</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">Idle Timeout</span>
          <span class="font-mono">90s</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">Request Timeout</span>
          <span class="font-mono">120s</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">Connect Timeout</span>
          <span class="font-mono">30s</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">TCP_NODELAY</span>
          <span class="font-mono text-success">Enabled</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">HTTP/2</span>
          <span class="font-mono text-success">Supported</span>
        </div>
      </div>
    </div>

    <!-- Database -->
    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 flex items-center gap-2 text-sm font-medium">
        <Database class="h-4 w-4 text-text-muted" />
        {$t('settings.storage')}
      </h3>
      <div class="space-y-2 text-sm">
        <div class="flex justify-between">
          <span class="text-text-muted">Database</span>
          <span class="font-mono">SQLite</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-muted">Location</span>
          <span class="font-mono text-xs">App Data Directory</span>
        </div>
      </div>
    </div>

    <!-- Features -->
    <div class="rounded-lg border border-border bg-surface p-4">
      <h3 class="mb-3 flex items-center gap-2 text-sm font-medium">
        <Code class="h-4 w-4 text-text-muted" />
        {$t('settings.features')}
      </h3>
      <div class="grid grid-cols-2 gap-2 text-sm">
        {#each ['REST', 'GraphQL', 'WebSocket', 'SSE', 'gRPC', 'Mock Server', 'Git Sync', 'Test Runner', 'API Docs', 'OAuth 2.0', 'AWS Sig v4', 'Hawk Auth'] as feature}
          <div class="flex items-center gap-2">
            <span class="h-2 w-2 rounded-full bg-success"></span>
            <span class="text-text-muted">{feature}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>
