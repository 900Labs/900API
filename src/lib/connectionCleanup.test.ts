import { describe, expect, it, vi } from 'vitest'
import { createConnectionLifecycle, disconnectQuietly } from './connectionCleanup'

describe('connection teardown', () => {
  it('disconnects an active backend connection', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined)
    await disconnectQuietly('ws_disconnect', 'connection-1', invoke)
    expect(invoke).toHaveBeenCalledWith('ws_disconnect', { id: 'connection-1' })
  })

  it('does not surface teardown disconnect errors', async () => {
    const invoke = vi.fn().mockRejectedValue(new Error('already closed'))
    await expect(disconnectQuietly('sse_disconnect', 'connection-2', invoke)).resolves.toBeUndefined()
  })

  it('does nothing without a connection id', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined)
    await disconnectQuietly('ws_disconnect', null, invoke)
    expect(invoke).not.toHaveBeenCalled()
  })

  it('disconnects again when teardown wins a pending connect race', async () => {
    let resolveConnect: (() => void) | undefined
    const connect = new Promise<void>((resolve) => {
      resolveConnect = resolve
    })
    const invoke = vi.fn().mockResolvedValue(undefined)
    const lifecycle = createConnectionLifecycle('ws_disconnect', invoke)

    const establishment = lifecycle.establish('connection-3', () => connect)
    lifecycle.destroy('connection-3')
    expect(invoke).toHaveBeenCalledTimes(1)

    resolveConnect?.()
    await expect(establishment).resolves.toEqual({ active: false })
    expect(invoke).toHaveBeenCalledTimes(2)
    expect(invoke).toHaveBeenNthCalledWith(1, 'ws_disconnect', { id: 'connection-3' })
    expect(invoke).toHaveBeenNthCalledWith(2, 'ws_disconnect', { id: 'connection-3' })
  })

  it('does not start a backend connection after teardown', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined)
    const connect = vi.fn().mockResolvedValue(undefined)
    const lifecycle = createConnectionLifecycle('sse_disconnect', invoke)

    lifecycle.destroy(null)

    await expect(lifecycle.establish('connection-4', connect)).resolves.toEqual({ active: false })
    expect(connect).not.toHaveBeenCalled()
    expect(invoke).not.toHaveBeenCalled()
  })
})
