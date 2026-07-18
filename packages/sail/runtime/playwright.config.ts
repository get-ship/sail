// Playwright config for the `timeline()` browser proof.
//
// No `webServer`: the spec builds a browser bundle of the runtime, injects
// it into an about:blank page with `setContent`, and compares what the
// REAL engine computes. There is nothing to serve, so there is nothing to
// start, wait for, or leave running.
//
// Run with `bunx playwright test` from `packages/sail/runtime`. Like every
// other Playwright suite in this repo it is deliberately outside
// `bun run test` (vitest) — see the root `vitest.config.ts`.

import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [['list']],
  use: {
    headless: true,
    trace: 'off',
    viewport: { width: 1280, height: 800 },
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
})
