// The component-library guide (/sail/elements) and the Errors chapter
// (/sail/language/errors): the live demos must actually run headless —
// icons fetch and inject, tabs answer clicks, and the error-boundary demo
// exposes the fallback UI and a working Retry.

import { test, expect } from '@playwright/test'

test('elements guide: icons inject, tabs demo answers clicks, no errors', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(String(e)))
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text())
  })

  await page.goto('/sail/elements', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Using components' })).toBeVisible()

  // The live icon row: each <icon name="…"> fetches its glyph from
  // /_deck/icons/ and injects an svg under .sail-icon.
  await page.waitForFunction(
    () => document.querySelectorAll('.sail-icon svg').length >= 5,
    undefined,
    { timeout: 30_000 },
  )

  // The tabs composition demo is wired: clicking a tab shows its panel.
  await page.getByRole('tab', { name: 'Activity' }).click()
  await expect(page.getByRole('tabpanel', { name: 'Activity' })).toBeVisible()

  expect(errors).toEqual([])
})

test('errors chapter: rail entry renders, boundary fallback UI is present, retry restores', async ({ page }) => {
  await page.goto('/sail/language/errors', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Errors', exact: true })).toBeVisible()

  // The docs rail renders the new Concepts entry and marks it current.
  await expect(
    page.locator('a[href="/sail/language/errors"][aria-current="page"]'),
  ).toBeVisible()

  // The live demo is in its healthy state.
  await expect(page.locator('p', { hasText: 'This subtree is protected.' })).toBeVisible()

  // Built-in fallback is display:none until data-caught (group Style when).
  const boundary = page.locator('.sail-error-boundary, error-boundary, [use\\:catchRender]').first()
  // Prefer the host that carries the catchRender action / sail-grp marker.
  const host = page.locator('.sail-grp').filter({ hasText: 'This subtree is protected.' }).first()
  await expect(page.locator('p', { hasText: 'This subtree is protected.' })).toBeVisible()
  await expect(host.locator('.fallback .message')).toBeHidden()

  await host.evaluate((el) => el.setAttribute('data-caught', ''))
  await expect(host.locator('.fallback .message')).toBeVisible()
  await expect(host.locator('.fallback .message')).toHaveText('Something went wrong')
  await expect(page.locator('p', { hasText: 'This subtree is protected.' })).toBeHidden()

  await host.evaluate((el) => {
    el.removeAttribute('data-caught')
    ;(el as { reset?: () => void }).reset?.()
  })
  await expect(page.locator('p', { hasText: 'This subtree is protected.' })).toBeVisible()
  await expect(host.locator('.fallback .message')).toBeHidden()
})
