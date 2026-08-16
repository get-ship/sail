import { test, expect, type Page } from '@playwright/test'

const pageErrors = (page: Page): string[] => {
  const errs: string[] = []
  page.on('pageerror', (e) => errs.push(e.message))
  return errs
}

test('pricing is a marketing page, not three lonely cards', async ({ page }) => {
  const errs = pageErrors(page)
  await page.goto('/pricing', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Pricing that grows with you', level: 1 })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Hobby' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Choose Studio' })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Compare plans' })).toBeVisible()
  await expect(page.getByText('Two months free on Studio when you pay yearly.')).toBeVisible()
  await page.getByRole('button', { name: 'Annually' }).click()
  await expect(page.getByText('billed yearly')).toHaveCount(2)
  expect(errs).toEqual([])
})

test('dashboard opens new-order dialog and filters drawer', async ({ page }) => {
  const errs = pageErrors(page)
  await page.goto('/dashboard', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Overview', level: 1 })).toBeVisible()

  await page.getByRole('button', { name: 'New order' }).first().click()
  await expect(page.locator('#new-order')).toBeVisible()
  await expect(page.getByText('New order').first()).toBeVisible()
  await page.getByRole('button', { name: 'Cancel' }).click()

  await page.getByRole('button', { name: 'Filters' }).click()
  const filters = page.locator('#order-filters')
  await expect(filters).toBeVisible()
  await expect(filters.getByText('Needed by')).toBeVisible()
  await page.getByRole('button', { name: 'Apply' }).click()

  await page.getByRole('tab', { name: 'Customers' }).click()
  await expect(page.getByText('No customer selected')).toBeVisible()
  expect(errs).toEqual([])
})

test('settings tabs stay in a readable form column', async ({ page }) => {
  const errs = pageErrors(page)
  await page.goto('/settings', { waitUntil: 'domcontentloaded' })
  await expect(page.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible()
  await page.getByRole('tab', { name: 'Team' }).click()
  await expect(page.getByText('Ada Lovelace')).toBeVisible()
  await page.getByRole('tab', { name: 'Billing' }).click()
  await expect(page.getByRole('button', { name: 'Delete workspace' })).toBeVisible()
  expect(errs).toEqual([])
})
