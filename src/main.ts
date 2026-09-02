import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

window.addEventListener('unhandledrejection', (event) => {
  console.error('[900api] Unhandled promise rejection:', event.reason)
})

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
