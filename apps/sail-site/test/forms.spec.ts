// Forms guide live-demo smoke — drives /sail/guide/forms end to end:
// errors gate on the first submit attempt, a valid submit walks through
// pending (900ms fake API) into the success alert, "Fail the next save"
// surfaces lastError, and Reset returns the form to its untouched state.

import { test, expect, type Page } from '@playwright/test'

// The snippets on the page quote the demo's own strings, so every text
// query must be scoped INSIDE the live form or it also matches the
// code blocks.
const goto = async (page: Page) => {
  await page.goto('/sail/guide/forms', { waitUntil: 'domcontentloaded' })
  const demo = page.getByTestId('forms-demo')
  await expect(demo).toBeVisible()
  return demo
}

test('forms guide: empty submit shows errors, no page reload', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (err) => errors.push(err.message))
  const demo = await goto(page)

  // Errors are gated on the first submit attempt — invisible on arrival.
  await expect(demo.getByText('a display name is required')).toBeHidden()

  await page.getByTestId('forms-submit').click()
  await expect(demo.getByText('a display name is required')).toBeVisible()
  await expect(demo.getByText('must include an @')).toBeVisible()

  // The submit handler owns preventDefault — still on the same page.
  expect(page.url()).toContain('/sail/guide/forms')
  expect(errors).toEqual([])
})

test('forms guide: valid submit shows pending, then success', async ({ page }) => {
  await goto(page)

  await page.getByTestId('forms-name').fill('Ada Lovelace')
  await page.getByTestId('forms-email').fill('ada@example.com')
  await page.getByTestId('forms-submit').click()

  // The 900ms fake API keeps the pending state observable.
  await expect(page.getByTestId('forms-submit')).toBeDisabled()
  await expect(page.getByTestId('forms-submit')).toContainText('Saving…')

  await expect(page.getByTestId('forms-success')).toBeVisible({ timeout: 3_000 })
  await expect(page.getByTestId('forms-success')).toContainText('ada@example.com')
  await expect(page.getByTestId('forms-submit')).toBeEnabled()
})

test('forms guide: sabotaged submit surfaces lastError; reset clears the form', async ({ page }) => {
  const demo = await goto(page)

  await page.getByTestId('forms-name').fill('Grace Hopper')
  await page.getByTestId('forms-email').fill('grace@example.com')
  await demo.getByRole('button', { name: /fail the next save/i }).click()
  await page.getByTestId('forms-submit').click()

  await expect(page.getByTestId('forms-error')).toBeVisible({ timeout: 3_000 })
  await expect(page.getByTestId('forms-error')).toContainText('503')

  // Reset: fields revert, error clears, error-gating rearms.
  await demo.getByRole('button', { name: /^reset$/i }).click()
  await expect(page.getByTestId('forms-name')).toHaveValue('')
  await expect(page.getByTestId('forms-email')).toHaveValue('')
  await expect(page.getByTestId('forms-error')).toBeHidden()
  await expect(demo.getByText('a display name is required')).toBeHidden()
})
