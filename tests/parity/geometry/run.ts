#!/usr/bin/env bun
/**
 * PARITY-1 geometry suite — taffy ↔ browser box tree.
 *
 * For each fixture under ./fixtures:
 *   1. Taffy boxes via `sail op SailAuthoring/LayoutCheck` (preprocess + check_preprocessed).
 *   2. Browser boxes via sail build SPA (or lowered-HTML fallback) + Playwright
 *      getBoundingClientRect on chromium, firefox, and webkit.
 *   3. Diff report (width/height + parent-relative x/y within tolerance).
 *
 * Paint (takumi ↔ browser style) lives in tests/parity/paint/run.ts (RENDER-1).
 *
 * Controller-driven content contract (minimal, suite honesty):
 *   Static render / geometry fixtures MUST use static template text only.
 *   Signal-driven or deferred controller content is out of scope for this suite
 *   and will intentionally show empty/placeholder boxes — do not add such
 *   fixtures here until an SSR-defaults contract is defined product-wide.
 *
 * Invoke:
 *   bun tests/parity/geometry/run.ts
 *   bun tests/parity/geometry/run.ts --browser chromium
 *   bun tests/parity/geometry/run.ts --browser chromium,firefox,webkit
 *   bun tests/parity/geometry/run.ts --fixture column-stack
 *   bun tests/parity/geometry/run.ts --strict
 *   SAIL_BIN=target/release/sail bun tests/parity/geometry/run.ts
 *
 * Exit codes (default / non-report-only):
 *   0 — harness ok AND structure_ok for every fixture (size Δ reported, non-fatal)
 *   1 — taffy/browser failure, structure fail, or (with --strict) size Δ
 * --report-only: exit 0 whenever taffy+browser ran; structure/size only in summary.
 */

import { spawnSync } from 'node:child_process'
import {
  mkdirSync,
  writeFileSync,
  readFileSync,
  readdirSync,
  existsSync,
  cpSync,
  rmSync,
  statSync,
} from 'node:fs'
import { dirname, join, resolve, basename } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createServer } from 'node:http'
import { createReadStream } from 'node:fs'
import { tmpdir } from 'node:os'
import net from 'node:net'
import { loadPlaywright, parseBrowsers, type BrowserName } from '../browsers.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(__dirname, '../../..')
const fixturesDir = join(__dirname, 'fixtures')
const outDir = join(__dirname, 'out')

// LayoutCheck reports fixed mobile/desktop viewports (see sail_compiler layout_viewports).
// Browser measurement uses the same desktop size so geometry lines up.
const VIEWPORT = { width: 1440, height: 900 }
const TOL = Number(process.env.PARITY_GEOM_TOL || 2.0) // px
const SAIL_BIN =
  process.env.SAIL_BIN ||
  (existsSync(join(repoRoot, 'target/release/sail'))
    ? join(repoRoot, 'target/release/sail')
    : join(repoRoot, 'target/debug/sail'))
const SAIL_COMPILER =
  process.env.SAIL_COMPILER ||
  (existsSync(join(repoRoot, 'target/release/sail-compiler'))
    ? join(repoRoot, 'target/release/sail-compiler')
    : join(repoRoot, 'target/debug/sail-compiler'))
const args = process.argv.slice(2)
const skipBrowser = args.includes('--taffy-only') || process.env.PARITY_TAFFY_ONLY === '1'
const browsers: BrowserName[] = skipBrowser ? [] : parseBrowsers(args)
const strict = args.includes('--strict') || process.env.PARITY_GEOM_STRICT === '1'
const reportOnly = args.includes('--report-only')
const strictSpa =
  args.includes('--strict-spa') || process.env.PARITY_GEOM_STRICT_SPA === '1'
const fixtureFilter = (() => {
  const i = args.indexOf('--fixture')
  return i >= 0 ? args[i + 1] : null
})()

function die(msg) {
  console.error(`error: ${msg}`)
  process.exit(1)
}

function freePort() {
  return new Promise((resolveP, reject) => {
    const s = net.createServer()
    s.listen(0, '127.0.0.1', () => {
      const { port } = s.address()
      s.close(() => resolveP(port))
    })
    s.on('error', reject)
  })
}

function listFixtures() {
  return readdirSync(fixturesDir)
    .filter((f) => f.endsWith('.sail'))
    .filter((f) => !fixtureFilter || f.startsWith(fixtureFilter) || f === `${fixtureFilter}.sail`)
    .sort()
}

function layoutCheck(source) {
  if (!existsSync(SAIL_BIN)) die(`missing sail binary at ${SAIL_BIN}`)
  const payload = JSON.stringify({ source })
  const r = spawnSync(
    SAIL_BIN,
    ['op', 'SailAuthoring/LayoutCheck', '--json', payload],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  )
  if (r.status !== 0) {
    throw new Error(`LayoutCheck failed: ${r.stderr || r.stdout}`)
  }
  const out = JSON.parse(r.stdout)
  const vp = out.viewports?.find(
    (v) =>
      Math.abs(v.viewport?.width - VIEWPORT.width) < 1 &&
      Math.abs(v.viewport?.height - VIEWPORT.height) < 1,
  )
  // LayoutCheck uses fixed mobile/desktop viewports; pick closest width or first.
  let report = vp
  if (!report) {
    const sorted = [...(out.viewports || [])].sort(
      (a, b) =>
        Math.abs((a.viewport?.width || 0) - VIEWPORT.width) -
        Math.abs((b.viewport?.width || 0) - VIEWPORT.width),
    )
    report = sorted[0]
  }
  if (!report || report.error) {
    throw new Error(`no taffy boxes: ${JSON.stringify(report || out)}`)
  }
  return { engine: out.engine, viewport: report.viewport, boxes: report.boxes || [] }
}

/**
 * Prefer a full `sail build` SPA so browser layout runs the real runtime path.
 * Falls back to lowered static HTML from sail-compiler __tpl emit.
 * Minimal temp projects often lack framework resolution — expect lowered-html
 * unless a full deck project is staged; use `--strict-spa` to fail on fallback.
 */
function stageAndBuild(fixturePath, stageRoot) {
  rmSync(stageRoot, { recursive: true, force: true })
  mkdirSync(join(stageRoot, 'routes'), { recursive: true })
  mkdirSync(join(stageRoot, 'public'), { recursive: true })
  const pageName = 'page.sail'
  cpSync(fixturePath, join(stageRoot, 'routes', pageName))
  writeFileSync(
    join(stageRoot, 'package.json'),
    JSON.stringify(
      {
        name: 'parity-geom-fixture',
        private: true,
        type: 'module',
      },
      null,
      2,
    ),
  )
  writeFileSync(join(stageRoot, 'public', '.gitkeep'), '')

  const out = join(stageRoot, 'dist')
  const build = spawnSync(
    SAIL_BIN,
    [
      'build',
      '--project',
      stageRoot,
      '--out',
      out,
      '--bundle-framework',
      '--base',
      'assets',
    ],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  )
  const log = (build.stdout || '') + (build.stderr || '')
  if (build.status === 0 && existsSync(join(out, 'index.html'))) {
    return { mode: 'sail-build', serveDir: out, log: log.slice(-1500) }
  }

  // Fallback: sail-compiler + __tpl extraction
  const compileOut = join(stageRoot, 'compile-out')
  mkdirSync(compileOut, { recursive: true })
  const comp = spawnSync(
    SAIL_COMPILER,
    ['build', join(stageRoot, 'routes', pageName), '--out', compileOut, '--clean'],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  )
  const clog = (comp.stdout || '') + (comp.stderr || '')
  if (comp.status !== 0) {
    throw new Error(`build failed:\n${log.slice(-2000)}\ncompile:\n${clog.slice(-2000)}`)
  }
  const jsFiles = readdirSync(compileOut).filter((f) => f.endsWith('.js'))
  if (!jsFiles.length) throw new Error(`no compiled js: ${clog.slice(-1000)}`)
  const jsText = readFileSync(join(compileOut, jsFiles[0]), 'utf8')
  const parts = extractTplHtml(jsText)
  const serve = join(stageRoot, 'lowered-static')
  writeLoweredBundle(serve, parts)
  const spaHint = (log || '').trim().slice(-400)
  console.warn(
    `warning: ${basename(fixturePath)}: sail build SPA unavailable; falling back to lowered-html` +
      (spaHint ? `\n  sail-build tail: ${spaHint.replace(/\s+/g, ' ').slice(0, 240)}` : ''),
  )
  return {
    mode: 'lowered-html',
    serveDir: serve,
    log: clog.slice(-1500),
    spa_fallback: true,
    spa_log_tail: spaHint,
  }
}

function extractTplHtml(jsText) {
  const parts = []
  const re = /__tpl\(\s*"((?:[^"\\]|\\.)*)"\s*\)/g
  let m
  while ((m = re.exec(jsText))) parts.push(decodeJsString(m[1]))
  const re2 = /__tpl\(\s*'((?:[^'\\]|\\.)*)'\s*\)/g
  while ((m = re2.exec(jsText))) parts.push(decodeJsString(m[1]))
  return parts
}

function decodeJsString(s) {
  return s
    .replace(/\\n/g, '\n')
    .replace(/\\r/g, '\r')
    .replace(/\\t/g, '\t')
    .replace(/\\"/g, '"')
    .replace(/\\'/g, "'")
    .replace(/\\\\/g, '\\')
}

function writeLoweredBundle(serveDir, parts) {
  rmSync(serveDir, { recursive: true, force: true })
  mkdirSync(serveDir, { recursive: true })
  const tokens = join(repoRoot, 'packages/sail/tokens/src')
  const cssFiles = ['tokens.css', 'themes.css', 'sail-base.css', 'element-skins.css']
  const links = []
  for (const name of cssFiles) {
    const src = join(tokens, name)
    if (!existsSync(src)) continue
    const dest = join(serveDir, name)
    cpSync(src, dest)
    links.push(name)
  }
  // fonts relative to sail-base
  const fontsSrc = join(tokens, 'fonts')
  if (existsSync(fontsSrc)) {
    cpSync(fontsSrc, join(serveDir, 'fonts'), { recursive: true })
  }
  const body =
    parts.map((h, i) => `<section class="tpl-frag" data-i="${i}">${h}</section>`).join('\n') ||
    '<p><em>no __tpl</em></p>'
  writeFileSync(
    join(serveDir, 'index.html'),
    `<!doctype html>
<html lang="en" theme="sail" data-mode="light">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>parity-geometry</title>
${links.map((c) => `<link rel="stylesheet" href="${c}"/>`).join('\n')}
<style>
  /* Fill viewport so min-h(screen)/100vh hosts match taffy's viewport height. */
  html, body { margin: 0; padding: 0; width: 100%; height: 100%; }
  #root { width: 100%; min-height: 100%; height: 100%; box-sizing: border-box; }
  .tpl-frag { display: contents; }
</style>
</head>
<body>
<div id="root">${body}</div>
</body>
</html>
`,
  )
}

type Measured =
  | { skipped: true; browser: BrowserName; reason: string }
  | {
      skipped: false
      browser: BrowserName
      boxes: Array<{
        tag: string
        class: string
        x: number
        y: number
        width: number
        height: number
        offset: { x: number; y: number; w: number; h: number }
      }>
    }

async function serveStatic(serveDir: string): Promise<{ url: string; close: () => Promise<void> }> {
  const port = await freePort()
  const server = await new Promise<import('node:http').Server>((resolveP, reject) => {
    const srv = createServer((req, res) => {
      let urlPath = decodeURIComponent((req.url || '/').split('?')[0])
      if (urlPath === '/') urlPath = '/index.html'
      const file = join(serveDir, urlPath.replace(/^\//, ''))
      if (!file.startsWith(serveDir) || !existsSync(file) || statSync(file).isDirectory()) {
        res.writeHead(404)
        res.end('not found')
        return
      }
      const ext = file.split('.').pop()
      const types: Record<string, string> = {
        html: 'text/html',
        css: 'text/css',
        js: 'application/javascript',
        woff2: 'font/woff2',
        png: 'image/png',
        svg: 'image/svg+xml',
      }
      res.writeHead(200, { 'Content-Type': types[ext || ''] || 'application/octet-stream' })
      createReadStream(file).pipe(res)
    })
    srv.listen(port, '127.0.0.1', () => resolveP(srv))
    srv.on('error', reject)
  })
  return {
    url: `http://127.0.0.1:${port}/`,
    close: () => new Promise((r) => server.close(() => r())),
  }
}

async function measureBrowser(url: string, name: BrowserName): Promise<Measured> {
  let pw
  try {
    pw = await loadPlaywright()
  } catch (e) {
    return { skipped: true, browser: name, reason: String((e as Error).message) }
  }
  const engine = pw[name]
  const launchOpts: { headless: boolean; executablePath?: string } = { headless: true }
  if (name === 'chromium' && process.env.CHROME_PATH && existsSync(process.env.CHROME_PATH)) {
    launchOpts.executablePath = process.env.CHROME_PATH
  }
  let browser
  try {
    browser = await engine.launch(launchOpts)
  } catch (e) {
    return {
      skipped: true,
      browser: name,
      reason: `${name} launch failed: ${String((e as Error).message).slice(0, 200)}`,
    }
  }
  try {
    const page = await browser.newPage({
      viewport: { width: VIEWPORT.width, height: VIEWPORT.height },
    })
    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 45000 })
    try {
      await page.waitForFunction(
        () => {
          const r = document.querySelector('#root')
          return r && (r.childElementCount > 0 || (r.textContent || '').trim().length > 0)
        },
        { timeout: 8000 },
      )
    } catch {
      /* static may already be filled */
    }
    await page.waitForTimeout(300)

    const boxes = await page.evaluate(() => {
      const root = document.querySelector('#root') || document.body
      const out: Array<{
        tag: string
        class: string
        x: number
        y: number
        width: number
        height: number
        offset: { x: number; y: number; w: number; h: number }
      }> = []
      const walk = (el: Element) => {
        const tag = el.tagName.toLowerCase()
        if (tag === 'script' || tag === 'style' || tag === 'link' || tag === 'meta') return
        const cs = getComputedStyle(el)
        if (cs.display === 'contents') {
          for (const c of el.children) walk(c)
          return
        }
        const r = el.getBoundingClientRect()
        const html = el as HTMLElement
        out.push({
          tag,
          class: el.className?.toString?.() || '',
          x: r.x,
          y: r.y,
          width: r.width,
          height: r.height,
          offset: { x: html.offsetLeft, y: html.offsetTop, w: html.offsetWidth, h: html.offsetHeight },
        })
        for (const c of el.children) walk(c)
      }
      if (root && root.id === 'root') {
        for (const c of root.children) walk(c)
      } else if (root) {
        walk(root)
      }
      return out
    })
    return { skipped: false, browser: name, boxes }
  } finally {
    await browser.close()
  }
}

function classKey(cls) {
  return (cls || '')
    .split(/\s+/)
    .filter(
      (c) =>
        c.startsWith('sail-') ||
        c.startsWith('s-w') ||
        c.startsWith('s-grow') ||
        c.startsWith('s-shrink') ||
        c.startsWith('s-min') ||
        c.includes('card') ||
        c.includes('pad'),
    )
    .slice(0, 8)
    .join(' ')
}

/** Stable match key: tag + filtered layout class tokens. */
function matchKey(box) {
  const tag = (box.tag || '').toLowerCase()
  const ck = classKey(box.class)
  return ck ? `${tag}|${ck}` : tag
}

/**
 * Pair boxes by (tag + classKey) with document-order within each key, then
 * fall back to remaining sequential slots. Structure fails when paired tags
 * diverge or counts differ.
 * Size (w/h) is the geometry gate; position is advisory (taffy parent-rel vs
 * browser absolute until absoluteize lands in LayoutCheck export).
 */
function diffBoxes(taffyBoxes, browserBoxes) {
  const tLayout = taffyBoxes.map((b, i) => ({ ...b, _i: i }))
  const bLayout = browserBoxes.map((b, i) => ({ ...b, _i: i }))

  // Bucket browser by key, preserve order.
  const bBuckets = new Map()
  for (const b of bLayout) {
    const k = matchKey(b)
    if (!bBuckets.has(k)) bBuckets.set(k, [])
    bBuckets.get(k).push(b)
  }
  const bUsed = new Set()
  const pairs = []
  for (const t of tLayout) {
    const k = matchKey(t)
    const bucket = bBuckets.get(k) || []
    let b = bucket.find((x) => !bUsed.has(x._i))
    if (!b) {
      // Soft fallback: same tag, unused.
      b = bLayout.find((x) => !bUsed.has(x._i) && (x.tag || '').toLowerCase() === (t.tag || '').toLowerCase())
    }
    if (b) {
      bUsed.add(b._i)
      pairs.push({ t, b, key: k })
    } else {
      pairs.push({ t, b: null, key: k })
    }
  }

  const diffs = []
  let matched = 0
  let sizeFail = 0
  let posFail = 0
  let errorN = 0
  let warnN = 0
  let tagDiverge = 0
  let compared = 0
  for (let i = 0; i < pairs.length; i++) {
    const { t, b } = pairs[i]
    if (!b) {
      tagDiverge++
      diffs.push({
        i,
        severity: 'error',
        reason: 'unmatched-taffy',
        taffy: { tag: t.tag, class: classKey(t.class), x: t.x, y: t.y, w: t.width, h: t.height },
        browser: null,
      })
      continue
    }
    compared++
    const tagsEqual = (t.tag || '').toLowerCase() === (b.tag || '').toLowerCase()
    if (!tagsEqual) {
      tagDiverge++
      diffs.push({
        i,
        severity: 'error',
        reason: 'tag-diverge',
        taffy: { tag: t.tag, class: classKey(t.class), w: t.width, h: t.height },
        browser: { tag: b.tag, class: classKey(b.class), w: b.width, h: b.height },
      })
      continue
    }
    const dw = Math.abs(t.width - b.width)
    const dh = Math.abs(t.height - b.height)
    const dx = Math.abs(t.x - b.x)
    const dy = Math.abs(t.y - b.y)
    const sizeOk = dw <= TOL && dh <= TOL
    const posOk = dx <= Math.max(TOL * 4, 16) && dy <= Math.max(TOL * 4, 16)
    if (sizeOk) matched++
    else sizeFail++
    if (!posOk) posFail++
    let severity = 'info'
    if (!sizeOk) {
      const maxD = Math.max(dw, dh)
      severity = maxD > 16 ? 'error' : 'warn'
      if (severity === 'error') errorN++
      else warnN++
    }
    if (!sizeOk || !posOk) {
      diffs.push({
        i,
        severity,
        taffy: {
          tag: t.tag,
          class: classKey(t.class),
          x: t.x,
          y: t.y,
          w: t.width,
          h: t.height,
        },
        browser: {
          tag: b.tag,
          class: classKey(b.class),
          x: b.x,
          y: b.y,
          w: b.width,
          h: b.height,
        },
        delta: { dw, dh, dx, dy },
        sizeOk,
        posOk,
      })
    }
  }
  const unusedBrowser = bLayout.filter((x) => !bUsed.has(x._i)).length
  const rootW =
    tLayout[0] && bLayout[0]
      ? Math.abs(tLayout[0].width - bLayout[0].width) <= TOL
      : false
  const structure_ok =
    tLayout.length === bLayout.length &&
    tagDiverge === 0 &&
    unusedBrowser === 0 &&
    rootW
  return {
    compared,
    taffy_count: tLayout.length,
    browser_count: bLayout.length,
    size_match: matched,
    size_fail: sizeFail,
    pos_fail: posFail,
    error_count: errorN,
    warn_count: warnN,
    tag_diverge: tagDiverge,
    unused_browser: unusedBrowser,
    structure_ok,
    root_width_ok: rootW,
    diffs: diffs.slice(0, 40),
  }
}

function tableRow(cols, widths) {
  return cols.map((c, i) => String(c).padEnd(widths[i])).join('  ')
}

async function main() {
  mkdirSync(outDir, { recursive: true })
  const fixtures = listFixtures()
  if (!fixtures.length) die('no fixtures')

  console.log(`PARITY-1 geometry — viewport ${VIEWPORT.width}x${VIEWPORT.height} tol=${TOL}px`)
  console.log(`sail=${SAIL_BIN}`)
  console.log(
    skipBrowser
      ? 'browsers= (taffy-only)'
      : `browsers=${browsers.join(',')}`,
  )
  console.log(`fixtures=${fixtures.length}${skipBrowser ? ' (taffy-only)' : ''}`)
  console.log('')

  const results = []
  for (const name of fixtures) {
    const path = join(fixturesDir, name)
    const source = readFileSync(path, 'utf8')
    const row = {
      fixture: name,
      taffy_ok: false,
      browser_ok: false,
      browser_mode: '',
      size_match: 0,
      size_fail: 0,
      compared: 0,
      pass: false,
      note: '',
    }
    try {
      const taffy = layoutCheck(source)
      row.taffy_ok = true
      row.taffy_boxes = taffy.boxes.length
      row.taffy_viewport = taffy.viewport

      if (skipBrowser) {
        row.note = 'taffy-only'
        row.taffy_pass = taffy.boxes.length > 0
        row.pass = row.taffy_pass
        row.structure_ok = null // not measured
        row.compared = 0
        writeFileSync(
          join(outDir, `${basename(name, '.sail')}.taffy.json`),
          JSON.stringify(taffy, null, 2),
        )
      } else {
        const stage = join(tmpdir(), `sail-parity-geom-${basename(name, '.sail')}`)
        const built = stageAndBuild(path, stage)
        row.browser_mode = built.mode
        if (strictSpa && built.mode !== 'sail-build') {
          row.note = 'strict-spa: lowered-html fallback'
          row.pass = false
          row.browser_ok = false
          row.structure_ok = false
        } else {
          const httpd = await serveStatic(built.serveDir)
          try {
            const perBrowser: Array<Record<string, unknown>> = []
            let allStruct = true
            let anyOk = false
            let notes: string[] = []
            let cmp = 0
            let szOk = 0
            let szFail = 0
            let errN = 0
            for (const bName of browsers) {
              const measured = await measureBrowser(httpd.url, bName)
              if (measured.skipped) {
                allStruct = false
                notes.push(`${bName}: ${measured.reason}`)
                perBrowser.push({ browser: bName, skipped: true, reason: measured.reason })
                continue
              }
              anyOk = true
              const diff = diffBoxes(taffy.boxes, measured.boxes)
              cmp += diff.compared
              szOk += diff.size_match
              szFail += diff.size_fail
              errN += diff.error_count
              const ok = diff.compared > 0 && diff.structure_ok
              if (!ok) allStruct = false
              if (!ok && diff.compared === 0) notes.push(`${bName}: no comparable hosts`)
              else if (diff.tag_diverge > 0) notes.push(`${bName}: tag diverge ${diff.tag_diverge}`)
              else if (diff.size_fail > 0) notes.push(`${bName}: geom Δ ${diff.error_count}e/${diff.warn_count}w`)
              perBrowser.push({
                browser: bName,
                boxes: measured.boxes.length,
                structure_ok: diff.structure_ok,
                size_fail: diff.size_fail,
                compared: diff.compared,
                tag_diverge: diff.tag_diverge,
              })
              writeFileSync(
                join(outDir, `${basename(name, '.sail')}.${bName}.report.json`),
                JSON.stringify(
                  {
                    fixture: name,
                    browser: bName,
                    taffy,
                    measured: { boxes: measured.boxes, mode: built.mode },
                    diff,
                  },
                  null,
                  2,
                ),
              )
            }
            row.browser_ok = anyOk
            row.browser_boxes = perBrowser
            row.compared = cmp
            row.size_match = szOk
            row.size_fail = szFail
            row.error_count = errN
            row.structure_ok = allStruct && anyOk
            row.pass = row.structure_ok
            row.strict_pass = row.pass && szFail === 0
            row.note = notes.join('; ')
            if (built.spa_fallback) {
              row.note = (row.note ? row.note + '; ' : '') + 'spa→lowered-html'
            }
          } finally {
            await httpd.close()
          }
        }
      }
    } catch (e) {
      row.note = String(e.message || e).slice(0, 240)
      row.pass = false
    }
    results.push(row)
  }

  // Scannable table
  const widths = [28, 6, 12, 5, 5, 6, 5, 6, 40]
  console.log(
    tableRow(
      ['fixture', 'taffy', 'browser', 'cmp', 'szOK', 'szFail', 'err', 'struct', 'note'],
      widths,
    ),
  )
  console.log(tableRow(widths.map((w) => '-'.repeat(w)), widths))
  for (const r of results) {
    const structLabel = skipBrowser
      ? 'n/a'
      : r.structure_ok === true
        ? 'yes'
        : r.structure_ok === false
          ? 'no'
          : 'n/a'
    console.log(
      tableRow(
        [
          r.fixture,
          r.taffy_ok ? String(r.taffy_boxes ?? 'ok') : 'FAIL',
          r.browser_ok ? r.browser_mode || 'ok' : skipBrowser ? 'skip' : 'FAIL',
          String(r.compared ?? 0),
          String(r.size_match ?? 0),
          String(r.size_fail ?? 0),
          String(r.error_count ?? 0),
          structLabel,
          (r.note || '').slice(0, 40),
        ],
        widths,
      ),
    )
  }

  const browserResults = results.filter((r) => !skipBrowser)
  const summary = {
    viewport: VIEWPORT,
    tol: TOL,
    browsers,
    skipBrowser,
    strict,
    strictSpa,
    reportOnly,
    results: results.map(({ taffy_viewport, ...r }) => r),
    taffy_passed: results.filter((r) => r.taffy_ok).length,
    structure_passed: browserResults.filter((r) => r.structure_ok === true).length,
    structure_total: browserResults.length,
    strict_passed: browserResults.filter((r) => r.strict_pass).length,
    total: results.length,
    report_dir: outDir,
  }
  writeFileSync(join(outDir, 'summary.json'), JSON.stringify(summary, null, 2))
  console.log('')
  if (skipBrowser) {
    console.log(
      `summary: taffy ${summary.taffy_passed}/${summary.total} → ${join(outDir, 'summary.json')}`,
    )
  } else {
    console.log(
      `summary: structure ${summary.structure_passed}/${summary.structure_total}; strict geom ${summary.strict_passed}/${summary.structure_total} → ${join(outDir, 'summary.json')}`,
    )
  }
  console.log(
    'paint: see tests/parity/paint/run.ts (takumi regions[].style ↔ browser computedStyle)',
  )
  const modeBits = []
  if (reportOnly) modeBits.push('report-only (structure non-fatal)')
  else modeBits.push('structure-gated')
  if (strict) modeBits.push('strict-size')
  if (strictSpa) modeBits.push('strict-spa')
  console.log(`mode: ${modeBits.join(', ')}`)
  console.log(
    'exit: 0=harness+structure_ok (default); --report-only=harness only; --strict=+size; 1=fail',
  )

  if (skipBrowser) {
    process.exit(results.every((r) => r.taffy_ok && r.taffy_pass !== false) ? 0 : 1)
  }
  const harnessOk = results.every((r) => r.taffy_ok && r.browser_ok)
  if (!harnessOk) process.exit(1)
  if (strict) {
    process.exit(results.every((r) => r.strict_pass) ? 0 : 1)
  }
  if (reportOnly) {
    // Explicit: structure/size only reported, not fatal.
    process.exit(0)
  }
  // Default: structure_ok required; size Δ non-fatal until --strict.
  process.exit(results.every((r) => r.pass) ? 0 : 1)
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
