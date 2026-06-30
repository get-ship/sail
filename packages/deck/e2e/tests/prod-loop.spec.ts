import { test, expect } from '@playwright/test'

// ── deck PROD — `deck build` output served statically ────────────────────────
//
// The dev loop (dev-loop.spec) proves `deck dev`. THIS proves the path a user
// actually SHIPS: `deck build --bundle-framework` → a self-contained dir → served
// by a dumb static host (serve-built.mjs), no deck process, no shipd. If the
// build emits a broken bundle (e.g. an un-importmapped bare pack import — the
// exact blank-page class deck-bundler now fails loud on), these go red.
//
// Runs on its own port against `--out ./.prod-build` so it never clobbers the
// dev server's `./app/.deck`. Absolute URLs (own webServer, no baseURL).

// Must match playwright.config.ts's PROD_PORT default (7882 — off the example
// app's 7880 to avoid a cross-harness collision).
const PROD_PORT = Number(process.env.DECK_E2E_PROD_PORT ?? 7882)
const BASE = `http://127.0.0.1:${PROD_PORT}`

test.describe('deck build — shipped static SPA', () => {
  test('the built shell is self-contained CSR with NO dev cruft', async ({ request }) => {
    const res = await request.get(`${BASE}/`)
    expect(res.status()).toBe(200)
    const html = await res.text()
    expect(html).toContain('id="root"')
    expect(html).toContain('importmap')
    // The framework singletons are materialized into the build (--bundle-framework),
    // so the importmap points at the app's OWN /_deck/* — not an external CDN.
    expect(html).toContain('/_deck/')
    // `--base assets`: app assets (entry + CSS) are referenced under `/assets/<file>`,
    // MATCHING their on-disk location — so the build is DIRECTLY static-host-servable
    // (no per-app serve route needed). Static-servability is the point of a build.
    expect(html).toContain('/assets/')
    expect(html).not.toContain('/deck-e2e-app/') // NOT the per-app serve namespace
    // A STATIC build carries NO serve-time injections: the `deck-render`/`deck-mode`
    // metas AND the live-reload client are added by `deck dev`'s serve path, never
    // baked — so a shipped build is clean (no dev cruft, nothing to strip).
    expect(html).not.toContain('live-reload.js')
    expect(html).not.toContain('deck-mode')
    // CSR: the route renders client-side, so its text is absent from raw bytes
    // (the empty #root + missing route text IS the csr tell, not a meta).
    expect(html).not.toContain('two plus two is 4')
  })

  test('the built entry mounts and renders the home route', async ({ page }) => {
    await page.goto(`${BASE}/`)
    await expect(page.locator('main.home h1')).toHaveText('deck e2e')
    // Proof the self-contained framework singletons loaded + the bundled packs
    // resolved: the route actually rendered.
    await expect(page.locator('.ssr-probe')).toHaveText('two plus two is 4')
  })

  test('client-side <a> navigation works from the built bundle (no full reload)', async ({
    page,
  }) => {
    await page.goto(`${BASE}/`)
    await page.evaluate(() => {
      ;(window as unknown as { __deckE2E?: string }).__deckE2E = 'kept'
    })
    await page.getByRole('link', { name: 'counter' }).click()
    await expect(page).toHaveURL(/\/counter$/)
    await expect(page.locator('.counter h1')).toHaveText('Counter')
    const sentinel = await page.evaluate(
      () => (window as unknown as { __deckE2E?: string }).__deckE2E,
    )
    expect(sentinel, 'window sentinel should survive a client-side navigation').toBe('kept')
  })

  test('signal reactivity works in the built bundle', async ({ page }) => {
    await page.goto(`${BASE}/counter`)
    const count = page.locator('.counter strong')
    await expect(count).toHaveText('0')
    await page.getByRole('button', { name: 'increment' }).click()
    await expect(count).toHaveText('1')
    await page.getByRole('button', { name: 'increment' }).click()
    await expect(count).toHaveText('2')
  })
})
