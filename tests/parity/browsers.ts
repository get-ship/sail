/**
 * Playwright engines for the taffy geometry suite.
 * Default: chromium + firefox + webkit (Playwright's WebKit build).
 *
 *   --browser chromium
 *   --browser chromium,firefox
 *   PARITY_BROWSERS=chromium
 */

export const BROWSER_NAMES = ['chromium', 'firefox', 'webkit'] as const
export type BrowserName = (typeof BROWSER_NAMES)[number]

export function isBrowserName(s: string): s is BrowserName {
  return (BROWSER_NAMES as readonly string[]).includes(s)
}

export function parseBrowsers(argv: string[]): BrowserName[] {
  const env = process.env.PARITY_BROWSERS
  const flagIdx = argv.indexOf('--browser')
  const raw = flagIdx >= 0 ? argv[flagIdx + 1] : env
  if (!raw || raw === 'all') return [...BROWSER_NAMES]
  const names = raw
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean)
  const out: BrowserName[] = []
  for (const n of names) {
    if (!isBrowserName(n)) {
      throw new Error(`unknown --browser ${n} (want ${BROWSER_NAMES.join('|')})`)
    }
    if (!out.includes(n)) out.push(n)
  }
  if (!out.length) throw new Error('empty --browser list')
  return out
}

export async function loadPlaywright(): Promise<typeof import('playwright')> {
  try {
    return await import('playwright')
  } catch (e) {
    throw new Error(`playwright import failed: ${(e as Error).message}`)
  }
}
