/**
 * Dirty leave dialog is modal isolation, not density bleed.
 *
 * Requires sail dev on STUDIO_URL (default http://127.0.0.1:7893) from product tree.
 * Screenshot: mission scratch pw-leave-dialog-product-tree.png
 *
 *   cd apps/studio && bunx vitest run test/theme-leave-dialog.e2e.test.ts
 *
 * @vitest-environment node
 */
import { describe, expect, it } from 'vitest'
import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'

const STUDIO = process.env.STUDIO_URL || 'http://127.0.0.1:7893'
const SCREENSHOT =
  process.env.LEAVE_DIALOG_SHOT ||
  '/home/madvib/dev/Ship/helm/data/missions/m-sail-studio/scratch/pw-leave-dialog-product-tree.png'
const SCRATCH_SHOT =
  '/home/madvib/dev/Ship/helm/data/missions/m-sail-studio/scratch/pw-leave-dialog-product-tree.png'

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

async function resolveThemeOverviewUrl(page: import('playwright').Page): Promise<string> {
  // Prefer enabled themes known on product-tree disk.
  const candidates = [
    `${STUDIO}/themes/my-theme-s99n`,
    `${STUDIO}/themes/my-theme-vbtk`,
    `${STUDIO}/themes/studio`,
    `${STUDIO}/themes/my-ship`,
    `${STUDIO}/themes`,
  ]
  for (const url of candidates) {
    const res = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30_000 })
    if (!res || res.status() >= 500) continue
    await page.waitForTimeout(2000)
    const hasOverview = await page.evaluate(() => {
      return !!(
        document.querySelector('[density-seg]') ||
        document.querySelector('[theme-facet-chrome]') ||
        document.querySelector('[panel="density"]')
      )
    })
    if (hasOverview) return page.url()
    // Themes list — open first project theme card if present
    if (url.endsWith('/themes')) {
      const opened = await page.evaluate(() => {
        const link =
          (document.querySelector('a[href*="/themes/"]') as HTMLAnchorElement | null) ||
          (document.querySelector('[theme-id]') as HTMLElement | null)
        if (link instanceof HTMLAnchorElement && link.href) {
          return link.href
        }
        if (link) {
          link.click()
          return 'clicked'
        }
        return null
      })
      if (opened && opened !== 'clicked') {
        await page.goto(opened, { waitUntil: 'domcontentloaded', timeout: 30_000 })
        await page.waitForTimeout(2500)
        const ok = await page.evaluate(
          () =>
            !!document.querySelector('[density-seg], [theme-facet-chrome]'),
        )
        if (ok) return page.url()
      } else if (opened === 'clicked') {
        await page.waitForTimeout(2500)
        const ok = await page.evaluate(
          () =>
            !!document.querySelector('[density-seg], [theme-facet-chrome]'),
        )
        if (ok) return page.url()
      }
    }
  }
  throw new Error(
    'No theme overview found (tried my-theme-s99n, my-theme-vbtk, studio, my-ship, themes list)',
  )
}

describe('theme leave dialog e2e (live studio)', () => {
  it('dirty → Back → modal leave dialog isolates density controls', async () => {
    if (!(await studioUp())) {
      throw new Error(
        `[FAIL] studio not reachable at ${STUDIO} — boot sail dev before leave dialog proof`,
      )
    }

    const { chromium } = await import('playwright')
    const browser = await chromium.launch({ headless: true })
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } })
    try {
      await resolveThemeOverviewUrl(page)
      await page.waitForSelector('[theme-facet-chrome], [density-seg]', {
        timeout: 20_000,
      })
      await page.waitForTimeout(1500)

      // Ensure density controls exist
      const densityCount = await page.locator('[density]').count()
      expect(densityCount, 'overview density controls present').toBeGreaterThanOrEqual(2)

      // Make dirty: click a density option that is not currently pressed
      const dirtyed = await page.evaluate(() => {
        const btns = [
          ...document.querySelectorAll('[density-seg] [density]'),
        ] as HTMLButtonElement[]
        const target =
          btns.find((b) => b.getAttribute('aria-pressed') !== 'true') || btns[0]
        if (!target) return { ok: false, reason: 'no density button' }
        target.click()
        return {
          ok: true,
          id: target.getAttribute('density'),
          label: (target.textContent || '').trim(),
        }
      })
      expect(dirtyed.ok, `density click: ${JSON.stringify(dirtyed)}`).toBe(true)
      await page.waitForTimeout(600)

      // Confirm dirty if chip exists; still proceed to Back either way
      const dirtyHint = await page.evaluate(() => {
        const chip = document.querySelector('[state-chip], [data-dirty]')
        const save = document.querySelector(
          '[theme-facet-chrome] button',
        ) as HTMLButtonElement | null
        return {
          chip: chip?.textContent?.trim() || '',
          body: document.body.innerText.slice(0, 400),
        }
      })
      void dirtyHint

      // Click Back (Themes crumb)
      await page.waitForSelector('[facet-back]', { timeout: 10_000 })
      await page.click('[facet-back]')
      await page.waitForTimeout(800)

      // Dialog must appear
      await page.waitForSelector('[theme-leave-dialog]', {
        state: 'visible',
        timeout: 10_000,
      })

      const modalState = await page.evaluate(() => {
        const d = document.querySelector(
          '[theme-leave-dialog]',
        ) as HTMLDialogElement | null
        if (!d) return null
        const openAttr = d.hasAttribute('open') || d.open === true
        const isModal =
          typeof d.matches === 'function' ? d.matches(':modal') : false
        const text = (d.textContent || '').replace(/\s+/g, ' ').trim()
        const hasCancel = !!d.querySelector('[leave-cancel]')
        const hasDiscard = !!d.querySelector('[leave-discard]')
        const hasSave = !!d.querySelector('[leave-save]')
        const labels = [...d.querySelectorAll('button')].map((b) =>
          (b.textContent || '').trim(),
        )
        const densityInside = [...d.querySelectorAll('[density]')].map((el) =>
          (el.textContent || '').trim(),
        )
        const rect = d.getBoundingClientRect()
        const centerX = rect.left + rect.width / 2
        const centerY = rect.top + rect.height / 2
        const topAtCenter = document.elementFromPoint(centerX, centerY)
        const topIsDialog =
          !!topAtCenter &&
          (topAtCenter === d ||
            !!topAtCenter.closest?.('[theme-leave-dialog]'))

        // Density controls should not be the hit target under dialog center
        const densityHits = [...document.querySelectorAll('[density]')].map(
          (el) => {
            const r = el.getBoundingClientRect()
            const cx = r.left + r.width / 2
            const cy = r.top + r.height / 2
            const hit = document.elementFromPoint(cx, cy)
            const blocked =
              !hit ||
              !!hit.closest?.('[theme-leave-dialog]') ||
              hit !== el
            return {
              id: el.getAttribute('density'),
              label: (el.textContent || '').trim(),
              blocked,
              hitTag: hit?.tagName,
              hitDialog: !!hit?.closest?.('[theme-leave-dialog]'),
            }
          },
        )

        return {
          openAttr,
          isModal,
          text,
          hasCancel,
          hasDiscard,
          hasSave,
          labels,
          densityInside,
          topIsDialog,
          densityHits,
          dialogBox: {
            w: rect.width,
            h: rect.height,
            top: rect.top,
            left: rect.left,
          },
        }
      })

      expect(modalState, 'dialog element present').toBeTruthy()
      expect(modalState!.openAttr, 'dialog open').toBe(true)
      // Prefer :modal (showModal top-layer); also accept open dialog with top-layer isolation
      expect(
        modalState!.isModal || modalState!.topIsDialog,
        `dialog must be :modal or top-layer (isModal=${modalState!.isModal}, topIsDialog=${modalState!.topIsDialog})`,
      ).toBe(true)

      expect(modalState!.hasCancel, 'Cancel action').toBe(true)
      expect(modalState!.hasDiscard, 'Discard action').toBe(true)
      expect(modalState!.hasSave, 'Save action').toBe(true)

      // Primary content is leave actions — not density Compact/Comfortable/Roomy
      const densityWords = /compact|comfortable|roomy/i
      expect(
        modalState!.densityInside.length,
        'no [density] nodes inside dialog',
      ).toBe(0)
      const primaryLabels = modalState!.labels.join(' | ')
      expect(primaryLabels).toMatch(/Cancel/i)
      expect(primaryLabels).toMatch(/Discard/i)
      expect(primaryLabels).toMatch(/Save/i)
      // Dialog body must not be density UI as primary content
      expect(
        densityWords.test(modalState!.text) &&
          !/unsaved|discard|cancel/i.test(modalState!.text),
        `dialog text should be leave copy, not density: ${modalState!.text}`,
      ).toBe(false)

      // Density controls must not be clickable through dialog
      const unblocked = modalState!.densityHits.filter((h) => !h.blocked)
      expect(
        unblocked,
        `density hit-through: ${JSON.stringify(modalState!.densityHits)}`,
      ).toEqual([])

      fs.mkdirSync(path.dirname(SCREENSHOT), { recursive: true })
      await page.screenshot({ path: SCREENSHOT, fullPage: false })
      if (SCRATCH_SHOT !== SCREENSHOT) {
        try {
          fs.mkdirSync(path.dirname(SCRATCH_SHOT), { recursive: true })
          fs.copyFileSync(SCREENSHOT, SCRATCH_SHOT)
        } catch {
          // scratch copy best-effort
        }
      }

      console.log(
        JSON.stringify(
          {
            ok: true,
            screenshot: SCREENSHOT,
            scratch: SCRATCH_SHOT,
            modal: {
              isModal: modalState!.isModal,
              topIsDialog: modalState!.topIsDialog,
              labels: modalState!.labels,
              dialogBox: modalState!.dialogBox,
            },
          },
          null,
          2,
        ),
      )
    } finally {
      await browser.close()
    }
  }, 90_000)
})
