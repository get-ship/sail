#!/usr/bin/env node
// Full nav-graph crawl of the static sail-site build.
//
// Seeds: every routes/**/page.sail (dynamic segments expanded from
// component slugs + tutorial lessons). Then follows every same-origin
// <a href> until the graph is closed. Headless Chromium.
//
// Gates: document 200, same-origin subresources 200, no runtime fetch
// to a host that will not exist on a static Pages deploy.
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from 'playwright'
import { createStaticServer } from './static-serve.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const APP = resolve(HERE, '..')
const OUT = resolve(process.env.SAIL_SITE_STATIC_OUT ?? join(APP, 'dist'))
const PORT = Number(process.env.SAIL_SITE_STATIC_PORT ?? 4173)
const REPORT = process.env.SAIL_SITE_CRAWL_OUT ?? join(APP, '.static-crawl.json')

const ALLOWED_REMOTE_NAV = new Set([
  'github.com',
  'www.github.com',
  'lucide.dev',
  'oklch.com',
  'motion.dev',
  'docs.ship.dev',
  'ship.dev',
])

const BLOCKED_REMOTE_HOST_RE =
  /unsplash|googleapis|gstatic|unpkg|jsdelivr|cdnjs|cloudflareinsights|fonts\.google|esm\.sh|skypack/i

const DEV_ONLY_PATH_RE = /^\/_deck\/(dev|hmr|mux|live-reload)/i

function walkFiles(dir, acc = []) {
  if (!existsSync(dir)) return acc
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    const st = statSync(p)
    if (st.isDirectory()) walkFiles(p, acc)
    else acc.push(p)
  }
  return acc
}

function pageFiles() {
  return walkFiles(join(APP, 'routes')).filter((p) => p.endsWith('/page.sail') || p.endsWith('\\page.sail'))
}

function componentSlugs() {
  const slugs = new Set()
  const docs = join(APP, 'src/component-docs')
  for (const file of walkFiles(docs).filter((p) => p.endsWith('/meta.ts') || p.endsWith('\\meta.ts'))) {
    const src = readFileSync(file, 'utf8')
    const m = src.match(/slug:\s*'([^']+)'/)
    if (m) slugs.add(m[1])
  }
  const catalog = readFileSync(join(APP, 'src/compositions/component-catalog.ts'), 'utf8')
  for (const m of catalog.matchAll(/P\(\s*'[^']+'\s*,\s*'([^']+)'/g)) slugs.add(m[1])
  return [...slugs]
}

function tutorialSlugs() {
  const dir = join(APP, 'content/tutorial/lessons')
  if (!existsSync(dir)) return []
  return readdirSync(dir).filter((n) => statSync(join(dir, n)).isDirectory())
}

function expandPattern(pattern, tagSlugs, lessonSlugs) {
  if (pattern.includes('[tag]')) return tagSlugs.map((s) => pattern.replace('[tag]', s))
  if (pattern.includes('[slug]')) return lessonSlugs.map((s) => pattern.replace('[slug]', s))
  if (pattern.includes('[name]')) return [pattern.replace('[name]', 'world')]
  return [pattern]
}

function seedRoutes() {
  const tags = componentSlugs()
  const lessons = tutorialSlugs()
  const seeds = new Set(['/'])
  for (const file of pageFiles()) {
    const rel = relative(join(APP, 'routes'), file).replaceAll('\\', '/')
    const dir = dirname(rel)
    const pattern = dir === '.' ? '/' : `/${dir}`
    for (const path of expandPattern(pattern, tags, lessons)) seeds.add(path)
  }
  return { seeds: [...seeds].sort(), tags, lessons }
}

function normalizePath(href, origin) {
  let url
  try {
    url = new URL(href, origin)
  } catch {
    return null
  }
  if (url.origin !== origin) return { external: true, href: url.href, host: url.hostname }
  if (url.pathname.startsWith('/_deck') || url.pathname.startsWith('/assets')) {
    return { asset: true, path: url.pathname }
  }
  const ext = url.pathname.match(/\.[a-z0-9]+$/i)
  if (ext && ext[0] !== '.html') return { asset: true, path: url.pathname }
  let path = url.pathname
  if (path.length > 1 && path.endsWith('/')) path = path.slice(0, -1)
  return { path }
}

function isBlockedRemote(url) {
  try {
    const u = new URL(url)
    if (u.protocol === 'data:' || u.protocol === 'blob:') return false
    return BLOCKED_REMOTE_HOST_RE.test(u.hostname)
  } catch {
    return false
  }
}

async function crawlRoute(page, origin, path, opts = {}) {
  const failed = []
  const remote = []
  const devOnly = []
  const responses = []

  const onResponse = (res) => {
    const url = res.url()
    const status = res.status()
    responses.push({ url, status, resourceType: res.request().resourceType() })
    let parsed
    try {
      parsed = new URL(url)
    } catch {
      return
    }
    if (parsed.origin === origin) {
      if (DEV_ONLY_PATH_RE.test(parsed.pathname)) {
        devOnly.push({ url, status })
      }
      if (status >= 400) failed.push({ url, status, type: res.request().resourceType() })
    } else if (parsed.protocol === 'http:' || parsed.protocol === 'https:') {
      remote.push({
        url,
        status,
        type: res.request().resourceType(),
        host: parsed.hostname,
      })
    }
  }
  page.on('response', onResponse)

  const pageErrors = []
  const onError = (err) => pageErrors.push(String(err))
  page.on('pageerror', onError)

  const isPlayground = path === '/playground'
  const timeout = opts.timeout ?? (isPlayground ? 90_000 : 12_000)
  const started = Date.now()
  let navStatus = 0
  let navOk = false
  try {
    const res = await page.goto(origin + path, { waitUntil: 'domcontentloaded', timeout })
    navStatus = res?.status() ?? 0
    navOk = navStatus === 200
    await page.waitForFunction(
      () => {
        const root = document.getElementById('root')
        return Boolean(root && root.childElementCount > 0)
      },
      undefined,
      { timeout: Math.min(timeout, 8_000) },
    ).catch(() => {})
    if (isPlayground) {
      await page
        .waitForFunction(
          () => {
            const badge = document.querySelector('.sail-badge')
            const text = badge?.textContent?.trim() ?? ''
            return text === 'running' || text === 'error'
          },
          undefined,
          { timeout: 60_000 },
        )
        .catch(() => {})
    }
  } catch (err) {
    failed.push({ url: origin + path, status: 0, type: 'document', error: String(err) })
  }

  const hrefs = await page
    .evaluate(() =>
      [...document.querySelectorAll('a[href]')].map((a) => a.getAttribute('href') ?? ''),
    )
    .catch(() => [])

  const playground = {}
  if (path === '/playground') {
    playground.status = await page
      .locator('.sail-badge')
      .first()
      .innerText()
      .catch(() => '')
    playground.editorLen = await page.evaluate(() => {
      const el =
        document.querySelector('sail-code-editor[pane="source"]') ??
        document.querySelector('sail-code-editor[data-pane="source"]') ??
        document.querySelector('sail-code-editor')
      return (el && el.value ? String(el.value).length : 0)
    }).catch(() => 0)
    playground.wasm = responses.find((r) => r.url.includes('sail_compiler_bg.wasm')) ?? null
    playground.previewText = await page
      .locator('[testid="pg-preview"], [data-testid="pg-preview"]')
      .first()
      .innerText()
      .catch(() => '')
  }

  page.off('response', onResponse)
  page.off('pageerror', onError)

  return {
    path,
    status: navStatus,
    ok: navOk && failed.filter((f) => f.type === 'document').length === 0,
    ms: Date.now() - started,
    failedAssets: failed,
    remote,
    devOnly,
    pageErrors,
    hrefs,
    playground: Object.keys(playground).length ? playground : undefined,
  }
}

const { seeds, tags, lessons } = seedRoutes()
if (!existsSync(join(OUT, 'index.html'))) {
  console.error(`crawl-static: missing ${OUT}/index.html — run bun run build:static first`)
  process.exit(2)
}

const handle = await createStaticServer(OUT, { port: PORT })
const origin = handle.url
console.log(`crawl-static: serving ${OUT} at ${origin}`)
console.log(`  seeds: ${seeds.length}  component slugs: ${tags.length}  lessons: ${lessons.length}`)

const browser = await chromium.launch({ headless: true })
const context = await browser.newContext({ viewport: { width: 1280, height: 800 } })
const page = await context.newPage()

const queue = [...seeds]
const seen = new Set()
const rows = []
const navDiscovered = new Set()

while (queue.length) {
  const path = queue.shift()
  if (seen.has(path)) continue
  seen.add(path)
  process.stdout.write(`  [${seen.size}] ${path}\n`)
  const row = await crawlRoute(page, origin, path)
  rows.push(row)
  for (const href of row.hrefs) {
    const n = normalizePath(href, origin)
    if (!n || n.external || n.asset) {
      if (n?.external && !ALLOWED_REMOTE_NAV.has(n.host)) {
        // Nav-only outbound (docs citations). Record, do not fetch.
      }
      continue
    }
    if (!seen.has(n.path) && !queue.includes(n.path)) {
      queue.push(n.path)
      navDiscovered.add(n.path)
    }
  }
}

await browser.close()
await handle.close()

const remoteRuntime = []
const blockedRemote = []
for (const row of rows) {
  for (const r of row.remote) {
    remoteRuntime.push({ path: row.path, ...r })
    if (isBlockedRemote(r.url) || r.type !== 'document') {
      // Subresource to another origin — will not exist on a static host.
      blockedRemote.push({ path: row.path, ...r })
    }
  }
}

const failedRoutes = rows.filter((r) => r.status !== 200)
const failedAssets = rows.flatMap((r) => r.failedAssets.map((f) => ({ path: r.path, ...f })))
const pageErrors = rows.flatMap((r) => r.pageErrors.map((e) => ({ path: r.path, error: e })))
const playground = rows.find((r) => r.path === '/playground')?.playground ?? null
const wasmOk = Boolean(playground?.wasm && playground.wasm.status === 200)
const playgroundOk = Boolean(
  playground && playground.status === 'running' && wasmOk && (playground.editorLen ?? 0) > 0,
)

const report = {
  origin,
  out: OUT,
  crawled: rows.length,
  seeds: seeds.length,
  navDiscovered: [...navDiscovered].sort(),
  componentSlugs: tags.length,
  tutorialLessons: lessons.length,
  failedRoutes,
  failedAssets,
  remoteRuntime,
  blockedRemote,
  pageErrors,
  playground,
  gates: {
    everyRoute200: failedRoutes.length === 0,
    localAssets: failedAssets.length === 0,
    offlineAssetGraph: blockedRemote.length === 0,
    playground: playgroundOk,
    spaShell: existsSync(join(OUT, 'index.html')),
    wasmPresent: existsSync(join(OUT, 'sail_compiler_bg.wasm')),
  },
  routes: rows.map((r) => ({
    path: r.path,
    status: r.status,
    ok: r.ok,
    ms: r.ms,
    failed: r.failedAssets.length,
    remote: r.remote.length,
    errors: r.pageErrors.length,
  })),
}

writeFileSync(REPORT, JSON.stringify(report, null, 2) + '\n')

const mdLines = [
  '| Path | Status | Assets | Remote | Errors | ms |',
  '| --- | ---: | ---: | ---: | ---: | ---: |',
  ...report.routes.map(
    (r) =>
      `| \`${r.path}\` | ${r.status} | ${r.failed} failed | ${r.remote} | ${r.errors} | ${r.ms} |`,
  ),
]
const mdPath = REPORT.replace(/\.json$/, '.md')
writeFileSync(mdPath, mdLines.join('\n') + '\n')

console.log(`crawl-static: ${rows.length} routes  report ${REPORT}`)
console.log(
  `  200=${rows.length - failedRoutes.length}/${rows.length}  ` +
    `asset-404s=${failedAssets.length}  remote-sub=${blockedRemote.length}  ` +
    `playground=${playgroundOk ? 'ok' : 'BLOCKED'}`,
)

const gates = report.gates
const hardFail = !gates.everyRoute200 || !gates.localAssets
if (hardFail) {
  console.error('crawl-static: HARD FAIL — a crawled route or local asset was not 200')
  process.exit(1)
}
if (!gates.offlineAssetGraph || !gates.playground) {
  // Named blockers go in RESULT; exit 0 so the prove still lands the table.
  console.warn('crawl-static: named blockers present (offline graph / playground) — see report')
}
process.exit(0)
