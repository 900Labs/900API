import { describe, expect, it, vi } from 'vitest'
import { invoke, isTauriRuntime, listen } from './tauri'

describe('browser-safe Tauri bridge', () => {
  it('detects a plain browser or test runtime', () => {
    expect(isTauriRuntime()).toBe(false)
  })

  it('returns isolated fallback values for read-only commands', async () => {
    const first = await invoke<unknown[]>('list_collections')
    first.push({ id: 'preview-only' })
    const second = await invoke<unknown[]>('list_collections')

    expect(second).toEqual([])
    await expect(invoke<string>('get_app_version')).resolves.toBe('0.2.1')
  })

  it('fails clearly for desktop-only actions', async () => {
    await expect(invoke('send_request')).rejects.toThrow(
      'requires the Tauri desktop runtime',
    )
  })

  it('returns a no-op unlisten function for browser previews', async () => {
    vi.spyOn(console, 'info').mockImplementation(() => {})
    const unlisten = await listen('ws-preview-message', () => {})
    expect(() => unlisten()).not.toThrow()
  })
})
