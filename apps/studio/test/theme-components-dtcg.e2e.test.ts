/**
 * Job C proof — Components facet DTCG UI (live studio Playwright).
 *
 * Requires sail dev on STUDIO_URL (default http://127.0.0.1:7893).
 * Screenshot: mission scratch pw-components-dtcg-product-tree.png
 *
 *   cd apps/studio && bunx vitest run test/theme-components-dtcg.e2e.test.ts
 *
 * @vitest-environment node
 */
import { describe, expect, it } from 'vitest'
import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'

const STUDIO = process.env.STUDIO_URL || 'http://127.0.0.1:7893'
const SCRATCH_SHOT =
  '/home/madvib/dev/Ship/helm/data/missions/m-sail-studio/scratch/pw-components-dtcg-product-tree.png'
const SCREENSHOT = process.env.COMPONENTS_DTCG_SHOT || SCRATCH_SHOT

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

/** Visible text only (textContent of body); attributes/ids ignored. */
function extractVisibleText(page: import('playwright').Page): Promise<string> {
  return page.evaluate(() => {
    const root =
      (document.querySelector('[data-theme-components-surface]') as HTMLElement | null) ||
      (document.querySelector('[data-components-chrome]') as HTMLElement | null) ||
      document.body
    return (root.innerText || '').replace(/\s+/g, ' ').trim()
  })
}

describe('Job C components DTCG e2e (live studio)', () => {
  it('Alert defaults resolve via DTCG; no var(--)/L1; pick writes override; Clear restores', async () => {
    if (!(await studioUp())) {
      throw new Error(
        `[FAIL] studio not reachable at ${STUDIO} — boot sail dev before Job C proof`,
      )
    }

    const { chromium } = await import('playwright')
    const browser = await chromium.launch({ headless: true })
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } })

    try {
      // Prefer project-editable themes on product tree; studio is catalog (view-only).
      const candidates = [
        `${STUDIO}/themes/my-theme-s99n/components`,
        `${STUDIO}/themes/my-theme-vbtk/components`,
        `${STUDIO}/themes/my-ship/components`,
        `${STUDIO}/themes/studio/components`,
      ]
      let landed = false
      for (const url of candidates) {
        const res = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30_000 })
        if (!res || res.status() >= 500) continue
        await page.waitForTimeout(2500)
        const ok = await page.evaluate(
          () =>
            !!document.querySelector(
              '[data-theme-components-surface], [data-components-workbench], [data-components-rail]',
            ),
        )
        if (ok) {
          landed = true
          break
        }
      }
      expect(landed, 'Components facet surface must load').toBe(true)

      await page.waitForSelector(
        '[data-components-rail], [data-l1-list], [data-components-inspector]',
        { timeout: 20_000 },
      )
      await page.waitForTimeout(800)

      // Select Alert face if not already selected.
      const selected = await page.evaluate(() => {
        const alertRow =
          (document.querySelector('[data-l1="alert"]') as HTMLElement | null) ||
          [...document.querySelectorAll('[data-l1-list] [data-l1], [data-components-rail] [data-l1]')].find(
            (el) => /alert/i.test(el.textContent || ''),
          )
        if (!alertRow) return { clicked: false, reason: 'no alert row' }
        if (alertRow.getAttribute('data-selected') !== 'true') {
          alertRow.click()
        }
        return { clicked: true, id: alertRow.getAttribute('data-l1') }
      })
      expect(selected.clicked, `select Alert: ${JSON.stringify(selected)}`).toBe(true)
      await page.waitForTimeout(1000)

      // Inspector must show component.alert.* color rows, then wait for DTCG hex resolve
      // (themeDoc loads async; docEpoch re-renders knobs once openTheme completes).
      await page.waitForSelector(
        '[data-token-path="component.alert.bg"], [data-component-knob-list] [data-token-path]',
        { timeout: 15_000 },
      )

      await page.waitForFunction(
        () => {
          const row = document.querySelector(
            '[data-token-path="component.alert.bg"]',
          ) as HTMLElement | null
          if (!row || row.tagName === 'SAIL-COLOR-PICKER') return false
          const texts = [...row.querySelectorAll('p')].map((p) => (p.textContent || '').trim())
          return texts.some((t) => /^Default\s*·\s*#[0-9a-fA-F]{6}/i.test(t) || /^#[0-9a-fA-F]{6}/i.test(t))
        },
        { timeout: 15_000 },
      )

      const readAlertRows = () =>
        page.evaluate(() => {
          const pickHexLabel = (row: HTMLElement): string => {
            const texts = [...row.querySelectorAll('p')].map((p) => (p.textContent || '').trim())
            return (
              texts.find((t) => /^Default\s*·\s*#/i.test(t)) ||
              texts.find((t) => /^#[0-9a-fA-F]{6,8}$/i.test(t)) ||
              texts.find((t) => /override/i.test(t)) ||
              texts.find((t) => /^Default/i.test(t)) ||
              ''
            )
          }
          const byPath: Record<
            string,
            { hexLabel: string; hex: string; hasClear: boolean; path: string }
          > = {}
          for (const row of [
            ...document.querySelectorAll('[data-token-path^="component.alert"]'),
          ] as HTMLElement[]) {
            const path = row.getAttribute('data-token-path') || ''
            if (!path.startsWith('component.alert')) continue
            if (row.tagName === 'SAIL-COLOR-PICKER') continue
            if (byPath[path]) continue
            const picker = row.querySelector(
              'sail-color-picker',
            ) as (HTMLElement & { value?: string }) | null
            const clearBtn = [...row.querySelectorAll('button')].find((b) =>
              /clear/i.test(b.textContent || ''),
            )
            byPath[path] = {
              path,
              hexLabel: pickHexLabel(row),
              hex: picker?.value || picker?.getAttribute('value') || '',
              hasClear: !!clearBtn,
            }
          }
          const inspector =
            (document.querySelector('[data-components-inspector]') as HTMLElement | null) ||
            (document.querySelector('[data-components-editor]') as HTMLElement | null)
          const surface =
            (document.querySelector('[data-theme-components-surface]') as HTMLElement | null) ||
            document.body
          const visible = (surface.innerText || '').replace(/\s+/g, ' ')
          const inspectorText = (inspector?.innerText || '').replace(/\s+/g, ' ')
          return {
            paths: Object.keys(byPath).sort(),
            alertBg: byPath['component.alert.bg'],
            alertFg: byPath['component.alert.fg'],
            alertStripe: byPath['component.alert.stripe'],
            visibleHasVar: /var\s*\(\s*--/i.test(visible),
            inspectorHasVar: /var\s*\(\s*--/i.test(inspectorText),
            visibleHasL1Word: /\bL1\b/.test(visible),
            inspectorHasL1Word: /\bL1\b/.test(inspectorText),
            varSnippets: (visible.match(/.{0,24}var\s*\(\s*--[^)]*\).{0,24}/gi) || []).slice(0, 5),
            l1Snippets: (visible.match(/.{0,24}\bL1\b.{0,24}/g) || []).slice(0, 5),
            canEdit: !document.querySelector('[data-catalog-hint]'),
            inspectorText: inspectorText.slice(0, 800),
          }
        })

      const baseline = await readAlertRows()

      // --- Acceptance: no CSS exhaust / no L1 jargon in user-visible text ---
      expect(
        baseline.visibleHasVar || baseline.inspectorHasVar,
        `user-visible var(-- forbidden: ${JSON.stringify(baseline.varSnippets)}`,
      ).toBe(false)
      expect(
        baseline.visibleHasL1Word || baseline.inspectorHasL1Word,
        `user-visible L1 forbidden: ${JSON.stringify(baseline.l1Snippets)}`,
      ).toBe(false)

      expect(baseline.alertBg, `component.alert.bg row present; paths=${baseline.paths.join(',')}`).toBeTruthy()
      expect(baseline.alertFg, 'component.alert.fg row present').toBeTruthy()
      expect(baseline.alertStripe, 'component.alert.stripe row present').toBeTruthy()

      // Defaults: Default · #hex (or override hex) — not empty gray lie, not var()
      for (const [name, row] of [
        ['bg', baseline.alertBg!],
        ['fg', baseline.alertFg!],
        ['stripe', baseline.alertStripe!],
      ] as const) {
        expect(row.hexLabel, `alert ${name} label`).not.toMatch(/var\s*\(/i)
        expect(
          row.hexLabel,
          `alert ${name} should show Default · #hex or override #hex, got "${row.hexLabel}"`,
        ).toMatch(/^(Default\s*·\s*)?#[0-9a-fA-F]{6,8}$|^Default\s*·\s*#[0-9a-fA-F]{6,8}$/i)
        // Swatch value should also be concrete hex when label has hex
        if (row.hex) {
          expect(row.hex, `alert ${name} picker hex`).toMatch(/^#[0-9a-f]{6,8}$/i)
          expect(row.hex.toLowerCase()).not.toBe('#888888')
          expect(row.hex.toLowerCase()).not.toBe('#808080')
        }
      }

      // Prefer non-overridden default state for pick proof; clear first if needed.
      if (baseline.alertBg!.hasClear) {
        await page.evaluate(() => {
          const row = document.querySelector(
            '[data-token-path="component.alert.bg"]',
          ) as HTMLElement | null
          const btn = [...(row?.querySelectorAll('button') ?? [])].find((b) =>
            /clear/i.test(b.textContent || ''),
          )
          btn?.click()
        })
        await page.waitForTimeout(800)
      }

      const beforePick = await page.evaluate(() => {
        const row = document.querySelector(
          '[data-token-path="component.alert.bg"]',
        ) as HTMLElement | null
        const picker = row?.querySelector(
          'sail-color-picker',
        ) as (HTMLElement & { value?: string }) | null
        const texts = [...(row?.querySelectorAll('p') ?? [])].map((p) =>
          (p.textContent || '').trim(),
        )
        const hexLabel =
          texts.find((t) => /^Default/i.test(t) || /^#[0-9a-f]{3,8}$/i.test(t)) || ''
        return {
          hex: picker?.value || picker?.getAttribute('value') || '',
          hexLabel,
          canEdit: !document.querySelector('[data-catalog-hint]'),
        }
      })

      expect(beforePick.canEdit, 'theme must be project-editable for pick/clear proof').toBe(
        true,
      )
      expect(beforePick.hexLabel, 'pre-pick label is Default · #hex').toMatch(
        /^Default\s*·\s*#[0-9a-fA-F]{6,8}$/i,
      )

      // Open Bg color picker and pick a different palette swatch.
      const bgBox = await page.evaluate(() => {
        const row = document.querySelector('[data-token-path="component.alert.bg"]')
        const chip =
          row?.querySelector('[data-token-chip]') ||
          row?.querySelector('sail-color-picker') ||
          row?.querySelector('[data-color-trigger-btn]')
        const r = (chip as HTMLElement | null)?.getBoundingClientRect()
        if (!r || r.width < 4) return null
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
      })
      expect(bgBox, 'Bg swatch hit target').toBeTruthy()
      await page.mouse.click(bgBox!.x, bgBox!.y)
      await page.waitForTimeout(500)

      const picked = await page.evaluate((current) => {
        const pickers = [
          ...document.querySelectorAll(
            'sail-color-picker[data-token-path="component.alert.bg"]',
          ),
        ]
        const el =
          pickers.find((p) => p.querySelector('[data-color-panel]')) || pickers[0]
        const panel = el?.querySelector('[data-color-panel]')
        const btns = [
          ...(panel?.querySelectorAll('button[data-hex]') ?? []),
        ] as HTMLButtonElement[]
        const target =
          btns.find(
            (b) =>
              (b.dataset.hex || '').toLowerCase() !== (current || '').toLowerCase(),
          ) || btns[0]
        if (!target) {
          return {
            ok: false as const,
            reason: 'no swatch',
            panel: !!panel,
            swatches: btns.length,
          }
        }
        const hex = target.dataset.hex!
        target.click()
        return { ok: true as const, hex, swatches: btns.length }
      }, beforePick.hex)

      expect(
        picked.ok,
        `palette pick must succeed: ${JSON.stringify(picked)}`,
      ).toBe(true)
      await page.waitForTimeout(1000)

      const afterPick = await page.waitForFunction(
        (pickedHex) => {
          const row = document.querySelector(
            '[data-token-path="component.alert.bg"]',
          ) as HTMLElement | null
          const picker = row?.querySelector(
            'sail-color-picker',
          ) as (HTMLElement & { value?: string }) | null
          const texts = [...(row?.querySelectorAll('p') ?? [])].map((p) =>
            (p.textContent || '').trim(),
          )
          const hexLabel =
            texts.find(
              (t) =>
                /^Default/i.test(t) ||
                /^#[0-9a-f]{3,8}$/i.test(t) ||
                /override/i.test(t),
            ) || ''
          const clearBtn = [...(row?.querySelectorAll('button') ?? [])].find((b) =>
            /clear/i.test(b.textContent || ''),
          )
          const hex = picker?.value || picker?.getAttribute('value') || ''
          const body = document.body.innerText
          const dirty =
            body.includes('Unsaved') ||
            !!document.querySelector('[data-unsaved], [data-dirty="true"]')
          const isOverride =
            !!clearBtn ||
            (!!hexLabel && !/^Default/i.test(hexLabel)) ||
            (!!hex &&
              !!pickedHex &&
              hex.toLowerCase() !== '' &&
              /^#[0-9a-f]{6}/i.test(hexLabel))
          return {
            ready: isOverride || dirty,
            hex,
            hexLabel,
            hasClear: !!clearBtn,
            dirty,
            pickedHex,
          }
        },
        (picked as { ok: true; hex: string }).hex,
        { timeout: 10_000 },
      )
      const overrideState = await afterPick.jsonValue()

      expect(
        overrideState.hasClear || !/^Default/i.test(overrideState.hexLabel),
        `pick must write component.alert.bg override: ${JSON.stringify(overrideState)}`,
      ).toBe(true)
      expect(overrideState.hexLabel, 'override label must not be var()').not.toMatch(
        /var\s*\(/i,
      )
      if (overrideState.hex) {
        expect(overrideState.hex).toMatch(/^#[0-9a-f]{6,8}$/i)
      }

      // Clear restores Default · #hex
      await page.evaluate(() => {
        const row = document.querySelector(
          '[data-token-path="component.alert.bg"]',
        ) as HTMLElement | null
        const btn = [...(row?.querySelectorAll('button') ?? [])].find((b) =>
          /clear/i.test(b.textContent || ''),
        )
        btn?.click()
      })
      await page.waitForTimeout(1000)

      const afterClear = await page.waitForFunction(
        () => {
          const row = document.querySelector(
            '[data-token-path="component.alert.bg"]',
          ) as HTMLElement | null
          const picker = row?.querySelector(
            'sail-color-picker',
          ) as (HTMLElement & { value?: string }) | null
          const texts = [...(row?.querySelectorAll('p') ?? [])].map((p) =>
            (p.textContent || '').trim(),
          )
          const hexLabel =
            texts.find((t) => /^Default/i.test(t) || /^#[0-9a-f]{3,8}$/i.test(t)) ||
            ''
          const clearBtn = [...(row?.querySelectorAll('button') ?? [])].find((b) =>
            /clear/i.test(b.textContent || ''),
          )
          return {
            ready: /^Default/i.test(hexLabel) && !clearBtn,
            hex: picker?.value || picker?.getAttribute('value') || '',
            hexLabel,
            hasClear: !!clearBtn,
          }
        },
        { timeout: 10_000 },
      )
      const cleared = await afterClear.jsonValue()

      expect(
        cleared.hexLabel,
        `Clear must restore Default · #hex: ${JSON.stringify(cleared)}`,
      ).toMatch(/^Default\s*·\s*#[0-9a-fA-F]{6,8}$/i)
      expect(cleared.hasClear, 'Clear button gone after restore').toBe(false)
      expect(cleared.hexLabel).not.toMatch(/var\s*\(/i)

      // Final visible-text scrub after interactions
      const finalText = await extractVisibleText(page)
      expect(finalText, 'final UI must not show var(--').not.toMatch(/var\s*\(\s*--/i)
      expect(finalText, 'final UI must not show L1 jargon').not.toMatch(/\bL1\b/)

      await page.screenshot({ path: SCREENSHOT, fullPage: false })
      try {
        fs.mkdirSync(path.dirname(SCRATCH_SHOT), { recursive: true })
        fs.copyFileSync(SCREENSHOT, SCRATCH_SHOT)
      } catch {
        // scratch copy best-effort
      }

      const report = {
        url: page.url(),
        baseline: {
          alertBg: baseline.alertBg,
          alertFg: baseline.alertFg,
          alertStripe: baseline.alertStripe,
          visibleHasVar: baseline.visibleHasVar,
          visibleHasL1Word: baseline.visibleHasL1Word,
          paths: baseline.paths,
        },
        beforePick,
        picked,
        overrideState,
        cleared,
        screenshot: SCREENSHOT,
      }
      console.log(JSON.stringify(report, null, 2))
    } finally {
      await browser.close()
    }
  }, 120_000)
})
