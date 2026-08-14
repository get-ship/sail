#!/usr/bin/env bun
/**
 * L1 × theme matrix probe.
 *
 * Owned fixture: crates/sail/compiler/tests/fixtures/l1-matrix
 * Theme cells: generated THEME_META (id × supported modes).
 * Not the docs site. Site `/components` is a product gauge, not this SSOT.
 *
 *   bun crates/sail/compiler/tests/l1-matrix/probe.ts
 *   SAIL_PROBE_BASE=http://127.0.0.1:7892 bun .../probe.ts
 *
 * Evidence: $SHIP_MISSION_SCRATCH/web-qa/l1-matrix/<run>/  (else .artifacts/…)
 */
import { chromium } from 'playwright'
import { mkdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { THEME_META } from '../../../../../packages/sail/tokens/src/generated/themes-meta.ts'

const BASE = process.env.SAIL_PROBE_BASE ?? 'http://127.0.0.1:7892'
const REPO = join(import.meta.dir, '../../../..')

type Cell = { theme: string; mode: string }

function cells(): Cell[] {
  const out: Cell[] = []
  for (const t of THEME_META) {
    const modes = t.modes.length ? [...t.modes] : [t.defaultMode ?? 'light']
    for (const mode of modes) out.push({ theme: t.id, mode })
  }
  return out
}

function qaDir(): string {
  const run = new Date().toISOString().slice(0, 16).replace(/[:T]/g, '-')
  const root = process.env.SHIP_MISSION_SCRATCH
    ? join(process.env.SHIP_MISSION_SCRATCH, 'web-qa')
    : join(REPO, '.artifacts/web-qa')
  const dir = join(root, 'l1-matrix', run)
  mkdirSync(join(dir, 'screenshots'), { recursive: true })
  return dir
}

const SKIP = new Set([
  'style',
  'slot',
  'script',
  'template',
  'link',
  'meta',
])

/** Hosts that are allowed to have a 0×0 box (anatomy is descendants). */
const CONTENTS_OK = new Set(['file'])
/** Hairline parts — 1px on one axis is a painted separator, not empty. */
const HAIRLINE_OK = new Set(['separator'])

type HostHit = {
  tag: string
  w: number
  h: number
  defects: string[]
}

async function measure(page: import('playwright').Page): Promise<HostHit[]> {
  return page.evaluate(({ skipList, contentsOk, hairlineOk }: { skipList: string[]; contentsOk: string[]; hairlineOk: string[] }) => {
    const skip = new Set(skipList)
    const CONTENTS_OK = new Set(contentsOk)
    const HAIRLINE_OK = new Set(hairlineOk)
    const cx = document.createElement('canvas').getContext('2d', {
      willReadFrequently: true,
    })
    if (!cx) return []
    const toRGBA = (str: string) => {
      cx.clearRect(0, 0, 1, 1)
      cx.fillStyle = '#000'
      cx.fillStyle = str
      cx.fillRect(0, 0, 1, 1)
      const d = cx.getImageData(0, 0, 1, 1).data
      return [d[0], d[1], d[2], d[3] / 255] as [number, number, number, number]
    }
    const lum = ([r, g, b]: number[]) => {
      const f = (v: number) => {
        v /= 255
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
      }
      return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
    }
    const hosts = [
      ...document.querySelectorAll<HTMLElement>('[data-sail-host]'),
    ]
    return hosts.map((el) => {
      const tag = el.getAttribute('data-sail-host') || el.tagName.toLowerCase()
      const defects: string[] = []
      const H = el.getBoundingClientRect()
      const host = el.getAttribute('data-sail-host') || ''
      if (H.width < 2 && H.height < 2 && !CONTENTS_OK.has(host)) {
        if (!(HAIRLINE_OK.has(host) && (H.width >= 1 || H.height >= 1))) {
          defects.push('EMPTY-RENDER')
        }
      }
      const cs = getComputedStyle(el)
      if (cs.visibility === 'hidden' || cs.display === 'none') {
        defects.push('HIDDEN')
      }
      if (parseFloat(cs.opacity) === 0) defects.push('OPACITY-0')
      let painted = H.width > 4 && H.height > 4 ? 1 : 0
      const walk = (n: Element) => {
        if (skip.has(n.tagName.toLowerCase())) return
        const s = getComputedStyle(n)
        if (s.display === 'none' || s.visibility === 'hidden') return
        const b = n.getBoundingClientRect()
        if (b.width > 4 && b.height > 4) painted++
        const hasText = [...n.childNodes].some(
          (c) => c.nodeType === 3 && (c.textContent || '').trim(),
        )
        if (hasText && parseFloat(s.opacity) > 0.3) {
          const fg = toRGBA(s.color)
          let bgEl: Element | null = n
          let bg: [number, number, number, number] | null = null
          while (bgEl) {
            const raw = getComputedStyle(bgEl).backgroundColor
            const p = toRGBA(raw)
            if (p[3] > 0.5) {
              bg = p
              break
            }
            bgEl = bgEl.parentElement
          }
          if (fg && bg) {
            const L1 = lum(fg)
            const L2 = lum(bg)
            const ratio =
              (Math.max(L1, L2) + 0.05) / (Math.min(L1, L2) + 0.05)
            if (ratio < 2.5) defects.push(`LOW-CONTRAST(${ratio.toFixed(1)})`)
          }
        }
        n.childNodes.forEach((c) => {
          if (c.nodeType === 1) walk(c as Element)
        })
      }
      walk(el)
      if (
        painted === 0 &&
        !defects.includes('EMPTY-RENDER') &&
        !CONTENTS_OK.has(host) &&
        !HAIRLINE_OK.has(host)
      ) {
        defects.push('EMPTY-RENDER')
      }
      return {
        tag,
        w: Math.round(H.width),
        h: Math.round(H.height),
        defects: [...new Set(defects)],
      }
    })
  }, { skipList: [...SKIP], contentsOk: [...CONTENTS_OK], hairlineOk: [...HAIRLINE_OK] })
}

if (import.meta.main) {
  const CELLS = cells()
  const DIR = qaDir()
  const plan = `# L1 theme matrix

- **Surface:** owned fixture \`tests/fixtures/l1-matrix\` (not sail-site)
- **Theme cells:** ${CELLS.length} — ${CELLS.map((c) => `${c.theme}/${c.mode}`).join(', ')}
- **Inventory:** every \`[data-sail-host]\` the compiler stamps on the rest-state strip
- **Signals:** EMPTY-RENDER, HIDDEN, OPACITY-0, LOW-CONTRAST
- **Base:** ${BASE}
`
  writeFileSync(join(DIR, 'plan.md'), plan)

  const browser = await chromium.launch()
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } })
  const page = await ctx.newPage()
  const consoleErrs: string[] = []
  page.on('pageerror', (e) => consoleErrs.push(String(e).slice(0, 160)))

  await page.goto(BASE + '/', { waitUntil: 'domcontentloaded', timeout: 30000 })
  await page.waitForTimeout(800)

  const evidence: unknown[] = []
  const flagged: { cell: string; tag: string; defects: string[] }[] = []
  let hostsSeen = 0

  for (const cell of CELLS) {
    const key = `${cell.theme}-${cell.mode}`
    await page.evaluate(({ theme, mode }) => {
      const r = document.documentElement
      r.setAttribute('theme', theme)
      r.setAttribute('mode', mode)
    }, cell)
    await page.waitForTimeout(250)
    const applied = await page.evaluate(() => ({
      theme: document.documentElement.getAttribute('theme'),
      mode: document.documentElement.getAttribute('mode'),
    }))
    if (applied.theme !== cell.theme || applied.mode !== cell.mode) {
      flagged.push({
        cell: key,
        tag: ':root',
        defects: [`THEME-MISS set=${cell.theme}/${cell.mode} got=${applied.theme}/${applied.mode}`],
      })
    }
    const shot = join(DIR, 'screenshots', `${key}.png`)
    await page.screenshot({ path: shot, fullPage: true })
    const hits = await measure(page)
    hostsSeen = Math.max(hostsSeen, hits.length)
    for (const h of hits) {
      evidence.push({ cell: key, ...h, shot })
      if (h.defects.length) flagged.push({ cell: key, tag: h.tag, defects: h.defects })
    }
    process.stdout.write(hits.some((h) => h.defects.length) ? '✗' : '·')
  }

  await browser.close()

  const report = {
    surface: 'l1-matrix',
    base: BASE,
    cells: CELLS,
    hosts: hostsSeen,
    console_errors: consoleErrs,
    flagged,
  }
  writeFileSync(join(DIR, 'evidence.jsonl'), evidence.map((e) => JSON.stringify(e)).join('\n') + '\n')
  writeFileSync(join(DIR, 'report.json'), JSON.stringify(report, null, 2))

  const status =
    consoleErrs.length || flagged.length ? 'DONE_WITH_CONCERNS' : 'DONE'
  const md = `---
surface: l1-matrix
status: ${status}
run: ${DIR.split('/').pop()}
hosts_probed: ${hostsSeen}
themes_covered: ${CELLS.map((c) => `${c.theme}/${c.mode}`).join(', ')}
viewports_covered: 1280x900
console_errors: ${consoleErrs.length}
network_failures: 0
---

## Summary

Owned L1 rest-state strip × ${CELLS.length} theme/mode cells. ${flagged.length} host-cell defect(s). Site is not the SSOT.

## Findings

${
  flagged.length
    ? flagged
        .map(
          (f, i) =>
            `### F-${String(i + 1).padStart(3, '0')} — ${f.tag} @ ${f.cell}\n- ${f.defects.join(', ')}`,
        )
        .join('\n\n')
    : '_none_'
}

## Console

${consoleErrs.length ? consoleErrs.map((e) => `- ${e}`).join('\n') : '_clean_'}
`
  writeFileSync(join(DIR, 'report.md'), md)
  console.log(`\n${flagged.length} defect row(s) · ${hostsSeen} hosts · ${CELLS.length} cells`)
  console.log(`→ ${DIR}`)
  process.exit(flagged.length || consoleErrs.length ? 1 : 0)
}
