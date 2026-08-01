#!/usr/bin/env node
/**
 * PARITY-1 geometry suite — taffy ↔ browser box tree.
 *
 * For each fixture under ./fixtures:
 *   1. Taffy boxes via `sail op SailAuthoring/LayoutCheck` (preprocess + check_preprocessed).
 *   2. Browser boxes via sail build SPA (or lowered-HTML fallback) + Playwright CDP/DOM.
 *   3. Diff report (width/height + parent-relative x/y within tolerance).
 *
 * Paint (takumi ↔ browser style) is stubbed until RENDER-1 fully lands in CI.
 *
 * Controller-driven content contract (minimal, suite honesty):
 *   Static render / geometry fixtures MUST use static template text only.
 *   Signal-driven or deferred controller content is out of scope for this suite
 *   and will intentionally show empty/placeholder boxes — do not add such
 *   fixtures here until an SSR-defaults contract is defined product-wide.
 *
 * Invoke:
 *   node tests/parity/geometry/run.mjs
 *   node tests/parity/geometry/run.mjs --fixture column-stack
 *   SAIL_BIN=target/release/sail node tests/parity/geometry/run.mjs
 *
 * Exit 0 if all fixtures within tolerance (or --report-only); 1 on hard failure.
 */

import { spawnSync, spawn } from 'node:child_process'
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
const CHROME =
  process.env.CHROME_PATH ||
  join(
    process.env.HOME || '',
    '.cache/ms-playwright/chromium-1228/chrome-linux64/chrome',
  )

const args = process.argv.slice(2)
// Scaffold default: produce reports + exit 0 when the harness itself works.
// `--strict` fails the process on size divergences (CI gate once corpus is stable).
const strict = args.includes('--strict') || process.env.PARITY_GEOM_STRICT === '1'
const reportOnly = args.includes('--report-only') || !strict
const fixtureFilter = (() => {
  const i = args.indexOf('--fixture')
  return i >= 0 ? args[i + 1] : null
})()
const skipBrowser = args.includes('--taffy-only') || process.env.PARITY_TAFFY_ONLY === '1'

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
 */
function stageAndBuild(fixturePath, stageRoot) {
  if (stageRoot.exists?.()) {
    /* no-op */
  }
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
  // Minimal deck project marker if needed
  writeFileSync(join(stageRoot, 'public', '.gitkeep'), '')

  const out = join(stageRoot, '.deck')
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
  return { mode: 'lowered-html', serveDir: serve, log: clog.slice(-1500) }
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
<html lang="en" data-theme="ship" data-mode="light">
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

async function measureBrowser(serveDir) {
  if (!existsSync(CHROME)) {
    return { skipped: true, reason: `chrome missing: ${CHROME}` }
  }
  // dynamic import playwright from repo
  let chromium
  try {
    ;({ chromium } = await import(
      join(repoRoot, 'node_modules/playwright/index.mjs').replace(
        'index.mjs',
        'index.js',
      )
    ))
  } catch {
    try {
      ;({ chromium } = await import('playwright'))
    } catch (e) {
      return { skipped: true, reason: `playwright import failed: ${e.message}` }
    }
  }

  const port = await freePort()
  const server = await new Promise((resolveP, reject) => {
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
      const types = {
        html: 'text/html',
        css: 'text/css',
        js: 'application/javascript',
        woff2: 'font/woff2',
        png: 'image/png',
        svg: 'image/svg+xml',
      }
      res.writeHead(200, { 'Content-Type': types[ext] || 'application/octet-stream' })
      createReadStream(file).pipe(res)
    })
    srv.listen(port, '127.0.0.1', () => resolveP(srv))
    srv.on('error', reject)
  })

  try {
    const browser = await chromium.launch({
      headless: true,
      executablePath: CHROME,
    })
    try {
      const page = await browser.newPage({
        viewport: { width: VIEWPORT.width, height: VIEWPORT.height },
      })
      await page.goto(`http://127.0.0.1:${port}/`, {
        waitUntil: 'domcontentloaded',
        timeout: 45000,
      })
      // SPA: wait for mount if present
      try {
        await page.waitForFunction(
          () => {
            const r = document.querySelector('#root')
            return r && (r.childElementCount > 0 || r.textContent.trim().length > 0)
          },
          { timeout: 8000 },
        )
      } catch {
        /* static may already be filled */
      }
      await page.waitForTimeout(300)

      // Absolute viewport coords via getBoundingClientRect (CDP-equivalent box model).
      // Parent-relative offset* is recorded for debug but not used for size gates.
      const boxes = await page.evaluate(() => {
        const root = document.querySelector('#root') || document.body
        const out = []
        const walk = (el) => {
          if (el.nodeType !== 1) return
          const tag = el.tagName.toLowerCase()
          if (tag === 'script' || tag === 'style' || tag === 'link' || tag === 'meta') return
          const cs = getComputedStyle(el)
          if (cs.display === 'contents') {
            for (const c of el.children) walk(c)
            return
          }
          const r = el.getBoundingClientRect()
          out.push({
            tag,
            class: el.className?.toString?.() || '',
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
            offset: { x: el.offsetLeft, y: el.offsetTop, w: el.offsetWidth, h: el.offsetHeight },
          })
          for (const c of el.children) walk(c)
        }
        if (root.id === 'root') {
          for (const c of root.children) walk(c)
        } else {
          walk(root)
        }
        return out
      })

      // Optional CDP sample on first box for protocol proof
      let cdpSample = null
      try {
        const client = await page.context().newCDPSession(page)
        const { root } = await client.send('DOM.getDocument', { depth: 1 })
        cdpSample = { documentNodeId: root.nodeId, protocol: 'CDP DOM.getDocument ok' }
      } catch (e) {
        cdpSample = { error: String(e.message || e) }
      }

      return { boxes, cdpSample, skipped: false }
    } finally {
      await browser.close()
    }
  } finally {
    await new Promise((r) => server.close(() => r()))
  }
}

/** Promote taffy parent-relative locations to absolute (viewport) coords. */
function absoluteizeTaffy(boxes) {
  // Pre-order walk with parent stack using tag/class tree isn't available —
  // BoxReport is flat. Approximate: location is parent-relative only for
  // direct children in taffy; without the parent map we keep as-is for size
  // gates and only use width/height as the hard metric. Root hosts already
  // have absolute-ish coords (x,y near 0).
  return boxes.map((b) => ({ ...b }))
}

function classKey(cls) {
  return (cls || '')
    .split(/\s+/)
    .filter((c) => c.startsWith('sail-') || c.startsWith('sx-w') || c.startsWith('sx-grow') || c.startsWith('sx-shrink') || c.startsWith('sx-min') || c.includes('card') || c.includes('pad'))
    .slice(0, 8)
    .join(' ')
}

/**
 * Align boxes by sequential order (same lowered tree). Size (w/h) is the gate;
 * position is advisory (taffy location is parent-relative; browser is absolute).
 * Severity: error if |Δ| > 16px on layout hosts; warn if > tol; info otherwise.
 */
function diffBoxes(taffyBoxes, browserBoxes) {
  const tLayout = absoluteizeTaffy(taffyBoxes)
  const bLayout = browserBoxes
  const n = Math.min(tLayout.length, bLayout.length)
  const diffs = []
  let matched = 0
  let sizeFail = 0
  let posFail = 0
  let errorN = 0
  let warnN = 0
  for (let i = 0; i < n; i++) {
    const t = tLayout[i]
    const b = bLayout[i]
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
  // Structural: counts equal and root width within tol
  const rootW =
    tLayout[0] && bLayout[0]
      ? Math.abs(tLayout[0].width - bLayout[0].width) <= TOL
      : false
  return {
    compared: n,
    taffy_count: tLayout.length,
    browser_count: bLayout.length,
    size_match: matched,
    size_fail: sizeFail,
    pos_fail: posFail,
    error_count: errorN,
    warn_count: warnN,
    structure_ok: tLayout.length === bLayout.length && rootW,
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
        row.pass = taffy.boxes.length > 0
        row.compared = 0
        // write taffy dump
        writeFileSync(
          join(outDir, `${basename(name, '.sail')}.taffy.json`),
          JSON.stringify(taffy, null, 2),
        )
      } else {
        const stage = join(tmpdir(), `sail-parity-geom-${basename(name, '.sail')}`)
        const built = stageAndBuild(path, stage)
        row.browser_mode = built.mode
        const measured = await measureBrowser(built.serveDir)
        if (measured.skipped) {
          row.note = measured.reason
          row.pass = false
          row.browser_ok = false
        } else {
          row.browser_ok = true
          row.browser_boxes = measured.boxes.length
          const diff = diffBoxes(taffy.boxes, measured.boxes)
          row.compared = diff.compared
          row.size_match = diff.size_match
          row.size_fail = diff.size_fail
          row.pos_fail = diff.pos_fail
          row.error_count = diff.error_count
          row.warn_count = diff.warn_count
          row.structure_ok = diff.structure_ok
          // Harness pass: suite ran, structure lines up. Geometry deltas are
          // reported with severity (thesis: divergences = compiler bugs).
          // Strict mode additionally requires size_fail === 0.
          row.pass = diff.compared > 0 && diff.structure_ok
          row.strict_pass = row.pass && diff.size_fail === 0
          if (!row.pass && diff.compared === 0) {
            row.note = 'no comparable hosts'
          } else if (diff.size_fail > 0) {
            row.note = `geom Δ: ${diff.error_count}e/${diff.warn_count}w`
          }
          writeFileSync(
            join(outDir, `${basename(name, '.sail')}.report.json`),
            JSON.stringify(
              {
                fixture: name,
                taffy,
                browser: { boxes: measured.boxes, cdp: measured.cdpSample, mode: built.mode },
                diff,
              },
              null,
              2,
            ),
          )
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
          r.structure_ok ? 'yes' : r.taffy_ok && skipBrowser ? 'n/a' : 'no',
          (r.note || '').slice(0, 40),
        ],
        widths,
      ),
    )
  }

  const summary = {
    viewport: VIEWPORT,
    tol: TOL,
    skipBrowser,
    strict,
    results: results.map(({ taffy_viewport, ...r }) => r),
    structure_passed: results.filter((r) => r.pass).length,
    strict_passed: results.filter((r) => r.strict_pass).length,
    total: results.length,
    report_dir: outDir,
  }
  writeFileSync(join(outDir, 'summary.json'), JSON.stringify(summary, null, 2))
  console.log('')
  console.log(
    `summary: structure ${summary.structure_passed}/${summary.total}; strict geom ${summary.strict_passed}/${summary.total} → ${join(outDir, 'summary.json')}`,
  )
  console.log(
    'paint: stubbed (takumi↔browser style parity waits on RENDER-1 lab stamp consumption)',
  )
  console.log(`mode: ${strict ? 'strict (exit 1 on size Δ)' : 'report (exit 0 if harness ok)'}`)

  if (skipBrowser) {
    process.exit(results.every((r) => r.taffy_ok) ? 0 : 1)
  }
  const harnessOk = results.every((r) => r.taffy_ok && r.browser_ok)
  if (!harnessOk) process.exit(1)
  if (strict) {
    process.exit(results.every((r) => r.strict_pass) ? 0 : 1)
  }
  process.exit(0)
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
