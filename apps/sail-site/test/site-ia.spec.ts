// JOB-SITE-IA-1 acceptance: nav, shells, SPA layout integrity, redirects, console.

import { test, expect, type Page } from '@playwright/test'

test.setTimeout(90_000)

const DOCS_RAIL = 'nav[label="Docs"], nav[testid="docs-rail"], [testid="docs-rail"]'
const SITE_NAV = '[testid="site-nav"], nav [label="Site"], row[label="Site"]'

async function noPageErrors(page: Page): Promise<string[]> {
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  return errors
}

test.describe('top nav', () => {
  test('Docs · Components · Playground targets', async ({ page }) => {
    await page.goto('/docs', { waitUntil: 'domcontentloaded' })
    await expect(page.locator('a[href="/docs"]').first()).toBeVisible()
    await expect(page.locator('a[href="/components"]').first()).toBeVisible()
    await expect(page.locator('a[href="/playground"]').first()).toBeVisible()
    // Old product switcher gone
    await expect(page.locator('a[href="/sail/overview"]')).toHaveCount(0)
    await expect(page.locator('navbar a[href="/deck"], navbar a[href="/ai"]')).toHaveCount(0)
  })
})

test.describe('docs shell', () => {
  test('rail + main on hub and nested concepts', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await page.goto('/docs', { waitUntil: 'domcontentloaded' })
    await expect(page.locator(DOCS_RAIL).first()).toBeVisible()
    await expect(page.locator('main').first()).toBeVisible()
    await expect(page.getByRole('heading', { name: /Sail docs/i })).toBeVisible()

    await page.goto('/docs/concepts/state', { waitUntil: 'domcontentloaded' })
    await expect(page.locator(DOCS_RAIL).first()).toBeVisible()
    await expect(
      page.locator('nav a[href="/docs/concepts/state"][aria-current="page"]').first(),
    ).toBeVisible()
  })
})

test.describe('workbench shell', () => {
  test('playground has no docs rail; editor + preview hosts', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await page.goto('/playground', { waitUntil: 'domcontentloaded' })
    await expect(page.locator(DOCS_RAIL)).toHaveCount(0)
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
    await expect(page.locator('[testid="pg-preview"], [data-testid="pg-preview"]')).toBeVisible()
    await expect(page.locator('[testid="fill-shell"], [testid="playground-shell"]')).toBeVisible()
  })
})

test.describe('SPA layout integrity', () => {
  test('rail stays across docs nav; gone on playground; returns on docs', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await page.goto('/docs', { waitUntil: 'domcontentloaded' })
    await expect(page.locator(DOCS_RAIL).first()).toBeVisible()

    // Client nav within docs
    await page.locator('nav a[href="/docs/concepts/state"]').first().click()
    await expect(page).toHaveURL(/\/docs\/concepts\/state/)
    await expect(page.locator(DOCS_RAIL).first()).toBeVisible()
    await expect(
      page.locator('nav a[href="/docs/concepts/state"][aria-current="page"]').first(),
    ).toBeVisible()

    // Client nav docs → playground: rail must unmount without hard refresh
    await page.locator('a[href="/playground"]').first().click()
    await expect(page).toHaveURL(/\/playground/)
    await expect(page.locator(DOCS_RAIL)).toHaveCount(0)

    // Client nav playground → docs: rail returns
    // Compact header has no product pills — use brand or direct link; goto would hard-load.
    // From playground, inject a docs link click via history through a header brand is home;
    // use evaluate navigate isn't SPA. Prefer locating any a[href="/docs"] if present,
    // else use page.evaluate with router if exposed. Fallback: click a programmatic anchor.
    const hasDocsLink = (await page.locator('a[href="/docs"]').count()) > 0
    if (hasDocsLink) {
      await page.locator('a[href="/docs"]').first().click()
    } else {
      // Compact mode: create temporary anchor and click for SPA router interception
      await page.evaluate(() => {
        const a = document.createElement('a')
        a.href = '/docs'
        a.textContent = 'docs'
        document.body.appendChild(a)
        a.click()
      })
    }
    await expect(page).toHaveURL(/\/docs/)
    await expect(page.locator(DOCS_RAIL).first()).toBeVisible({ timeout: 15_000 })
  })
})

test.describe('header mode', () => {
  test('Docs · Components · Playground on site chrome (docs + home)', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })

    for (const path of ['/docs', '/'] as const) {
      await page.goto(path, { waitUntil: 'domcontentloaded' })
      await expect(page.locator('[testid="site-nav"]').first()).toBeVisible()
      await expect(page.locator('a[href="/docs"]').filter({ hasText: 'Docs' }).first()).toBeVisible()
      await expect(
        page.locator('a[href="/components"]').filter({ hasText: 'Components' }).first(),
      ).toBeVisible()
      await expect(
        page.locator('a[href="/playground"]').filter({ hasText: 'Playground' }).first(),
      ).toBeVisible()
    }
  })

  test('playground is single workbench chrome (no site navbar tax)', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await page.goto('/playground', { waitUntil: 'domcontentloaded' })
    // Site product bar is gone — pixels go to the editor.
    await expect(page.locator('.sail-navbar')).toHaveCount(0)
    await expect(page.locator('[testid="site-nav"]')).toHaveCount(0)
    await expect(page.locator('[testid="pg-toolbar"], [testid="pg-back"]').first()).toBeVisible()
    await expect(page.locator('a[href="/docs"]').first()).toBeVisible()
    // Theme lives in the workbench toolbar, not a second site bar.
    await expect(
      page.locator('[testid="pg-toolbar"] button[popovertarget], [testid="pg-toolbar"] button[aria-label*="Theme"]').first(),
    ).toBeVisible({ timeout: 15_000 })
    // SOURCE row tax removed
    await expect(page.getByText('SOURCE', { exact: true })).toHaveCount(0)
  })
})

test.describe('redirect smoke', () => {
  test('/sail/overview and /deck land on docs URLs', async ({ page }) => {
    await page.goto('/sail/overview', { waitUntil: 'networkidle' })
    await expect(page).toHaveURL(/\/docs/, { timeout: 15_000 })

    await page.goto('/deck', { waitUntil: 'networkidle' })
    await expect(page).toHaveURL(/\/docs\/app/, { timeout: 15_000 })
  })
})

test.describe('console', () => {
  for (const path of ['/docs', '/components', '/playground'] as const) {
    test(`no pageerror on ${path}`, async ({ page }) => {
      const errors = await noPageErrors(page)
      await page.goto(path, { waitUntil: 'domcontentloaded' })
      // Allow hydrations
      await page.waitForTimeout(1500)
      expect(errors, errors.join('\n')).toEqual([])
    })
  }
})
