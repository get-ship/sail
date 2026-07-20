// Playwright config for the SAIL site smoke test. Spins up the deck dev
// server (the canonical sail-site serving loop) and drives three surface
// checks: landing, /components, /sail/tutorial/<slug>.

import { defineConfig, devices } from '@playwright/test'

const PORT = process.env.SAIL_SITE_TEST_PORT ?? '7891'

export default defineConfig({
  testDir: './test',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    trace: 'off',
    headless: true,
    viewport: { width: 1280, height: 800 },
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
  webServer: {
    command: `bun run gen && (cd ../.. && cargo run -q -p deck-cli -- dev --project apps/sail-site --addr 127.0.0.1:${PORT})`,
    cwd: __dirname,
    url: `http://127.0.0.1:${PORT}`,
    reuseExistingServer: process.env.SAIL_SITE_REUSE_SERVER === '1',
    timeout: 120_000,
    stdout: 'pipe',
    stderr: 'pipe',
  },
})
