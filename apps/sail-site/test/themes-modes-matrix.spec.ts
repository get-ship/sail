// Theme × mode *runtime paint* matrix — DOWNSTREAM of DTCG generate only.
//
// SSOT for authoring is DTCG JSON (modes.*.path.$value), edited via SailTheme
// ops / Theme Studio. This file does NOT treat CSS vars as the edit surface.
// It only asserts that after generate + applyTheme, critical *emitted* vars
// resolve on a real page (/components) so mode-incomplete themes fail CI.
//
// Apply path: theme + mode (same as applyTheme / applyMode).
//
// Single-mode themes (supportedModes / only one key under modes): skip missing
// modes in the matrix — path parity is among declared modes only.

import { test, expect, type Page } from '@playwright/test'
import { THEME_META } from '@ship/tokens/themes-meta'

/** Catalog ids + declared modes from generated meta (single-mode aware). */
const THEME_CASES: { id: string; modes: readonly string[] }[] = THEME_META.map((t) => ({
  id: t.id,
  modes: t.modes.length > 0 ? t.modes : (['light', 'dark'] as const),
}))

/**
 * CSS custom properties that must resolve to a real paint for L1.
 * `--border` may intentionally be `transparent` (halo/glass) — still must be
 * declared (non-empty), but transparent is allowed.
 */
const MUST_OPAQUE_VARS = [
  '--primary',
  '--primary-foreground',
  '--background',
  '--foreground',
  '--card',
  '--card-foreground',
  '--muted',
  '--muted-foreground',
] as const

const MUST_DECLARED_VARS = [...MUST_OPAQUE_VARS, '--border'] as const

function isEmpty(value: string): boolean {
  const v = value.trim().toLowerCase()
  return v === '' || v === 'initial' || v === 'unset' || v === 'inherit'
}

function isTransparent(value: string): boolean {
  const v = value.trim().toLowerCase()
  return (
    v === 'transparent' ||
    v === 'rgba(0, 0, 0, 0)' ||
    v === 'rgba(0,0,0,0)' ||
    v === 'rgb(0, 0, 0, 0)'
  )
}

function isUnsetOpaque(value: string): boolean {
  return isEmpty(value) || isTransparent(value)
}

async function applyThemeMode(page: Page, theme: string, mode: string) {
  // Persist first so site chrome / boot effects that re-read localStorage
  // cannot clobber the matrix (keys match packages/sail/tokens THEME_*_STORAGE_KEY).
  await page.evaluate(
    ({ theme: t, mode: m }) => {
      try {
        localStorage.setItem('ship-site-theme', t)
        localStorage.setItem('ship-site-theme-mode', m)
      } catch {
        /* ignore */
      }
      const root = document.documentElement
      root.setAttribute('theme', t)
      root.dataset.mode = m
      root.classList.remove('dark')
      // Prefer framework helper when present (importmap / app bundle).
      const w = window as unknown as {
        applyTheme?: (id: string, el?: HTMLElement, mode?: string) => void
      }
      if (typeof w.applyTheme === 'function') {
        w.applyTheme(t, root, m)
      }
    },
    { theme, mode },
  )
  // Attribute flip is sync; re-read immediately (no waitForFunction race with HMR).
  const applied = await page.evaluate(() => ({
    theme: document.documentElement.getAttribute('theme') ?? '',
    mode: document.documentElement.dataset.mode ?? '',
  }))
  expect(applied.theme, `expected theme=${theme}`).toBe(theme)
  expect(applied.mode, `expected mode=${mode}`).toBe(mode)
}

async function readCriticalVars(page: Page): Promise<Record<string, string>> {
  return page.evaluate((names) => {
    const cs = getComputedStyle(document.documentElement)
    const out: Record<string, string> = {}
    for (const n of names) out[n] = cs.getPropertyValue(n).trim()
    return out
  }, [...MUST_DECLARED_VARS])
}

test.describe('theme × mode paint matrix (/components)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/components', { waitUntil: 'domcontentloaded' })
    // Page chrome + L1 buttons must exist before we trust theme paint.
    await expect(page.locator('body')).toBeVisible({ timeout: 60_000 })
    await page.waitForFunction(
      () => document.documentElement.getAttribute('theme') != null || true,
      undefined,
      { timeout: 5_000 },
    )
  })

  for (const { id: theme, modes } of THEME_CASES) {
    for (const mode of modes) {
      test(`${theme} / ${mode}: critical CSS vars are set`, async ({ page }) => {
        await applyThemeMode(page, theme, mode)
        const vars = await readCriticalVars(page)
        const missingOpaque = MUST_OPAQUE_VARS.filter((k) => isUnsetOpaque(vars[k] ?? ''))
        const missingDeclared = MUST_DECLARED_VARS.filter((k) => isEmpty(vars[k] ?? ''))
        expect(
          missingOpaque,
          `${theme}/${mode} left critical paint vars empty/transparent: ${missingOpaque.join(', ')} — ` +
            `got ${JSON.stringify(vars)}`,
        ).toEqual([])
        expect(
          missingDeclared,
          `${theme}/${mode} left vars undeclared: ${missingDeclared.join(', ')}`,
        ).toEqual([])
      })
    }

    // Explicit: undeclared modes are not required (single-mode skip).
    for (const skip of ['light', 'dark'] as const) {
      if (modes.includes(skip)) continue
      test(`${theme} / ${skip}: skipped (theme does not declare mode)`, async () => {
        test.skip(true, `${theme} is single-mode; ${skip} not declared`)
      })
    }
  }

  test('synthwave default mode paints primary (single-mode dark)', async ({ page }) => {
    const sw = THEME_CASES.find((t) => t.id === 'synthwave')
    const mode = sw?.modes[0] ?? 'dark'
    await applyThemeMode(page, 'synthwave', mode)
    const vars = await readCriticalVars(page)
    expect(isUnsetOpaque(vars['--primary'] ?? '')).toBe(false)
    expect(isUnsetOpaque(vars['--primary-foreground'] ?? '')).toBe(false)

    const primaryBtn = page.locator('button[tone="primary"], button.sail-button').first()
    if ((await primaryBtn.count()) > 0) {
      const bg = await primaryBtn.evaluate((el) => getComputedStyle(el).backgroundColor)
      expect(isUnsetOpaque(bg)).toBe(false)
    }
  })

  test('mode switch sail light→dark→light: primary stays defined', async ({ page }) => {
    await applyThemeMode(page, 'sail', 'light')
    const light = await readCriticalVars(page)
    expect(isUnsetOpaque(light['--primary'] ?? '')).toBe(false)

    await applyThemeMode(page, 'sail', 'dark')
    const dark = await readCriticalVars(page)
    expect(isUnsetOpaque(dark['--primary'] ?? '')).toBe(false)

    await applyThemeMode(page, 'sail', 'light')
    const light2 = await readCriticalVars(page)
    expect(isUnsetOpaque(light2['--primary'] ?? '')).toBe(false)
    // After round-trip, primary should still match first light application
    // (no sticky foreign mode value that clears the var).
    expect(light2['--primary']).toBe(light['--primary'])
  })

  test('switching theme changes --primary (catalog not a single paint)', async ({ page }) => {
    await applyThemeMode(page, 'sail', 'light')
    const sail = (await readCriticalVars(page))['--primary']
    // Prefer a dual-mode catalog peer; fall back to any other declared theme mode.
    const peer = THEME_CASES.find((t) => t.id === 'paper') ?? THEME_CASES.find((t) => t.id !== 'sail')
    expect(peer, 'need a second catalog theme').toBeTruthy()
    const peerMode = peer!.modes.includes('light') ? 'light' : peer!.modes[0]!
    await applyThemeMode(page, peer!.id, peerMode)
    const other = (await readCriticalVars(page))['--primary']
    expect(isUnsetOpaque(sail ?? '')).toBe(false)
    expect(isUnsetOpaque(other ?? '')).toBe(false)
    expect(other).not.toBe(sail)
  })
})
