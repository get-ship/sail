import { defineConfig, devices } from '@playwright/test'

const PORT = process.env.STUDIO_E2E_PORT ?? '7894'

export default defineConfig({
  testDir: './test',
  testMatch: '**/*.e2e.spec.ts',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    headless: true,
    trace: 'retain-on-failure',
    viewport: { width: 1280, height: 800 },
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 800 } },
    },
  ],
  webServer: {
    command: `cd ../.. && SAIL_COMPILER_NO_LIMITS=1 cargo run -q -p deck-cli --offline -- dev --project apps/studio --addr 127.0.0.1:${PORT}`,
    url: `http://127.0.0.1:${PORT}`,
    reuseExistingServer: true,
    timeout: 180_000,
    stdout: 'pipe',
    stderr: 'pipe',
  },
})
