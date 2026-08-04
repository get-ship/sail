/**
 * Job 1 / Job B proof — type role edit changes preview face (host CSS var / computed font).
 *
 * Requires sail dev on STUDIO_URL (default http://127.0.0.1:7893) from product tree.
 * Screenshot: mission scratch pw-type-face-product-tree.png
 *
 *   cd apps/studio && bunx vitest run test/theme-type-face.e2e.test.ts
 *
 * @vitest-environment node
 */
import { describe, expect, it } from 'vitest'
import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'

const STUDIO = process.env.STUDIO_URL || 'http://127.0.0.1:7893'
const SCREENSHOT =
  process.env.TYPE_FACE_SHOT ||
  '/home/madvib/dev/Ship/helm/data/missions/m-sail-studio/scratch/pw-type-face-product-tree.png'
const SCRATCH_SHOT =
  '/home/madvib/dev/Ship/helm/data/missions/m-sail-studio/scratch/pw-type-face-product-tree.png'

/** Families available on overview Display select (see theme-overview-edit.ts). */
const DISTINCTIVE = [
  'Source Serif 4',
  'Syne',
  'Cal Sans',
  'Satoshi',
  'Fraunces',
  'Inter',
] as const

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
        document.querySelector('[data-type-role="display"]') ||
        document.querySelector('[data-preview-specimen]') ||
        document.querySelector('[data-theme-overview-surface]')
      )
    })
    if (hasOverview) return page.url()
    if (url.endsWith('/themes')) {
      const opened = await page.evaluate(() => {
        const link =
          (document.querySelector('a[href*="/themes/"]') as HTMLAnchorElement | null) ||
          (document.querySelector('[data-theme-id]') as HTMLElement | null)
        if (link instanceof HTMLAnchorElement && link.href) return link.href
        if (link) {
          link.click()
          return 'clicked'
        }
        return null
      })
      if (opened && opened !== 'clicked') {
        await page.goto(opened, { waitUntil: 'domcontentloaded', timeout: 30_000 })
        await page.waitForTimeout(2500)
      } else if (opened === 'clicked') {
        await page.waitForTimeout(2500)
      }
      const ok = await page.evaluate(
        () => !!document.querySelector('[data-type-role="display"], [data-preview-specimen]'),
      )
      if (ok) return page.url()
    }
  }
  throw new Error(
    'No theme overview found (tried my-theme-s99n, my-theme-vbtk, studio, my-ship, themes list)',
  )
}

function faceToken(name: string): string {
  // Match family token in CSS stacks (quoted or bare).
  return name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

describe('Job 1 type face e2e (live studio product tree)', () => {
  it('set Display type role → preview host --display-font / computed face updates', async () => {
    if (!(await studioUp())) {
      throw new Error(
        `[FAIL] studio not reachable at ${STUDIO} — boot sail dev before Job 1 proof`,
      )
    }

    const { chromium } = await import('playwright')
    const browser = await chromium.launch({ headless: true })
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } })
    try {
      await resolveThemeOverviewUrl(page)
      await page.waitForSelector('[data-type-role="display"]', { timeout: 20_000 })
      await page.waitForSelector('[data-preview-specimen], [data-type-samples]', {
        timeout: 20_000,
      })
      await page.waitForTimeout(1500)

      // Baseline paint props before edit
      const before = await page.evaluate(() => {
        const host =
          (document.querySelector('[data-preview-specimen]') as HTMLElement | null) ||
          (document.querySelector('[data-overview-specimen]') as HTMLElement | null)
        const select = document.querySelector(
          '[data-type-role="display"]',
        ) as HTMLSelectElement | null
        const samples = document.querySelector('[data-type-samples]')
        const displayEl = samples?.querySelector('p') as HTMLElement | null
        const hostDisplay =
          host?.style.getPropertyValue('--display-font')?.trim() ||
          (host ? getComputedStyle(host).getPropertyValue('--display-font').trim() : '')
        const computed = displayEl ? getComputedStyle(displayEl).fontFamily : ''
        const opts = select
          ? [...select.options].map((o) => o.value)
          : ([] as string[])
        return {
          selectValue: select?.value ?? '',
          selectDisabled: !!select?.disabled,
          opts,
          hostDisplay,
          computed,
          draftPaint: host?.getAttribute('data-draft-paint') ?? null,
          hasHost: !!host,
          hasSamples: !!samples,
        }
      })

      expect(before.hasHost || before.hasSamples, `preview host/samples present: ${JSON.stringify(before)}`).toBe(
        true,
      )
      expect(before.selectDisabled, 'Display select must be editable (project theme)').toBe(
        false,
      )

      // Pick a distinctive family different from current select value
      const current = before.selectValue
      const want =
        DISTINCTIVE.find((f) => f !== current && before.opts.includes(f)) ||
        before.opts.find((o) => o && o !== current) ||
        null
      expect(want, `need alternate Display option (current=${current}, opts=${before.opts.join('|')})`).toBeTruthy()

      // Prefer serif/sans names that are NOT pixel aliases when proving overwrite
      const chosen = want as string

      const setResult = await page.evaluate((family) => {
        const select = document.querySelector(
          '[data-type-role="display"]',
        ) as HTMLSelectElement | null
        if (!select) return { ok: false as const, reason: 'no select' }
        // Ensure option exists
        let opt = [...select.options].find((o) => o.value === family)
        if (!opt) {
          opt = document.createElement('option')
          opt.value = family
          opt.textContent = family
          select.appendChild(opt)
        }
        select.value = family
        select.dispatchEvent(new Event('change', { bubbles: true }))
        return { ok: true as const, value: select.value }
      }, chosen)

      expect(setResult.ok, `set Display: ${JSON.stringify(setResult)}`).toBe(true)
      // Wait for draft paint rAF + store emit
      await page.waitForTimeout(1200)

      // Poll until host prop updates or timeout
      const after = await page.waitForFunction(
        (family) => {
          const host =
            (document.querySelector('[data-preview-specimen]') as HTMLElement | null) ||
            (document.querySelector('[data-overview-specimen]') as HTMLElement | null)
          const samples = document.querySelector('[data-type-samples]')
          const displayEl = samples?.querySelector('p') as HTMLElement | null
          const hostInline = host?.style.getPropertyValue('--display-font')?.trim() || ''
          const hostComputed = host
            ? getComputedStyle(host).getPropertyValue('--display-font').trim()
            : ''
          const hostDisplay = hostInline || hostComputed
          const computed = displayEl ? getComputedStyle(displayEl).fontFamily : ''
          const sampleStyle = displayEl?.style.fontFamily || ''
          const draftPaint = host?.getAttribute('data-draft-paint') ?? null
          const select = document.querySelector(
            '[data-type-role="display"]',
          ) as HTMLSelectElement | null
          const label = (displayEl?.textContent || '').trim()
          const needle = family.toLowerCase()
          const hit =
            hostDisplay.toLowerCase().includes(needle) ||
            computed.toLowerCase().includes(needle)
          return {
            ready: hit || draftPaint === 'apply' || hostDisplay.length > 0,
            hostDisplay,
            hostInline,
            hostComputed,
            computed,
            sampleStyle,
            draftPaint,
            selectValue: select?.value ?? '',
            label,
            family,
          }
        },
        chosen,
        { timeout: 10_000 },
      )

      const face = await after.jsonValue()

      await page.screenshot({ path: SCREENSHOT, fullPage: false })
      try {
        fs.mkdirSync(path.dirname(SCRATCH_SHOT), { recursive: true })
        if (SCREENSHOT !== SCRATCH_SHOT) {
          fs.copyFileSync(SCREENSHOT, SCRATCH_SHOT)
        }
      } catch {
        // scratch copy best-effort
      }

      const re = new RegExp(faceToken(chosen), 'i')
      const hostStack = face.hostDisplay || face.hostInline || face.hostComputed
      const computedFace = face.computed || ''

      // Primary: host CSS custom property from applyScope must carry the stack
      const hostOk = re.test(hostStack)
      // Secondary: computed font-family on type sample (may fall back if webfont missing)
      const computedOk = re.test(computedFace)

      // Pixel/alias regression: after setting a serif/sans name, must not still be press-start only
      const stillPixel =
        /press\s*start|font-press-start/i.test(hostStack) && !re.test(hostStack)

      const report = {
        before: {
          selectValue: before.selectValue,
          hostDisplay: before.hostDisplay,
          computed: before.computed,
          draftPaint: before.draftPaint,
        },
        chosen,
        after: face,
        hostOk,
        computedOk,
        stillPixel,
        screenshot: SCREENSHOT,
      }

      console.log(JSON.stringify(report, null, 2))

      expect(face.selectValue, 'select value tracks chosen family').toBe(chosen)
      expect(
        stillPixel,
        `host still pixel/press-start after set to ${chosen}: ${hostStack}`,
      ).toBe(false)

      // Job 5d: draft host paint authority is theme= + applyScope; no data-theme-preview
      const attrProof = await page.evaluate(() => {
        const host =
          (document.querySelector('[data-preview-specimen]') as HTMLElement | null) ||
          (document.querySelector('[data-overview-specimen]') as HTMLElement | null)
        return {
          theme: host?.getAttribute('theme') ?? null,
          dataThemePreview: host?.getAttribute('data-theme-preview') ?? null,
          draftPaint: host?.getAttribute('data-draft-paint') ?? null,
        }
      })
      expect(
        attrProof.theme,
        'draft host must carry theme= after applyScope',
      ).toBeTruthy()
      expect(
        attrProof.dataThemePreview,
        'draft host must not set data-theme-preview',
      ).toBeNull()
      expect(attrProof.draftPaint, 'draft paint path').toBe('apply')

      // PASS if host prop has the family (preferred when webfont not installed)
      // OR computed face contains it.
      expect(
        hostOk || computedOk,
        [
          `[FAIL] Display → ${chosen} did not update preview face.`,
          `host --display-font: ${hostStack || '(empty)'}`,
          `computed font-family: ${computedFace || '(empty)'}`,
          `draftPaint: ${face.draftPaint}`,
          `select: ${face.selectValue}`,
          `label text: ${face.label}`,
        ].join('\n'),
      ).toBe(true)

      // If host prop is present, it must not be an unresolved var() of the old pixel chain
      if (hostStack) {
        expect(
          hostStack,
          'host --display-font should be concrete stack, not only var() alias',
        ).not.toMatch(/^var\(--font-press-start/)
      }
    } finally {
      await browser.close()
    }
  }, 90_000)
})
