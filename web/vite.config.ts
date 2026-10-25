import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig, type Plugin } from 'vite'

// The built app may only talk to the origin it was served from: no analytics,
// no CDNs, no fonts from elsewhere. Your data never leaves the page.
const contentSecurityPolicy = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data:",
  "font-src 'self'",
  "connect-src 'self'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
].join('; ')

const csp: Plugin = {
  name: 'investor-csp',
  apply: 'build',
  transformIndexHtml: (html) =>
    html.replace('<head>', `<head>\n    <meta http-equiv="Content-Security-Policy" content="${contentSecurityPolicy}" />`),
}

export default defineConfig({
  base: './',
  plugins: [svelte(), csp],
  // Broker configs and tax packs live at the repository root.
  server: { fs: { allow: ['..'] } },
})
