/**
 * render phase — browser ground truth screenshot → run_dir/render.png.
 * Blocking: missing render ⇒ failed run. Writes buffers directly (no tool-specific save).
 */

import { createServer } from 'node:http'
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { extname, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import {
  CONTRACT_VERSION,
  type PhaseResult,
  type RunManifest,
  emptyPhaseResult,
} from '../contract/types.ts'
import { fileSha256 } from '../config/hash.ts'
import { resolveChrome, resolveSailBin } from '../naming/paths.ts'
import { isPathInside } from '../naming/safe-path.ts'

const MIME: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.woff2': 'font/woff2',
}

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const s = createServer()
    s.listen(0, '127.0.0.1', () => {
      const addr = s.address()
      if (addr && typeof addr === 'object') {
        const port = addr.port
        s.close(() => resolve(port))
      } else {
        s.close(() => reject(new Error('no port')))
      }
    })
    s.on('error', reject)
  })
}

async function serveStatic(root: string, port: number): Promise<{ close: () => Promise<void> }> {
  const rootAbs = resolve(root)
  const server = createServer((req, res) => {
    const url = (req.url ?? '/').split('?')[0] || '/'
    const rel = url === '/' ? '/index.html' : url
    const file = resolve(rootAbs, '.' + rel)
    if (!isPathInside(rootAbs, file) || !existsSync(file) || !statSync(file).isFile()) {
      res.writeHead(404)
      res.end('not found')
      return
    }
    const ext = extname(file)
    res.writeHead(200, { 'Content-Type': MIME[ext] ?? 'application/octet-stream' })
    res.end(readFileSync(file))
  })
  await new Promise<void>((resolve, reject) => {
    server.listen(port, '127.0.0.1', () => resolve())
    server.on('error', reject)
  })
  return {
    close: () =>
      new Promise((resolve) => {
        server.close(() => resolve())
      }),
  }
}

/** Minimal HTML wrapper for sail CSR dist or raw sandbox. */
function ensureIndexHtml(sandbox: string): string | null {
  const candidates = [
    join(sandbox, 'dist/index.html'),
    join(sandbox, 'index.html'),
    join(sandbox, 'out/index.html'),
  ]
  for (const c of candidates) {
    if (existsSync(c)) return c.endsWith('index.html') ? join(c, '..') : c
  }
  // Build a tiny static preview from page content if no index
  const page = join(sandbox, 'routes/page.sail')
  if (existsSync(page)) {
    const preview = join(sandbox, '.bench-render-preview')
    const html = `<!doctype html><html><head><meta charset="utf-8"><title>bench render</title>
<style>body{font-family:system-ui;padding:2rem;max-width:48rem;margin:auto;line-height:1.5}
pre{white-space:pre-wrap;background:#f6f6f6;padding:1rem;border-radius:8px}</style>
</head><body><h1>sail source preview</h1><p>Browser ground-truth preview of authored source (no CSR bundle).</p>
<pre>${escapeHtml(readFileSync(page, 'utf8'))}</pre></body></html>`
    mkdirSync(preview, { recursive: true })
    writeFileSync(join(preview, 'index.html'), html)
    return preview
  }
  return null
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

async function screenshotWithPlaywright(
  chromePath: string,
  url: string,
  outPng: string,
): Promise<{ ok: boolean; error?: string; bytes?: number }> {
  // Prefer playwright if installed in sail monorepo; else puppeteer-less CDP via chrome itself is hard —
  // use playwright programmatic API when available.
  try {
    const pw = await import('playwright')
    const browser = await pw.chromium.launch({
      executablePath: chromePath,
      headless: true,
    })
    try {
      const page = await browser.newPage({ viewport: { width: 1280, height: 720 } })
      await page.goto(url, { waitUntil: 'networkidle', timeout: 30_000 })
      await page.screenshot({ path: outPng, fullPage: true, type: 'png' })
    } finally {
      await browser.close()
    }
    if (existsSync(outPng)) {
      const st = statSync(outPng)
      return { ok: st.size > 0, bytes: st.size }
    }
    return { ok: false, error: 'screenshot file missing after playwright' }
  } catch (e) {
    // Fallback: chrome headless --screenshot (limited but no playwright dep)
    const r = spawnSync(
      chromePath,
      [
        '--headless=new',
        '--disable-gpu',
        '--no-sandbox',
        '--window-size=1280,720',
        `--screenshot=${outPng}`,
        url,
      ],
      { encoding: 'utf8', timeout: 60_000 },
    )
    if (existsSync(outPng) && statSync(outPng).size > 0) {
      return { ok: true, bytes: statSync(outPng).size }
    }
    const msg = e instanceof Error ? e.message : String(e)
    return {
      ok: false,
      error: `playwright/chrome screenshot failed: ${msg}; chrome_exit=${r.status} ${r.stderr?.slice(0, 200)}`,
    }
  }
}

function trySailBuild(sandbox: string, sailBin: string | null): void {
  if (!sailBin) return
  // Best-effort build for CSR; ignore failure — we fall back to source preview.
  spawnSync(sailBin, ['build'], {
    cwd: sandbox,
    encoding: 'utf8',
    timeout: 120_000,
  })
}

export async function renderPhase(manifest: RunManifest): Promise<PhaseResult> {
  const t0 = Date.now()
  const result = emptyPhaseResult('render')
  result.contract_version = CONTRACT_VERSION

  const outRel = manifest.paths.render
  const outPng = join(manifest.run_dir, outRel)
  const chrome = resolveChrome()
  const sailBin =
    (manifest.env.SAIL_BIN && existsSync(manifest.env.SAIL_BIN) ? manifest.env.SAIL_BIN : null) ??
    resolveSailBin()

  if (!chrome) {
    result.ok = false
    result.verdict = 'fail'
    result.flags.push('render_chrome_missing')
    result.missing.render = 'chrome_not_found'
    result.error = {
      code: 'RENDER_NO_CHROME',
      message: 'chromium not found (set CHROME_PATH or install playwright browsers)',
    }
    result.artifacts = [
      {
        name: 'render.png',
        rel_path: outRel,
        blocking: true,
        status: 'missing',
        reason: 'chrome_not_found',
      },
    ]
    // Marker file so the gap is physical, not silent
    writeFileSync(
      join(manifest.run_dir, 'render.MISSING.json'),
      JSON.stringify({ reason: 'chrome_not_found', ts: new Date().toISOString() }, null, 2) + '\n',
    )
    result.wall_ms = Date.now() - t0
    return result
  }

  try {
    trySailBuild(manifest.sandbox_dir, sailBin)
    const serveRoot = ensureIndexHtml(manifest.sandbox_dir)
    if (!serveRoot) {
      result.ok = false
      result.verdict = 'fail'
      result.flags.push('render_no_static_root')
      result.missing.render = 'no_static_root'
      result.artifacts = [
        {
          name: 'render.png',
          rel_path: outRel,
          blocking: true,
          status: 'missing',
          reason: 'no_static_root',
        },
      ]
      writeFileSync(
        join(manifest.run_dir, 'render.MISSING.json'),
        JSON.stringify({ reason: 'no_static_root', ts: new Date().toISOString() }, null, 2) + '\n',
      )
      result.wall_ms = Date.now() - t0
      return result
    }

    const port = await freePort()
    const server = await serveStatic(serveRoot, port)
    const url = `http://127.0.0.1:${port}/`
    try {
      const shot = await screenshotWithPlaywright(chrome, url, outPng)
      if (!shot.ok) {
        result.ok = false
        result.verdict = 'fail'
        result.flags.push('render_screenshot_failed')
        result.missing.render = shot.error ?? 'screenshot_failed'
        result.error = { code: 'RENDER_SHOT', message: shot.error ?? 'screenshot failed' }
        result.artifacts = [
          {
            name: 'render.png',
            rel_path: outRel,
            blocking: true,
            status: 'missing',
            reason: shot.error ?? 'screenshot_failed',
          },
        ]
        writeFileSync(
          join(manifest.run_dir, 'render.MISSING.json'),
          JSON.stringify({ reason: shot.error, ts: new Date().toISOString() }, null, 2) + '\n',
        )
      } else {
        const buf = readFileSync(outPng)
        result.ok = true
        result.verdict = 'pass'
        result.detail = {
          method: 'browser',
          url,
          viewport: { width: 1280, height: 720 },
          chrome,
          serve_root: serveRoot,
        }
        result.metrics = { render_bytes: shot.bytes }
        result.artifacts = [
          {
            name: 'render.png',
            rel_path: outRel,
            blocking: true,
            status: 'present',
            sha256: fileSha256(buf),
            bytes: buf.length,
          },
        ]
      }
    } finally {
      await server.close()
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    result.ok = false
    result.verdict = 'error'
    result.error = { code: 'RENDER', message: msg }
    result.flags.push('render_error')
    result.missing.render = msg
    result.artifacts = [
      {
        name: 'render.png',
        rel_path: outRel,
        blocking: true,
        status: 'missing',
        reason: msg,
      },
    ]
    writeFileSync(
      join(manifest.run_dir, 'render.MISSING.json'),
      JSON.stringify({ reason: msg, ts: new Date().toISOString() }, null, 2) + '\n',
    )
  }

  result.wall_ms = Date.now() - t0
  return result
}
