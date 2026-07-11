import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { listen as tauriListen, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event'

type InvokeArgs = Record<string, unknown>

type TauriWindow = Window & {
  __TAURI_INTERNALS__?: {
    invoke?: unknown
  }
}

const browserPreviewFallbacks: Record<string, unknown> = {
  get_app_version: '0.2.0',
  list_collections: [],
  list_environments: [],
  list_history: [],
  list_requests: [],
  generate_all_docs: [],
  plugin_list: [],
  sync_get_config: null,
  sync_git_status: null,
  sync_list_collections: [],
  team_list_workspaces: [],
  team_get_activity: [],
}

export function isTauriRuntime(): boolean {
  if (typeof window === 'undefined') return false
  return typeof (window as TauriWindow).__TAURI_INTERNALS__?.invoke === 'function'
}

function cloneFallback<T>(value: unknown): T {
  if (typeof structuredClone === 'function') return structuredClone(value) as T
  return JSON.parse(JSON.stringify(value)) as T
}

export async function invoke<T = unknown>(command: string, args?: InvokeArgs): Promise<T> {
  if (isTauriRuntime()) {
    return tauriInvoke<T>(command, args)
  }

  if (Object.hasOwn(browserPreviewFallbacks, command)) {
    return cloneFallback<T>(browserPreviewFallbacks[command])
  }

  throw new Error(
    `"${command}" requires the Tauri desktop runtime. Use npm run tauri:dev or a built 900API desktop app for this action.`,
  )
}

export async function listen<T>(
  event: string,
  handler: EventCallback<T>,
): Promise<UnlistenFn> {
  if (isTauriRuntime()) {
    return tauriListen<T>(event, handler)
  }

  console.info(`"${event}" event stream requires the Tauri desktop runtime.`)
  return () => {}
}
