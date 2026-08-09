// Free playground — editor + same-doc preview + recipes, no tutorial chrome.

import { test, expect, type Page } from '@playwright/test'

test.setTimeout(90_000)

const editorValue = (page: Page): Promise<string> =>
  page.evaluate(() => {
    const el =
      document.querySelector('sail-code-editor[pane="source"]') ??
      document.querySelector('sail-code-editor[data-pane="source"]') ??
      document.querySelector('sail-code-editor')
    return (el as unknown as { value?: string })?.value ?? ''
  })

const statusText = (page: Page): Promise<string> =>
  page.locator('.sail-badge').first().innerText()

async function openPlayground(page: Page): Promise<void> {
  await page.goto('/sail/playground', { waitUntil: 'domcontentloaded' })
  await page.waitForFunction(
    () => {
      const el =
        document.querySelector('sail-code-editor[pane="source"]') ??
        document.querySelector('sail-code-editor[data-pane="source"]') ??
        document.querySelector('sail-code-editor')
      return Boolean((el as unknown as { value?: string })?.value)
    },
    undefined,
    { timeout: 60_000 },
  )
}

test('loads with editor and blank (or default) recipe', async ({ page }) => {
  await openPlayground(page)
  const src = await editorValue(page)
  expect(src.length).toBeGreaterThan(0)
  expect(src).toMatch(/Hello Sail|<column/i)
  await expect(page.locator('[testid="pg-preview"], [data-testid="pg-preview"]')).toBeVisible()
  // No iframe host path.
  expect(
    await page.locator('iframe[testid="pg-preview"], iframe[data-testid="pg-preview"]').count(),
  ).toBe(0)
})

test('run compiles blank and mounts a non-empty same-doc preview', async ({ page }) => {
  const errors: string[] = []
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text())
  })
  await openPlayground(page)

  await expect.poll(() => statusText(page), { timeout: 60_000 }).toBe('running')

  const preview = page.locator('[testid="pg-preview"], [data-testid="pg-preview"]')
  await expect(preview).toBeVisible()
  await expect(preview).not.toBeEmpty()
  await expect(preview).toContainText(/Hello Sail|Primary button/i)
  expect(await page.locator('iframe').count()).toBe(0)
  expect(errors.filter((e) => /not declared in scope/i.test(e))).toEqual([])
})

test('selecting a recipe loads source and updates preview', async ({ page }) => {
  await openPlayground(page)
  await expect.poll(() => statusText(page), { timeout: 60_000 }).toBe('running')

  const select = page.locator('[testid="pg-recipe"], select[label="Recipe"], select').first()
  await expect(select).toBeVisible()

  // First tutorial recipe after Blank is 01-a-page-is-markup (settings page).
  await select.selectOption('01-a-page-is-markup')

  await expect
    .poll(async () => editorValue(page), { timeout: 30_000 })
    .toMatch(/Workspace settings|settings/i)

  await expect.poll(() => statusText(page), { timeout: 60_000 }).toBe('running')

  const preview = page.locator('[testid="pg-preview"], [data-testid="pg-preview"]')
  await expect(preview).not.toBeEmpty()
  await expect(preview).toContainText(/Workspace settings|Profile|Danger zone/i)
})
