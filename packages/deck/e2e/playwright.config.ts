import { defineConfig, devices } from '@playwright/test'

// The harness drives the REAL `deck` CLI standalone — no shipd, no host
// adapter — so the behaviors we assert are exactly what a deck user gets.
// `cargo run` self-bootstraps the binary; `--project ./app` points at the
// file-routed app fixture in this package.
const HOST = '127.0.0.1'
const PORT = Number(process.env.DECK_E2E_PORT ?? 7878)
const BASE_URL = `http://${HOST}:${PORT}`
// The SSR fixture (`app-ssr`, render: ssr) runs on its own port. Its spec uses
// absolute URLs, so it needs no per-project baseURL.
const SSR_PORT = Number(process.env.DECK_E2E_SSR_PORT ?? 7879)

export default defineConfig({
  testDir: './tests',
  // One dev server, serialized — HMR specs mutate the app on disk, so they
  // cannot run concurrently against a shared server.
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
  webServer: [
    {
      command: `cargo run -q -p deck-cli -- dev --project ./app --addr ${HOST}:${PORT}`,
      url: BASE_URL,
      // First run may compile the deck CLI; allow generous startup.
      timeout: 180_000,
      reuseExistingServer: !process.env.CI,
      stdout: 'pipe',
      stderr: 'pipe',
    },
    {
      // The SSR fixture (render: ssr) — proves server first-paint + hydration.
      command: `cargo run -q -p deck-cli -- dev --project ./app-ssr --addr ${HOST}:${SSR_PORT}`,
      url: `http://${HOST}:${SSR_PORT}`,
      timeout: 180_000,
      reuseExistingServer: !process.env.CI,
      stdout: 'pipe',
      stderr: 'pipe',
    },
  ],
})
