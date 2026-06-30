import { test, expect } from '@playwright/test'

// Drives the REAL `deck dev` CLI against the playground. Verifies the app
// renders, wears the deck theme (not ship's), is reactive, and that the
// signal graph is glitch-free.

test('overview renders, deck-themed, dark-first', async ({ page }) => {
  await page.goto('/')
  // The deck palette is active (the shell controller branded <html>).
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'deck')
  // Dark-first: no light override unless chosen.
  await expect(page.locator('html')).not.toHaveAttribute('data-mode', 'light')
  // The hero is server/client-rendered into the shell's route slot.
  await expect(page.locator('.hero h1')).toContainText('standalone')
  // The deck brand, not a ship one.
  await expect(page.locator('.brand .word')).toContainText('deck')
})

test('the live counter is reactive (a route signal)', async ({ page }) => {
  await page.goto('/')
  const card = page.locator('.card').filter({ hasText: 'Live reactivity' })
  await expect(card.locator('.big')).toHaveText('0')
  await card.getByRole('button', { name: '+' }).click()
  await card.getByRole('button', { name: '+' }).click()
  await expect(card.locator('.big')).toHaveText('2')
})

test('the diamond is glitch-free: d recomputes once per change', async ({ page }) => {
  await page.goto('/reactivity')

  const dVal = page.locator('.node.join .val')
  const dRecomputes = page.locator('.stat', { hasText: 'd recomputed' }).locator('.v')
  const aChanges = page.locator('.stat', { hasText: 'you changed a' }).locator('.v')

  // Initial: a=1 → b=2, c=11, d=13. The effect ran once at first paint.
  await expect(dVal).toHaveText('13')
  await expect(dRecomputes).toHaveText('1×')
  await expect(aChanges).toHaveText('0×')

  // One change to `a` → b AND c change, both feed d.
  await page.locator('.node.signal').getByRole('button', { name: '+ 1' }).click()

  // d updated to 16 (4 + 12)…
  await expect(dVal).toHaveText('16')
  // …and recomputed EXACTLY once more (2, not 3). A glitchy system would
  // double-fire here. This is the proof.
  await expect(aChanges).toHaveText('1×')
  await expect(dRecomputes).toHaveText('2×')
})

test('store state survives client-side navigation', async ({ page }) => {
  await page.goto('/')
  const visits = page.locator('.card').filter({ hasText: 'Durable state' }).locator('.big')
  const first = Number(await visits.textContent())
  expect(first).toBeGreaterThanOrEqual(1)

  // Navigate away and back via the in-app router (no full reload).
  await page.getByRole('link', { name: 'Reactivity', exact: true }).click()
  await expect(page.locator('.node.join')).toBeVisible()
  await page.getByRole('link', { name: 'Overview', exact: true }).click()

  // The persisted store climbed; a route signal would have reset to 0.
  await expect(visits).toHaveText(String(first + 1))
})
