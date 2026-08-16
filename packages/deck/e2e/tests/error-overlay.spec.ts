import { test, expect } from '@playwright/test'

// ── deck dev — runtime error overlay ─────────────────────────────────────────
//
// Contract: an uncaught effect/setup throw with no `<sail-error-boundary>`
// in scope must not reach only `console.error` — a page that rendered nothing
// with dozens of throws needs a visible failure surface.
//
// `/error-boom` (app/routes/error-boom/page.sail) throws from an effect on
// mount with no boundary anywhere in the route — the exact no-boundary path.
// These specs prove: the dev overlay renders for it, a healthy page shows no
// overlay, and the same app built for PROD carries zero overlay bytes.

const PROD_PORT = Number(process.env.DECK_E2E_PROD_PORT ?? 7882)
const PROD_BASE = `http://127.0.0.1:${PROD_PORT}`

test.describe('deck dev — runtime error overlay', () => {
  test('a throwing effect with no boundary renders the dev overlay panel', async ({ page }) => {
    const consoleErrors: string[] = []
    page.on('console', (msg) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })

    await page.goto('/error-boom')

    const overlay = page.locator('#__sail_error_overlay')
    await expect(overlay).toBeVisible()
    await expect(overlay).toContainText('deliberate boom for overlay e2e')
    await expect(overlay).toContainText('effect threw')

    // The event channel is IN ADDITION to console.error, never instead of —
    // the existing fallback log must still fire.
    expect(consoleErrors.some((m) => m.includes('[sail] effect threw'))).toBe(true)

    // Dismiss button clears the panel.
    await overlay.locator('button', { hasText: '×' }).click()
    await expect(overlay).toHaveCount(0)
  })

  test('client navigation off a throwing route clears the overlay', async ({ page }) => {
    await page.goto('/error-boom')
    await expect(page.locator('#__sail_error_overlay')).toBeVisible()
    await page.getByRole('link', { name: 'counter' }).click()
    await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
    await expect(page.locator('#__sail_error_overlay')).toHaveCount(0)
  })

  test('a healthy page shows no overlay', async ({ page }) => {
    await page.goto('/counter', { waitUntil: 'load' })
    await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
    await expect(page.locator('#__sail_error_overlay')).toHaveCount(0)
  })

  test('dev-mode HTML includes the overlay script; served source pins the injection', async ({
    request,
  }) => {
    const res = await request.get('/error-boom')
    const html = await res.text()
    expect(html).toContain('/_deck/error-overlay.js')
  })
})

test.describe('deck build — shipped static app has NO overlay bytes', () => {
  test('the built PROD shell carries zero error-overlay bytes', async ({ request }) => {
    const res = await request.get(`${PROD_BASE}/`)
    expect(res.status()).toBe(200)
    const html = await res.text()
    expect(html).not.toContain('error-overlay')
  })
})
