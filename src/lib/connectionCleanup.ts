import { invoke } from './tauri'

type DisconnectCommand = 'ws_disconnect' | 'sse_disconnect'
type InvokeFunction = (command: string, args?: Record<string, unknown>) => Promise<unknown>

export type ConnectionEstablishment = {
  active: boolean
  error?: unknown
}

export async function disconnectQuietly(
  command: DisconnectCommand,
  id: string | null,
  invokeFunction: InvokeFunction = invoke,
): Promise<void> {
  if (!id) return
  try {
    await invokeFunction(command, { id })
  } catch {
    // Component teardown cannot display or act on a disconnect error.
  }
}

export function createConnectionLifecycle(
  command: DisconnectCommand,
  invokeFunction: InvokeFunction = invoke,
) {
  let destroyed = false
  const cancelled = new Set<string>()

  return {
    isDestroyed(): boolean {
      return destroyed
    },

    async establish(id: string, connect: () => Promise<unknown>): Promise<ConnectionEstablishment> {
      if (destroyed || cancelled.has(id)) return { active: false }

      try {
        await connect()
      } catch (error) {
        return destroyed || cancelled.has(id) ? { active: false } : { active: false, error }
      }

      if (destroyed || cancelled.has(id)) {
        await disconnectQuietly(command, id, invokeFunction)
        return { active: false }
      }

      return { active: true }
    },

    async cancel(id: string | null): Promise<void> {
      if (!id) return
      cancelled.add(id)
      await invokeFunction(command, { id })
    },

    destroy(id: string | null): void {
      destroyed = true
      if (id) cancelled.add(id)
      void disconnectQuietly(command, id, invokeFunction)
    },
  }
}
