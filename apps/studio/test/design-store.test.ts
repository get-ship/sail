/**
 * @vitest-environment happy-dom
 */
import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { readFileSync } from 'node:fs'
import { createCliOpsClient } from './helpers/ops-adapter-cli'
import {
  __resetDesignForTests,
  __setWorkingDocForTests,
  applyRadiusToDoc,
  defaultRadiusFamily,
  densityOf,
  diffThemeDocs,
  extractRadius,
  flattenValues,
  getDesignState,
  isMonotonic,
  listTokenRows,
  loadThemes,
  overrideComponentRadius,
  parseThemeJson,
  previewInApp,
  setDesignDensity,
  setDesignOpsClient,
  setDesignProjectRoot,
  setOverridesOnly,
  setRadiusStep,
  setScaleStep,
  setScaleTogether,
  setTokenSearch,
  setTypeRole,
  specimenSource,
  stringifyTheme,
  uiTokenRows,
  validateMonotonic,
  type DesignOpsClient,
} from '../src/design'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')

let cli: ReturnType<typeof createCliOpsClient>

beforeAll(() => {
  cli = createCliOpsClient(REPO)
}, 120_000)

afterEach(() => {
  __resetDesignForTests()
})

describe('scale monotonicity (client)', () => {
  it('accepts non-decreasing radius steps', () => {
    const f = defaultRadiusFamily()
    expect(isMonotonic(f.steps, f.ordered)).toBe(true)
    expect(validateMonotonic(f.steps, f.ordered)).toBeNull()
  })

  it('clamps decreasing steps with plain-language note (no silent clamp)', () => {
    const f = defaultRadiusFamily(false)
    const { family, clampNote } = setScaleStep(f, 'md', 2) // sm=4, md=2 → clamp to sm
    expect(clampNote).toMatch(/can't go below|adjusted/i)
    expect(family.steps.find((s) => s.key === 'md')?.valuePx).toBe(4)
    expect(isMonotonic(family.steps, family.ordered)).toBe(true)
  })

  it('scale-together keeps family monotonic after drag', () => {
    const f = defaultRadiusFamily(true)
    const { family, error } = setScaleStep(f, 'lg', 20)
    expect(error).toBeNull()
    expect(isMonotonic(family.steps, family.ordered)).toBe(true)
    expect(family.steps.find((s) => s.key === 'lg')?.valuePx).toBe(20)
  })
})

describe('theme doc density + inheritance + diff', () => {
  it('density write only appears when changed (no CSS paths)', async () => {
    const base = {
      $extensions: { sail: { themeId: 'toy', density: 'default' } },
      radius: {
        sm: { $value: '4px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '14px', $type: 'dimension' },
        $extensions: { ordered: ['sm', 'md', 'lg', 'xl'] },
      },
    }
    __setWorkingDocForTests(base)
    expect(getDesignState().diffRows.length).toBe(0)
    // flattenValues surfaces ship density for export
    expect(flattenValues(base).get('$extensions.sail.density')).toBe('default')
    await setDesignDensity('compact')
    const s = getDesignState()
    expect(s.density).toBe('compact')
    expect(densityOf(s.themeDoc)).toBe('compact')
    // Exactly the density mode write — no CSS paths
    expect(s.diffRows).toEqual([
      {
        path: '$extensions.sail.density',
        from: 'default',
        to: 'compact',
      },
    ])
    expect(s.overrideCount).toBe(1)
    expect(s.diffRows.every((r) => !r.path.endsWith('.css'))).toBe(true)
  })

  it('diffThemeDocs includes density when only density changes', () => {
    const a = { $extensions: { sail: { themeId: 't', density: 'default' } } }
    const b = { $extensions: { sail: { themeId: 't', density: 'spacious' } } }
    const rows = diffThemeDocs(a, b)
    expect(rows).toEqual([
      {
        path: '$extensions.sail.density',
        from: 'default',
        to: 'spacious',
      },
    ])
  })

  it('override + reset inheritance produces correct DTCG diff', async () => {
    const { setTokenValue, clearTokenValue } = await import('../src/design')
    const base = {
      $extensions: { ship: { themeId: 'toy' } },
      radius: {
        sm: { $value: '4px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '14px', $type: 'dimension' },
      },
    }
    __setWorkingDocForTests(base)
    await setTokenValue('color.card.default', '#22c55e')
    let s = getDesignState()
    expect(s.inheritance.find((n) => n.id === 'theme')).toBeTruthy()
    expect(s.diffRows.some((r) => r.path.includes('color.card'))).toBe(true)
    await clearTokenValue('color.card.default')
    s = getDesignState()
    expect(s.diffRows.some((r) => r.path.includes('color.card'))).toBe(false)
  })

  it('applyRadiusToDoc writes ordered extension', () => {
    const doc = applyRadiusToDoc({}, defaultRadiusFamily())
    const radius = doc.radius as { $extensions?: { ordered?: string[] } }
    expect(radius.$extensions?.ordered).toEqual(['sm', 'md', 'lg', 'xl'])
  })
})

describe('specimen source', () => {
  it('embeds density attribute for compact', () => {
    const src = specimenSource({ density: 'compact' })
    expect(src).toContain('density="compact"')
    expect(src).toContain('Primary')
    expect(src).not.toMatch(/\.css/)
  })
})

const GATE_COLOR = {
  background: { $value: 'oklch(0.98 0 0)', $type: 'color' },
  foreground: { $value: 'oklch(0.20 0 0)', $type: 'color' },
  primary: {
    default: { $value: 'oklch(0.45 0.12 250)', $type: 'color' },
    foreground: { $value: 'oklch(0.98 0 0)', $type: 'color' },
  },
  muted: {
    default: { $value: 'oklch(0.92 0 0)', $type: 'color' },
    foreground: { $value: 'oklch(0.35 0 0)', $type: 'color' },
  },
  brandAccent: {
    default: { $value: 'oklch(0.45 0.16 300)', $type: 'color' },
    foreground: { $value: 'oklch(0.98 0 0)', $type: 'color' },
  },
}

function radiusWriteDoc(
  themeId: string,
  radius: Record<string, unknown>,
): Record<string, unknown> {
  return {
    $extensions: { sail: { themeId, defaultMode: 'light', layer: 'theme' } },
    modes: {
      light: {
        color: GATE_COLOR,
        radius,
      },
    },
  }
}

describe('ops SailTheme write + monotonicity parity', () => {
  it('ops refuses non-monotonic scale (named NonMonotonicScale)', async () => {
    const themesDir = 'apps/studio/themes'
    const path = `${themesDir}/__tool7-bad-radius.json`
    const bad = radiusWriteDoc('tool7-bad-radius', {
      sm: { $value: '12px', $type: 'dimension' },
      md: { $value: '4px', $type: 'dimension' },
      lg: { $value: '10px', $type: 'dimension' },
      xl: { $value: '14px', $type: 'dimension' },
      $extensions: { ordered: ['sm', 'md', 'lg', 'xl'] },
    })
    await expect(
      cli.sailThemeWrite!({
        path,
        content: stringifyTheme(bad),
      }),
    ).rejects.toThrow(/NonMonotonicScale/)
  }, 60_000)

  it('ops accepts monotonic radius write then read roundtrip', async () => {
    const path = 'apps/studio/themes/__tool7-good-radius.json'
    const good = radiusWriteDoc('tool7-good-radius', {
      sm: { $value: '4px', $type: 'dimension' },
      md: { $value: '6px', $type: 'dimension' },
      lg: { $value: '10px', $type: 'dimension' },
      xl: { $value: '16px', $type: 'dimension' },
      $extensions: { ordered: ['sm', 'md', 'lg', 'xl'] },
    })
    try {
      const w = await cli.sailThemeWrite!({
        path,
        content: stringifyTheme(good),
      })
      expect(w.ok).toBe(true)
      const r = await cli.sailThemeRead!({ path })
      const doc = parseThemeJson(r.content)
      const fam = extractRadius(doc)
      expect(isMonotonic(fam.steps, fam.ordered)).toBe(true)
      expect(fam.steps.find((s) => s.key === 'xl')?.raw).toBe('16px')
    } finally {
      // cleanup via write empty delete — best-effort unlink
      try {
        const { unlinkSync } = await import('node:fs')
        unlinkSync(join(REPO, path))
      } catch {
        /* */
      }
    }
  }, 60_000)

  it('design store clamps non-monotonic radius drag with a note (writes stay mono)', async () => {
    setDesignOpsClient(cli as DesignOpsClient)
    setDesignProjectRoot('apps/studio')
    const bad = {
      $extensions: { ship: { themeId: 'store-bad' } },
      radius: {
        sm: { $value: '20px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '14px', $type: 'dimension' },
        $extensions: { ordered: ['sm', 'md', 'lg', 'xl'] },
      },
    }
    __setWorkingDocForTests(bad)
    // Force independent steps — drag md below sm → clamp + explanation
    setScaleTogether(false)
    await setRadiusStep('md', 2)
    const s = getDesignState()
    expect(s.lastClampNote).toMatch(/adjusted|can't/i)
    expect(validateMonotonic(s.radius.steps, s.radius.ordered)).toBeNull()
  }, 30_000)

  it('TokenUsage blast radius for radius-lg', async () => {
    const r = await cli.sailInspectTokenUsage!({ token: 'radius-lg' })
    expect(r.components ?? 0).toBeGreaterThan(0)
    expect(Array.isArray(r.sites)).toBe(true)
  }, 60_000)

  it('SailTheme/List catalog has sail (not ship/contrast)', async () => {
    const listed = await cli.sailThemeList!({
      path: 'packages/sail/tokens/model/themes',
    })
    const files = listed.files ?? []
    expect(files).toContain('sail.json')
    expect(files).not.toContain('ship.json')
    expect(files).not.toContain('contrast.json')
  }, 30_000)
})

describe('design surface source gates', () => {
  it('design modules never read CSS for token values', () => {
    const root = join(REPO, 'apps/studio/src/design')
    const files = ['store.ts', 'theme-doc.ts', 'scale.ts', 'specimen.ts', 'types.ts', 'index.ts']
    for (const f of files) {
      const t = readFileSync(join(root, f), 'utf8')
      // No CSS file reads / fetch of themes.css
      expect(t).not.toMatch(/readFileSync\([^)]*\.css/)
      expect(t).not.toMatch(/themes\.css|tokens\.css/)
      expect(t).not.toMatch(/getComputedStyle/)
    }
  })

  it('design route redirects to Themes; chrome keeps loadThemes', () => {
    const leaf = readFileSync(join(REPO, 'apps/studio/routes/design/page.sail'), 'utf8')
    const chrome = readFileSync(
      join(REPO, 'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail'),
      'utf8',
    )
    const shellVm = readFileSync(join(REPO, 'apps/studio/src/compositions/shell/shell-page-vm.ts'), 'utf8')
    expect(leaf).toMatch(/navigate\(`\$\{base\}\/themes`|\/themes/)
    expect(leaf).toMatch(/data-design-redirect|Opening Themes/)
    expect(shellVm).toMatch(/from ['"]@ship\/studio\/design['"]|from ['"]\.\.\/\.\.\/design/)
    expect(shellVm).toMatch(/SailTheme|loadThemes|setDesignOpsClient/)
  })

  it('Themes: no permanent Used by chrome; save confirm stays simple', () => {
    const chrome = readFileSync(
      join(REPO, 'apps/studio/src/theme-facet-chrome/theme-facet-chrome.sail'),
      'utf8',
    )
    const session = readFileSync(
      join(REPO, 'apps/studio/src/compositions/theme-session/theme-session.ts'),
      'utf8',
    )
    const actions = readFileSync(
      join(REPO, 'apps/studio/src/compositions/theme-session/theme-session-actions.ts'),
      'utf8',
    )
    const color = readFileSync(
      join(REPO, 'apps/studio/src/theme-color-surface/theme-color-surface.sail'),
      'utf8',
    )
    const shape = readFileSync(
      join(REPO, 'apps/studio/src/theme-shape-surface/theme-shape-surface.sail'),
      'utf8',
    )
    expect(chrome).not.toMatch(/used-by/)
    expect(chrome).not.toMatch(/used-by-band/)
    expect(chrome).toMatch(/saveButtonLabel|Confirm save/)
    expect(session).toMatch(/usedByLabel/)
    expect(session).toMatch(/saveReviewOpen/)
    expect(session).toMatch(/refreshBlast|refreshUsedBy/)
    expect(actions).toMatch(/requestSaveTheme/)
    expect(color).not.toMatch(/used-by/)
    expect(shape).not.toMatch(/used-by/)
    expect(chrome).not.toMatch(/blast radius|blast-radius/i)
    expect(color).not.toMatch(/blast radius|used-by-token/i)
    expect(shape).not.toMatch(/blast radius|used-by-token/i)
    const themesVm = readFileSync(
      join(REPO, 'apps/studio/src/compositions/themes-surface/themes-surface-vm.ts'),
      'utf8',
    )
    const nav = readFileSync(
      join(REPO, 'apps/studio/src/compositions/theme-session/theme-nav.ts'),
      'utf8',
    )
    expect(chrome).toMatch(/theme-leave-dialog/)
  })

  it('usage refresh ignores stale async settles (blastGen)', async () => {
    const {
      refreshBlast,
      refreshBlastAggregate,
      setDesignOpsClient,
      getDesignState,
    } = await import('../src/design')
    let resolveSlow: (() => void) | null = null
    const slow = new Promise<void>((r) => {
      resolveSlow = r
    })
    setDesignOpsClient({
      sailThemeList: async () => ({ files: [] }),
      sailThemeRead: async () => ({ content: '{}' }),
      sailThemeWrite: async () => ({ ok: true }),
      sailThemeFork: async () => ({ ok: true }),
      sailInspectTokenUsage: async ({ token }: { token: string }) => {
        if (token === 'primary') {
          await slow
          return {
            components: 99,
            componentIds: ['stale-a', 'stale-b'],
            sites: [{ file: 'stale.sail' }],
          }
        }
        return {
          components: 3,
          componentIds: ['fresh-a', 'fresh-b', 'fresh-c'],
          sites: [{ file: 'a.sail' }, { file: 'b.sail' }],
        }
      },
    })
    const stale = refreshBlast('primary')
    await refreshBlastAggregate(['radius-lg'], 2)
    expect(getDesignState().blast?.components).toBe(3)
    expect(getDesignState().blast?.label).toMatch(/Save updates 3 components across 2 files/)
    resolveSlow?.()
    await stale
    // Stale primary settle must not overwrite fresher aggregate.
    expect(getDesignState().blast?.components).toBe(3)
    expect(getDesignState().blast?.label).toMatch(/Save updates 3 components across 2 files/)
    expect(getDesignState().blast?.componentIds).toEqual(['fresh-a', 'fresh-b', 'fresh-c'])
  })

  it('token groups inventory is full surface (~400 leaves)', async () => {
    const { listTokenGroups, tokenStats, humanLabel, colorSummary } = await import(
      '../src/design/token-groups'
    )
    const stats = tokenStats()
    expect(stats.tokenCount).toBeGreaterThan(300)
    expect(stats.groupCount).toBeGreaterThan(5)
    const groups = listTokenGroups()
    expect(groups.some((g) => g.id === 'color')).toBe(true)
    expect(groups.some((g) => g.id === 'radius')).toBe(true)
    expect(groups.some((g) => g.id === 'space')).toBe(true)
    expect(groups.some((g) => g.id === 'size')).toBe(true)
    expect(groups.some((g) => g.id === 'type')).toBe(true)
    expect(groups.some((g) => g.id === 'elevation')).toBe(true)
    expect(humanLabel('radius.lg')).toMatch(/Large/i)
    expect(colorSummary('oklch(0.61 0.23 303)', 'accent')).not.toMatch(/^oklch/)
  })

  it('search spans all groups; diff/copy use human labels', async () => {
    const {
      __setWorkingDocForTests,
      setTokenGroup,
      setTokenSearch,
      uiGroupTokens,
      formatDiffLine,
      exportDiffText,
      setDesignDensity,
      getDesignState,
    } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 't', density: 'default' } },
      radius: {
        sm: { $value: '4px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '14px', $type: 'dimension' },
      },
    })
    setTokenGroup('color')
    setTokenSearch('radius')
    const hits = uiGroupTokens()
    expect(hits.length).toBeGreaterThan(0)
    expect(hits.some((r) => r.groupId === 'radius' || r.path.startsWith('radius'))).toBe(true)
    setTokenSearch('')
    await setDesignDensity('compact')
    const line = formatDiffLine(getDesignState().diffRows[0]!)
    expect(line).not.toMatch(/\$extensions/)
    expect(line.toLowerCase()).toMatch(/density/)
    expect(line).not.toMatch(/^oklch/)
    const exported = exportDiffText()
    expect(exported).not.toMatch(/\$extensions\.ship\.density/)
    expect(exported.toLowerCase()).toMatch(/density/)
  })

  it('chrome boot loads themes and icon glyphs (no activity-signal rail)', () => {
    const shellVm = readFileSync(join(REPO, 'apps/studio/src/compositions/shell/shell-page-vm.ts'), 'utf8')
    expect(shellVm).toMatch(/loadThemes/)
    expect(shellVm).toMatch(/loadIconGlyphs/)
    expect(shellVm).not.toMatch(/setActivity/)
  })

  it('loadThemes without client is neutral empty (not error)', async () => {
    setDesignOpsClient(null)
    await loadThemes()
    expect(getDesignState().error).toBeNull()
    expect(getDesignState().loading).toBe(false)
  })

  it('formatUsedByLabel product copy (never blast)', async () => {
    const { formatUsedByLabel } = await import('../src/design')
    expect(formatUsedByLabel({ components: 0, files: 0 })).toBe('Not used in this project yet')
    expect(formatUsedByLabel({ components: 12, files: 4 })).toBe('Used by · 12 components')
    expect(formatUsedByLabel({ components: 1, files: 1 })).toBe('Used by · 1 component')
    expect(formatUsedByLabel({ components: 12, files: 4, dirtyCount: 3 })).toBe(
      'Save updates 12 components across 4 files',
    )
    expect(formatUsedByLabel({ components: 0, files: 0, offline: true })).toBe(
      'Usage unavailable offline',
    )
    expect(formatUsedByLabel({ components: 12, files: 4 })).not.toMatch(/blast/i)
  })
})

describe('type role token write', () => {
  it('setTypeRole patches typography font tokens', async () => {
    const base = { $extensions: { ship: { themeId: 'ty' } } }
    __setWorkingDocForTests(base)
    await setTypeRole('display', 'Fraunces')
    const s = getDesignState()
    expect(s.typeRoles.display).toBe('Fraunces')
    expect(s.diffRows.some((r) => r.path.includes('typography.font'))).toBe(true)
  })
})

describe('brand color + mode-aware paths + decisions', () => {
  it('setBrandColor writes oklch primary under modes when theme is modes-in-file', async () => {
    const { setBrandColor, getPathValue, uiDecisionCards, deriveBrandRamp, brandContrastFlags } =
      await import('../src/design')
    const base = {
      $extensions: { ship: { themeId: 'brand-toy' } },
      modes: {
        light: {
          color: {
            primary: {
              default: { $value: 'oklch(0.6 0.15 40)', $type: 'color' },
              foreground: { $value: 'oklch(0.98 0.01 90)', $type: 'color' },
            },
            background: { default: { $value: 'oklch(0.98 0 0)', $type: 'color' } },
            foreground: { default: { $value: 'oklch(0.2 0 0)', $type: 'color' } },
          },
        },
      },
    }
    __setWorkingDocForTests(base)
    await setBrandColor('#3b82f6')
    const s = getDesignState()
    const written = getPathValue(s.themeDoc, 'color.primary.default', 'light')
    expect(written).toMatch(/^oklch\(/)
    expect(s.diffRows.some((r) => r.path.includes('color.primary'))).toBe(true)
    // Co-writes primary-foreground for AA
    expect(getPathValue(s.themeDoc, 'color.primary.foreground', 'light')).toMatch(/^oklch\(/)
    const cards = uiDecisionCards(s)
    expect(cards).toHaveLength(5)
    expect(cards.map((c) => c.label)).toEqual([
      'Brand color',
      'Type',
      'Spacing density',
      'Corners',
      'Icons',
    ])
    const ramp = deriveBrandRamp(written!)
    expect(ramp).toHaveLength(5)
    expect(ramp[0]!.value).toMatch(/^oklch\(/)
    const flags = brandContrastFlags({
      primary: written!,
      primaryFg: 'oklch(0.98 0.01 90)',
      background: 'oklch(0.98 0 0)',
      foreground: 'oklch(0.2 0 0)',
    })
    expect(flags.some((f) => f.id === 'fg-on-primary')).toBe(true)
  })

  it('mode-aware getPathValue reads modes.dark primary', async () => {
    const { getPathValue } = await import('../src/design')
    const doc = {
      modes: {
        dark: {
          color: {
            primary: { default: { $value: 'oklch(0.72 0.17 38)', $type: 'color' } },
          },
        },
      },
    }
    expect(getPathValue(doc, 'color.primary.default', 'dark')).toBe('oklch(0.72 0.17 38)')
    // Read may fall back across modes
    expect(getPathValue(doc, 'color.primary.default', 'light')).toBe('oklch(0.72 0.17 38)')
  })

  it('setBrandColor while light does not clobber dark-only primary', async () => {
    const { setBrandColor, setDesignMode, getPathValue } = await import('../src/design')
    const base = {
      $extensions: { ship: { themeId: 'asymm' } },
      modes: {
        dark: {
          color: {
            primary: { default: { $value: 'oklch(0.72 0.17 38)', $type: 'color' } },
          },
        },
      },
    }
    __setWorkingDocForTests(base)
    setDesignMode('light')
    await setBrandColor('#22c55e')
    const s = getDesignState()
    const light = getPathValue(s.themeDoc, 'color.primary.default', 'light')
    const darkLeaf = (s.themeDoc as { modes?: { dark?: { color?: { primary?: { default?: { $value?: string } } } } } })
      ?.modes?.dark?.color?.primary?.default?.$value
    expect(light).toMatch(/^oklch\(/)
    expect(darkLeaf).toBe('oklch(0.72 0.17 38)')
    // Physical write target is modes.light, not dark
    const lightLeaf = (s.themeDoc as { modes?: { light?: { color?: { primary?: { default?: { $value?: string } } } } } })
      ?.modes?.light?.color?.primary?.default?.$value
    expect(lightLeaf).toBe(light)
  })

  it('radius step writes modes.{mode}.radius on modes-only themes (not root)', async () => {
    const { setRadiusStep, setDesignMode } = await import('../src/design')
    const base = {
      $extensions: { ship: { themeId: 'rad-modes' } },
      modes: {
        light: {
          radius: {
            sm: { $value: '4px', $type: 'dimension' },
            md: { $value: '6px', $type: 'dimension' },
            lg: { $value: '10px', $type: 'dimension' },
            xl: { $value: '14px', $type: 'dimension' },
            $extensions: { ordered: ['sm', 'md', 'lg', 'xl'] },
          },
        },
      },
    }
    __setWorkingDocForTests(base)
    setDesignMode('light')
    await setRadiusStep('lg', 18)
    const s = getDesignState()
    const doc = s.themeDoc as {
      radius?: unknown
      modes?: { light?: { radius?: { lg?: { $value?: string } } } }
    }
    expect(doc.radius).toBeUndefined()
    expect(doc.modes?.light?.radius?.lg?.$value).toBe('18px')
  })

  it('type roles extract heading.font and write under active mode', async () => {
    const { extractTypeRoles, setTypeRole, setDesignMode } = await import('../src/design')
    const base = {
      $extensions: { ship: { themeId: 'type-toy' } },
      modes: {
        light: {
          typography: {
            heading: { font: { $value: 'Fraunces, ui-serif, Georgia, serif', $type: 'fontFamily' } },
            body: { font: { $value: '{font-body}', $type: 'fontFamily' } },
            font: { body: { $value: '{font-body-app}', $type: 'fontFamily' } },
          },
        },
      },
    }
    __setWorkingDocForTests(base)
    setDesignMode('light')
    const roles = extractTypeRoles(base, 'light')
    expect(roles.display).toBe('Fraunces')
    // alias alone is not a face name → default body
    expect(roles.body).toBe('DM Sans')
    await setTypeRole('display', 'Playfair Display')
    const s = getDesignState()
    const heading = (
      s.themeDoc as {
        modes?: { light?: { typography?: { heading?: { font?: { $value?: string } } } } }
      }
    )?.modes?.light?.typography?.heading?.font?.$value
    expect(heading).toMatch(/Playfair Display/)
  })

  it('type roles never treat CSS var(--font-*) cascade as a face name', async () => {
    const { extractTypeRoles } = await import('../src/design')
    // Generator emit uses var(--font-display) → app stack. Product UI must show Syne, not the CSS.
    const leaked = {
      modes: {
        light: {
          typography: {
            heading: {
              font: { $value: 'var(--font-display)', $type: 'fontFamily' },
            },
            body: {
              font: { $value: 'var(--font-body-app)', $type: 'fontFamily' },
            },
            monoFont: {
              $value: 'var(--font-mono), ui-monospace, monospace',
              $type: 'fontFamily',
            },
          },
        },
      },
    }
    const roles = extractTypeRoles(leaked, 'light')
    expect(roles.display).toBe('Syne')
    expect(roles.body).toBe('DM Sans')
    expect(roles.mono).toBe('JetBrains Mono')
    expect(roles.display).not.toMatch(/var\(/)
    expect(roles.body).not.toMatch(/var\(/)
    expect(roles.mono).not.toMatch(/var\(/)
  })

  it('requestSaveTheme two-step: review open then confirm', async () => {
    const {
      requestSaveTheme,
      setDesignOpsClient,
      setDesignDensity,
      __setThemePathForTests,
    } = await import('../src/design')
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'save-gate', density: 'default' } },
    })
    __setThemePathForTests('apps/studio/themes/save-gate.json')
    let writes = 0
    setDesignOpsClient({
      sailThemeList: async () => ({ files: [] }),
      sailThemeRead: async () => ({ content: '{}' }),
      sailThemeWrite: async () => {
        writes += 1
        return { ok: true }
      },
      sailThemeFork: async () => ({ ok: true }),
    })
    await setDesignDensity('compact')
    expect(getDesignState().dirty).toBe(true)
    const first = await requestSaveTheme()
    expect(first).toBe(false)
    expect(getDesignState().saveReviewOpen).toBe(true)
    expect(writes).toBe(0)
    const second = await requestSaveTheme()
    expect(second).toBe(true)
    expect(writes).toBe(1)
    expect(getDesignState().saveReviewOpen).toBe(false)
  })

  it('blastTokensFromDiffs maps dirty paths to usage tokens', async () => {
    const { blastTokensFromDiffs } = await import('../src/design')
    expect(blastTokensFromDiffs([{ path: 'modes.light.radius.lg' }])).toContain('radius-lg')
    expect(blastTokensFromDiffs([{ path: 'color.primary.default' }])).toContain('primary')
    expect(
      blastTokensFromDiffs([
        { path: 'modes.light.radius.md' },
        { path: 'color.primary.default' },
      ]).length,
    ).toBeGreaterThan(1)
  })
})

describe('preview in app + token filters', () => {
  it('previewInApp posts theme-preview and emits status (no documentElement CSSOM)', () => {
    const base = {
      $extensions: { sail: { themeId: 'ship', density: 'compact' } },
      radius: {
        sm: { $value: '4px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '14px', $type: 'dimension' },
      },
    }
    __setWorkingDocForTests(base)
    const posted: unknown[] = []
    const orig = window.postMessage.bind(window)
    window.postMessage = ((data: unknown, ...rest: unknown[]) => {
      posted.push(data)
      return orig(data, ...(rest as [string]))
    }) as typeof window.postMessage
    try {
      const r = previewInApp()
      expect(r.ok).toBe(true)
      expect(r.patchKeys).toContain('--radius-lg')
      // documentElement attrs/CSSOM removed — postMessage bridge carries the patch.
      expect(
        posted.some(
          (m) =>
            m &&
            typeof m === 'object' &&
            (m as { type?: string; channel?: string }).type === 'theme-preview' &&
            (m as { channel?: string }).channel === 'sail-studio',
        ),
      ).toBe(true)
      expect(getDesignState().statusNote).toMatch(/preview ship/)
      expect(getDesignState().statusNote).toMatch(/density compact/)
    } finally {
      window.postMessage = orig
    }
  })

  it('loadThemes preserves dirty working doc on catalog refresh', async () => {
    const base = {
      $extensions: { ship: { themeId: 'ship', density: 'default' } },
      radius: {
        sm: { $value: '4px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '14px', $type: 'dimension' },
      },
    }
    __setWorkingDocForTests(base)
    await setDesignDensity('compact')
    expect(getDesignState().dirty).toBe(true)
    const before = getDesignState().themeDoc
    let readCalls = 0
    const lists: string[] = []
    const mock = {
      sailThemeList: async ({ path }: { path: string }) => {
        lists.push(path)
        return { files: ['ship.json'] }
      },
      sailThemeRead: async () => {
        readCalls += 1
        throw new Error('openTheme must not run when dirty without themeId')
      },
      sailThemeWrite: async () => ({ ok: true }),
      sailThemeFork: async () => ({ ok: true }),
    } as DesignOpsClient
    setDesignOpsClient(mock)
    await loadThemes()
    expect(getDesignState().dirty).toBe(true)
    expect(getDesignState().themeDoc).toEqual(before)
    expect(getDesignState().density).toBe('compact')
    expect(lists.length).toBeGreaterThan(0)
    expect(readCalls).toBe(0)
    // Explicit themeId still attempts reopen (fork / force path); errors are humanized.
    await loadThemes({ themeId: 'ship' })
    expect(readCalls).toBeGreaterThan(0)
    expect(getDesignState().error).toMatch(/Could not open that theme/)
    expect(getDesignState().error).not.toMatch(/openTheme must not run/)
  })

  it('token search + overrides-only filter the inventory', async () => {
    const base = {
      $extensions: { ship: { themeId: 't', density: 'default' } },
      radius: {
        sm: { $value: '4px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
      },
    }
    __setWorkingDocForTests(base)
    await setDesignDensity('compact')
    await overrideComponentRadius('button', '9999px')
    setOverridesOnly(true)
    let rows = uiTokenRows()
    expect(rows.length).toBeGreaterThan(0)
    expect(rows.every((r) => r.dirty)).toBe(true)
    setTokenSearch('density')
    rows = uiTokenRows()
    expect(rows.some((r) => r.path.includes('density'))).toBe(true)
    expect(rows.every((r) => r.path.toLowerCase().includes('density') || r.value.includes('compact'))).toBe(
      true,
    )
    // pure listTokenRows
    const listed = listTokenRows(getDesignState().themeDoc, base, {
      overridesOnly: true,
    })
    expect(listed.some((r) => r.path === '$extensions.sail.density')).toBe(true)
  })
})
