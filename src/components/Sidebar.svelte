<script lang="ts">
  import { Send, FolderTree, Settings, FileText, FlaskConical, BookOpen, Network, Radio, Download, Cable, Server, GitBranch, Globe, Puzzle, Users } from '@lucide/svelte'
  import { t, locale, locales, type Locale } from '../lib/i18n'

  type ViewName = 'requests' | 'graphql' | 'websocket' | 'sse' | 'grpc' | 'mock' | 'sync' | 'collections' | 'environments' | 'tests' | 'docs' | 'plugins' | 'team' | 'settings'

  interface NavItem {
    view: ViewName
    labelKey: string
    icon: typeof Send
  }

  let { activeView = $bindable() }: { activeView: ViewName } = $props()

  const navItems: NavItem[] = [
    { view: 'requests', labelKey: 'nav.requests', icon: Send },
    { view: 'graphql', labelKey: 'nav.graphql', icon: Network },
    { view: 'websocket', labelKey: 'nav.websocket', icon: Radio },
    { view: 'sse', labelKey: 'nav.sse', icon: Download },
    { view: 'grpc', labelKey: 'nav.grpc', icon: Cable },
    { view: 'mock', labelKey: 'nav.mock', icon: Server },
    { view: 'sync', labelKey: 'nav.sync', icon: GitBranch },
    { view: 'collections', labelKey: 'nav.collections', icon: FolderTree },
    { view: 'environments', labelKey: 'nav.environments', icon: Settings },
    { view: 'tests', labelKey: 'nav.tests', icon: FlaskConical },
    { view: 'docs', labelKey: 'nav.docs', icon: BookOpen },
    { view: 'plugins', labelKey: 'nav.plugins', icon: Puzzle },
    { view: 'team', labelKey: 'nav.team', icon: Users },
    { view: 'settings', labelKey: 'nav.settings', icon: Settings },
  ]

  let showLocaleMenu = $state(false)
</script>

<nav class="flex w-14 flex-col items-center gap-1 border-r border-border bg-surface py-3 lg:w-56">
  {#each navItems as item (item.view)}
    <button
      class="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors {activeView === item.view ? 'bg-accent text-white' : 'text-text-muted hover:bg-surface-hover hover:text-text'}"
      onclick={() => (activeView = item.view)}
    >
      <item.icon class="h-5 w-5 shrink-0" />
      <span class="hidden lg:inline">{$t(item.labelKey)}</span>
    </button>
  {/each}

  <div class="mt-auto px-2 py-2">
    <div class="relative">
      {#if showLocaleMenu}
        <div class="absolute bottom-full left-0 mb-2 w-48 rounded-lg border border-border bg-surface p-1 shadow-lg">
          {#each locales as l}
            <button
              class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm transition-colors {$locale === l.value ? 'bg-accent text-white' : 'text-text-muted hover:bg-surface-hover'}"
              onclick={() => { locale.set(l.value); showLocaleMenu = false }}
            >
              <span>{l.flag}</span>
              <span>{l.label}</span>
            </button>
          {/each}
        </div>
      {/if}
      <button
        class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-text-muted transition-colors hover:bg-surface-hover hover:text-text"
        onclick={() => (showLocaleMenu = !showLocaleMenu)}
      >
        <Globe class="h-5 w-5 shrink-0" />
        <span class="hidden lg:inline">{locales.find((l) => l.value === $locale)?.label ?? 'English'}</span>
      </button>
    </div>
    <span class="hidden text-center text-xs text-text-muted lg:block">900API v0.1.0</span>
  </div>
</nav>
