/**
 * STUDIO-BROWSER-E2E / STUDIO-BOOT-1 — boot smoke: every studio surface body
 * mounts with zero pageerrors of any kind (mount-scope + product runtime).
 *
 * Requires sail dev (default http://127.0.0.1:7891):
 *
 *   ./target/debug/sail dev --project apps/studio --addr 127.0.0.1:7891
 *   cd apps/studio && bunx vitest run test/boot-smoke.e2e.test.ts
 *
 * Optional: STUDIO_EVIDENCE_DIR — write per-route screenshots (png) into that dir.
 *
 * @vitest-environment node
 */
import { describe, expect, it } from 'vitest'
import http from 'node:http'
import { mkdir, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const STUDIO = process.env.STUDIO_URL || 'http://127.0.0.1:7891'
const EVIDENCE_DIR = process.env.STUDIO_EVIDENCE_DIR || ''

/**
 * Product surfaces from STUDIO-BOOT-1, plus `/` which is the dual-scope canary:
 * root's page controller calls ambient `mount()` to redirect. That path goes
 * through deck-route's host `mount` → setup scope; if deck-route inlined
 * effect-scope, CURRENT_SCOPE forks and SailMountOutsideSetupError fires here
 * first (shell chrome still paints — it is a custom element).
 */
const ROUTES: { path: string; label: string; expectSurface?: string }[] = [
  { path: '/', label: 'root-redirect', expectSurface: 'edit-surface' },
  { path: '/edit', label: 'edit', expectSurface: 'edit-surface' },
  { path: '/components', label: 'components', expectSurface: 'components-surface' },
  { path: '/themes', label: 'themes', expectSurface: 'themes-surface' },
  { path: '/tasks', label: 'tasks', expectSurface: 'tasks-surface' },
  { path: '/config', label: 'config', expectSurface: 'config-surface' },
]

async function studioUp(): Promise<boolean> {
  return new Promise((resolve) => {
    try {
      const u = new URL(STUDIO)
      const req = http.request(
        {
          hostname: u.hostname,
          port: u.port || 80,
          path: '/',
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

describe('studio boot smoke (live sail dev)', () => {
  it('studio routes mount surface bodies with zero pageerrors', async () => {
    if (!(await studioUp())) {
      throw new Error(
        `[FAIL] studio not reachable at ${STUDIO} — boot sail dev before boot-smoke`,
      )
    }

    if (EVIDENCE_DIR) {
      await mkdir(EVIDENCE_DIR, { recursive: true })
    }

    const { chromium } = await import('playwright')
    const browser = await chromium.launch({ headless: true })
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } })
    const pageerrors: string[] = []
    page.on('pageerror', (e) => pageerrors.push(String(e.stack || e.message || e)))

    try {
      for (const route of ROUTES) {
        pageerrors.length = 0
        const res = await page.goto(`${STUDIO}${route.path}`, {
          waitUntil: 'networkidle',
          timeout: 45_000,
        })
        expect(res && res.status() < 500, `${route.label} status ${res?.status()}`).toBe(true)
        // Shell chrome (activity rail) always paints; wait for it then settle.
        await page.waitForSelector('[aria-label="Studio activities"]', {
          timeout: 20_000,
        })
        // Root redirect needs a beat for ambient mount → navigate → leaf mount.
        await page.waitForTimeout(route.path === '/' ? 1500 : 800)

        if (route.expectSurface) {
          await page.waitForSelector(route.expectSurface, { timeout: 20_000 })
        }

        // Zero pageerrors of any kind (not only SailMountOutsideSetupError).
        expect(
          pageerrors,
          `${route.label}: pageerrors:\n${pageerrors.join('\n---\n')}`,
        ).toEqual([])

        if (EVIDENCE_DIR) {
          const shot = join(EVIDENCE_DIR, `${route.label}.png`)
          await page.screenshot({ path: shot, fullPage: true })
          await writeFile(
            join(EVIDENCE_DIR, `${route.label}.pageerrors.json`),
            JSON.stringify({ route: route.path, label: route.label, pageerrors: [] }, null, 2) +
              '\n',
          )
        }
      }
    } finally {
      await browser.close()
    }
  }, 180_000)
})
