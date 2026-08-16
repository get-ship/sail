import { test, expect } from '@playwright/test'
import { readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

// ── deck HMR — edit→reload loop (state-preserving, no full document reload) ───
//
// The app-granular remount loop end-to-end against the real `deck dev` (keep
// signal values, swap CSS — not Vite per-module HMR):
// edit a route's .sail on disk → deck dev rebuilds the SPA → emits `app` over the
// HMR SSE → the live-reload client re-imports the freshly-bundled entry → the
// HMR-aware bootstrap calls `swapModules` (new chunk URLs) on the running
// router + re-mounts the current route preserving signal state. No full
// document reload. Restores the file in `finally` so the fixture stays clean.

const here = dirname(fileURLToPath(import.meta.url))
const PAGE = join(here, '..', 'app', 'routes', 'hmr-edit', 'page.sail')

test.describe('deck HMR — edit→reload loop', () => {
  test('editing a route on disk soft-reloads it: code updates, state survives, no full reload', async ({
    page,
  }) => {
    const original = readFileSync(PAGE, 'utf8')
    try {
      await page.goto('/hmr-edit')
      await expect(page.getByRole('heading', { level: 1 })).toHaveText('HMR original')

      // A sentinel on window: survives a SOFT reload, wiped by a full reload.
      await page.evaluate(() => {
        ;(window as unknown as { __deckHmrSentinel?: string }).__deckHmrSentinel = 'kept'
      })

      // Drive controller state to a non-default value.
      const count = page.locator('strong')
      const inc = page.getByRole('button', { name: 'increment' })
      await inc.click()
      await inc.click()
      await expect(count).toHaveText('2')

      // Edit the route source on disk → triggers the dev-watcher rebuild + HMR.
      writeFileSync(PAGE, original.replace('HMR original', 'HMR edited'))

      // The code update arrives via HMR (debounce + rebuild + SSE + re-import).
      await expect(page.getByRole('heading', { level: 1 })).toHaveText('HMR edited', {
        timeout: 30_000,
      })
      // Controller state preserved across the soft reload...
      await expect(count).toHaveText('2')
      // ...it was a SOFT reload (a full document reload would have wiped this)...
      const sentinel = await page.evaluate(
        () => (window as unknown as { __deckHmrSentinel?: string }).__deckHmrSentinel,
      )
      expect(sentinel, 'window sentinel must survive a soft reload (no full reload)').toBe('kept')
      // ...and the re-mounted route is still reactive.
      await inc.click()
      await expect(count).toHaveText('3')
    } finally {
      writeFileSync(PAGE, original)
    }
  })
})
