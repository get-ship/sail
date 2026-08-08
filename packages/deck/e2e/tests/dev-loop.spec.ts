import { test, expect } from '@playwright/test'

// ── deck dev — standalone SPA runtime loop ───────────────────────────────────
//
// Drives the REAL `deck dev` CLI (booted by playwright.config webServer) against
// the routes/ app fixture in ./app — no shipd, no host adapter. These are the
// behavioral conditions the dev SPA path actually satisfies, verified by manual
// probe before they were written here (no aspirational assertions):
//
//   • deck serves a CSR SPA shell (importmap + #root + deck-render:csr)
//   • the client entry mounts the router and renders the home route
//   • <a> navigation is client-side (mountRouter intercepts; no document reload)
//   • the counter hydrates to 0 and signal reactivity drives DOM updates
//
// NOTE on rendering: the dev SPA path is CLIENT-rendered. Per-route SSR holds
// for the named-`view` path (see deck-conformance/e2e/counter.sh) but NOT here.
// The shell test pins `deck-render:csr` on purpose: if deck ever adds dev-time
// per-route SSR, that assertion goes red and we update it consciously.

test.describe('deck dev — standalone SPA runtime loop', () => {
  test('serves the CSR SPA shell (importmap + #root, route content not in raw bytes)', async ({
    request,
  }) => {
    const res = await request.get('/')
    expect(res.status()).toBe(200)
    const html = await res.text()
    expect(html).toContain('id="root"')
    expect(html).toContain('importmap')
    expect(html).toContain('content="csr"')
    // Shell only — the route renders client-side, so its text is absent from
    // the raw server bytes.
    expect(html).not.toContain('two plus two is 4')
  })

  test('client entry mounts the router and renders the home route', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('heading', { name: 'deck e2e' })).toBeVisible()
    await expect(page.getByText('two plus two is 4')).toBeVisible()
  })

  test('<a> navigation is client-side — mounts the counter without a full reload', async ({
    page,
  }) => {
    await page.goto('/')
    // A sentinel on window survives iff navigation is client-side (no document
    // reload). If the router does a full navigation, this read returns null.
    await page.evaluate(() => {
      ;(window as unknown as { __deckE2E?: string }).__deckE2E = 'kept'
    })
    await page.getByRole('link', { name: 'counter' }).click()
    await expect(page).toHaveURL(/\/counter$/)
    await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
    const sentinel = await page.evaluate(
      () => (window as unknown as { __deckE2E?: string }).__deckE2E,
    )
    expect(sentinel, 'window sentinel should survive a client-side navigation').toBe('kept')
  })

  test('counter hydrates to 0 and signal reactivity drives DOM updates', async ({ page }) => {
    await page.goto('/counter')
    const count = page.locator('strong')
    await expect(count).toHaveText('0')
    await page.getByRole('button', { name: 'increment' }).click()
    await expect(count).toHaveText('1')
    await page.getByRole('button', { name: 'increment' }).click()
    await expect(count).toHaveText('2')
  })
})
