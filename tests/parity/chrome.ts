/**
 * Shared Chromium resolution for PARITY-1 suites (geometry + paint).
 *
 * Order:
 *   1. CHROME_PATH env (if path exists)
 *   2. playwright / playwright-core `chromium.executablePath()`
 *   3. Newest chrome under ~/.cache/ms-playwright (same search as CI)
 *   4. Pinned chromium-1228 path (historical local fallback)
 *
 * CI should leave CHROME_PATH unset and rely on (2)/(3) after
 * `playwright install chromium`.
 */

import { existsSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { homedir } from 'node:os'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

function pinnedFallback() {
  return join(homedir(), '.cache/ms-playwright/chromium-1228/chrome-linux64/chrome')
}

/** Walk cache dir for chrome-linux chrome binaries; prefer highest version path. */
function findInMsPlaywrightCache() {
  const root = join(homedir(), '.cache/ms-playwright')
  if (!existsSync(root)) return null
  const found = []
  const walk = (dir, depth) => {
    if (depth > 4) return
    let entries
    try {
      entries = readdirSync(dir)
    } catch {
      return
    }
    for (const name of entries) {
      const p = join(dir, name)
      let st
      try {
        st = statSync(p)
      } catch {
        continue
      }
      if (st.isDirectory()) {
        if (name.startsWith('chrome-linux')) {
          const chrome = join(p, 'chrome')
          if (existsSync(chrome)) found.push(chrome)
        }
        walk(p, depth + 1)
      }
    }
  }
  walk(root, 0)
  if (!found.length) return null
  found.sort()
  return found[found.length - 1]
}

function playwrightExecutable() {
  for (const mod of ['playwright', 'playwright-core']) {
    try {
      const pw = require(mod)
      const path = pw.chromium?.executablePath?.()
      if (path && existsSync(path)) return path
    } catch {
      /* try next */
    }
  }
  return null
}

/**
 * Resolve a Chromium binary path, or null if none found.
 * @returns {{ path: string|null, source: string }}
 */
export function resolveChrome() {
  const env = process.env.CHROME_PATH
  if (env && existsSync(env)) {
    return { path: env, source: 'CHROME_PATH' }
  }
  // Env set but missing — fall through rather than hard-fail; suite will skip.
  const fromPw = playwrightExecutable()
  if (fromPw) return { path: fromPw, source: 'playwright.executablePath' }

  const fromCache = findInMsPlaywrightCache()
  if (fromCache) return { path: fromCache, source: 'ms-playwright-cache' }

  const pinned = pinnedFallback()
  if (existsSync(pinned)) return { path: pinned, source: 'pinned-chromium-1228' }

  return { path: env || pinned, source: env ? 'CHROME_PATH-missing' : 'none' }
}
