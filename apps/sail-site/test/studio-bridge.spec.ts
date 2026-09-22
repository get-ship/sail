/**
 * Sail Studio bridge under sail dev — replaces retired sail-devtools e2e.
 * Bridge: FAB + hotkeys + mount outside #root.
 */
import { test, expect } from '@playwright/test'

async function waitBridge(page: import('@playwright/test').Page) {
  await page.waitForFunction(
    () =>
      document.getElementById('sail-studio-root') != null &&
      document.getElementById('sail-studio-fab') != null,
    { timeout: 30_000 },
  )
}

test('attach: studio bridge host + FAB outside #root, DEV meta present', async ({ page }) => {
  await page.goto('/')
  await waitBridge(page)

  const info = await page.evaluate(() => {
    const host = document.getElementById('sail-studio-root')
    const fab = document.getElementById('sail-studio-fab')
    const root = document.getElementById('root')
    return {
      host: !!host,
      fab: !!fab,
      outsideRoot: !!(host && root && !root.contains(host)),
      shadow: !!host?.shadowRoot,
      contain: host?.style.contain || '',
      devMeta:
        document.querySelector('meta[name="deck-mode"]')?.getAttribute('content') ===
        'development',
      bridgeScript: [...document.querySelectorAll('script')].some(
        (s) => (s as HTMLScriptElement).src?.includes('/_deck/sail-studio-bridge.js'),
      ),
    }
  })

  expect(info.host, 'studio host mounted').toBe(true)
  expect(info.fab, 'FAB mounted').toBe(true)
  expect(info.outsideRoot, 'host outside #root').toBe(true)
  expect(info.shadow, 'shadow root containment').toBe(true)
  expect(info.contain).toContain('strict')
  expect(info.devMeta).toBe(true)
})

test('hotkey ⌘⇧S toggles studio open', async ({ page }) => {
  await page.goto('/')
  await waitBridge(page)

  await page.keyboard.press('Meta+Shift+S')
  await page.waitForFunction(() => {
    const host = document.getElementById('sail-studio-root')
    return host && host.style.display !== 'none'
  })

  await page.keyboard.press('Escape')
  await page.waitForFunction(() => {
    const host = document.getElementById('sail-studio-root')
    return host && host.style.display === 'none'
  })
})

test('FAB opens studio and injects shell chrome (or /_studio body)', async ({ page }) => {
  await page.goto('/')
  await waitBridge(page)
  await page.click('#sail-studio-fab')
  await page.waitForFunction(() => {
    const host = document.getElementById('sail-studio-root')
    return host && host.style.display !== 'none'
  })
  // Overlay fill fetches /_studio and injects #root into shadow #f.
  // When multi-app Studio is mounted, shell markers appear; otherwise fail-soft "Studio".
  await page.waitForFunction(() => {
    const host = document.getElementById('sail-studio-root')
    const frame = host?.shadowRoot?.getElementById('f')
    if (!frame) return false
    return (
      frame.querySelector('[data-sail-studio-shell]') != null ||
      frame.textContent === 'Studio' ||
      (frame.innerHTML?.length ?? 0) > 0
    )
  })
})
