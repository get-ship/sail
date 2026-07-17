// Theming — drive /theming, exercise the light/dark toggle, exercise the
// custom-crimson theme. The page's controller toggles data-mode on <html>
// and data-theme for the custom theme; both must reflect in the document
// before the next paint.

import { test, expect } from '@playwright/test'

test('theming page: light/dark toggle flips data-mode', async ({ page }) => {
  await page.goto('/theming', { waitUntil: 'domcontentloaded' })
  await page.waitForFunction(
    () => document.querySelectorAll('.theme-toggle-row button').length > 0,
    undefined,
    { timeout: 30_000 },
  )

  const initial = await page.evaluate(() => ({
    mode: document.documentElement.dataset.mode ?? '',
    theme: document.documentElement.dataset.theme ?? '',
  }))

  // The page has explicit "Light" + "Dark" buttons. Click the one that's
  // NOT currently active.
  const targetMode = initial.mode === 'dark' ? 'light' : 'dark'
  await page.getByRole('button', { name: new RegExp(`^${targetMode}$`, 'i') }).click()

  await page.waitForFunction(
    (m: string) => (document.documentElement.dataset.mode ?? '') === m,
    targetMode,
    { timeout: 10_000 },
  )

  const after = await page.evaluate(() => ({
    mode: document.documentElement.dataset.mode ?? '',
  }))
  expect(after.mode).toBe(targetMode)
})

test('theming page: custom theme switches data-theme', async ({ page }) => {
  await page.goto('/theming', { waitUntil: 'domcontentloaded' })
  await page.waitForFunction(
    () => document.querySelectorAll('.theme-toggle-row button').length > 0,
    undefined,
    { timeout: 30_000 },
  )

  const initial = await page.evaluate(() => ({
    theme: document.documentElement.dataset.theme ?? '',
  }))
  expect(initial.theme).not.toBe('crimson-red')

  // Click the "Apply crimson-red theme" button.
  const apply = page.getByRole('button', { name: /apply crimson-red theme/i })
  await apply.click()

  await page.waitForFunction(
    () => (document.documentElement.dataset.theme ?? '') === 'crimson-red',
    undefined,
    { timeout: 10_000 },
  )

  const after = await page.evaluate(() => ({
    theme: document.documentElement.dataset.theme ?? '',
    primary: getComputedStyle(document.documentElement).getPropertyValue('--primary'),
  }))
  expect(after.theme).toBe('crimson-red')
  // The custom theme overrides --primary to crimson. Token reads should
  // resolve to the override (not the empty string), proving the live
  // theme switch actually re-skins the page.
  expect(after.primary).toMatch(/oklch/)
})
