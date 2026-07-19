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
  // /_deck/icons/ and injects an svg.sail-icon-svg.
  await page.waitForFunction(
    () => document.querySelectorAll('svg.sail-icon-svg').length >= 5,
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

  // The boundary carries a built-in fallback with a message and a Retry.
  const boundary = page.locator('.sail-error-boundary')
  await expect(boundary.locator('.message')).toBeHidden()
  await expect(boundary.locator('.retry')).toBeHidden()

  // Simulate the caught state (the runtime's throw→catch path for a text
  // expression inside a {#if} branch is not yet wired in this branch, so we
  // drive the boundary via the same attribute the action sets on a real catch).
  await boundary.evaluate((el) => el.setAttribute('data-caught', ''))
  await expect(boundary.locator('.message')).toBeVisible()
  await expect(boundary.locator('.message')).toHaveText('Something went wrong')
  await expect(page.locator('p', { hasText: 'This subtree is protected.' })).toBeHidden()

  // Retry clears the fault and the healthy content comes back.
  await boundary.evaluate((el) => { (el as { reset?: () => void }).reset?.() })
  await expect(page.locator('p', { hasText: 'This subtree is protected.' })).toBeVisible()
  await expect(boundary.locator('.message')).toBeHidden()
})
