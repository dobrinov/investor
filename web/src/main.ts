import { mount } from 'svelte'
import '@fontsource/ibm-plex-sans/400.css'
import '@fontsource/ibm-plex-sans/500.css'
import '@fontsource/ibm-plex-sans/600.css'
import '@fontsource/ibm-plex-mono/400.css'
import '@fontsource/ibm-plex-mono/500.css'
import './app.css'
import App from './App.svelte'
import { loadCore } from './lib/core'

export default mount(App, { target: document.getElementById('app')! })

// Installed from the browser, the app opens offline too. The desktop app
// ships its files with it and needs no cache.
if (import.meta.env.PROD && 'serviceWorker' in navigator && !('__TAURI_INTERNALS__' in window)) {
  void navigator.serviceWorker
    .register('./sw.js')
    .then(() => navigator.serviceWorker.ready)
    // The engine loads only when a file is opened; fetch it now so it is kept too.
    .then(() => loadCore())
    .then(async () => {
      // What this first visit already loaded, so the next one works offline.
      const loaded = performance.getEntriesByType('resource').map((r) => r.name)
      const ours = [location.pathname, ...loaded].filter((url) => new URL(url, location.href).origin === location.origin)
      const cache = await caches.open('investor-v1')
      await Promise.all(ours.map((url) => cache.add(url).catch(() => {})))
    })
    .catch(() => {})
}
