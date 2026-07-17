// Playwright config for the SUI site smoke test. Spins up the deck dev
// server (the canonical sail-site serving loop) and drives three surface
// checks: landing, /components, /tutorial/01-hello-signal.

import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './test',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: 'http://127.0.0.1:7878',
    trace: 'off',
    headless: true,
    viewport: { width: 1280, height: 800 },
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
  webServer: {
    command: 'bun run dev',
    cwd: __dirname,
    url: 'http://127.0.0.1:7878',
    reuseExistingServer: true,
    timeout: 120_000,
    stdout: 'pipe',
    stderr: 'pipe',
  },
})
