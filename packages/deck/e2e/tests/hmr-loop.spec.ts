import { test, expect } from '@playwright/test'

// ── deck HMR — state-preserving re-mount primitive ───────────────────────────
//
// mountRouter exposes a `__deckHmr` seam: `reloadCurrent()` re-mounts the route
// on screen, `swapModule(pattern, url)` points one route at a fresh chunk then
// remounts it, and `swapModules(fresh)` replaces the whole module map then
// remounts. All preserve signal state via `restoreScope` (capture the live
// signal VALUES, re-seed the fresh mount). Deck HMR is app-granular remount,
// not Vite: the live-reload client re-imports the rebuilt entry, which calls
// `swapModules`. `swapModule` is a test / optional-transport seam — not the
// default. Here we prove `reloadCurrent` directly: state survives a re-mount,
// and the re-mounted DOM stays reactive.

test.describe('deck HMR — state-preserving re-mount', () => {
  test('reloadCurrent re-mounts the route, preserving signal state + reactivity', async ({
    page,
  }) => {
    await page.goto('/counter')
    const count = page.locator('strong')
    const inc = page.getByRole('button', { name: 'increment' })
    await expect(count).toHaveText('0')

    // Drive the signal to a non-default value.
    await inc.click()
    await inc.click()
    await inc.click()
    await expect(count).toHaveText('3')

    // Re-mount the current route (the HMR primitive). A naive re-mount would
    // reset the counter to 0; restoreScope must carry the live value across.
    const reloaded = await page.evaluate(async () => {
      const hmr = (globalThis as unknown as { __deckHmr?: { reloadCurrent: () => Promise<void> } })
        .__deckHmr
      if (!hmr) return false
      await hmr.reloadCurrent()
      return true
    })
    expect(reloaded, '__deckHmr.reloadCurrent must be exposed by mountRouter').toBe(true)

    // State preserved across the re-mount...
    await expect(count).toHaveText('3')
    // ...and the freshly-mounted DOM is still reactive.
    await inc.click()
    await expect(count).toHaveText('4')
  })
})
