import { test, expect, type Page } from '@playwright/test'
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

// ── deck HMR — stress / edge (app-granular remount, not Vite) ────────────────
//
// Contract lives in `hmr-edit.spec.ts` (edit route → remount, keep signals, no
// full document reload). These cases pin the edges around that loop:
//   • `@/` store / composition edits remount and keep PAGE signals
//   • a new route file is either navigable after remount, or 404 until a
//     full document load (honest: `swapModules` does not replace the route table)
//   • compile overlay (`#__sail_live_reload_overlay`) on type-gate / compile
//     fail, clears on the next successful rebuild (and on client nav via
//     `sail:error-clear`). A `.sail`-only save while the type-gate is still
//     red must not emit `module-changed` / clear the overlay.
//   • runtime overlay (`#__sail_error_overlay`) clears on client nav
//   • one save → one `module-changed` (no rebuild/SSE storm)
//   • `public/` edit does not crash; favicon URL is not cache-busted
//
// Two overlay IDs stay distinct. Do not weaken `hmr-edit`.

const here = dirname(fileURLToPath(import.meta.url))
const APP = join(here, '..', 'app')
const STORE = join(APP, 'src', 'stores', 'hmr-tally.ts')
const COMPOSITION = join(APP, 'src', 'compositions', 'hmr-label.ts')
const OVERLAY_PAGE = join(APP, 'routes', 'hmr-overlay', 'page.sail')
const HMR_EDIT = join(APP, 'routes', 'hmr-edit', 'page.sail')
const FAVICON = join(APP, 'public', 'favicon.svg')
const NEW_ROUTE_DIR = join(APP, 'routes', 'hmr-new-route')
const NEW_ROUTE = join(NEW_ROUTE_DIR, 'page.sail')

const COMPILE_OVERLAY = '#__sail_live_reload_overlay'
const RUNTIME_OVERLAY = '#__sail_error_overlay'

type HmrEvent = { type: 'module-changed' | 'compile-error'; data: { target?: string } }

async function attachHmrTap(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as {
      __deckHmrTap?: HmrEvent[]
      __deckHmrTapEs?: EventSource
    }
    w.__deckHmrTapEs?.close()
    w.__deckHmrTap = []
    const es = new EventSource('/_deck/dev/hmr')
    w.__deckHmrTapEs = es
    const push = (type: HmrEvent['type']) => (raw: MessageEvent) => {
      try {
        w.__deckHmrTap!.push({ type, data: JSON.parse(String(raw.data)) })
      } catch {
        /* ignore malformed */
      }
    }
    es.addEventListener('module-changed', push('module-changed'))
    es.addEventListener('compile-error', push('compile-error'))
    return new Promise<void>((resolve, reject) => {
      const t = window.setTimeout(() => reject(new Error('HMR EventSource open timeout')), 10_000)
      es.onopen = () => {
        window.clearTimeout(t)
        resolve()
      }
      es.onerror = () => {
        /* EventSource retries; wait for onopen */
      }
    })
  })
}

async function hmrEvents(page: Page): Promise<HmrEvent[]> {
  return page.evaluate(
    () => (window as unknown as { __deckHmrTap?: HmrEvent[] }).__deckHmrTap ?? [],
  )
}

async function waitForAppChanged(page: Page, since: number, timeoutMs = 30_000): Promise<void> {
  await expect
    .poll(
      async () =>
        (await hmrEvents(page)).filter(
          (e) => e.type === 'module-changed' && e.data.target === 'app',
        ).length,
      { timeout: timeoutMs },
    )
    .toBeGreaterThan(since)
}

function setWindowSentinel(page: Page) {
  return page.evaluate(() => {
    ;(window as unknown as { __deckHmrSentinel?: string }).__deckHmrSentinel = 'kept'
  })
}

function readSentinel(page: Page) {
  return page.evaluate(
    () => (window as unknown as { __deckHmrSentinel?: string }).__deckHmrSentinel,
  )
}

test.describe('deck HMR — stress / edge', () => {
  test('@/ composition edit remounts and keeps page signals (or documents a full reload)', async ({
    page,
  }) => {
    const original = readFileSync(COMPOSITION, 'utf8')
    try {
      await page.goto('/hmr-store')
      await expect(page.getByRole('heading', { level: 1 })).toHaveText('HMR store')
      await expect(page.getByTestId('composition-label')).toHaveText('composition original')
      await setWindowSentinel(page)
      await page.getByRole('button', { name: 'inc-page' }).click()
      await page.getByRole('button', { name: 'inc-page' }).click()
      await expect(page.getByTestId('page-count')).toHaveText('2')

      await attachHmrTap(page)
      writeFileSync(COMPOSITION, original.replace('composition original', 'composition edited'))
      await expect(page.getByTestId('composition-label')).toHaveText('composition edited', {
        timeout: 30_000,
      })

      const sentinel = await readSentinel(page)
      if (sentinel === 'kept') {
        await expect(page.getByTestId('page-count')).toHaveText('2')
        await page.getByRole('button', { name: 'inc-page' }).click()
        await expect(page.getByTestId('page-count')).toHaveText('3')
      } else {
        // Honest fallback: this edit forced a full document reload.
        await expect(page.getByTestId('page-count')).toHaveText('0')
      }
    } finally {
      writeFileSync(COMPOSITION, original)
    }
  })

  test('@/ store edit remounts and keeps page signals (or documents a full reload)', async ({
    page,
  }) => {
    const original = readFileSync(STORE, 'utf8')
    try {
      await page.goto('/hmr-store')
      await expect(page.getByTestId('store-label')).toHaveText('store original')
      await setWindowSentinel(page)
      await page.getByRole('button', { name: 'inc-page' }).click()
      await expect(page.getByTestId('page-count')).toHaveText('1')

      await attachHmrTap(page)
      writeFileSync(STORE, original.replace('store original', 'store edited'))
      await expect(page.getByTestId('store-label')).toHaveText('store edited', { timeout: 30_000 })

      const sentinel = await readSentinel(page)
      if (sentinel === 'kept') {
        await expect(page.getByTestId('page-count')).toHaveText('1')
      } else {
        await expect(page.getByTestId('page-count')).toHaveText('0')
      }
    } finally {
      writeFileSync(STORE, original)
    }
  })

  test('new route file: navigable after remount, or 404 until a full document load', async ({
    page,
  }) => {
    rmSync(NEW_ROUTE_DIR, { recursive: true, force: true })
    try {
      await page.goto('/counter')
      await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
      await setWindowSentinel(page)
      await attachHmrTap(page)
      const before = (await hmrEvents(page)).filter(
        (e) => e.type === 'module-changed' && e.data.target === 'app',
      ).length

      mkdirSync(NEW_ROUTE_DIR, { recursive: true })
      writeFileSync(
        NEW_ROUTE,
        `<div>\n  <h1>HMR new route</h1>\n</div>\n`,
      )
      await waitForAppChanged(page, before)

      // Client-side nav — this is the remount path, not a document load.
      await page.evaluate(() => {
        const a = document.createElement('a')
        a.href = '/hmr-new-route'
        a.textContent = 'new-route'
        document.body.appendChild(a)
      })
      await page.getByRole('link', { name: 'new-route' }).click()

      const heading = page.getByRole('heading', { name: 'HMR new route' })
      const notFound = page.locator('.deck-not-found, ship-error-state')
      await expect(heading.or(notFound)).toBeVisible({ timeout: 10_000 })

      if (await heading.isVisible()) {
        // Remount registered the new pattern (or the route table was swapped).
        expect(await readSentinel(page), 'client nav must not full-reload').toBe('kept')
      } else {
        // Honest today: swapModules updates chunks, not DYNAMIC_ROUTE_PATTERNS
        // / SHAPES — client nav 404s until a full document load remounts with
        // the rebuilt entry's route table.
        await expect(notFound).toBeVisible()
        expect(await readSentinel(page)).toBe('kept')
        await page.goto('/hmr-new-route')
        await expect(heading).toBeVisible()
      }
    } finally {
      rmSync(NEW_ROUTE_DIR, { recursive: true, force: true })
    }
  })

  test('type-gate fail shows compile overlay; next successful rebuild clears it', async ({
    page,
  }) => {
    const original = readFileSync(STORE, 'utf8')
    try {
      await page.goto('/hmr-store')
      await expect(page.getByRole('heading', { name: 'HMR store' })).toBeVisible()
      await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0)
      await attachHmrTap(page)

      writeFileSync(
        STORE,
        original.replace(
          "export const storeBanner = 'store original'",
          "export const storeBanner: number = 'type-gate fail'",
        ),
      )
      await expect(page.locator(COMPILE_OVERLAY)).toBeVisible({ timeout: 30_000 })
      await expect(page.locator(COMPILE_OVERLAY)).toContainText('Compile error')

      writeFileSync(STORE, original)
      await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0, { timeout: 30_000 })
      await expect(page.getByRole('heading', { name: 'HMR store' })).toBeVisible()
    } finally {
      writeFileSync(STORE, original)
    }
  })

  test('compile fail shows compile overlay; next successful rebuild clears it', async ({
    page,
  }) => {
    const original = readFileSync(OVERLAY_PAGE, 'utf8')
    try {
      await page.goto('/hmr-overlay')
      await expect(page.getByRole('heading', { name: 'HMR overlay' })).toBeVisible()
      await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0)
      await attachHmrTap(page)

      writeFileSync(OVERLAY_PAGE, `${original}\nthis is not valid sail {{{\n`)
      await expect(page.locator(COMPILE_OVERLAY)).toBeVisible({ timeout: 30_000 })
      await expect(page.locator(COMPILE_OVERLAY)).toContainText('Compile error')
      await expect(page.locator(RUNTIME_OVERLAY)).toHaveCount(0)

      writeFileSync(OVERLAY_PAGE, original)
      await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0, { timeout: 30_000 })
      await expect(page.getByRole('heading', { name: 'HMR overlay' })).toBeVisible()
    } finally {
      writeFileSync(OVERLAY_PAGE, original)
    }
  })

  test('compile overlay clears on client navigation', async ({ page }) => {
    const original = readFileSync(STORE, 'utf8')
    try {
      await page.goto('/hmr-store')
      await expect(page.getByRole('heading', { name: 'HMR store' })).toBeVisible()
      await attachHmrTap(page)
      writeFileSync(
        STORE,
        original.replace(
          "export const storeBanner = 'store original'",
          "export const storeBanner: number = 'type-gate fail'",
        ),
      )
      await expect(page.locator(COMPILE_OVERLAY)).toBeVisible({ timeout: 30_000 })

      // Compile overlay is `inset:0` and would swallow a real mouse click.
      // Native `HTMLAnchorElement.click()` still bubbles to the router's
      // document listener → mountCurrent → `sail:error-clear`.
      await page.evaluate(() => {
        const a = document.createElement('a')
        a.href = '/counter'
        a.textContent = 'counter'
        document.body.appendChild(a)
        a.click()
      })
      await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
      await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0)
      await expect(page.locator(RUNTIME_OVERLAY)).toHaveCount(0)
    } finally {
      writeFileSync(STORE, original)
    }
  })

  test('sail-only save while type-gate is red keeps compile overlay', async ({ page }) => {
    const originalStore = readFileSync(STORE, 'utf8')
    const originalSail = readFileSync(HMR_EDIT, 'utf8')
    try {
      await page.goto('/hmr-store')
      await expect(page.getByRole('heading', { name: 'HMR store' })).toBeVisible()
      await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0)
      await attachHmrTap(page)

      writeFileSync(
        STORE,
        originalStore.replace(
          "export const storeBanner = 'store original'",
          "export const storeBanner: number = 'type-gate fail'",
        ),
      )
      await expect(page.locator(COMPILE_OVERLAY)).toBeVisible({ timeout: 30_000 })
      const before = (await hmrEvents(page)).filter(
        (e) => e.type === 'module-changed' && e.data.target === 'app',
      ).length

      writeFileSync(HMR_EDIT, originalSail.replace('HMR original', 'HMR still red'))
      await new Promise((r) => setTimeout(r, 2500))

      await expect(page.locator(COMPILE_OVERLAY)).toBeVisible()
      const after = (await hmrEvents(page)).filter(
        (e) => e.type === 'module-changed' && e.data.target === 'app',
      ).length
      expect(after, 'sail-only tick while type-gate is red must not rebuild').toBe(before)

      writeFileSync(STORE, originalStore)
      await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0, { timeout: 30_000 })
    } finally {
      writeFileSync(STORE, originalStore)
      writeFileSync(HMR_EDIT, originalSail)
    }
  })

  test('runtime overlay clears on client navigation', async ({ page }) => {
    await page.goto('/error-boom')
    await expect(page.locator(RUNTIME_OVERLAY)).toBeVisible()
    await page.getByRole('link', { name: 'counter' }).click()
    await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
    await expect(page.locator(RUNTIME_OVERLAY)).toHaveCount(0)
    await expect(page.locator(COMPILE_OVERLAY)).toHaveCount(0)
  })

  test('one save emits one app module-changed (no rebuild/SSE storm)', async ({ page }) => {
    const original = readFileSync(HMR_EDIT, 'utf8')
    try {
      await page.goto('/hmr-edit')
      await expect(page.getByRole('heading', { level: 1 })).toHaveText('HMR original')
      await attachHmrTap(page)
      const before = (await hmrEvents(page)).length

      writeFileSync(HMR_EDIT, original.replace('HMR original', 'HMR once'))
      await expect(page.getByRole('heading', { level: 1 })).toHaveText('HMR once', {
        timeout: 30_000,
      })

      // Settle window: debounce is 150ms; a storm would keep firing.
      await new Promise((r) => setTimeout(r, 2500))
      const after = await hmrEvents(page)
      const appChanged = after
        .slice(before)
        .filter((e) => e.type === 'module-changed' && e.data.target === 'app')
      expect(
        appChanged.length,
        `one save must emit one app module-changed, got ${appChanged.length}`,
      ).toBe(1)
    } finally {
      writeFileSync(HMR_EDIT, original)
    }
  })

  test('public/ edit does not crash; favicon is not cache-busted', async ({ page, request }) => {
    const original = readFileSync(FAVICON, 'utf8')
    try {
      await page.goto('/counter')
      await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
      await attachHmrTap(page)
      const before = (await hmrEvents(page)).filter(
        (e) => e.type === 'module-changed' && e.data.target === 'app',
      ).length

      writeFileSync(FAVICON, original.replace('#09f', '#f00'))
      await waitForAppChanged(page, before)

      // Dev still serves the page — a public/ tick must not take the server down.
      await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
      const res = await request.get('/favicon.svg')
      expect(res.status()).toBe(200)
      expect(await res.text()).toContain('#f00')

      // Documented cache-bust limit: public assets keep a stable URL (no hash,
      // no query). Dev sends Cache-Control: no-cache on GET, but the document
      // `<link rel="icon">` is not rewritten, so the tab icon may stay stale.
    } finally {
      writeFileSync(FAVICON, original)
    }
  })
})
