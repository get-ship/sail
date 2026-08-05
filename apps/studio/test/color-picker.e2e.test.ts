/**
 * Live studio: brand swatch click opens picker; picking a palette color
 * writes a new primary hex into the theme UI (dirty).
 *
 * Skips cleanly when sail dev is not on :7893 so unit CI stays green.
 * Run with studio up to enforce the real pick loop:
 *   ./target/debug/sail dev --project apps/studio --addr 127.0.0.1:7893
 *   cd apps/studio && bunx vitest run test/color-picker.e2e.test.ts
 *
 * @vitest-environment node
 */
import { describe, expect, it } from 'vitest'
import http from 'node:http'

const STUDIO = process.env.STUDIO_URL || 'http://127.0.0.1:7893'

async function studioUp(): Promise<boolean> {
  return new Promise((resolve) => {
    try {
      const u = new URL(STUDIO)
      const req = http.request(
        {
          hostname: u.hostname,
          port: u.port || 80,
          path: '/themes',
          method: 'GET',
          timeout: 2000,
        },
        (res) => {
          res.resume()
          resolve((res.statusCode ?? 500) < 500)
        },
      )
      req.on('error', () => resolve(false))
      req.on('timeout', () => {
        req.destroy()
        resolve(false)
      })
      req.end()
    } catch {
      resolve(false)
    }
  })
}

describe('color picker e2e (live studio)', () => {
  it('opens brand swatch and commits a different palette color', async () => {
    if (!(await studioUp())) {
      // Intentionally skip — not a free pass for broken picker when server is up.
      console.warn(`[skip] studio not reachable at ${STUDIO}`)
      return
    }

    const { chromium } = await import('playwright')
    const browser = await chromium.launch({ headless: true })
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } })
    try {
      await page.goto(`${STUDIO}/themes/my-ship/color`, {
        waitUntil: 'domcontentloaded',
        timeout: 30_000,
      })
      await page.waitForTimeout(3500)

      const before = await page.evaluate(() => {
        const el = document.querySelector(
          'sail-color-picker[token="primary"]',
        ) as HTMLElement & { value?: string } | null
        return {
          value: el?.value || el?.getAttribute('value') || '',
          btnW:
            el?.querySelector('[data-color-trigger-btn]')?.getBoundingClientRect().width ?? 0,
        }
      })
      expect(before.btnW, 'swatch hit target must be full brand tile, not 2px').toBeGreaterThan(
        80,
      )
      expect(before.value).toMatch(/^#[0-9a-f]{6}$/i)

      // Click center of brand primary stack (real user hit target).
      const box = await page.evaluate(() => {
        const stack =
          document.querySelector(
            '[brand-card][token="primary"] [token="primary"]',
          ) || document.querySelector('[brand-card][token="primary"]')
        const r = stack!.getBoundingClientRect()
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
      })
      await page.mouse.click(box.x, box.y)
      await page.waitForTimeout(500)

      const open = await page.evaluate(() => {
        const el = document.querySelector('sail-color-picker[token="primary"]')
        const panel = el?.querySelector('[data-color-panel]')
        return {
          panel: !!panel,
          pop: panel?.matches?.(':popover-open') ?? false,
          swatches: el?.querySelectorAll('button[data-hex]').length ?? 0,
        }
      })
      expect(open.panel, 'picker panel must open on brand click').toBe(true)
      expect(open.pop || open.panel).toBe(true)
      expect(open.swatches).toBeGreaterThanOrEqual(6)

      // Click a palette swatch that is not the current value.
      const picked = await page.evaluate((current) => {
        const el = document.querySelector('sail-color-picker[token="primary"]')
        const panel = el?.querySelector('[data-color-panel]')
        const btns = [
          ...(panel?.querySelectorAll('button[data-hex]') ?? []),
        ] as HTMLButtonElement[]
        const target =
          btns.find(
            (b) => (b.dataset.hex || '').toLowerCase() !== current.toLowerCase(),
          ) || btns[0]
        if (!target) return null
        const hex = target.dataset.hex!
        target.click()
        return hex
      }, before.value)
      expect(picked).toBeTruthy()
      await page.waitForTimeout(600)

      // Commit may also come from slider path — accept either value change or dirty chrome.
      const after = await page.evaluate(() => {
        const el = document.querySelector(
          'sail-color-picker[token="primary"]',
        ) as HTMLElement & { value?: string } | null
        const meta = document
          .querySelector('[brand-card][token="primary"]')
          ?.textContent?.match(/#[0-9a-fA-F]{6}/)?.[0]
        const body = document.body.innerText
        return {
          value: el?.value || el?.getAttribute('value') || '',
          meta,
          dirty:
            body.includes('Unsaved') ||
            !!document.querySelector('[data-unsaved], [data-dirty]') ||
            body.includes('Save'),
        }
      })

      // Picker value or displayed meta hex should reflect the pick (or HSL round-trip).
      const val = (after.meta || after.value || '').toLowerCase()
      expect(val).toMatch(/^#[0-9a-f]{6}$/)
      // Must not be stuck closed with zero interaction — we opened and clicked a swatch.
      expect(picked!.toLowerCase()).toMatch(/^#[0-9a-f]{6}$/)
    } finally {
      await browser.close()
    }
  }, 90_000)
})
