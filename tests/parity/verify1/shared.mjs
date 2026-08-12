/**
 * Shared harness for VERIFY-1: LayoutCheck, compile, browser measure, diffs.
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
import { join, resolve, dirname, basename } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createServer } from 'node:http'
import { createReadStream } from 'node:fs'
import { tmpdir, cpus } from 'node:os'
import net from 'node:net'
import { createRequire } from 'node:module'
import { resolveChrome } from '../chrome.mjs'
import { VOCAB } from './vocab.mjs'

const __dirname = dirname(fileURLToPath(import.meta.url))
export const repoRoot = resolve(__dirname, '../../..')
export const outDir = join(__dirname, 'out')

export const VIEWPORT = { width: 1440, height: 900 }
export const PAINT_VIEWPORT = { width: 640, height: 900 }
export const TOL = Number(process.env.VERIFY1_GEOM_TOL || VOCAB.geometry_tolerance_px)

export const SAIL_BIN =
  process.env.SAIL_BIN ||
  (existsSync(join(repoRoot, 'target/release/sail'))
    ? join(repoRoot, 'target/release/sail')
    : join(repoRoot, 'target/debug/sail'))

const chromeResolved = resolveChrome()
export const CHROME = chromeResolved.path
export const CHROME_SOURCE = chromeResolved.source

const require = createRequire(import.meta.url)

export function ensureOut() {
  mkdirSync(outDir, { recursive: true })
  mkdirSync(join(outDir, 'corpus'), { recursive: true })
  mkdirSync(join(outDir, 'repros'), { recursive: true })
}

/** LayoutCheck for one source; returns desktop viewport boxes + diagnostics. */
export function layoutCheck(source) {
  const r = spawnSync(
    SAIL_BIN,
    ['op', 'SailAuthoring/LayoutCheck', '--json', JSON.stringify({ source })],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  )
  if (r.status !== 0) {
    return {
      ok: false,
      error: (r.stderr || r.stdout || 'LayoutCheck failed').slice(0, 500),
    }
  }
  let out
  try {
    out = JSON.parse(r.stdout)
  } catch (e) {
    return { ok: false, error: `json: ${e.message}` }
  }
  const vp =
    out.viewports?.find(
      (v) =>
        Math.abs(v.viewport?.width - VIEWPORT.width) < 1 &&
        Math.abs(v.viewport?.height - VIEWPORT.height) < 1,
    ) ||
    [...(out.viewports || [])].sort(
      (a, b) =>
        Math.abs((a.viewport?.width || 0) - VIEWPORT.width) -
        Math.abs((b.viewport?.width || 0) - VIEWPORT.width),
    )[0]
  return {
    ok: true,
    engine: out.engine,
    diagnostics: out.diagnostics || [],
    boxes: vp?.boxes || [],
    viewport: vp?.viewport || VIEWPORT,
    error: vp?.error || null,
  }
}

/**
 * Parallel LayoutCheck pool.
 * @param {{ id: string, source: string }[]} cases
 * @param {{ concurrency?: number, onProgress?: (n:number,total:number)=>void }} opts
 */
export async function layoutCheckAll(cases, opts = {}) {
  const concurrency = opts.concurrency || Math.min(8, Math.max(2, cpus().length))
  const results = new Array(cases.length)
  let next = 0
  let done = 0
  async function worker() {
    while (true) {
      const i = next++
      if (i >= cases.length) return
      results[i] = { id: cases[i].id, ...layoutCheck(cases[i].source) }
      done++
      if (opts.onProgress && done % 50 === 0) opts.onProgress(done, cases.length)
    }
  }
  await Promise.all(Array.from({ length: concurrency }, () => worker()))
  if (opts.onProgress) opts.onProgress(cases.length, cases.length)
  return results
}

/**
 * Batch-compile .sail files; return map id → { html, css }.
 * CSS comes from the compiler-injected `__suiStyles` block (s-* atomics are
 * NOT all present in static tokens CSS — only skins + sail-* primitives).
 * On batch failure, falls back to per-file compile.
 */
export function compileBatch(cases, stageRoot) {
  rmSync(stageRoot, { recursive: true, force: true })
  mkdirSync(stageRoot, { recursive: true })
  for (const c of cases) {
    writeFileSync(join(stageRoot, `${c.id}.sail`), c.source)
  }
  const compileOut = join(stageRoot, '_out')
  const r = spawnSync(
    SAIL_BIN,
    ['compile', stageRoot, '--out', compileOut, '--clean'],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 },
  )
  /** @type {Map<string, { html: string, css: string }>} */
  const htmlById = new Map()
  if (existsSync(compileOut)) {
    for (const f of readdirSync(compileOut).filter((x) => x.endsWith('.js'))) {
      const id = basename(f, '.js')
      const js = readFileSync(join(compileOut, f), 'utf8')
      htmlById.set(id, {
        html: extractTplHtml(js).join('') || '',
        css: extractSuiStyles(js),
      })
    }
  }
  if (htmlById.size < cases.length) {
    for (const c of cases) {
      if (htmlById.has(c.id)) continue
      const oneDir = join(stageRoot, `_one_${c.id}`)
      mkdirSync(oneDir, { recursive: true })
      const sailPath = join(oneDir, `${c.id}.sail`)
      writeFileSync(sailPath, c.source)
      const oneOut = join(oneDir, 'out')
      const r1 = spawnSync(
        SAIL_BIN,
        ['compile', sailPath, '--out', oneOut, '--clean'],
        { cwd: repoRoot, encoding: 'utf8', maxBuffer: 8 * 1024 * 1024 },
      )
      if (r1.status !== 0 || !existsSync(oneOut)) continue
      const jsFile = readdirSync(oneOut).find((x) => x.endsWith('.js'))
      if (!jsFile) continue
      const js = readFileSync(join(oneOut, jsFile), 'utf8')
      htmlById.set(c.id, {
        html: extractTplHtml(js).join('') || '',
        css: extractSuiStyles(js),
      })
    }
  }
  return {
    ok: htmlById.size > 0,
    error: r.status !== 0 && htmlById.size === 0 ? (r.stderr || '').slice(0, 400) : null,
    htmlById,
    compiled: htmlById.size,
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

/** Extract compiler-injected `@layer sail { … }` atomic CSS from compiled JS. */
function extractSuiStyles(jsText) {
  const m = jsText.match(/__suiStyles\s*=\s*"((?:[^"\\]|\\.)*)"/)
  if (!m) return ''
  return decodeJsString(m[1])
}

/** Normalize compile map entries that may be string (legacy) or {html,css}. */
export function entryHtml(entry) {
  if (!entry) return ''
  return typeof entry === 'string' ? entry : entry.html || ''
}
export function entryCss(entry) {
  if (!entry || typeof entry === 'string') return ''
  return entry.css || ''
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

/**
 * Write a static HTML shell that hosts many case fragments.
 * @param {string} serveDir
 * @param {{ id: string, html: string, css?: string }[]} casesHtml
 * @param {{ width?: number }} [opts]
 */
export function writeMultiCaseHtml(serveDir, casesHtml, opts = {}) {
  const width = opts.width || VIEWPORT.width
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
  const fontsSrc = join(tokens, 'fonts')
  if (existsSync(fontsSrc)) cpSync(fontsSrc, join(serveDir, 'fonts'), { recursive: true })

  // Dedup compiler-injected sx atomics across cases
  const cssChunks = []
  const seenCss = new Set()
  for (const c of casesHtml) {
    const css = (c.css || '').trim()
    if (!css || seenCss.has(css)) continue
    seenCss.add(css)
    cssChunks.push(css)
  }

  const body = casesHtml
    .map(
      ({ id, html }) =>
        `<section class="v1-case" data-case="${id}" style="position:relative;width:${width}px;box-sizing:border-box;margin:0;padding:0;border:0;">${html}</section>`,
    )
    .join('\n')

  writeFileSync(
    join(serveDir, 'index.html'),
    `<!doctype html>
<html lang="en" theme="ship" data-mode="light">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>verify1-corpus</title>
${links.map((c) => `<link rel="stylesheet" href="${c}"/>`).join('\n')}
<style>
  html, body { margin: 0; padding: 0; width: 100%; }
  /* Each case is an independent layout island at viewport width */
  .v1-case { width: ${width}px; min-height: 1px; box-sizing: border-box; }
  .tpl-frag { display: contents; }
</style>
<style id="v1-s-atomics">
/* Compiler-injected s-* / sail-* atomics from compiled modules */
${cssChunks.join('\n')}
</style>
</head>
<body>
${body}
</body>
</html>
`,
  )
}

async function freePort() {
  return new Promise((resolveP, reject) => {
    const s = net.createServer()
    s.listen(0, '127.0.0.1', () => {
      const { port } = s.address()
      s.close(() => resolveP(port))
    })
    s.on('error', reject)
  })
}

async function importPlaywright() {
  const local = join(repoRoot, 'node_modules/playwright')
  if (existsSync(local)) {
    try {
      return await import(join(local, 'index.js'))
    } catch {
      /* fall through to package specifier */
    }
  }
  return import('playwright')
}

/**
 * Launch browser once; measure many cases from a multi-case HTML page.
 * Returns Map<id, boxes[]>
 */
export async function measureBrowserCases(serveDir, caseIds, viewport = VIEWPORT) {
  if (!CHROME || !existsSync(CHROME)) {
    return { skipped: true, reason: `chrome missing (${CHROME_SOURCE}): ${CHROME}`, map: new Map() }
  }
  let chromium
  try {
    ;({ chromium } = await importPlaywright())
  } catch (e) {
    return { skipped: true, reason: `playwright: ${e.message}`, map: new Map() }
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

  const map = new Map()
  let lastErr = null
  try {
    let browser = null
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        browser = await chromium.launch({
          headless: true,
          executablePath: CHROME,
          args: ['--disable-dev-shm-usage', '--no-sandbox'],
        })
        break
      } catch (e) {
        lastErr = e
        await new Promise((r) => setTimeout(r, 500 * (attempt + 1)))
      }
    }
    if (!browser) {
      return {
        skipped: true,
        reason: `chrome launch failed: ${lastErr?.message || lastErr}`,
        map,
      }
    }
    try {
      const page = await browser.newPage({
        viewport: { width: viewport.width, height: viewport.height },
      })
      // Tall page: cases stack; viewport width matters for layout width.
      await page.setViewportSize({ width: viewport.width, height: Math.min(viewport.height, 900) })
      await page.goto(`http://127.0.0.1:${port}/`, {
        waitUntil: 'domcontentloaded',
        timeout: 60000,
      })
      // Wait fonts
      try {
        await page.evaluate(() => document.fonts?.ready)
      } catch {
        /* ok */
      }
      await page.waitForTimeout(200)

      const measured = await page.evaluate(() => {
        const out = {}
        for (const sec of document.querySelectorAll('.v1-case')) {
          const id = sec.getAttribute('data-case')
          const boxes = []
          const walk = (el) => {
            if (el.nodeType !== 1) return
            const tag = el.tagName.toLowerCase()
            if (tag === 'script' || tag === 'style' || tag === 'link' || tag === 'meta') return
            if (el.classList?.contains('v1-case')) {
              for (const c of el.children) walk(c)
              return
            }
            const cs = getComputedStyle(el)
            if (cs.display === 'contents') {
              for (const c of el.children) walk(c)
              return
            }
            const r = el.getBoundingClientRect()
            // Case-relative coords: subtract section origin
            boxes.push({
              tag,
              class: el.className?.toString?.() || '',
              x: r.x,
              y: r.y,
              width: r.width,
              height: r.height,
              clientWidth: el.clientWidth,
              clientHeight: el.clientHeight,
              scrollWidth: el.scrollWidth,
              scrollHeight: el.scrollHeight,
              overflowX: cs.overflowX,
              overflowY: cs.overflowY,
              display: cs.display,
            })
            for (const c of el.children) walk(c)
          }
          const origin = sec.getBoundingClientRect()
          walk(sec)
          // Normalize to section-relative
          for (const b of boxes) {
            b.x -= origin.x
            b.y -= origin.y
          }
          out[id] = boxes
        }
        return out
      })
      for (const id of caseIds) {
        map.set(id, measured[id] || [])
      }
    } finally {
      await browser.close().catch(() => {})
    }
  } finally {
    await new Promise((r) => server.close(() => r()))
  }
  return { skipped: false, map }
}

function classKey(cls) {
  return (cls || '')
    .split(/\s+/)
    .filter(
      (c) =>
        c.startsWith('sail-') ||
        c.startsWith('s-w') ||
        c.startsWith('s-h') ||
        c.startsWith('s-grow') ||
        c.startsWith('s-shrink') ||
        c.startsWith('s-min') ||
        c.startsWith('s-max') ||
        c.startsWith('s-pad') ||
        c.startsWith('s-overflow') ||
        c.startsWith('s-p') ||
        c.includes('card') ||
        c.includes('pad'),
    )
    .slice(0, 10)
    .join(' ')
}

function matchKey(box) {
  const tag = (box.tag || '').toLowerCase()
  const ck = classKey(box.class)
  return ck ? `${tag}|${ck}` : tag
}

/**
 * Diff taffy boxes vs browser boxes (size gate).
 * Position is advisory (parent-rel vs absoluteized).
 */
export function diffBoxes(taffyBoxes, browserBoxes, tol = TOL) {
  const tLayout = (taffyBoxes || []).map((b, i) => ({ ...b, _i: i }))
  const bLayout = (browserBoxes || []).map((b, i) => ({ ...b, _i: i }))
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
      b = bLayout.find(
        (x) => !bUsed.has(x._i) && (x.tag || '').toLowerCase() === (t.tag || '').toLowerCase(),
      )
    }
    if (b) {
      bUsed.add(b._i)
      pairs.push({ t, b, key: k })
    } else {
      pairs.push({ t, b: null, key: k })
    }
  }

  let sizeMatch = 0
  let sizeFail = 0
  let compared = 0
  let tagDiverge = 0
  const diffs = []
  for (let i = 0; i < pairs.length; i++) {
    const { t, b } = pairs[i]
    if (!b) {
      tagDiverge++
      diffs.push({ i, reason: 'unmatched-taffy', t, b: null })
      continue
    }
    compared++
    if ((t.tag || '').toLowerCase() !== (b.tag || '').toLowerCase()) {
      tagDiverge++
      diffs.push({ i, reason: 'tag-diverge', t, b })
      continue
    }
    const dw = Math.abs((t.width ?? 0) - (b.width ?? 0))
    const dh = Math.abs((t.height ?? 0) - (b.height ?? 0))
    const sizeOk = dw <= tol && dh <= tol
    if (sizeOk) sizeMatch++
    else {
      sizeFail++
      diffs.push({
        i,
        reason: 'size',
        dw,
        dh,
        t: { tag: t.tag, class: classKey(t.class), w: t.width, h: t.height },
        b: { tag: b.tag, class: classKey(b.class), w: b.width, h: b.height },
      })
    }
  }
  return {
    compared,
    size_match: sizeMatch,
    size_fail: sizeFail,
    tag_diverge: tagDiverge,
    unused_browser: bLayout.filter((x) => !bUsed.has(x._i)).length,
    structure_ok:
      tLayout.length === bLayout.length && tagDiverge === 0 && bLayout.length > 0,
    size_ok: sizeFail === 0 && compared > 0,
    diffs: diffs.slice(0, 12),
  }
}

/** Map diagnostic kinds to error class ids. */
export function classifyDiag(diag) {
  const msg = (diag.message || '').toLowerCase()
  const kind = diag.kind || ''
  // Order matters: silent-clip messages also contain "overflows its parent".
  if (
    kind === 'collapsed-into-overflow-hidden-parent' ||
    kind === 'unbounded-growth-into-overflow-hidden-parent' ||
    msg.includes('overflow:hidden') ||
    msg.includes('silently clipped') ||
    msg.includes('clipped silent') ||
    msg.includes('bounds against a 0px parent')
  ) {
    return 'silent-clip'
  }
  if (kind === 'child-overflows-parent' || msg.includes('overflows its parent')) {
    return 'child-overflows-parent'
  }
  if (kind === 'zero-main-axis' || msg.includes('measures 0px on its main axis')) {
    return 'height-chain'
  }
  return null
}

/**
 * Browser ground truth for error classes from measured boxes.
 * Heuristics (documented in verdict):
 *  - silent-clip: some box has overflow hidden/clip AND scroll > client on main axis with client main ≈ 0 or child larger
 *  - child-overflows-parent: child height/width > parent (with visible overflow)
 *  - height-chain: element height ≤ 0.5 with children of non-zero total height
 */
export function browserErrorTruth(boxes) {
  const truth = {
    'silent-clip': false,
    'child-overflows-parent': false,
    'height-chain': false,
  }
  if (!boxes?.length) return truth

  // Build simple parent-child by nesting y/x containment order (document order walk already nested)
  // Use sequential walk: parent candidates are earlier boxes that contain later ones.
  for (let i = 0; i < boxes.length; i++) {
    const p = boxes[i]
    const pOverflowSilent =
      p.overflowY === 'hidden' ||
      p.overflowY === 'clip' ||
      p.overflowX === 'hidden' ||
      p.overflowX === 'clip'
    if (pOverflowSilent) {
      if (
        (p.scrollHeight || 0) > (p.clientHeight || 0) + 1 ||
        (p.scrollWidth || 0) > (p.clientWidth || 0) + 1
      ) {
        truth['silent-clip'] = true
      }
      // collapsed parent with positive scroll or child
      if ((p.height || 0) <= 0.5 && ((p.scrollHeight || 0) > 0.5 || (p.clientHeight || 0) === 0)) {
        // look for any later box that might be a child with size
        for (let j = i + 1; j < Math.min(i + 8, boxes.length); j++) {
          if ((boxes[j].height || 0) > 0.5) {
            truth['silent-clip'] = true
            truth['height-chain'] = true
            break
          }
        }
      }
    }
    if ((p.height || 0) <= 0.5) {
      for (let j = i + 1; j < Math.min(i + 6, boxes.length); j++) {
        if ((boxes[j].height || 0) > 0.5) {
          truth['height-chain'] = true
          break
        }
      }
    }
  }

  // child overflows parent (document-order containment)
  for (let i = 0; i < boxes.length; i++) {
    const p = boxes[i]
    for (let j = i + 1; j < boxes.length; j++) {
      const c = boxes[j]
      // crude: child starts within parent y and is taller than parent
      if (
        c.y >= p.y - 1 &&
        c.y < p.y + Math.max(p.height, 1) &&
        c.height > p.height + 2 &&
        p.height > 0.5 &&
        p.overflowY !== 'hidden' &&
        p.overflowY !== 'clip'
      ) {
        truth['child-overflows-parent'] = true
      }
    }
  }
  return truth
}

export function takeRepro(caseObj, meta, max = 40) {
  return {
    id: caseObj.id,
    source: caseObj.source,
    dims: caseObj.dims || null,
    family: caseObj.family,
    ...meta,
  }
}

export function writeJson(path, obj) {
  writeFileSync(path, JSON.stringify(obj, null, 2))
}

/** Run sail render for paint region styles. */
export function sailRender(sourcePath, viewport = PAINT_VIEWPORT) {
  const outPng = join(tmpdir(), `v1-paint-${basename(sourcePath, '.sail')}.png`)
  const r = spawnSync(
    SAIL_BIN,
    [
      'render',
      sourcePath,
      '--viewport',
      `${viewport.width}x${viewport.height}`,
      '--out',
      outPng,
      '--json',
      '--theme',
      'ship',
      '--mode',
      'light',
    ],
    { cwd: repoRoot, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  )
  const jsonPath = outPng.replace(/\.png$/, '.json')
  if (!existsSync(jsonPath)) {
    return { ok: false, error: (r.stderr || r.stdout || 'render failed').slice(0, 400) }
  }
  try {
    const data = JSON.parse(readFileSync(jsonPath, 'utf8'))
    return { ok: true, regions: data.regions || [], ms: data.ms, png: outPng }
  } catch (e) {
    return { ok: false, error: e.message }
  }
}

/**
 * Measure browser computed styles for L1 hosts in a single-case page.
 */
export async function measureBrowserStyles(serveDir, viewport = PAINT_VIEWPORT) {
  if (!CHROME || !existsSync(CHROME)) {
    return { skipped: true, reason: `chrome missing`, hosts: [] }
  }
  let chromium
  try {
    ;({ chromium } = await importPlaywright())
  } catch (e) {
    return { skipped: true, reason: e.message, hosts: [] }
  }
  const L1 = new Set([
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
        viewport: { width: viewport.width, height: viewport.height },
      })
      await page.goto(`http://127.0.0.1:${port}/`, {
        waitUntil: 'domcontentloaded',
        timeout: 30000,
      })
      await page.waitForTimeout(150)
      const hosts = await page.evaluate((l1list) => {
        const L1 = new Set(l1list)
        // Convert any CSS color (oklch/lab/…) to rgba via canvas pixel sample.
        const toRgba = (c) => {
          if (!c || c === 'transparent') return 'rgba(0, 0, 0, 0)'
          try {
            const canvas = document.createElement('canvas')
            canvas.width = 1
            canvas.height = 1
            const ctx = canvas.getContext('2d', { willReadFrequently: true })
            ctx.clearRect(0, 0, 1, 1)
            ctx.fillStyle = c
            ctx.fillRect(0, 0, 1, 1)
            const [r, g, b, a] = ctx.getImageData(0, 0, 1, 1).data
            return `rgba(${r}, ${g}, ${b}, ${(a / 255).toFixed(3)})`
          } catch {
            return c
          }
        }
        const out = []
        const walk = (el) => {
          if (el.nodeType !== 1) return
          const tag = el.tagName.toLowerCase()
          if (L1.has(tag) || el.hasAttribute('data-sail-host')) {
            const cs = getComputedStyle(el)
            const r = el.getBoundingClientRect()
            out.push({
              tag,
              x: r.x,
              y: r.y,
              w: r.width,
              h: r.height,
              backgroundColor: toRgba(cs.backgroundColor),
              color: toRgba(cs.color),
              borderRadius: cs.borderRadius,
              borderTopWidth: cs.borderTopWidth,
              borderTopColor: toRgba(cs.borderTopColor),
              opacity: cs.opacity,
            })
          }
          for (const c of el.children) walk(c)
        }
        walk(document.body)
        return out
      }, [...L1])
      return { skipped: false, hosts }
    } finally {
      await browser.close()
    }
  } finally {
    await new Promise((r) => server.close(() => r()))
  }
}

function parseCssColor(s) {
  if (!s) return null
  const str = String(s).trim()
  const m = str.match(/rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)/i)
  if (m) return [Number(m[1]), Number(m[2]), Number(m[3])]
  // #rrggbb
  if (str.startsWith('#')) {
    const h = str.slice(1)
    const full =
      h.length === 3
        ? h
            .split('')
            .map((x) => x + x)
            .join('')
        : h.slice(0, 6)
    if (/^[0-9a-fA-F]{6}$/.test(full)) {
      const n = parseInt(full, 16)
      return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
    }
  }
  return null
}

export function paintDiff(takumiRegions, browserHosts, colorTol = 28, pxTol = 2) {
  const tHosts = (takumiRegions || []).filter((r) => {
    const tag = (r.tag || '').toLowerCase()
    return (
      r.style &&
      (tag === 'button' ||
        tag === 'card' ||
        tag === 'badge' ||
        tag === 'input' ||
        tag === 'field' ||
        tag === 'navbar' ||
        tag === 'sidebar' ||
        tag === 'dialog' ||
        tag === 'table' ||
        tag === 'switch')
    )
  })
  let propsCompared = 0
  let propsMatch = 0
  let propsFail = 0
  const fails = []
  const used = new Set()
  for (const t of tHosts) {
    const tag = (t.tag || '').toLowerCase()
    const b = browserHosts.find((h, i) => !used.has(i) && h.tag === tag)
    if (!b) continue
    used.add(browserHosts.indexOf(b))
    const pairs = [
      ['backgroundColor', t.style.backgroundColor, b.backgroundColor, 'color'],
      ['color', t.style.color, b.color, 'color'],
      ['borderRadius', t.style.borderRadius, b.borderRadius, 'px'],
    ]
    for (const [name, tv, bv, kind] of pairs) {
      if (tv == null || tv === '') continue
      propsCompared++
      if (kind === 'color') {
        const tc = parseCssColor(tv)
        const bc = parseCssColor(bv)
        if (!tc || !bc) {
          propsFail++
          fails.push({ name, tv, bv, reason: 'parse' })
          continue
        }
        const d = Math.max(
          Math.abs(tc[0] - bc[0]),
          Math.abs(tc[1] - bc[1]),
          Math.abs(tc[2] - bc[2]),
        )
        if (d <= colorTol) propsMatch++
        else {
          propsFail++
          fails.push({ name, d, tv, bv })
        }
      } else {
        const tn = parseFloat(tv)
        const bn = parseFloat(bv)
        if (Number.isFinite(tn) && Number.isFinite(bn) && Math.abs(tn - bn) <= pxTol) propsMatch++
        else {
          propsFail++
          fails.push({ name, tv, bv })
        }
      }
    }
  }
  return {
    takumi_hosts: tHosts.length,
    browser_hosts: browserHosts.length,
    props_compared: propsCompared,
    props_match: propsMatch,
    props_fail: propsFail,
    match_rate: propsCompared ? propsMatch / propsCompared : null,
    fails: fails.slice(0, 8),
  }
}
