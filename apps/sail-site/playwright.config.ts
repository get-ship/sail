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
    // Sail authors `testid=` (not data-testid).
    testIdAttribute: 'testid',
    trace: 'off',
    headless: true,
    viewport: { width: 1280, height: 800 },
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
  webServer: {
    // Prefer --bin sail (deck-cli has multiple binaries). Full gen is optional via
    // SAIL_SITE_TEST_GEN=1 when wasm/routes need refresh; default is lean for smoke.
    command:
      process.env.SAIL_SITE_TEST_GEN === '1'
        ? `bun run gen && (cd ../.. && cargo run -q -p deck-cli --bin sail -- dev --project apps/sail-site --addr 127.0.0.1:${PORT})`
        : `cd ../.. && cargo run -q -p deck-cli --bin sail -- dev --project apps/sail-site --addr 127.0.0.1:${PORT}`,
    // cwd defaults to config directory (avoid bare `__dirname` under "type":"module")
    url: `http://127.0.0.1:${PORT}`,
    reuseExistingServer: !process.env.CI || process.env.SAIL_SITE_REUSE_SERVER === '1',
    timeout: 180_000,
    stdout: 'pipe',
    stderr: 'pipe',
  },
})
