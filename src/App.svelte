<script lang="ts">
  import Sidebar from './components/Sidebar.svelte'
  import RequestBuilder from './components/requests/RequestBuilder.svelte'
  import GraphQLBuilder from './components/requests/GraphQLBuilder.svelte'
  import WebSocketClient from './components/requests/WebSocketClient.svelte'
  import SseClient from './components/requests/SseClient.svelte'
  import GrpcClient from './components/requests/GrpcClient.svelte'
  import MockServer from './components/requests/MockServer.svelte'
  import GitSync from './components/requests/GitSync.svelte'
  import CollectionTree from './components/collections/CollectionTree.svelte'
  import EnvironmentManager from './components/environments/EnvironmentManager.svelte'
  import TestRunner from './components/requests/TestRunner.svelte'
  import ApiDocs from './components/requests/ApiDocs.svelte'
  import SettingsView from './components/requests/Settings.svelte'
  import PluginManager from './components/requests/PluginManager.svelte'
  import TeamWorkflows from './components/requests/TeamWorkflows.svelte'
  import { invoke } from '@tauri-apps/api/core'

  let activeView = $state<'requests' | 'graphql' | 'websocket' | 'sse' | 'grpc' | 'mock' | 'sync' | 'collections' | 'environments' | 'tests' | 'docs' | 'plugins' | 'team' | 'settings'>('requests')
  let appVersion = $state('0.1.0')

  async function checkBackend() {
    try {
      const version = await invoke<string>('get_app_version')
      appVersion = version
    } catch {
      // Backend not available in pure browser dev mode
    }
  }

  $effect(() => {
    checkBackend()
  })
</script>

<div class="flex h-screen w-screen overflow-hidden bg-bg text-text">
  <Sidebar bind:activeView />
  <main class="flex-1 overflow-hidden">
    {#if activeView === 'requests'}
      <RequestBuilder />
    {:else if activeView === 'graphql'}
      <GraphQLBuilder />
    {:else if activeView === 'websocket'}
      <WebSocketClient />
    {:else if activeView === 'sse'}
      <SseClient />
    {:else if activeView === 'grpc'}
      <GrpcClient />
    {:else if activeView === 'mock'}
      <MockServer />
    {:else if activeView === 'sync'}
      <GitSync />
    {:else if activeView === 'collections'}
      <CollectionTree />
    {:else if activeView === 'environments'}
      <EnvironmentManager />
    {:else if activeView === 'tests'}
      <TestRunner />
    {:else if activeView === 'docs'}
      <ApiDocs />
    {:else if activeView === 'plugins'}
      <PluginManager />
    {:else if activeView === 'team'}
      <TeamWorkflows />
    {:else if activeView === 'settings'}
      <SettingsView />
    {/if}
  </main>
</div>
