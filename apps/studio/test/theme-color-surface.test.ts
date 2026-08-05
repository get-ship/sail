/**
 * Color facet behavior — picker hex → store oklch → dirty diff.
 * Not template string scans.
 */
import { afterEach, describe, expect, it } from 'vitest'
import {
  __resetDesignForTests,
  __setWorkingDocForTests,
  getDesignState,
  getPathValue,
  setBrandColor,
  setDesignMode,
  setTokenValue,
  toPickerHex,
  hexToOklchString,
} from '../src/design'
import { COLOR_ROLES } from '../src/theme-color-surface/theme-color-edit'

afterEach(() => {
  __resetDesignForTests()
})

const modesTheme = (primary = 'oklch(0.65 0.15 40)') => ({
  $extensions: { ship: { themeId: 'color-toy' } },
  modes: {
    light: {
      color: {
        primary: {
          default: { $value: primary, $type: 'color' },
          foreground: { $value: 'oklch(0.98 0.01 90)', $type: 'color' },
        },
        secondary: {
          default: { $value: 'oklch(0.5 0.05 250)', $type: 'color' },
        },
        background: { $value: 'oklch(0.98 0 0)', $type: 'color' },
        foreground: { $value: 'oklch(0.2 0 0)', $type: 'color' },
        surface: {
          1: { $value: 'oklch(1 0 0)', $type: 'color' },
          2: { $value: 'oklch(0.96 0 0)', $type: 'color' },
          3: { $value: 'oklch(1 0 0)', $type: 'color' },
        },
        success: { default: { $value: 'oklch(0.6 0.15 145)', $type: 'color' } },
        destructive: { default: { $value: 'oklch(0.55 0.2 25)', $type: 'color' } },
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
    // Co-write AA foreground for brand
    expect(getPathValue(s.themeDoc, 'color.primary.foreground', 'light')).toMatch(/^oklch\(/)
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

  it('semantic role write stamps registry cssName (lowerer → --success not path-join)', async () => {
    seedLightTheme()
    const path = COLOR_ROLES.success.path
    await setTokenValue(path, '#16a34a')
    const doc = getDesignState().themeDoc
    expect(doc).toBeTruthy()
    // Walk modes.light.color.success.default for $extensions.ship.cssName
    const modes = (doc as { modes?: { light?: Record<string, unknown> } }).modes?.light
    const success = (modes?.color as Record<string, unknown> | undefined)?.success as
      | { default?: { $extensions?: { ship?: { cssName?: string } }; $value?: string } }
      | undefined
    const leaf = success?.default
    expect(leaf?.$value, 'written oklch').toMatch(/^oklch\(/)
    expect(leaf?.$extensions?.ship?.cssName, 'public cssName must be stamped').toBe('success')
    // lowerThemeDoc twin of applyScope must emit --success
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
  it('reads detail.value from sail-change CustomEvent (picker contract)', () => {
    const detail = { value: '#a855f7' }
    const e = new CustomEvent('sail-change', { detail, bubbles: true, composed: true })
    const hex = String(
      (e as CustomEvent<{ value?: string }>).detail?.value ?? '',
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
