import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig, type Plugin } from 'vite'

// The built app may only talk to the origin it was served from: no analytics,
// no CDNs, no fonts from elsewhere. Your data never leaves the page.
// The desktop build (`vite build --mode desktop`) also talks to its own shell
// over Tauri's local IPC channel to open and save files.
const contentSecurityPolicy = (desktop: boolean) => [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data:",
  "font-src 'self'",
  desktop ? "connect-src 'self' ipc: http://ipc.localhost" : "connect-src 'self'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
].join('; ')

const csp = (desktop: boolean): Plugin => ({
  name: 'investor-csp',
  apply: 'build',
  transformIndexHtml: (html) =>
    html.replace('<head>', `<head>\n    <meta http-equiv="Content-Security-Policy" content="${contentSecurityPolicy(desktop)}" />`),
})

export default defineConfig(({ mode }) => ({
  base: './',
  plugins: [svelte(), csp(mode === 'desktop')],
  // The marketing page at the root, the app itself under app/.
  build: { rollupOptions: { input: { main: 'index.html', app: 'app/index.html' } } },
  // Broker configs and tax packs live at the repository root.
  server: { fs: { allow: ['..'] } },
}))
