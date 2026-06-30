import { test, expect } from '@playwright/test'

// ── deck example — standalone SPA runtime loop ──────────────────────────────
//
// Drives the REAL `deck dev` CLI against the Mise example app. These assert the
// runtime behaviors the app demonstrates — verified in a real headless browser,
// not from the static bytes. The dev SPA path is CSR, so route content only
// appears after JS loads; specs wait on locators (never `networkidle`).

test.describe('deck example — Mise (ship-free showcase)', () => {
  test('serves a CSR SPA shell (importmap + #root, route text not in raw bytes)', async ({
    request,
  }) => {
    const res = await request.get('/')
    expect(res.status()).toBe(200)
    const html = await res.text()
    expect(html).toContain('id="root"')
    expect(html).toContain('importmap')
    expect(html).toContain('content="csr"')
    // Shell only — the route renders client-side.
    expect(html).not.toContain('Skillet Shakshuka')
  })

  test('home route renders inside the layout chrome after JS', async ({ page }) => {
    await page.goto('/', { waitUntil: 'load' })
    // Route content (light DOM, mounted into the chrome's route slot). The h1
    // carries a <sui-badge>demo</sui-badge>, so match on substring.
    await expect(page.locator('.home h1')).toContainText('Mise')
    // Layout chrome persists: nav links live in the sui-app-shell shadow DOM
    // (Playwright pierces open shadow roots).
    await expect(page.getByRole('link', { name: 'Recipes', exact: true })).toBeVisible()
    await expect(page.getByRole('button', { name: /Dark|Light/ })).toBeVisible()
  })

  test('client-side <a> nav keeps the chrome and renders the recipes list', async ({ page }) => {
    await page.goto('/', { waitUntil: 'load' })
    // Sentinel survives iff navigation is client-side (no document reload).
    await page.evaluate(() => {
      ;(window as unknown as { __mise?: string }).__mise = 'kept'
    })
    await page.getByRole('link', { name: 'Recipes', exact: true }).click()
    await expect(page).toHaveURL(/\/recipes$/)
    await expect(page.locator('.recipes h1')).toHaveText('Recipes')
    await expect(page.locator('.recipes .card-link')).toHaveCount(4)
    const sentinel = await page.evaluate(
      () => (window as unknown as { __mise?: string }).__mise,
    )
    expect(sentinel, 'window sentinel should survive a client-side navigation').toBe('kept')
  })

  test('dynamic [id] route reads the param signal and looks it up in the client', async ({
    page,
  }) => {
    await page.goto('/recipes/shakshuka', { waitUntil: 'load' })
    await expect(page.locator('.detail h1')).toHaveText('Skillet Shakshuka')
    // Steps come from the client record for this id.
    await expect(page.locator('.detail .steps li').first()).toContainText('Sweat onion')
  })

  test('the [id] param echoes for an unknown id (proves route.params.id() is live)', async ({
    page,
  }) => {
    await page.goto('/recipes/made-up-xyz', { waitUntil: 'load' })
    await expect(page.locator('.detail .missing')).toBeVisible()
    await expect(page.locator('.detail .missing code')).toHaveText('made-up-xyz')
  })

  test('counter increments on click (signal reactivity drives the DOM)', async ({ page }) => {
    await page.goto('/kitchen', { waitUntil: 'load' })
    const count = page.locator('.kitchen .big')
    await expect(count).toHaveText('0')
    const incr = page.getByRole('button', { name: 'Increment' })
    await incr.click()
    await expect(count).toHaveText('1')
    await incr.click()
    await expect(count).toHaveText('2')
  })

  test('the add-recipe form mutates client state and navigates to the new recipe', async ({
    page,
  }) => {
    await page.goto('/kitchen', { waitUntil: 'load' })
    await page.locator('.kitchen input[name="title"]').fill('Garlic Noodles')
    await page.locator('.kitchen input[name="minutes"]').fill('18')
    await page.locator('.kitchen form.add-form button[type="submit"]').click()
    // The mutation appended a recipe and navigated to its dynamic route.
    await expect(page).toHaveURL(/\/recipes\/garlic-noodles$/)
    await expect(page.locator('.detail h1')).toHaveText('Garlic Noodles')
    // And the shared client now lists 5 recipes.
    await page.getByRole('link', { name: 'Recipes', exact: true }).click()
    await expect(page.locator('.recipes .card-link')).toHaveCount(5)
  })
})
