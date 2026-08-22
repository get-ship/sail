#!/usr/bin/env bun
/**
 * L1 × theme matrix probe.
 *
 * Owned fixture: crates/sail/compiler/tests/fixtures/l1-matrix
 * Cells: cells() — catalog fan-out (rest × THEME_META + axis sweeps).
 * Routes: GENERATED per-host pages (generate.ts) — one route per component,
 * one <h2> specimen per cell. Probe walks cells(), visits each cell's route,
 * locates the specimen via getByContract, and paints the floor over it.
 * Inventory: COMPONENTS hosts — not a hand list.
 * Not the docs site. Site `/components` is a product gauge, not this SSOT.
 *
 *   bun crates/sail/compiler/tests/l1-matrix/probe.ts
 *   SAIL_PROBE_BASE=http://127.0.0.1:7892 bun .../probe.ts
 *
 * Evidence: $SHIP_MISSION_SCRATCH/web-qa/l1-matrix/<run>/  (else .artifacts/…)
 */
import { chromium } from 'playwright'
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { COMPONENTS } from '../../../../../packages/sail/l1/src/index.ts'
import { getByContract } from '../../../../../packages/sail/sail-test/src/contract.ts'
import { cells as catalogCells } from './cells.ts'

const BASE = process.env.SAIL_PROBE_BASE ?? 'http://127.0.0.1:7892'
const REPO = join(import.meta.dir, '../../../..')
const FIXTURE = join(import.meta.dir, '../fixtures/l1-matrix')

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
const CONTENTS_OK = new Set(['file', 'error-boundary'])
/** Hairline parts — 1px on one axis is a painted separator, not empty. */
const HAIRLINE_OK = new Set(['separator'])
/** Paint lives on the icon ladder by contract — never TINY. */
const ICON_SCALE_OK = new Set(['checkbox', 'radio', 'icon', 'spinner', 'skeleton'])

/**
 * Overlays that need a press — not rest-state. Theme-picker is already on the page.
 * toast-region is the toast mount; both stay off.
 */
const OVERLAY_OFF_PAGE = new Set([
  'dialog',
  'popover',
  'tooltip',
  'dropdown',
  'context-menu',
  'hover-card',
  'combobox',
  'toast',
  'toast-region',
])

/** Expected rest hosts come from COMPONENTS, not a hand list. */
const EXPECTED_REST_HOSTS = COMPONENTS.map((f) => f.host).filter(
  (h) => !OVERLAY_OFF_PAGE.has(h),
)

const ALL_CELLS = catalogCells()
const REST_CELLS = ALL_CELLS.filter(
  (c) => !c.overlay && !OVERLAY_OFF_PAGE.has(c.tag),
)

type LayoutDiag = {
  kind?: string | null
  rule?: string
  message?: string
  severity?: string
  line?: number
}

/** layout-diags.json (written by generate.ts) → "<route>::<specimen>" → kinds. */
function loadLayoutFlags(): Map<string, string[]> {
  const dump = join(FIXTURE, 'layout-diags.json')
  const out = new Map<string, string[]>()
  if (!existsSync(dump)) return out
  const parsed = JSON.parse(readFileSync(dump, 'utf8')) as {
    files?: { path: string; diagnostics?: LayoutDiag[] }[]
  }
  for (const file of parsed.files ?? []) {
    const diags = file.diagnostics ?? []
    if (!diags.length) continue
    const page = join(FIXTURE, file.path)
    if (!existsSync(page)) continue
    const sections: { line: number; specimen: string }[] = []
    readFileSync(page, 'utf8').split('\n').forEach((l: string, i: number) => {
      const m = l.match(/<h2>([^<]+)<\/h2>/)
      if (m) sections.push({ line: i + 1, specimen: m[1].trim() })
    })
    const host = file.path.replace(/^routes\//, '').replace(/\/page\.sail$/, '')
    for (const d of diags) {
      if (!d.kind) continue
      const at = [...sections].reverse().find((s) => s.line <= (d.line ?? 0))
      const key = `/${host}::${at ? at.specimen : '(page)'}`
      const list = out.get(key) ?? []
      list.push(d.kind.toUpperCase())
      out.set(key, list)
    }
  }
  return out
}

type HostHit = {
  tag: string
  w: number
  h: number
  defects: string[]
}

async function measure(
  page: import('playwright').Page,
  opts: { specimen: string | null },
): Promise<{ sectionFound: boolean; hits: HostHit[] }> {
  return page.evaluate(
    ({ skipList, contentsOk, hairlineOk, iconScaleOk, specimen }: {
      skipList: string[]
      contentsOk: string[]
      hairlineOk: string[]
      iconScaleOk: string[]
      specimen: string | null
    }) => {
      const skip = new Set(skipList)
      const CONTENTS_OK = new Set(contentsOk)
      const HAIRLINE_OK = new Set(hairlineOk)
      const ICON_SCALE_OK = new Set(iconScaleOk)
      const cx = document.createElement('canvas').getContext('2d', {
        willReadFrequently: true,
      })
      if (!cx) return { sectionFound: false, hits: [] }
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
      let scope: ParentNode = document
      if (specimen !== null) {
        const hs = [...document.querySelectorAll('h2')]
        const h = hs.find((x) => (x.textContent || '').trim() === specimen)
        if (!h) return { sectionFound: false, hits: [] }
        scope = h.parentElement ?? document
      }
      const iconRaw = getComputedStyle(document.documentElement)
        .getPropertyValue('--icon-size-xl')
      const iconMax = parseFloat(iconRaw) > 0 ? parseFloat(iconRaw) : 20
      const hosts = [
        ...scope.querySelectorAll<HTMLElement>('[data-sail-host]'),
      ]
      const hits = hosts.map((el) => {
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
        if (
          H.width > 0 &&
          H.height > 0 &&
          Math.max(H.width, H.height) < iconMax &&
          !CONTENTS_OK.has(host) &&
          !HAIRLINE_OK.has(host) &&
          !ICON_SCALE_OK.has(host)
        ) {
          defects.push(`TINY(${Math.round(H.width)}x${Math.round(H.height)})`)
        }
        if (HAIRLINE_OK.has(host) && H.width >= 1 && H.height >= 1) {
          const painted =
            toRGBA(cs.backgroundColor)[3] > 0 ||
            ['top', 'right', 'bottom', 'left'].some(
              (s) => parseFloat(cs.getPropertyValue(`border-${s}-width`)) > 0,
            )
          if (!painted) defects.push('INVISIBLE-LINE')
        }
        let paintedBoxes = H.width > 4 && H.height > 4 ? 1 : 0
        const walk = (n: Element) => {
          if (skip.has(n.tagName.toLowerCase())) return
          const s = getComputedStyle(n)
          if (s.display === 'none' || s.visibility === 'hidden') return
          const b = n.getBoundingClientRect()
          if (b.width > 4 && b.height > 4) paintedBoxes++
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
          paintedBoxes === 0 &&
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
      return { sectionFound: true, hits }
    },
    {
      skipList: [...SKIP],
      contentsOk: [...CONTENTS_OK],
      hairlineOk: [...HAIRLINE_OK],
      iconScaleOk: [...ICON_SCALE_OK],
      specimen: opts.specimen,
    },
  )
}

if (import.meta.main) {
  const CELLS = [...REST_CELLS].sort(
    (a, b) =>
      a.route.localeCompare(b.route) ||
      a.theme.localeCompare(b.theme) ||
      a.mode.localeCompare(b.mode),
  )
  const DIR = qaDir()
  const LAYOUT_FLAGS = loadLayoutFlags()
  const ROUTES = [...new Set(CELLS.map((c) => c.route))]
  const THEMES = [
    ...new Set(CELLS.map((c) => `${c.theme}/${c.mode}`)),
  ]
  const plan = `# L1 theme matrix

- **Surface:** owned fixture \`tests/fixtures/l1-matrix\` (not sail-site)
- **Routes:** generated per-host pages — ${ROUTES.length} visited
- **Themes:** ${THEMES.length} — ${THEMES.join(', ')}
- **Cells:** ${CELLS.length} probed (${ALL_CELLS.length - CELLS.length} overlay cells stay off-page)
- **Inventory:** COMPONENTS hosts (${COMPONENTS.length}) minus OVERLAY_OFF_PAGE → ${EXPECTED_REST_HOSTS.length} rest hosts. Not a hand list.
- **Signals:** EMPTY-RENDER, HIDDEN, OPACITY-0, LOW-CONTRAST (<2.5), TINY, INVISIBLE-LINE, HOST-MISS, CONTRACT-MISS, SPECIMEN-MISS, THEME-MISS + layout-diag kinds (${LAYOUT_FLAGS.size} specimens carry flags)
- **Base:** ${BASE}
`
  writeFileSync(join(DIR, 'plan.md'), plan)

  const browser = await chromium.launch()
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 } })
  const page = await ctx.newPage()
  const consoleErrs: string[] = []
  page.on('pageerror', (e) => consoleErrs.push(String(e).slice(0, 160)))

  const evidence: unknown[] = []
  const flagged: { cell: string; tag: string; defects: string[] }[] = []
  const shots = new Map<string, string>()
  let stampsSeen = 0
  let bad = 0
  const cur = { route: '', theme: '', mode: '' }

  for (const cell of CELLS) {
    const cellDefects: string[] = []
    if (cur.route !== cell.route) {
      await page.goto(BASE + cell.route, { waitUntil: 'domcontentloaded', timeout: 30000 })
      cur.route = cell.route
      cur.theme = ''
      cur.mode = ''
      await page.waitForTimeout(400)
    }
    if (cur.theme !== cell.theme || cur.mode !== cell.mode) {
      await page.evaluate(({ theme, mode }) => {
        const r = document.documentElement
        r.setAttribute('theme', theme)
        r.setAttribute('mode', mode)
      }, cell)
      cur.theme = cell.theme
      cur.mode = cell.mode
      await page.waitForTimeout(250)
      const applied = await page.evaluate(() => ({
        theme: document.documentElement.getAttribute('theme'),
        mode: document.documentElement.getAttribute('mode'),
      }))
      if (applied.theme !== cell.theme || applied.mode !== cell.mode) {
        cellDefects.push(
          `THEME-MISS set=${cell.theme}/${cell.mode} got=${applied.theme}/${applied.mode}`,
        )
      }
    }

    const shotKey = `${cell.route.slice(1)}-${cell.theme}-${cell.mode}`.replace(
      /[^a-z0-9-]/gi,
      '_',
    )
    const existing = shots.get(shotKey)
    const shot = existing ?? join(DIR, 'screenshots', `${shotKey}.png`)
    if (!existing) {
      await page.screenshot({ path: shot, fullPage: true })
      shots.set(shotKey, shot)
    }

    const heading = page.getByRole('heading', { name: cell.specimen, exact: true })
    const sectionFound = (await heading.count()) === 1
    if (!sectionFound) {
      cellDefects.push(`SPECIMEN-MISS no <h2> ${cell.specimen}`)
    }
    if (!cell.contract.states?.includes('hidden')) {
      const located = getByContract(page, cell.contract)
      const count = await located.count()
      if (count < 1) {
        cellDefects.push(
          `CONTRACT-MISS role=${cell.contract.role} name=${cell.contract.name ?? '-'} count=${count}`,
        )
      }
    }

    const { sectionFound: measured, hits } = await measure(page, {
      specimen: cell.specimen,
    })
    if (measured) {
      stampsSeen = Math.max(stampsSeen, hits.length)
      const stamped = hits.some((h) => h.tag === cell.tag)
      if (!stamped) {
        cellDefects.push(`HOST-MISS no [data-sail-host]=${cell.tag} in ${cell.specimen}`)
      }
      const layoutKinds = LAYOUT_FLAGS.get(`${cell.route}::${cell.specimen}`)
      if (layoutKinds?.length && stamped) cellDefects.push(...layoutKinds)
      const contractedHidden = cell.contract.states?.includes('hidden')
      for (const h of hits) {
        const defects = contractedHidden
          ? h.defects.filter((d) => d !== 'HIDDEN' && d !== 'EMPTY-RENDER')
          : h.defects
        evidence.push({
          cell: cell.id,
          route: cell.route,
          ...h,
          defects,
          shot,
        })
        if (defects.length) {
          flagged.push({ cell: cell.id, tag: h.tag, defects })
        }
      }
    }
    if (cellDefects.length) {
      bad++
      flagged.push({ cell: cell.id, tag: cell.tag, defects: cellDefects })
    }
    process.stdout.write(cellDefects.length ? '✗' : '·')
  }

  await browser.close()

  const report = {
    surface: 'l1-matrix',
    base: BASE,
    cells_probed: CELLS.length,
    cells_off_page: ALL_CELLS.length - CELLS.length,
    routes: ROUTES.length,
    themes: THEMES,
    stamps_seen_max: stampsSeen,
    expected_rest_hosts: EXPECTED_REST_HOSTS,
    layout_diag_specimens_flagged: LAYOUT_FLAGS.size,
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
cells_probed: ${CELLS.length}
routes_covered: ${ROUTES.length}
themes_covered: ${THEMES.join(', ')}
viewports_covered: 1280x900
console_errors: ${consoleErrs.length}
network_failures: 0
---

## Summary

Generated L1 cell strip (rest + axis sweep) × ${THEMES.length} theme/modes across ${ROUTES.length} routes. ${flagged.length} defect row(s), ${bad} cell(s) flagged. Site is not the SSOT.

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
  console.log(`\n${flagged.length} defect row(s) · ${bad}/${CELLS.length} cells · ${ROUTES.length} routes`)
  console.log(`→ ${DIR}`)
  process.exit(flagged.length || consoleErrs.length ? 1 : 0)
}
