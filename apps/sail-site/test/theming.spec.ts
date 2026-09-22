// Theming — drive /docs/concepts/theming. Themes are applyTheme + persistTheme
// (theme + mode). Dark is a mode, not a twin theme id.

import { test, expect } from '@playwright/test'

test('theming page: picking a theme sets theme', async ({ page }) => {
  await page.goto('/docs/concepts/theming', { waitUntil: 'domcontentloaded' })
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
    () => (document.documentElement.getAttribute('theme') ?? '') === 'console',
    undefined,
    { timeout: 15_000 },
  )

  await page.evaluate(() => {
    const buttons = [...document.querySelectorAll('button')]
    buttons.find((b) => /Paper/i.test(b.textContent ?? ''))?.click()
  })
  await page.waitForFunction(
    () => (document.documentElement.getAttribute('theme') ?? '') === 'paper',
    undefined,
    { timeout: 15_000 },
  )

  const after = await page.evaluate(() => ({
    theme: document.documentElement.getAttribute('theme') ?? '',
    primary: getComputedStyle(document.documentElement).getPropertyValue('--primary').trim(),
  }))
  expect(after.theme).toBe('paper')
  expect(after.primary.length).toBeGreaterThan(0)
})

test('theming page: dark aesthetic sets theme + mode dark', async ({ page }) => {
  await page.goto('/docs/concepts/theming', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Theming', exact: true })).toBeVisible({
    timeout: 30_000,
  })

  const consoleBtn = page.getByRole('button', { name: /Console/i }).first()
  await expect(consoleBtn).toBeVisible()
  await consoleBtn.click()

  await page.waitForFunction(
    () =>
      (document.documentElement.getAttribute('theme') ?? '') === 'console' &&
      (document.documentElement.dataset.mode ?? '') === 'dark',
    undefined,
    { timeout: 10_000 },
  )

  const after = await page.evaluate(() => ({
    theme: document.documentElement.getAttribute('theme') ?? '',
    mode: document.documentElement.dataset.mode ?? '',
    darkClass: document.documentElement.classList.contains('dark'),
  }))
  expect(after.theme).toBe('console')
  expect(after.mode).toBe('dark')
  expect(after.darkClass).toBe(false)
})
