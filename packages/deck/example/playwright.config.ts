import { defineConfig, devices } from '@playwright/test'

// Drives the REAL `deck` CLI standalone (no shipd, no host adapter) against
// this example app. `cargo run` self-bootstraps the binary; `--project .`
// points at this package's `routes/` tree.
const HOST = '127.0.0.1'
const PORT = Number(process.env.DECK_EXAMPLE_PORT ?? 7880)
const BASE_URL = `http://${HOST}:${PORT}`

export default defineConfig({
  testDir: './tests',
  // The dev server streams live-reload SSE that never idles — specs use
  // `load` + locator waits, never `networkidle`.
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
    // First run may compile the deck CLI; allow generous startup.
    timeout: 180_000,
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
  },
})
