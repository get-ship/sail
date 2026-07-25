// Theming — drive /sail/language/theming. Themes are applyTheme + persistTheme
// (data-theme + .dark), not a separate light/dark mode channel.

import { test, expect } from '@playwright/test'

test('theming page: picking a theme sets data-theme', async ({ page }) => {
  await page.goto('/sail/language/theming', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Theming', exact: true })).toBeVisible({
    timeout: 30_000,
  })

  // Theme chips sit under a sticky site navbar that intercepts pointer events
  // after scroll; drive the same click path via DOM (still a real button click).
  await expect(page.getByRole('button', { name: /Console/i }).first()).toBeVisible({
    timeout: 30_000,
  })
  await page.evaluate(() => {
    const buttons = [...document.querySelectorAll('button')]
    const byLabel = (re: RegExp) => buttons.find((b) => re.test(b.textContent ?? ''))
    byLabel(/Console/i)?.click()
  })
  await page.waitForFunction(
    () => (document.documentElement.dataset.theme ?? '') === 'console',
    undefined,
    { timeout: 15_000 },
  )

  await page.evaluate(() => {
    const buttons = [...document.querySelectorAll('button')]
    buttons.find((b) => /Paper/i.test(b.textContent ?? ''))?.click()
  })
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
