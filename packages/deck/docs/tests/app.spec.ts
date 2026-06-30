import { test, expect } from '@playwright/test'

// The deck docs site IS a deck app, driven here through the REAL `deck dev` CLI.
// This suite is the native-path release gate: it exercises file routing, the
// layout/chrome, dynamic [param] routes, the data seam, reactivity, and the
// code-block authoring pattern — the actual crates a deck release ships.

test('home renders, wears the deck theme, dark-first', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'deck')
  await expect(page.locator('html')).not.toHaveAttribute('data-mode', 'light')
  await expect(page.locator('.doc h1')).toContainText('standalone web framework')
  await expect(page.locator('.brand .word')).toContainText('deck')
})

test('sidebar nav routes client-side and the layout chrome persists', async ({ page }) => {
  await page.goto('/')
  // A marker a FULL reload would wipe — proves the navigation is client-side.
  await page.evaluate(() => ((window as unknown as { __noReload: boolean }).__noReload = true))

  await page.getByRole('link', { name: 'File routing' }).click()
  await expect(page).toHaveURL(/\/routing$/)
  await expect(page.locator('.doc h1')).toHaveText('File routing')
  // The chrome (sidebar) is still mounted; only the route slot swapped.
  await expect(page.getByRole('link', { name: 'Overview' })).toBeVisible()
  await expect(await page.evaluate(() => (window as unknown as { __noReload?: boolean }).__noReload)).toBe(true)
})

test('a dynamic [param] route reads the URL segment as a signal', async ({ page }) => {
  await page.goto('/examples/greet/ada')
  await expect(page.locator('.doc h1')).toHaveText('Hello, ada.')
  await page.goto('/examples/greet/deck')
  await expect(page.locator('.doc h1')).toHaveText('Hello, deck.')
})

test('the data-seam store survives client-side navigation', async ({ page }) => {
  await page.goto('/data')
  const visits = page.locator('.demo .big')
  await page.getByRole('button', { name: 'Record a visit' }).click()
  const after = Number(await visits.textContent())
  expect(after).toBeGreaterThanOrEqual(1)

  // Navigate away and back via the in-app router; the store lives above it.
  await page.getByRole('link', { name: 'Reactivity', exact: true }).click()
  await expect(page.locator('.node.join')).toBeVisible()
  await page.getByRole('link', { name: 'The data seam', exact: true }).click()

  // Held — a route signal would have reset to 0.
  await expect(visits).toHaveText(String(after))
})

test('the reactivity diamond is glitch-free: d recomputes once per change', async ({ page }) => {
  await page.goto('/reactivity')
  const dVal = page.locator('.node.join .val')
  const dRecomputes = page.locator('.stat', { hasText: 'd recomputed' }).locator('.v')
  const aChanges = page.locator('.stat', { hasText: 'you changed a' }).locator('.v')

  await expect(dVal).toHaveText('13')
  await expect(dRecomputes).toHaveText('1×')
  await expect(aChanges).toHaveText('0×')

  await page.locator('.node.signal').getByRole('button', { name: '+ 1' }).click()

  await expect(dVal).toHaveText('16')
  await expect(aChanges).toHaveText('1×')
  // 2, not 3 — the diamond never double-fires.
  await expect(dRecomputes).toHaveText('2×')
})

test('code examples render as literal text, not interpolated', async ({ page }) => {
  await page.goto('/getting-started')
  // The snippet pattern keeps `{count()}` literal in a code block — if it were
  // interpolated the braces would be gone (or it would have failed to compile).
  await expect(page.getByText('clicked {count()} times')).toBeVisible()
})
