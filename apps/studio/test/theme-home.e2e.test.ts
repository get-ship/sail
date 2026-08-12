/**
 * Themes home open path (Playwright).
 *
 * Requires sail dev on STUDIO_URL (default http://127.0.0.1:7893) from product tree.
 *
 *   cd apps/studio && bunx vitest run test/theme-home.e2e.test.ts
 *
 * @vitest-environment node
 */
import { describe, expect, it } from 'vitest'
import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'

const STUDIO = process.env.STUDIO_URL || 'http://127.0.0.1:7893'
const SCREENSHOT =
  process.env.THEME_HOME_SHOT ||
  path.join(import.meta.dirname, '../../../.artifacts/pw-theme-home.png')

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

describe('theme home e2e (live studio)', () => {
  it('open /themes → select project theme → Open lands on /themes/:id', async () => {
    if (!(await studioUp())) {
      throw new Error(
        `[FAIL] studio not reachable at ${STUDIO} — boot sail dev before theme-home proof`,
      )
    }

    const { chromium } = await import('playwright')
    const browser = await chromium.launch({ headless: true })
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } })

    try {
      const res = await page.goto(`${STUDIO}/themes`, {
        waitUntil: 'domcontentloaded',
        timeout: 30_000,
      })
      expect(res && res.status() < 500, `themes status ${res?.status()}`).toBe(true)

      await page.waitForSelector('[themes-surface]', { timeout: 20_000 })
      // Library list should appear (project and/or catalog groups).
      await page.waitForFunction(
        () => {
          const surface = document.querySelector('[themes-surface]')
          if (!surface) return false
          return !!(
            document.querySelector('[theme-row]') ||
            document.querySelector('[themes-empty]') ||
            document.querySelector('[themes-loading]')
          )
        },
        { timeout: 20_000 },
      )

      // Honesty smoke on live DOM + theme={id} paint authority
      const honesty = await page.evaluate(() => {
        const text = document.body?.innerText || ''
        const swatch = document.querySelector(
          '[theme-swatches]',
        ) as HTMLElement | null
        const previewHost = document.querySelector(
          '[themes-preview] [preview-specimen], [color-story-grid], [type-story]',
        ) as HTMLElement | null
        const themeHosts = document.querySelectorAll(
          '[themes-surface] [theme]',
        )
        const loadBearingPreview = document.querySelectorAll(
          '[themes-surface] [data-theme-preview]',
        )
        const swatchTheme = swatch?.getAttribute('theme') || ''
        // Table paint: --primary should resolve from themes.css via [theme=id]
        const primary =
          swatch && swatchTheme
            ? getComputedStyle(swatch).getPropertyValue('--primary').trim()
            : ''
        return {
          hasEnablePacks: /Enable catalog packs/i.test(text),
          hasForkWord: /\bfork\b/i.test(text),
          hasEnableSwitch: !!document.querySelector('[data-theme-enable]'),
          hasDefaultStar: !!document.querySelector('[data-theme-default]'),
          hasSurface: !!document.querySelector('[themes-surface]'),
          projectRowCount: document.querySelectorAll(
            '[theme-row][theme-scope="project"]',
          ).length,
          catalogAddCount: document.querySelectorAll('[theme-add-project]').length,
          themeHostCount: themeHosts.length,
          loadBearingPreviewCount: loadBearingPreview.length,
          swatchTheme,
          swatchPrimary: primary,
          previewHostTheme: previewHost?.getAttribute('theme') || null,
        }
      })
      expect(honesty.hasSurface).toBe(true)
      expect(honesty.hasEnablePacks, 'blurb must not say Enable catalog packs').toBe(false)
      expect(honesty.hasForkWord, 'UI must not say fork').toBe(false)
      expect(honesty.hasEnableSwitch, 'no fake enable switch').toBe(false)
      expect(honesty.hasDefaultStar, 'no default star theater').toBe(false)
      // Hard cutover: home hosts use theme=; zero data-theme-preview residue
      expect(
        honesty.themeHostCount,
        'home cards/preview should set theme= for table paint',
      ).toBeGreaterThan(0)
      expect(
        honesty.loadBearingPreviewCount,
        'home must not use data-theme-preview as paint path',
      ).toBe(0)
      if (honesty.swatchTheme) {
        expect(
          honesty.swatchPrimary.length,
          `swatch theme=${honesty.swatchTheme} should resolve --primary from tables`,
        ).toBeGreaterThan(0)
      }

      // Prefer project row → Open
      let opened = false
      if (honesty.projectRowCount > 0) {
        await page.click('[theme-row][theme-scope="project"]')
        await page.waitForTimeout(400)
        const openBtn = page.locator('[preview-open]').first()
        await openBtn.waitFor({ state: 'visible', timeout: 10_000 })
        await openBtn.click()
        await page.waitForTimeout(2500)
        const url = page.url()
        opened =
          /\/themes\/[^/?#]+/.test(url) && !url.endsWith('/themes') && !url.endsWith('/themes/')
        expect(opened, `expected /themes/:id, got ${url}`).toBe(true)
        // Facet chrome or overview markers
        const onFacet = await page.evaluate(
          () =>
            !!document.querySelector(
              '[theme-facet-chrome], [theme-overview-surface], [density-seg], [preview-specimen]',
            ),
        )
        expect(onFacet, 'should land on theme detail / overview').toBe(true)
      } else {
        // Catalog-only fallback: select catalog, Open still navigates (view-only)
        const cat = page.locator('[theme-row][theme-scope="catalog"]').first()
        if ((await cat.count()) > 0) {
          await cat.click()
          await page.waitForTimeout(400)
          const openBtn = page.locator('[preview-open]').first()
          await openBtn.waitFor({ state: 'visible', timeout: 10_000 })
          await openBtn.click()
          await page.waitForTimeout(2500)
          const url = page.url()
          opened =
            /\/themes\/[^/?#]+/.test(url) &&
            !url.endsWith('/themes') &&
            !url.endsWith('/themes/')
          expect(opened, `catalog open → /themes/:id, got ${url}`).toBe(true)
        } else {
          throw new Error('No project or catalog theme rows on /themes')
        }
      }

      // Screenshot for handoff
      try {
        fs.mkdirSync(path.dirname(SCREENSHOT), { recursive: true })
        await page.screenshot({ path: SCREENSHOT, fullPage: true })
      } catch {
        /* non-blocking */
      }
    } finally {
      await browser.close()
    }
  }, 90_000)
})
