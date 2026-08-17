/**
 * Studio shell — operator-visible contract.
 *
 * Encodes the geometry the operator can see: activity rail, editor fill,
 * themes catalog, per-component L1 stage, painted icons, clean console.
 * Red tests are the contract. Do not "fix" the product from this file.
 *
 *   cd apps/studio && bunx playwright test test/studio-shell.e2e.spec.ts
 *
 * Server: reuse http://127.0.0.1:7894 (playwright.config reuseExistingServer).
 */
import { test, expect, type Page, type Request, type Response } from '@playwright/test'

test.setTimeout(90_000)

const VIEWPORT = { width: 1280, height: 800 } as const
const RAIL_WIDTH_PX = 48
const RAIL_WIDTH_SLACK = 8
const EDITOR_STATUS_SLACK = 80

/**
 * Documented pageerror waiver list.
 * Icon glyph 404s are NOT waivers — they fail the network test.
 */
const PAGEERROR_WAIVERS: readonly RegExp[] = [
  // none today — operator-visible runtime errors fail the contract
]

const ICON_GLYPH_404 = /\/_deck\/icons\/[^/?#]+\.(svg|png|webp)(?:[?#]|$)/i

type Rect = {
  x: number
  y: number
  w: number
  h: number
  display: string
  visibility: string
}

type IconInfo = Rect & {
  name: string
  empty: boolean
  hasGlyph: boolean
  inRail: boolean
}

type Capture = {
  vp: { w: number; h: number }
  rail: Rect | null
  editor: Rect | null
  status: Rect | null
  openingSplash: boolean
  bodyHead: string
  icons: IconInfo[]
}

function fmtRect(r: Rect | null): string {
  if (!r) return 'null'
  return `${r.w.toFixed(1)}×${r.h.toFixed(1)} @ (${r.x.toFixed(1)},${r.y.toFixed(1)}) display=${r.display}`
}

function attachConsole(page: Page): { pageerrors: string[]; icon404s: string[] } {
  const pageerrors: string[] = []
  const icon404s: string[] = []
  page.on('pageerror', (e) => {
    const msg = String(e.stack || e.message || e)
    if (PAGEERROR_WAIVERS.some((re) => re.test(msg))) return
    pageerrors.push(msg)
  })
  const note404 = (url: string) => {
    if (ICON_GLYPH_404.test(url) && !icon404s.includes(url)) icon404s.push(url)
  }
  page.on('response', (res: Response) => {
    if (res.status() === 404) note404(res.url())
  })
  page.on('requestfailed', (req: Request) => {
    const url = req.url()
    if (ICON_GLYPH_404.test(url)) note404(url)
  })
  return { pageerrors, icon404s }
}

function activityRail(page: Page) {
  return page
    .getByRole('navigation', { name: 'Studio activities' })
    .or(page.locator('[aria-label="Studio activities"], [label="Studio activities"]'))
    .first()
}

async function gotoSettled(page: Page, path: string): Promise<void> {
  const res = await page.goto(path, { waitUntil: 'domcontentloaded', timeout: 45_000 })
  expect(res, `${path} must respond`).toBeTruthy()
  expect(res!.status(), `${path} status ${res!.status()}`).toBeLessThan(500)
  await activityRail(page).waitFor({ state: 'visible', timeout: 20_000 })
  await page.waitForTimeout(800)
}

async function captureShell(page: Page): Promise<Capture> {
  return page.evaluate(() => {
    const box = (el: Element | null): Rect | null => {
      if (!el) return null
      const r = el.getBoundingClientRect()
      const cs = getComputedStyle(el)
      return {
        x: r.x,
        y: r.y,
        w: r.width,
        h: r.height,
        display: cs.display,
        visibility: cs.visibility,
      }
    }
    const rail =
      document.querySelector('[aria-label="Studio activities"]') ||
      document.querySelector('[label="Studio activities"]') ||
      document.querySelector('nav')
    const editor =
      document.querySelector('[edit-surface]') ||
      document.querySelector('[surface="edit"]') ||
      document.querySelector('[studio-surface]')
    const status = document.querySelector('[studio-status]')
    const surfaceText = ((editor as HTMLElement | null)?.innerText || '').replace(/\s+/g, ' ').trim()
    const openingSplash =
      /^Opening Edit/.test(surfaceText) && surfaceText.length < 48 && !document.querySelector('[edit-surface]')
    const icons = [...document.querySelectorAll('icon, [data-sail-host="icon"]')].map((el) => {
      const r = box(el)!
      const glyph = el.querySelector('svg, img, use, path')
      const glyphBox = glyph ? glyph.getBoundingClientRect() : null
      const hasGlyph = !!(
        glyph &&
        glyphBox &&
        glyphBox.width > 0 &&
        glyphBox.height > 0
      )
      return {
        ...r,
        name: el.getAttribute('name') || '',
        empty: !el.innerHTML.trim(),
        hasGlyph,
        inRail: !!(rail && rail.contains(el)),
      }
    })
    return {
      vp: { w: window.innerWidth, h: window.innerHeight },
      rail: box(rail),
      editor: box(editor),
      status: box(status),
      openingSplash,
      bodyHead: (document.body.innerText || '').replace(/\s+/g, ' ').trim().slice(0, 240),
      icons,
    }
  })
}

function visibleSized(icons: IconInfo[], pred: (i: IconInfo) => boolean = () => true): IconInfo[] {
  return icons.filter(
    (i) => pred(i) && i.w > 2 && i.h > 2 && i.display !== 'none' && i.visibility !== 'hidden',
  )
}

test.describe('studio shell (operator-visible contract)', () => {
  test('activity rail is a full-height size-12 strip, not collapsed, not overlapping the editor', async ({
    page,
  }) => {
    await gotoSettled(page, '/edit')
    const shot = await captureShell(page)
    const rail = shot.rail
    expect(rail, `activity rail missing. body=${shot.bodyHead}`).toBeTruthy()

    const msg = `rail ${fmtRect(rail)} vp=${shot.vp.w}×${shot.vp.h} editor ${fmtRect(shot.editor)}`
    expect(rail!.w, `rail width collapsed — ${msg}`).toBeGreaterThan(RAIL_WIDTH_PX - RAIL_WIDTH_SLACK)
    expect(rail!.w, `rail wider than size-12 (~${RAIL_WIDTH_PX}px) — ${msg}`).toBeLessThan(
      RAIL_WIDTH_PX + RAIL_WIDTH_SLACK + 8,
    )
    expect(rail!.h, `rail not full viewport height — ${msg}`).toBeGreaterThan(shot.vp.h - 4)
    expect(rail!.x, `rail not flush left — ${msg}`).toBeLessThan(2)
    expect(rail!.display, msg).not.toBe('none')
    expect(rail!.visibility, msg).not.toBe('hidden')

    if (shot.editor) {
      expect(
        rail!.x + rail!.w,
        `rail overlaps editor — ${msg}`,
      ).toBeLessThanOrEqual(shot.editor.x + 2)
    }

    const railIcons = visibleSized(shot.icons, (i) => i.inRail)
    expect(
      railIcons.length,
      `rail icons not visible (w>0,h>0). ${msg} icons=${JSON.stringify(railIcons.slice(0, 8))}`,
    ).toBeGreaterThanOrEqual(3)
    const emptyRail = railIcons.filter((i) => i.empty || !i.hasGlyph)
    expect(
      emptyRail,
      `rail icons empty/broken — ${msg} empty=${JSON.stringify(emptyRail.map((i) => i.name))}`,
    ).toEqual([])
  })

  test('editor fills the view: width ≈ viewport − rail, height ≈ viewport', async ({ page }) => {
    await gotoSettled(page, '/edit')
    await page.waitForSelector('[edit-surface], [surface="edit"]', { timeout: 20_000 })
    const shot = await captureShell(page)
    const editor = shot.editor
    const railW = shot.rail?.w ?? RAIL_WIDTH_PX
    const expectedW = shot.vp.w - railW
    const msg = `editor ${fmtRect(editor)} vp=${shot.vp.w}×${shot.vp.h} rail=${fmtRect(shot.rail)} status=${fmtRect(shot.status)} splash=${shot.openingSplash} head=${shot.bodyHead}`

    expect(editor, `main workspace missing — ${msg}`).toBeTruthy()
    expect(shot.openingSplash, `workspace left as Opening Edit… — ${msg}`).toBe(false)
    expect(editor!.h, `editor is a strip / zero height — ${msg}`).toBeGreaterThan(200)
    expect(editor!.w, `editor width not ≈ viewport − rail (${expectedW}) — ${msg}`).toBeGreaterThan(
      expectedW - 24,
    )
    expect(editor!.w, `editor wider than leftover after rail — ${msg}`).toBeLessThan(expectedW + 8)
    expect(editor!.h, `editor height not ≈ viewport — ${msg}`).toBeGreaterThan(
      shot.vp.h - EDITOR_STATUS_SLACK,
    )
    if (shot.rail) {
      expect(editor!.x, `editor should sit to the right of the rail — ${msg}`).toBeGreaterThanOrEqual(
        shot.rail.x + shot.rail.w - 2,
      )
    }
  })

  test('themes catalog shows more than a title — a theme row with a painted box', async ({
    page,
  }) => {
    await gotoSettled(page, '/themes')
    await page.waitForSelector('[themes-surface], [surface="themes"]', { timeout: 20_000 })
    await page.waitForFunction(
      () =>
        !!document.querySelector('[theme-row]') ||
        !!document.querySelector('[themes-empty]') ||
        !!document.querySelector('[themes-loading]'),
      { timeout: 20_000 },
    )

    const catalog = await page.evaluate(() => {
      const box = (el: Element | null) => {
        if (!el) return null
        const r = el.getBoundingClientRect()
        return { x: r.x, y: r.y, w: r.width, h: r.height }
      }
      const surface = document.querySelector('[themes-surface], [surface="themes"]')
      const rows = [...document.querySelectorAll('[theme-row]')]
      const paintedIn = (root: Element) =>
        [...root.querySelectorAll('*')].filter((el) => {
          const cs = getComputedStyle(el)
          const bg = cs.backgroundColor
          if (!bg || bg === 'transparent' || bg === 'rgba(0, 0, 0, 0)') return false
          const r = el.getBoundingClientRect()
          return r.width >= 8 && r.height >= 8
        })
      const rowPaint = rows.map((row) => {
        const cells = paintedIn(row)
        const first = cells[0]
        const r = first ? first.getBoundingClientRect() : null
        return {
          text: (row.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 48),
          row: box(row),
          painted: cells.length,
          firstPaint: r
            ? { w: r.width, h: r.height, bg: getComputedStyle(first!).backgroundColor }
            : null,
        }
      })
      const titleOnly =
        /themes/i.test(document.body.innerText || '') &&
        rows.length === 0 &&
        !document.querySelector('[theme-swatches], [color-story]')
      return {
        surface: box(surface),
        titleOnly,
        rowCount: rows.length,
        rowPaint,
        head: (document.body.innerText || '').replace(/\s+/g, ' ').trim().slice(0, 200),
      }
    })

    const msg = `themes surface=${JSON.stringify(catalog.surface)} rows=${catalog.rowCount} paint=${JSON.stringify(catalog.rowPaint.slice(0, 3))} head=${catalog.head}`
    expect(catalog.titleOnly, `themes is title-only — ${msg}`).toBe(false)
    expect(catalog.rowCount, `no theme row/card — ${msg}`).toBeGreaterThan(0)
    const painted = catalog.rowPaint.filter((r) => r.painted > 0 && r.firstPaint)
    expect(painted.length, `no theme row with a painted box — ${msg}`).toBeGreaterThan(0)
    expect(painted[0]!.firstPaint!.w, msg).toBeGreaterThan(8)
    expect(painted[0]!.firstPaint!.h, msg).toBeGreaterThan(8)
  })

  test('per-component L1 rail lists hosts; default stage has a painted specimen', async ({
    page,
  }) => {
    const candidates = [
      '/themes/studio/components',
      '/themes/my-ship/components',
      '/themes/sail/components',
    ]
    let landed = ''
    for (const url of candidates) {
      const res = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30_000 })
      if (!res || res.status() >= 500) continue
      await page.waitForTimeout(1200)
      const ok = await page.evaluate(
        () =>
          !!document.querySelector(
            '[theme-components-surface], [components-workbench], [components-rail], [l1-list]',
          ),
      )
      if (ok) {
        landed = url
        break
      }
    }
    expect(landed, `no working L1 components URL among ${candidates.join(', ')}`).not.toBe('')

    await page.waitForSelector('[components-rail], [l1-list]', { timeout: 20_000 })
    await page.waitForTimeout(600)

    const stage = await page.evaluate(() => {
      const box = (el: Element | null) => {
        if (!el) return null
        const r = el.getBoundingClientRect()
        const cs = getComputedStyle(el)
        return { x: r.x, y: r.y, w: r.width, h: r.height, display: cs.display }
      }
      const hosts = [...document.querySelectorAll('[l1-list] [l1], [components-rail] [l1]')].map(
        (el) => ({
          id: el.getAttribute('l1') || el.getAttribute('data-l1') || '',
          text: (el.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 40),
          selected: el.getAttribute('selected') === 'true',
          ...box(el)!,
        }),
      )
      const specimen =
        document.querySelector('[preview-specimen], [components-specimen]') ||
        document.querySelector('[l1-live-stage]') ||
        document.querySelector('[l1-stage-host]') ||
        document.querySelector('[components-stage]')
      const painted = specimen
        ? [...specimen.querySelectorAll('button, a, badge, input, [data-sail-host], *')].filter(
            (el) => {
              const r = el.getBoundingClientRect()
              const cs = getComputedStyle(el)
              if (r.width < 8 || r.height < 8 || cs.display === 'none') return false
              const bg = cs.backgroundColor
              return !!(bg && bg !== 'transparent' && bg !== 'rgba(0, 0, 0, 0)')
            },
          )
        : []
      const firstPaint = painted[0]
      const fr = firstPaint ? firstPaint.getBoundingClientRect() : null
      return {
        url: location.pathname,
        hosts,
        specimen: box(specimen),
        paintedCount: painted.length,
        firstPaint: fr
          ? {
              tag: firstPaint!.tagName.toLowerCase(),
              w: fr.width,
              h: fr.height,
              bg: getComputedStyle(firstPaint!).backgroundColor,
            }
          : null,
        head: (document.body.innerText || '').replace(/\s+/g, ' ').trim().slice(0, 200),
      }
    })

    const msg = `url=${stage.url} hosts=${stage.hosts.length} specimen=${JSON.stringify(stage.specimen)} paint=${JSON.stringify(stage.firstPaint)} head=${stage.head}`
    expect(stage.hosts.length, `component rail lists no hosts — ${msg}`).toBeGreaterThan(0)
    expect(stage.specimen, `live stage missing — ${msg}`).toBeTruthy()
    expect(stage.specimen!.w, `stage 0-width — ${msg}`).toBeGreaterThan(40)
    expect(stage.specimen!.h, `stage 0-height / empty — ${msg}`).toBeGreaterThan(40)
    expect(stage.paintedCount, `stage has no painted specimen — ${msg}`).toBeGreaterThan(0)
  })

  test('icons on /edit and the activity rail are painted (not empty boxes)', async ({ page }) => {
    const net = attachConsole(page)
    await gotoSettled(page, '/edit')
    await page.waitForTimeout(600)
    const shot = await captureShell(page)
    const onPage = visibleSized(shot.icons)
    const onRail = onPage.filter((i) => i.inRail)
    const paintedPage = onPage.filter((i) => i.hasGlyph && !i.empty)
    const paintedRail = onRail.filter((i) => i.hasGlyph && !i.empty)
    const msg = [
      `page icons sized=${onPage.length} painted=${paintedPage.length}`,
      `rail sized=${onRail.length} painted=${paintedRail.length}`,
      `rail sample=${JSON.stringify(onRail.slice(0, 8).map((i) => ({ name: i.name, w: i.w, h: i.h, empty: i.empty, hasGlyph: i.hasGlyph })))}`,
      `icon404s=${net.icon404s.join(', ') || '(none)'}`,
    ].join(' | ')

    expect(onPage.length, `fewer than 3 sized icons on /edit — ${msg}`).toBeGreaterThanOrEqual(3)
    expect(onRail.length, `fewer than 3 sized icons on the activity rail — ${msg}`).toBeGreaterThanOrEqual(
      3,
    )
    expect(paintedPage.length, `icons on /edit are empty/broken — ${msg}`).toBeGreaterThanOrEqual(3)
    expect(paintedRail.length, `activity-rail icons are empty/broken — ${msg}`).toBeGreaterThanOrEqual(
      3,
    )
    expect(net.icon404s, `icon glyph 404s (operator: icons fucked) — ${msg}`).toEqual([])
  })

  test('pageerror empty except documented waivers; icon glyph 404s fail', async ({ page }) => {
    const net = attachConsole(page)
    await gotoSettled(page, '/edit')
    await page.waitForTimeout(1000)
    expect(net.pageerrors, `pageerror:\n${net.pageerrors.join('\n---\n')}`).toEqual([])
    expect(
      net.icon404s,
      `network 404 for icon glyphs is a fail, not a waiver:\n${net.icon404s.join('\n')}`,
    ).toEqual([])
  })
})
