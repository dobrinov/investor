import { defineConfig, devices } from '@playwright/test'

// Browser tests against the built site (run `pnpm build` first).
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  retries: process.env.CI ? 1 : 0,
  use: {
    baseURL: 'http://localhost:4174',
    serviceWorkers: 'block',
    ...devices['Desktop Chrome'],
  },
  webServer: {
    command: 'pnpm exec vite preview --port 4174 --strictPort',
    url: 'http://localhost:4174',
    reuseExistingServer: !process.env.CI,
  },
})
