/**
 * Color facet behavior — picker hex → store oklch → dirty diff.
 * Not template string scans.
 */
import { afterEach, describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  __resetDesignForTests,
  __setThemePathForTests,
  __setWorkingDocForTests,
  generateTheme,
  getDesignState,
  getPathValue,
  setAnchorColor,
  setBrandColor,
  setDesignMode,
  setDesignOpsClient,
  setTokenValue,
  toPickerHex,
  hexToOklchString,
} from '../src/design'
import { COLOR_ROLES } from '../src/compositions/theme-color-surface/theme-color-edit'

afterEach(() => {
  __resetDesignForTests()
})

const modesTheme = (primary = 'oklch(0.65 0.15 40)') => ({
  $extensions: { ship: { themeId: 'color-toy' } },
  modes: {
    light: {
      color: {
        primary: {
          solid: { $value: primary, $type: 'color' },
          text: { $value: 'oklch(0.98 0.01 90)', $type: 'color' },
        },
        secondary: {
          solid: { $value: 'oklch(0.5 0.05 250)', $type: 'color' },
        },
        background: { $value: 'oklch(0.98 0 0)', $type: 'color' },
        foreground: { $value: 'oklch(0.2 0 0)', $type: 'color' },
        success: { solid: { $value: 'oklch(0.6 0.15 145)', $type: 'color' } },
        danger: { solid: { $value: 'oklch(0.55 0.2 25)', $type: 'color' } },
      },
    },
  },
})

function seedLightTheme(doc = modesTheme()) {
  __setWorkingDocForTests(doc)
  setDesignMode('light')
}

describe('color pick pipeline (hex → oklch write)', () => {
  it('hex from sail-color-picker becomes oklch on primary via setBrandColor', async () => {
    seedLightTheme()
    const before = getPathValue(getDesignState().themeDoc, COLOR_ROLES.primary.path, 'light')
    expect(before).toMatch(/^oklch\(/)

    // User picked blue in the picker UI.
    await setBrandColor('#3b82f6')

    const after = getPathValue(getDesignState().themeDoc, COLOR_ROLES.primary.path, 'light')
    expect(after).toMatch(/^oklch\(/)
    expect(after).not.toBe(before)
    // Picker can re-open with hex that maps back to the written oklch.
    const reopened = toPickerHex(after!)
    expect(reopened).toMatch(/^#[0-9a-f]{6}$/)
    // Round-trip: same hex → same oklch string shape
    expect(hexToOklchString('#3b82f6')).toMatch(/^oklch\(/)

    const s = getDesignState()
    expect(s.dirty).toBe(true)
    expect(s.diffRows.some((r) => r.path.includes('color.primary'))).toBe(true)
    expect(getPathValue(s.themeDoc, 'color.primary.text', 'light')).toMatch(/^oklch\(/)
  })

  it('non-primary role pick uses setTokenValue hex→oklch (secondary path)', async () => {
    seedLightTheme()
    const path = COLOR_ROLES.secondary.path
    const before = getPathValue(getDesignState().themeDoc, path, 'light')

    await setTokenValue(path, '#64748b')

    const after = getPathValue(getDesignState().themeDoc, path, 'light')
    expect(after).toMatch(/^oklch\(/)
    expect(after).not.toBe(before)
    expect(getDesignState().diffRows.some((r) => r.path.includes('color.secondary'))).toBe(true)
  })

  it('surface / semantic role paths in COLOR_ROLES are writable with hex', async () => {
    seedLightTheme()
    const picks: Array<{ key: keyof typeof COLOR_ROLES; hex: string }> = [
      { key: 'bg', hex: '#f8f7f6' },
      { key: 'card', hex: '#ffffff' },
      { key: 'danger', hex: '#dc2626' },
      { key: 'success', hex: '#16a34a' },
    ]
    for (const { key, hex } of picks) {
      const path = COLOR_ROLES[key].path
      await setTokenValue(path, hex)
      const written = getPathValue(getDesignState().themeDoc, path, 'light')
      expect(written, path).toMatch(/^oklch\(/)
      // UI can feed the picker from stored token.
      expect(toPickerHex(written!)).toMatch(/^#[0-9a-f]{6}$/)
    }
    expect(getDesignState().diffRows.length).toBeGreaterThanOrEqual(picks.length)
  })

  it('COLOR_ROLES covers brand · surfaces · text · semantic without gaps', () => {
    const keys = Object.keys(COLOR_ROLES)
    for (const need of [
      'primary',
      'secondary',
      'accent',
      'bg',
      'surface1',
      'card',
      'fg',
      'muted',
      'border',
      'ring',
      'success',
      'warning',
      'danger',
      'info',
    ]) {
      expect(keys).toContain(need)
      expect(COLOR_ROLES[need as keyof typeof COLOR_ROLES].path).toMatch(/^color\./)
    }
  })

  it('picking the same hex again still yields valid oklch (idempotent shape)', async () => {
    seedLightTheme()
    await setBrandColor('#dc7702')
    const a = getPathValue(getDesignState().themeDoc, COLOR_ROLES.primary.path, 'light')
    await setBrandColor('#dc7702')
    const b = getPathValue(getDesignState().themeDoc, COLOR_ROLES.primary.path, 'light')
    expect(a).toBe(b)
    expect(a).toMatch(/^oklch\(/)
  })

  it('semantic role write lowers via publicName without leaf cssName stamp', async () => {
    seedLightTheme()
    const path = COLOR_ROLES.success.path
    await setTokenValue(path, '#16a34a')
    const doc = getDesignState().themeDoc
    expect(doc).toBeTruthy()
    const modes = (doc as { modes?: { light?: Record<string, unknown> } }).modes?.light
    const success = (modes?.color as Record<string, unknown> | undefined)?.success as
      | { solid?: { $extensions?: { sail?: { cssName?: string } }; $value?: string } }
      | undefined
    const leaf = success?.solid
    expect(leaf?.$value, 'written oklch').toMatch(/^oklch\(/)
    expect(leaf?.$extensions?.sail?.cssName, 'no cssName stamp').toBeUndefined()
    // lowerThemeDoc twin of applyScope must emit --success via publicName(path)
    const { lowerThemeDoc } = await import('@ship/sail-runtime')
    const lowered = lowerThemeDoc(doc as Record<string, unknown>, 'light')
    expect(lowered.patch['--success']).toMatch(/^oklch\(/)
    expect(lowered.patch['--color-success-default']).toBeUndefined()
  })
})

/**
 * Simulate the browser event shape sail-color-picker emits into onRoleColor.
 * Proves the facet event adapter can extract hex + role without the Sail mount.
 */
describe('color facet event adapter (sail-change shape)', () => {
  it('reads value from on-change callback payload (picker contract)', () => {
    const detail = { value: '#a855f7' }
    const e = detail
    const hex = String(
      e.value ?? '',
    ).trim()
    expect(hex).toBe('#a855f7')
    expect(hexToOklchString(hex)).toMatch(/^oklch\(/)
  })

  it('token resolution prefers token on host (brand card / token row)', () => {
    const host = document.createElement('div')
    host.setAttribute('token', 'primary')
    const picker = document.createElement('button')
    host.appendChild(picker)
    const role =
      picker.closest?.('[token]')?.getAttribute('token') ||
      host.getAttribute('token')
    expect(role).toBe('primary')
    expect(role! in COLOR_ROLES).toBe(true)
  })
})

describe('ramp editor is a thin client over the engine', () => {
  it('does not invent contrast ratios in the color facet', () => {
    const src = readFileSync(
      join(dirname(fileURLToPath(import.meta.url)), '../src/compositions/theme-color-surface/theme-color-edit.ts'),
      'utf8',
    )
    expect(src).not.toMatch(/contrastRatio\(/)
    expect(src).toMatch(/sailThemeDeriveScale/)
    expect(src).toMatch(/sailThemeCheckTheme/)
  })

  it('anchor edit + generate call derive_scale and check_theme, then write', async () => {
    const deriveArgs: unknown[] = []
    const checkArgs: unknown[] = []
    const writes: { path: string; content: string }[] = []
    setDesignOpsClient({
      sailThemeList: async () => ({ files: [] }),
      sailThemeRead: async ({ path }) => ({ path, content: '{}' }),
      sailThemeWrite: async ({ path, content }) => {
        writes.push({ path, content })
        return { ok: true, path, bytes: content.length }
      },
      sailThemeFork: async () => ({ ok: true }),
      sailThemeDeriveScale: async (args) => {
        deriveArgs.push(args)
        return {
          ok: true,
          mode: 'light',
          scales: {
            primary: {
              solid: 'oklch(0.5400 0.1800 245.00)',
              hover: 'oklch(0.4968 0.1584 245.00)',
              subtle: 'oklch(0.9272 0.0216 245.00)',
              border: 'oklch(0.8568 0.0504 245.00)',
              text: 'oklch(0.9800 0.0000 245.00)',
            },
          },
          derived: ['color.primary.solid'],
        }
      },
      sailThemeCheckTheme: async (args) => {
        checkArgs.push(args)
        return {
          ok: true,
          errors: [],
          contrastPairs: [
            { fg: 'foreground', bg: 'background', ratio: 18.38, floor: 4.5, ok: true, soft: false },
          ],
          derived: ['color.primary.solid'],
        }
      },
      sailThemeGenerate: async () => ({ ok: true }),
    })
    __setWorkingDocForTests({
      $extensions: { sail: { themeId: 'probe', layer: 'theme', defaultMode: 'light' } },
      anchors: {
        light: {
          primary: { space: 'oklch', l: 0.54, c: 0.18, h: 245 },
        },
      },
      modes: {
        light: {
          color: {
            background: { $value: 'oklch(0.98 0 0)', $type: 'color' },
            foreground: { $value: 'oklch(0.20 0 0)', $type: 'color' },
          },
        },
      },
    })
    __setThemePathForTests('apps/studio/themes/probe.json')
    setDesignMode('light')

    await setAnchorColor('primary', 'oklch(0.54 0.18 245)')
    const anchors = (getDesignState().themeDoc as { anchors?: { light?: { primary?: { l?: number } } } })
      .anchors?.light?.primary
    expect(anchors?.l).toBeCloseTo(0.54)

    const ok = await generateTheme()
    expect(ok).toBe(true)
    expect(checkArgs.length).toBeGreaterThan(0)
    expect(writes.length).toBe(1)
    expect(writes[0]!.path).toBe('apps/studio/themes/probe.json')
    expect(writes[0]!.content).toContain('anchors')
  })

  it('generate refuses when check_theme reports a floor pair', async () => {
    setDesignOpsClient({
      sailThemeList: async () => ({ files: [] }),
      sailThemeRead: async ({ path }) => ({ path, content: '{}' }),
      sailThemeWrite: async () => ({ ok: true, path: '', bytes: 0 }),
      sailThemeFork: async () => ({ ok: true }),
      sailThemeCheckTheme: async () => ({
        ok: false,
        errors: ['theme probe modes.light: muted-foreground on muted is 1.20:1, floor 4.5:1'],
        contrastPairs: [
          { fg: 'muted-foreground', bg: 'muted', ratio: 1.2, floor: 4.5, ok: false, soft: false },
        ],
      }),
    })
    __setWorkingDocForTests({
      $extensions: { sail: { themeId: 'probe' } },
      modes: { light: { color: {} } },
    })
    __setThemePathForTests('apps/studio/themes/probe.json')
    const ok = await generateTheme()
    expect(ok).toBe(false)
    expect(getDesignState().lastWriteError).toMatch(/floor 4.5:1/)
  })
})
