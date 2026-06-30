import { test, expect } from '@playwright/test'

// ── deck dev — routes/ SSR (render: ssr) ─────────────────────────────────────
//
// Drives the `app-ssr` fixture (deck.jsonc { render: ssr }) on its own port.
// Proves deck's OWN routes/ path server-first-paints via the ship-js (V8)
// engine and the client hydrates it — verified empirically before pinning:
//
//   • the matched route's markup is in the RAW server bytes, inside #root
//     (server-rendered), not an empty CSR shell
//   • the index route SSRs too (not just named routes)
//   • the SSR'd DOM hydrates and stays reactive, with no hydration mismatch
//
// Contrast dev-loop.spec (the CSR app/) which pins the default csr contract.
// Absolute URLs (the SSR fixture is a second webServer on SSR_PORT).

const SSR = `http://127.0.0.1:${process.env.DECK_E2E_SSR_PORT ?? 7879}`

test.describe('deck dev — routes/ SSR (render: ssr)', () => {
  test('server first-paints the route into #root (markup in raw bytes)', async ({
    request,
  }) => {
    const res = await request.get(`${SSR}/counter`)
    expect(res.status()).toBe(200)
    const html = await res.text()
    expect(html).toContain('content="ssr"')
    // The route is rendered server-side: its markup is in the raw bytes, inside
    // #root — not the empty `<div id="root"></div>` of a CSR shell.
    expect(html).toMatch(/id="root">\s*<div class="counter"/)
    expect(html).toContain('Counter')
  })

  test('the index route SSRs too (not just named routes)', async ({ request }) => {
    const html = await (await request.get(`${SSR}/`)).text()
    expect(html).toMatch(/id="root">\s*<main class="home"/)
  })

  test('SSR DOM hydrates and stays reactive — no mismatch', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (e) => errors.push(e.message))
    await page.goto(`${SSR}/counter`)
    const count = page.locator('.counter strong')
    // SSR rendered the controller, so the count is 0 in first paint; the client
    // hydrates that DOM rather than rebuilding it.
    await expect(count).toHaveText('0')
    await page.getByRole('button', { name: 'increment' }).click()
    await expect(count).toHaveText('1')
    expect(errors, 'no hydration/runtime errors').toEqual([])
  })
})
