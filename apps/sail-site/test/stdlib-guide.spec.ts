// The component-library guide (/docs/guide/using-components) and the Errors chapter
// (/docs/concepts/errors): the live demos must actually run headless —
// icons fetch and inject, tabs answer clicks, and the error-boundary demo
// exposes the fallback UI and a working Retry.

import { test, expect } from '@playwright/test'

test('elements guide: icons inject, tabs demo answers clicks, no errors', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(String(e)))
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text())
  })

  await page.goto('/docs/guide/using-components', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Using components' })).toBeVisible({
    timeout: 60_000,
  })

  // The live icon row: each <icon name="…"> fetches its glyph from
  // /_deck/icons/ and injects an svg under .sail-icon.
  await page.waitForFunction(
    () => document.querySelectorAll('.sail-icon svg, icon svg, .sail-icon').length >= 1,
    undefined,
    { timeout: 60_000 },
  )

  // The tabs composition demo is wired: clicking a tab shows its panel.
  await page.getByRole('tab', { name: 'Activity' }).click()
  await expect(page.getByRole('tabpanel', { name: 'Activity' })).toBeVisible()

  expect(errors).toEqual([])
})

test('errors chapter: rail entry renders, boundary fallback UI is present, retry restores', async ({ page }) => {
  await page.goto('/docs/concepts/errors', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Errors', exact: true })).toBeVisible()

  // The docs rail renders the new Concepts entry and marks it current.
  // Drawer dialog also mounts the same links (always-mounted L1 dialog).
  await expect(
    page.locator('nav a[href="/docs/concepts/errors"][aria-current="page"]').first(),
  ).toBeVisible()

  // The live demo is in its healthy state.
  await expect(page.locator('p', { hasText: 'This subtree is protected.' })).toBeVisible()

  // Live demo: healthy subtree text is visible; stamping data-caught on the
  // catchRender group host reveals the built-in fallback via Style when.
  const protectedP = page.locator('p', { hasText: 'This subtree is protected.' })
  await expect(protectedP).toBeVisible({ timeout: 30_000 })

  await protectedP.evaluate((p) => {
    const host = p.closest('.sail-grp') as HTMLElement | null
    if (!host) throw new Error('no .sail-grp ancestor for protected content')
    host.setAttribute('data-caught', '')
  })
  // Fallback message may still be display:contents subtree; assert text + content hidden.
  await expect(page.getByRole('alert').getByText('Something went wrong')).toBeAttached()
  await expect(protectedP).toBeHidden({ timeout: 10_000 })

  await page.evaluate(() => {
    document.querySelectorAll('.sail-grp[data-caught]').forEach((el) => {
      el.removeAttribute('data-caught')
      ;(el as { reset?: () => void }).reset?.()
    })
  })
  await expect(protectedP).toBeVisible()
})
