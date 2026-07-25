// Theming — drive /sail/language/theming. Themes are applyTheme + persistTheme
// (data-theme + .dark), not a separate light/dark mode channel.

import { test, expect } from '@playwright/test'

test('theming page: picking a theme sets data-theme', async ({ page }) => {
  await page.goto('/sail/language/theming', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Theming', exact: true })).toBeVisible({
    timeout: 30_000,
  })

  // Theme chips are labeled with THEMES[].label. Use Console (always dark) then
  // Paper so we always cross a theme boundary regardless of storage default.
  const consoleBtn = page.getByRole('button', { name: /Console/i }).first()
  await expect(consoleBtn).toBeVisible({ timeout: 30_000 })
  await consoleBtn.click()
  await page.waitForFunction(
    () => (document.documentElement.dataset.theme ?? '') === 'console',
    undefined,
    { timeout: 15_000 },
  )

  const paper = page.getByRole('button', { name: /Paper/i }).first()
  await paper.click()
  await page.waitForFunction(
    () => (document.documentElement.dataset.theme ?? '') === 'paper',
    undefined,
    { timeout: 15_000 },
  )

  const after = await page.evaluate(() => ({
    theme: document.documentElement.dataset.theme ?? '',
    primary: getComputedStyle(document.documentElement).getPropertyValue('--primary').trim(),
  }))
  expect(after.theme).toBe('paper')
  expect(after.primary.length).toBeGreaterThan(0)
})

test('theming page: dark theme stamps .dark and data-theme', async ({ page }) => {
  await page.goto('/sail/language/theming', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Theming', exact: true })).toBeVisible({
    timeout: 30_000,
  })

  const dark = page.getByRole('button', { name: /SHIP Dark/i })
  await expect(dark).toBeVisible()
  await dark.click()

  await page.waitForFunction(
    () =>
      (document.documentElement.dataset.theme ?? '') === 'ship-dark' &&
      document.documentElement.classList.contains('dark'),
    undefined,
    { timeout: 10_000 },
  )

  const after = await page.evaluate(() => ({
    theme: document.documentElement.dataset.theme ?? '',
    dark: document.documentElement.classList.contains('dark'),
  }))
  expect(after.theme).toBe('ship-dark')
  expect(after.dark).toBe(true)
})
