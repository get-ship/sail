import { test, expect } from '@playwright/test'

// ── deck — per-route code-splitting is LAZY ──────────────────────────────────
//
// The bundler emits one `page-<hash>.js` chunk per route and a `modules` map of
// `() => import(...)` thunks; `mountRouter` runs only the CURRENT route's thunk.
// dev-loop proves navigation WORKS; this proves it's actually code-split + lazy:
// the counter's chunk must arrive over the network ON navigation, not upfront
// with the entry. If a refactor ever eagerly bundles all routes into the entry
// (killing the split), this goes red.

test('routes are code-split: the counter chunk loads on-demand at nav, not upfront', async ({
  page,
}) => {
  const pageChunks = new Set<string>()
  page.on('request', (r) => {
    if (/\/(assets\/)?page-[\w-]+\.js(\?|$)/.test(r.url())) pageChunks.add(r.url())
  })

  await page.goto('/')
  await expect(page.getByRole('heading', { name: 'deck e2e' })).toBeVisible()
  // The home route pulled its OWN chunk (proof routes are split, not inlined in
  // the entry — an all-in-one entry would load zero page-*.js).
  const afterHome = new Set(pageChunks)
  expect(afterHome.size, 'home loaded a per-route chunk (routes are split)').toBeGreaterThanOrEqual(
    1,
  )

  // Navigate — the counter's chunk must be fetched NOW (lazy), i.e. a page chunk
  // that was NOT already loaded for the home route.
  await page.getByRole('link', { name: 'counter' }).click()
  await expect(page.getByRole('heading', { name: 'Counter' })).toBeVisible()
  const fresh = [...pageChunks].filter((u) => !afterHome.has(u))
  expect(
    fresh.length,
    'navigating to /counter fetched a NEW per-route chunk on demand (lazy)',
  ).toBeGreaterThanOrEqual(1)
})
