#!/usr/bin/env node
/**
 * PARITY-1 paint suite — takumi ↔ browser computed style (RENDER-1 skins).
 *
 * For each fixture under ./fixtures:
 *   1. Takumi paint via `sail op SailRender/Screenshot` (regions[].style).
 *   2. Browser computedStyle via lowered-HTML (or sail-build SPA) + Playwright.
 *   3. Diff per load-bearing L1 host: bg/fg/border color, radius, border width.
 *
 * Not pixel parity — colors within channel tolerance; radii/widths within px tol.
 *
 * Geometry (taffy↔browser boxes) lives in tests/parity/geometry/run.mjs.
 *
 * Controller-driven content (suite honesty):
 *   Fixtures MUST use static template text only. Signal-driven / deferred
 *   controller content is covered by sail-render SSR-RENDER tests; this suite
 *   stays on static L1 fixtures.
 *
 * Invoke:
 *   node tests/parity/paint/run.mjs
 *   node tests/parity/paint/run.mjs --fixture l1-chrome-row
 *   node tests/parity/paint/run.mjs --strict          # style_fail === 0
 *   node tests/parity/paint/run.mjs --report-only     # harness ok only
 *   SAIL_BIN=target/release/sail node tests/parity/paint/run.mjs
 *
 * Exit codes (default / non-report-only):
 *   0 — harness ok AND structure_ok for every fixture (style Δ non-fatal)
 *   1 — screenshot/browser failure, structure fail, or (with --strict) style Δ
 * --report-only: exit 0 whenever takumi+browser ran; style only in summary.
 * --strict-spa: refuse lowered-html fallback (SPA path still optional for CI).
 *
 * structure_ok = L1 tag pairing + non-empty region styles + props_compared ≥ min.
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
import { resolveChrome } from '../chrome.mjs'

const __dirname = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(__dirname, '../../..')
const fixturesDir = join(__dirname, 'fixtures')
const outDir = join(__dirname, 'out')

const VIEWPORT = { width: 640, height: 900 }
const COLOR_TOL = Number(process.env.PARITY_PAINT_COLOR_TOL || 28) // per-channel 0–255
const PX_TOL = Number(process.env.PARITY_PAINT_PX_TOL || 2.0)
/** Min style props that must actually compare for structure_ok (not tag-only). */
const MIN_PROPS =
  Number(process.env.PARITY_PAINT_MIN_PROPS || 1) >= 0
    ? Number(process.env.PARITY_PAINT_MIN_PROPS || 1)
    : 1
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
const chromeResolved = resolveChrome()
const CHROME = chromeResolved.path

/** Load-bearing L1 hosts RENDER-1 covered — paint compare targets. */
const L1_HOSTS = new Set([
  'button',
  'card',
  'field',
  'input',
  'switch',
  'badge',
  'navbar',
  'sidebar',
  'dialog',
  'table',
])

const args = process.argv.slice(2)
const strict = args.includes('--strict') || process.env.PARITY_PAINT_STRICT === '1'
const reportOnly = args.includes('--report-only')
const strictSpa =
  args.includes('--strict-spa') || process.env.PARITY_PAINT_STRICT_SPA === '1'
const fixtureFilter = (() => {
  const i = args.indexOf('--fixture')
  return i >= 0 ? args[i + 1] : null
})()
const skipBrowser = args.includes('--takumi-only') || process.env.PARITY_TAKUMI_ONLY === '1'

function styleNonEmpty(s) {
  if (!s || typeof s !== 'object') return false
  return Object.values(s).some((v) => v != null && v !== '')
}

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

/** Workspace-relative path for SailRender/Screenshot `path` confinement. */
function workspaceRel(absPath) {
  const rel = absPath.startsWith(repoRoot)
    ? absPath.slice(repoRoot.length).replace(/^\//, '')
    : absPath
  return rel
}

function takumiScreenshot(fixtureAbs) {
  if (!existsSync(SAIL_BIN)) die(`missing sail binary at ${SAIL_BIN}`)
  const rel = workspaceRel(fixtureAbs)
  const payload = JSON.stringify({
    path: rel,
    viewport: { width: VIEWPORT.width, height: VIEWPORT.height },
    theme: 'ship',
    mode: 'light',
  })
  const r = spawnSync(SAIL_BIN, ['op', 'SailRender/Screenshot', '--json', payload], {
    cwd: repoRoot,
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  })
  if (r.status !== 0) {
    throw new Error(`SailRender/Screenshot failed: ${r.stderr || r.stdout}`)
  }
  const out = JSON.parse(r.stdout)
  return {
    theme: out.theme,
    viewport: out.viewport,
    ms: out.ms,
    regions: out.regions || [],
    dynamics: out.dynamics || [],
    // Drop huge PNG from fixture reports unless debugging.
    hasPng: !!(out.imagePng && out.imagePng.length > 32),
  }
}

/**
 * Prefer sail-build SPA; fall back to lowered __tpl HTML + token CSS so L1
 * skins (element-skins / framework CSS) apply in the browser.
 */
function stageAndBuild(fixturePath, stageRoot) {
  rmSync(stageRoot, { recursive: true, force: true })
  mkdirSync(join(stageRoot, 'routes'), { recursive: true })
  mkdirSync(join(stageRoot, 'public'), { recursive: true })
  cpSync(fixturePath, join(stageRoot, 'routes', 'page.sail'))
  writeFileSync(
    join(stageRoot, 'package.json'),
    JSON.stringify({ name: 'parity-paint-fixture', private: true, type: 'module' }, null, 2),
  )
  writeFileSync(join(stageRoot, 'public', '.gitkeep'), '')

  const out = join(stageRoot, '.deck')
  const build = spawnSync(
    SAIL_BIN,
    ['build', '--project', stageRoot, '--out', out, '--bundle-framework', '--base', 'assets'],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  )
  const log = (build.stdout || '') + (build.stderr || '')
  if (build.status === 0 && existsSync(join(out, 'index.html'))) {
    return { mode: 'sail-build', serveDir: out, log: log.slice(-1500) }
  }

  const compileOut = join(stageRoot, 'compile-out')
  mkdirSync(compileOut, { recursive: true })
  const comp = spawnSync(
    SAIL_COMPILER,
    ['build', join(stageRoot, 'routes', 'page.sail'), '--out', compileOut, '--clean'],
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
  console.warn(
    `warning: ${basename(fixturePath)}: sail build SPA unavailable; falling back to lowered-html`,
  )
  return { mode: 'lowered-html', serveDir: serve, log: clog.slice(-1500), spa_fallback: true }
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
    cpSync(src, join(serveDir, name))
    links.push(name)
  }
  // Framework L1 skins may also live under packages/components or compiler stdlib
  // emit; element-skins.css is the browser path. If missing, copy any skins.css.
  const skinsExtra = join(repoRoot, 'packages/sail/tokens/src/element-skins.css')
  if (!existsSync(skinsExtra)) {
    // no-op; tokens package layout may vary
  }
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
<html lang="en" theme="ship" data-mode="light">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>parity-paint</title>
${links.map((c) => `<link rel="stylesheet" href="${c}"/>`).join('\n')}
<style>
  html, body { margin: 0; padding: 0; width: 100%; min-height: 100%; }
  #root { width: 100%; box-sizing: border-box; }
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

async function measureBrowserStyles(serveDir) {
  if (!CHROME || !existsSync(CHROME)) {
    return {
      skipped: true,
      reason: `chrome missing (source=${chromeResolved.source}): ${CHROME || 'null'}`,
    }
  }
  let chromium
  try {
    ;({ chromium } = await import(
      join(repoRoot, 'node_modules/playwright/index.mjs').replace('index.mjs', 'index.js')
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
    const browser = await chromium.launch({ headless: true, executablePath: CHROME })
    try {
      const page = await browser.newPage({
        viewport: { width: VIEWPORT.width, height: VIEWPORT.height },
      })
      await page.goto(`http://127.0.0.1:${port}/`, {
        waitUntil: 'domcontentloaded',
        timeout: 45000,
      })
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
      await page.waitForTimeout(200)

      const hosts = [...L1_HOSTS]
      const styles = await page.evaluate((hostTags) => {
        const root = document.querySelector('#root') || document.body
        const set = new Set(hostTags)
        const out = []
        const walk = (el) => {
          if (el.nodeType !== 1) return
          const rawTag = el.tagName.toLowerCase()
          if (rawTag === 'script' || rawTag === 'style' || rawTag === 'link' || rawTag === 'meta')
            return
          const cs = getComputedStyle(el)
          if (cs.display === 'contents') {
            for (const c of el.children) walk(c)
            return
          }
          // Author L1 host: prefer data-sail-host (card/badge/field lower to div/span).
          const host =
            (el.getAttribute && el.getAttribute('data-sail-host')) ||
            (set.has(rawTag) ? rawTag : null)
          if (host && set.has(host)) {
            // Prefer first non-zero border side for width/color (uniform skins).
            const bw = [
              cs.borderTopWidth,
              cs.borderRightWidth,
              cs.borderBottomWidth,
              cs.borderLeftWidth,
            ]
            const bc = [
              cs.borderTopColor,
              cs.borderRightColor,
              cs.borderBottomColor,
              cs.borderLeftColor,
            ]
            let borderWidth = bw[0]
            let borderColor = bc[0]
            for (let i = 0; i < 4; i++) {
              if (parseFloat(bw[i]) > 0) {
                borderWidth = bw[i]
                borderColor = bc[i]
                break
              }
            }
            out.push({
              tag: host,
              htmlTag: rawTag,
              class: el.className?.toString?.() || '',
              backgroundColor: cs.backgroundColor,
              color: cs.color,
              borderColor,
              borderWidth,
              borderRadius: cs.borderTopLeftRadius,
              opacity: cs.opacity,
            })
          }
          for (const c of el.children) walk(c)
        }
        if (root.id === 'root') {
          for (const c of root.children) walk(c)
        } else {
          walk(root)
        }
        return out
      }, hosts)

      return { styles, skipped: false }
    } finally {
      await browser.close()
    }
  } finally {
    await new Promise((r) => server.close(() => r()))
  }
}

/** Parse `rgb()`, `rgba()`, or `rgba(r, g, b, a)` with a 0–255. */
function parseColor(s) {
  if (s == null || s === '' || s === 'transparent') return null
  const str = String(s).trim().toLowerCase()
  if (str === 'transparent') return null
  let m = str.match(
    /^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*([\d.]+))?\s*\)$/,
  )
  if (m) {
    const r = Math.round(Number(m[1]))
    const g = Math.round(Number(m[2]))
    const b = Math.round(Number(m[3]))
    let a = 255
    if (m[4] != null) {
      const av = Number(m[4])
      a = av <= 1 ? Math.round(av * 255) : Math.round(av)
    }
    return { r, g, b, a }
  }
  // modern `rgb(r g b / a)`
  m = str.match(
    /^rgba?\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)(?:\s*\/\s*([\d.]+%?))?\s*\)$/,
  )
  if (m) {
    const r = Math.round(Number(m[1]))
    const g = Math.round(Number(m[2]))
    const b = Math.round(Number(m[3]))
    let a = 255
    if (m[4] != null) {
      let av = m[4]
      if (String(av).endsWith('%')) av = Number(av) / 100
      else av = Number(av)
      a = av <= 1 ? Math.round(av * 255) : Math.round(av)
    }
    return { r, g, b, a }
  }
  if (str.startsWith('#')) {
    const h = str.slice(1)
    if (h.length === 3) {
      return {
        r: parseInt(h[0] + h[0], 16),
        g: parseInt(h[1] + h[1], 16),
        b: parseInt(h[2] + h[2], 16),
        a: 255,
      }
    }
    if (h.length === 6 || h.length === 8) {
      return {
        r: parseInt(h.slice(0, 2), 16),
        g: parseInt(h.slice(2, 4), 16),
        b: parseInt(h.slice(4, 6), 16),
        a: h.length === 8 ? parseInt(h.slice(6, 8), 16) : 255,
      }
    }
  }
  return null
}

function colorDelta(a, b) {
  if (!a || !b) return null
  return Math.max(
    Math.abs(a.r - b.r),
    Math.abs(a.g - b.g),
    Math.abs(a.b - b.b),
    // Alpha only counts when both sides are meaningfully opaque-ish or both semi.
    Math.abs(a.a - b.a) > 8 ? Math.abs(a.a - b.a) : 0,
  )
}

function parsePx(s) {
  if (s == null || s === '') return null
  if (typeof s === 'number' && Number.isFinite(s)) return s
  const m = String(s).trim().match(/^([\d.]+)px/)
  if (m) return Number(m[1])
  const n = Number(s)
  return Number.isFinite(n) ? n : null
}

/**
 * Pair L1 hosts by tag in document order.
 *
 * Structure (default gate) requires more than tag pairing:
 *   - every takumi L1 host matched in browser
 *   - at least one host carries non-empty takumi `style` (region styles present)
 *   - props_compared >= MIN_PROPS (actual style slots compared, not empty pass)
 *
 * Style Δ (color/radius/width/opacity) is separate: non-fatal unless --strict.
 */
function diffPaint(takumiRegions, browserStyles) {
  const tHosts = (takumiRegions || [])
    .filter((r) => r.tag && L1_HOSTS.has(r.tag))
    .map((r, i) => ({ ...r, _i: i }))
  const bHosts = (browserStyles || []).map((b, i) => ({ ...b, _i: i }))

  const bBuckets = new Map()
  for (const b of bHosts) {
    if (!bBuckets.has(b.tag)) bBuckets.set(b.tag, [])
    bBuckets.get(b.tag).push(b)
  }
  const bUsed = new Set()
  const pairs = []
  for (const t of tHosts) {
    const bucket = bBuckets.get(t.tag) || []
    const b = bucket.find((x) => !bUsed.has(x._i))
    if (b) {
      bUsed.add(b._i)
      pairs.push({ t, b })
    } else {
      pairs.push({ t, b: null })
    }
  }

  const diffs = []
  let compared = 0
  let styleMatch = 0
  let styleFail = 0
  let errorN = 0
  let warnN = 0
  let tagDiverge = 0
  let propsCompared = 0
  let propsFail = 0
  let hostsWithStyle = 0
  let hostsStyleEmpty = 0

  for (let i = 0; i < pairs.length; i++) {
    const { t, b } = pairs[i]
    if (!b) {
      tagDiverge++
      diffs.push({
        i,
        severity: 'error',
        reason: 'unmatched-takumi',
        tag: t.tag,
        takumi: t.style || null,
        browser: null,
      })
      continue
    }
    compared++
    const ts = t.style || {}
    if (styleNonEmpty(ts)) hostsWithStyle++
    else hostsStyleEmpty++
    const propDiffs = []

    // Colors
    for (const key of ['backgroundColor', 'color', 'borderColor']) {
      const tc = parseColor(ts[key])
      const bc = parseColor(b[key])
      if (!tc || !bc) continue
      // Skip near-transparent takumi fills (no real chrome).
      if (tc.a < 8) continue
      propsCompared++
      const d = colorDelta(tc, bc)
      if (d == null) continue
      if (d > COLOR_TOL) {
        propsFail++
        propDiffs.push({ prop: key, delta: d, takumi: ts[key], browser: b[key] })
      }
    }

    // Radius
    {
      const tr = ts.borderRadius
      const br = parsePx(b.borderRadius)
      if (tr != null && br != null) {
        propsCompared++
        const d = Math.abs(tr - br)
        if (d > PX_TOL) {
          propsFail++
          propDiffs.push({ prop: 'borderRadius', delta: d, takumi: tr, browser: br })
        }
      }
    }

    // Border width
    {
      const tw = ts.borderWidth
      const bw = parsePx(b.borderWidth)
      if (tw != null && bw != null) {
        propsCompared++
        const d = Math.abs(tw - bw)
        if (d > PX_TOL) {
          propsFail++
          propDiffs.push({ prop: 'borderWidth', delta: d, takumi: tw, browser: bw })
        }
      }
    }

    // Opacity (RegionStyle surface; compare when takumi exports it)
    {
      const to = ts.opacity
      const bo = parsePx(b.opacity) ?? (b.opacity != null ? Number(b.opacity) : null)
      if (to != null && bo != null && Number.isFinite(bo)) {
        propsCompared++
        // browser opacity is 0–1; takumi is 0–1 float
        const d = Math.abs(Number(to) - bo)
        if (d > 0.05) {
          propsFail++
          propDiffs.push({ prop: 'opacity', delta: d, takumi: to, browser: bo })
        }
      }
    }

    const ok = propDiffs.length === 0
    if (ok) styleMatch++
    else {
      styleFail++
      const maxD = Math.max(...propDiffs.map((p) => p.delta || 0))
      const severity =
        propDiffs.some((p) => p.prop === 'backgroundColor' && p.delta > COLOR_TOL * 2) ||
        maxD > COLOR_TOL * 2
          ? 'error'
          : 'warn'
      if (severity === 'error') errorN++
      else warnN++
      diffs.push({
        i,
        severity,
        tag: t.tag,
        props: propDiffs,
        takumi: ts,
        browser: {
          backgroundColor: b.backgroundColor,
          color: b.color,
          borderColor: b.borderColor,
          borderWidth: b.borderWidth,
          borderRadius: b.borderRadius,
          opacity: b.opacity,
        },
      })
    }
  }

  const unusedBrowser = bHosts.filter((x) => !bUsed.has(x._i)).length
  // Structure floor: tag pairing + styles present + at least MIN_PROPS compared.
  // Prevents "empty style drop still passes" regressions (O2).
  const pairing_ok = tagDiverge === 0 && tHosts.length > 0 && compared === tHosts.length
  const style_presence_ok = hostsWithStyle > 0
  const props_floor_ok = propsCompared >= MIN_PROPS
  const structure_ok = pairing_ok && style_presence_ok && props_floor_ok

  return {
    compared,
    takumi_l1: tHosts.length,
    browser_l1: bHosts.length,
    style_match: styleMatch,
    style_fail: styleFail,
    props_compared: propsCompared,
    props_fail: propsFail,
    hosts_with_style: hostsWithStyle,
    hosts_style_empty: hostsStyleEmpty,
    min_props: MIN_PROPS,
    pairing_ok,
    style_presence_ok,
    props_floor_ok,
    error_count: errorN,
    warn_count: warnN,
    tag_diverge: tagDiverge,
    unused_browser: unusedBrowser,
    structure_ok,
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

  console.log(
    `PARITY-1 paint — viewport ${VIEWPORT.width}x${VIEWPORT.height} colorTol=${COLOR_TOL} pxTol=${PX_TOL} minProps=${MIN_PROPS}`,
  )
  console.log(`sail=${SAIL_BIN}`)
  console.log(
    `chrome=${CHROME || 'null'} (${chromeResolved.source})${skipBrowser ? ' (takumi-only)' : ''}`,
  )
  console.log(`fixtures=${fixtures.length}${skipBrowser ? ' (takumi-only)' : ''}`)
  console.log('')

  const results = []
  for (const name of fixtures) {
    const path = join(fixturesDir, name)
    const row = {
      fixture: name,
      takumi_ok: false,
      browser_ok: false,
      browser_mode: '',
      style_match: 0,
      style_fail: 0,
      compared: 0,
      pass: false,
      note: '',
    }
    try {
      const shot = takumiScreenshot(path)
      row.takumi_ok = true
      row.takumi_regions = shot.regions.length
      row.takumi_l1 = shot.regions.filter((r) => r.tag && L1_HOSTS.has(r.tag)).length
      row.takumi_ms = shot.ms
      row.has_png = shot.hasPng

      if (skipBrowser) {
        row.note = 'takumi-only'
        row.pass = row.takumi_l1 > 0
        row.structure_ok = null
      } else {
        const stage = join(tmpdir(), `sail-parity-paint-${basename(name, '.sail')}`)
        const built = stageAndBuild(path, stage)
        row.browser_mode = built.mode
        if (strictSpa && built.mode !== 'sail-build') {
          row.note = 'strict-spa: lowered-html fallback'
          row.pass = false
          row.browser_ok = false
          row.structure_ok = false
        } else {
        const measured = await measureBrowserStyles(built.serveDir)
        if (measured.skipped) {
          row.note = measured.reason
          row.pass = false
          row.browser_ok = false
        } else {
          row.browser_ok = true
          row.browser_l1 = measured.styles.length
          const diff = diffPaint(shot.regions, measured.styles)
          row.compared = diff.compared
          row.style_match = diff.style_match
          row.style_fail = diff.style_fail
          row.props_compared = diff.props_compared
          row.props_fail = diff.props_fail
          row.hosts_with_style = diff.hosts_with_style
          row.error_count = diff.error_count
          row.warn_count = diff.warn_count
          row.structure_ok = diff.structure_ok
          row.tag_diverge = diff.tag_diverge
          row.pass = diff.compared > 0 && diff.structure_ok
          // Strict also requires props floor (empty compare must not pass --strict).
          row.strict_pass =
            row.pass && diff.style_fail === 0 && diff.props_compared >= MIN_PROPS
          if (!row.pass && diff.compared === 0) {
            row.note = 'no comparable L1 hosts'
          } else if (diff.tag_diverge > 0) {
            row.note = `tag diverge ${diff.tag_diverge}`
          } else if (!diff.style_presence_ok) {
            row.note = 'no region styles on L1 hosts'
          } else if (!diff.props_floor_ok) {
            row.note = `props_compared ${diff.props_compared}<${MIN_PROPS}`
          } else if (diff.style_fail > 0) {
            row.note = `style Δ: ${diff.error_count}e/${diff.warn_count}w`
          }
          if (built.spa_fallback) {
            row.note = (row.note ? row.note + '; ' : '') + 'spa→lowered-html'
          }
          writeFileSync(
            join(outDir, `${basename(name, '.sail')}.report.json`),
            JSON.stringify(
              {
                fixture: name,
                viewport: VIEWPORT,
                colorTol: COLOR_TOL,
                pxTol: PX_TOL,
                takumi: {
                  theme: shot.theme,
                  ms: shot.ms,
                  regions: shot.regions,
                  dynamics: shot.dynamics,
                },
                browser: {
                  styles: measured.styles,
                  mode: built.mode,
                  spa_fallback: !!built.spa_fallback,
                },
                diff,
              },
              null,
              2,
            ),
          )
        }
        }
      }
    } catch (e) {
      row.note = String(e.message || e).slice(0, 240)
      row.pass = false
    }
    results.push(row)
  }

  const widths = [28, 8, 12, 5, 5, 6, 5, 6, 40]
  console.log(
    tableRow(
      ['fixture', 'takumi', 'browser', 'cmp', 'stOK', 'stFail', 'err', 'struct', 'note'],
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
          r.takumi_ok ? String(r.takumi_l1 ?? 'ok') : 'FAIL',
          r.browser_ok ? r.browser_mode || 'ok' : skipBrowser ? 'skip' : 'FAIL',
          String(r.compared ?? 0),
          String(r.style_match ?? 0),
          String(r.style_fail ?? 0),
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
    suite: 'paint',
    viewport: VIEWPORT,
    colorTol: COLOR_TOL,
    pxTol: PX_TOL,
    minProps: MIN_PROPS,
    chrome: { path: CHROME, source: chromeResolved.source },
    skipBrowser,
    strict,
    strictSpa,
    reportOnly,
    results: results.map((r) => r),
    takumi_passed: results.filter((r) => r.takumi_ok).length,
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
      `summary: takumi ${summary.takumi_passed}/${summary.total} → ${join(outDir, 'summary.json')}`,
    )
  } else {
    console.log(
      `summary: structure ${summary.structure_passed}/${summary.structure_total}; strict style ${summary.strict_passed}/${summary.structure_total} → ${join(outDir, 'summary.json')}`,
    )
  }
  const modeBits = []
  if (reportOnly) modeBits.push('report-only (structure non-fatal)')
  else modeBits.push('structure-gated (pair+stylePresence+propsFloor)')
  if (strict) modeBits.push('strict-style')
  if (strictSpa) modeBits.push('strict-spa')
  console.log(`mode: ${modeBits.join(', ')}`)
  console.log(
    'exit: 0=harness+structure_ok (default); --report-only=harness only; --strict=+style; 1=fail',
  )
  console.log(
    'note: geometry structure gate lives in tests/parity/geometry/run.mjs',
  )

  if (skipBrowser) {
    process.exit(results.every((r) => r.takumi_ok && r.pass) ? 0 : 1)
  }
  const harnessOk = results.every((r) => r.takumi_ok && r.browser_ok)
  if (!harnessOk) process.exit(1)
  if (strict) {
    process.exit(results.every((r) => r.strict_pass) ? 0 : 1)
  }
  if (reportOnly) {
    process.exit(0)
  }
  process.exit(results.every((r) => r.pass) ? 0 : 1)
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
