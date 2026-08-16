import { defineConfig, devices } from '@playwright/test'

// The harness drives the REAL `deck` CLI standalone — no shipd, no host
// adapter — so the behaviors we assert are exactly what a deck user gets.
// `cargo run` self-bootstraps the binary; `--project ./app` points at the
// file-routed app fixture in this package.
const HOST = '127.0.0.1'
const PORT = Number(process.env.DECK_E2E_PORT ?? 7878)
const BASE_URL = `http://${HOST}:${PORT}`
// SSR is a mode of THE Project (`./app`), not a second museum. The runner
// copies `./app` and overlays `deck.render: ssr` on that one config file.
// Absolute URLs (own port), so this spec needs no per-project baseURL.
const SSR_PORT = Number(process.env.DECK_E2E_SSR_PORT ?? 7879)
// The PROD fixture: `deck build --bundle-framework` output, static-served (no
// deck process). Built to an isolated `--out` so it never clobbers the dev
// server's `./app/dist`. Its spec uses absolute URLs.
// Distinct from the example app's dev server (which defaults to 7880) so the
// two harnesses never collide — a shared port + `reuseExistingServer` made the
// prod bake silently serve a stale dev server.
const PROD_PORT = Number(process.env.DECK_E2E_PROD_PORT ?? 7882)

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
      // Same Project as CSR (`./app`), isolated copy so two `sail dev`s do not
      // fight over `dist/` / generated files. Overlay flips `deck.render`.
      command: `rm -rf ./.ssr-src && cp -a ./app ./.ssr-src && rm -rf ./.ssr-src/dist ./.ssr-src/.sail && sed -i 's/"render": "csr"/"render": "ssr"/' ./.ssr-src/sail.config.jsonc && cargo run -q -p deck-cli -- dev --project ./.ssr-src --addr ${HOST}:${SSR_PORT}`,
      url: `http://${HOST}:${SSR_PORT}`,
      timeout: 180_000,
      reuseExistingServer: !process.env.CI,
      stdout: 'pipe',
      stderr: 'pipe',
    },
    {
      // The PROD fixture: build the app self-contained, then serve the static
      // output with a dumb file server (no deck process) — the shipped path.
      // Build a COPY (`.prod-src`) so the build's generated `routes.gen.ts`
      // never races the dev server (port 7878), which writes the same file in
      // `./app` on startup. `--out` isolates the bundle into `.prod-build`.
      command: `rm -rf ./.prod-src ./.prod-build && cp -r ./app ./.prod-src && cargo run -q -p deck-cli -- build --project ./.prod-src --out ./.prod-build --bundle-framework --base assets && bun serve-built.ts ./.prod-build ${PROD_PORT}`,
      url: `http://${HOST}:${PROD_PORT}`,
      timeout: 180_000,
      // NEVER reuse: this fixture bakes a static artifact that must reflect the
      // CURRENT code. Reusing a prior server would serve a stale build (or, on a
      // port clash, a dev server) — which is exactly how 4 prod specs went green-
      // but-wrong. Always tear down `.prod-*` and rebuild from the live binary.
      reuseExistingServer: false,
      stdout: 'pipe',
      stderr: 'pipe',
    },
  ],
})
