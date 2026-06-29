import { writable } from 'svelte/store'

export type SavedRequestData = {
  id: string
  collectionId: string
  name: string
  method: string
  url: string
  headers: string
  params: string
  bodyType: string
  body: string
  authType: string
  authConfig: string
  preRequestScript: string
  testScript: string
}

export const loadRequestStore = writable<SavedRequestData | null>(null)

export type EnvironmentData = {
  id: string
  name: string
  variables: { key: string; value: string; enabled: boolean }[]
}

export const activeEnvironmentStore = writable<EnvironmentData | null>(null)
