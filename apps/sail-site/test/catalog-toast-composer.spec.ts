// JOB-SITE-CATALOG-1 gates:
//   C-TOAST-1 — dismiss → Show toast remounts
//   C-COMP-1  — sail-composer default is multiline grow; three lines visible

import { test, expect } from '@playwright/test'

test('toast detail: dismiss then Show toast remounts', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (err) => errors.push(err.message))

  await page.goto('/components/toast', { waitUntil: 'domcontentloaded' })

  const stage = page.locator('[slot="stage"]')
  await expect(stage).toBeVisible({ timeout: 30_000 })

  const toast = () => stage.getByRole('status')
  await expect(toast()).toHaveCount(1)
  await expect(toast()).toContainText('Saved.')

  // L1 built-in close only (data-toast-close / aria-label=Dismiss).
  await toast().getByRole('button', { name: /dismiss/i }).click()
  await expect(toast()).toHaveCount(0)

  await stage.getByRole('button', { name: 'Show toast' }).click()
  await expect(toast()).toHaveCount(1)
  await expect(toast()).toContainText('Saved.')

  expect(errors).toEqual([])
})

test('composer detail: multiline textarea grows for three lines', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (err) => errors.push(err.message))

  await page.goto('/components/sail-composer', { waitUntil: 'domcontentloaded' })

  const composer = page.locator('sail-composer').first()
  await expect(composer).toBeVisible({ timeout: 30_000 })

  const ta = composer.locator('textarea')
  await expect(ta).toBeVisible()
  await expect(composer.locator('input')).toHaveCount(0)

  const h1 = await ta.evaluate((el) => (el as HTMLTextAreaElement).getBoundingClientRect().height)

  await ta.fill('line one\nline two\nline three')
  // Autosize reflows on input; wait for layout to settle taller than one line.
  await expect
    .poll(async () => ta.evaluate((el) => (el as HTMLTextAreaElement).getBoundingClientRect().height))
    .toBeGreaterThan(h1 + 8)

  const value = await ta.inputValue()
  expect(value.split('\n')).toHaveLength(3)

  expect(errors).toEqual([])
})
