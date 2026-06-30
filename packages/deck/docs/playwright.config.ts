import { defineConfig, devices } from '@playwright/test'

// Drives the REAL `deck` CLI standalone (no shipd) against the playground app.
// Distinct port (7884) so it never collides with the example app (7880) or the
// e2e harness (7878/7882).
const HOST = '127.0.0.1'
const PORT = Number(process.env.DECK_PLAYGROUND_PORT ?? 7884)
const BASE_URL = `http://${HOST}:${PORT}`

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env.CI,
  retries: 0,
  reporter: process.env.CI ? [['github'], ['list']] : 'list',
  use: {
    baseURL: BASE_URL,
    headless: true,
    trace: 'retain-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: `cargo run -q -p deck-cli -- dev --project . --addr ${HOST}:${PORT}`,
    url: BASE_URL,
    timeout: 180_000,
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
  },
})
