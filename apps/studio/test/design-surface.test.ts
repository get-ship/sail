import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { readFileSync } from 'node:fs'
import { createCliOpsClient } from '../src/edit/ops-adapter-cli'
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
  resetInheritance,
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
      $extensions: { ship: { themeId: 'toy', density: 'default' } },
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
    expect(flattenValues(base).get('$extensions.ship.density')).toBe('default')
    await setDesignDensity('compact')
    const s = getDesignState()
    expect(s.density).toBe('compact')
    expect(densityOf(s.themeDoc)).toBe('compact')
    // Exactly the density mode write — no CSS paths
    expect(s.diffRows).toEqual([
      {
        path: '$extensions.ship.density',
        from: 'default',
        to: 'compact',
      },
    ])
    expect(s.overrideCount).toBe(1)
    expect(s.diffRows.every((r) => !r.path.endsWith('.css'))).toBe(true)
  })

  it('diffThemeDocs includes density when only density changes', () => {
    const a = { $extensions: { ship: { themeId: 't', density: 'default' } } }
    const b = { $extensions: { ship: { themeId: 't', density: 'spacious' } } }
    const rows = diffThemeDocs(a, b)
    expect(rows).toEqual([
      {
        path: '$extensions.ship.density',
        from: 'default',
        to: 'spacious',
      },
    ])
  })

  it('override + reset inheritance produces correct DTCG diff', async () => {
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
    await overrideComponentRadius('button', '9999px')
    let s = getDesignState()
    expect(s.inheritance.find((n) => n.id === 'button')?.isOverride).toBe(true)
    expect(s.diffRows.some((r) => r.path === 'component.button.radius')).toBe(true)
    await resetInheritance('button')
    s = getDesignState()
    expect(s.inheritance.find((n) => n.id === 'button')?.isOverride).toBe(false)
    expect(s.diffRows.some((r) => r.path === 'component.button.radius')).toBe(false)
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
    expect(src).toContain('data-density="compact"')
    expect(src).toContain('Primary')
    expect(src).not.toMatch(/\.css/)
  })
})

describe('ops SailTheme write + monotonicity parity', () => {
  it('ops refuses non-monotonic scale (named NonMonotonicScale)', async () => {
    const themesDir = 'apps/studio/tokens/themes'
    const path = `${themesDir}/__tool7-bad-radius.json`
    const bad = {
      $extensions: { ship: { themeId: 'tool7-bad-radius' } },
      radius: {
        sm: { $value: '12px', $type: 'dimension' },
        md: { $value: '4px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '14px', $type: 'dimension' },
        $extensions: { ordered: ['sm', 'md', 'lg', 'xl'] },
      },
    }
    await expect(
      cli.sailThemeWrite!({
        path,
        content: stringifyTheme(bad),
      }),
    ).rejects.toThrow(/NonMonotonicScale/)
  }, 60_000)

  it('ops accepts monotonic radius write then read roundtrip', async () => {
    const path = 'apps/studio/tokens/themes/__tool7-good-radius.json'
    const good = {
      $extensions: { ship: { themeId: 'tool7-good-radius' } },
      radius: {
        sm: { $value: '4px', $type: 'dimension' },
        md: { $value: '6px', $type: 'dimension' },
        lg: { $value: '10px', $type: 'dimension' },
        xl: { $value: '16px', $type: 'dimension' },
        $extensions: { ordered: ['sm', 'md', 'lg', 'xl'] },
      },
    }
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

  it('SailTheme/List catalog has ship', async () => {
    const listed = await cli.sailThemeList!({
      path: 'packages/sail/tokens/model/themes',
    })
    expect(listed.files ?? []).toContain('ship.json')
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

  it('design route mounts surface; chrome keeps loadThemes', () => {
    const leaf = readFileSync(join(REPO, 'apps/studio/routes/design/page.sail'), 'utf8')
    const chrome = readFileSync(
      join(REPO, 'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail'),
      'utf8',
    )
    const shellVm = readFileSync(join(REPO, 'apps/studio/src/shell/shell-page-vm.ts'), 'utf8')
    expect(leaf).toMatch(/<design-surface\b/)
    expect(shellVm).toMatch(/from ['"]@ship\/studio\/design['"]/)
    expect(shellVm).toMatch(/SailTheme|loadThemes|setDesignOpsClient/)
    expect(chrome).not.toMatch(/data-design-surface/)
    expect(chrome).not.toMatch(/data-decisions-rail/)
    expect(chrome).not.toMatch(/data-specimen-canvas/)
  })

  it('design-surface + inspector are ban-clean with workbench markers', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/design-surface/design-surface.sail'),
      'utf8',
    )
    const inspector = readFileSync(
      join(REPO, 'apps/studio/src/design-inspector/design-inspector.sail'),
      'utf8',
    )
    const surfaceVm = readFileSync(
      join(REPO, 'apps/studio/src/design-surface/design-surface-vm.ts'),
      'utf8',
    )
    const inspectorVm = readFileSync(
      join(REPO, 'apps/studio/src/design-inspector/design-inspector-vm.ts'),
      'utf8',
    )
    expect(surface).toMatch(/data-design-surface/)
    expect(surface).toMatch(/data-group-rail|data-decisions-rail/)
    expect(surface).toMatch(/data-decision-bar/)
    expect(surface).toMatch(/data-decision=/)
    expect(surface).toMatch(/Brand color/)
    expect(surface).toMatch(/Spacing density/)
    expect(surface).toMatch(/data-token-list|data-token-editor/)
    expect(surface).toMatch(/sail-color-picker/)
    expect(surface).toMatch(/data-brand-ramp|data-brand-panel/)
    expect(surface).toMatch(/data-save-review|Confirm save/)
    expect(inspector).toMatch(/data-specimen-live|data-specimen-canvas/)
    expect(inspector).toMatch(/data-radius-sliders/)
    expect(inspector).toMatch(/data-blast-radius/)
    expect(inspector).toMatch(/data-inheritance/)
    expect(inspector).toMatch(/base → theme → override/)
    // No jargon front-door labels
    expect(surface).not.toMatch(/>DECIDE</)
    expect(surface).not.toMatch(/Brand hue/)
    expect(surface).not.toMatch(/hue-rotate|DTCG/)
    // User copy — no pair-session notes
    expect(surface).not.toMatch(/Corners follow the family/)
    expect(inspector).not.toMatch(/refused on write/)
    expect(inspector).not.toMatch(/monotonic sm/)
    for (const vm of [surfaceVm, inspectorVm]) {
      expect(vm).toMatch(/from ['"]@ship\/studio\/design['"]/)
      expect(vm).toMatch(/subscribeDesign/)
    }
    // Surface does not dual-boot loadThemes on mount; retry may call it.
    const mountBody =
      surfaceVm.match(/mountAmbient\s*\(\s*\(\)\s*=>\s*\{([\s\S]*?)\n  \}\)/)?.[1] ?? ''
    expect(mountBody).not.toMatch(/loadThemes/)
    expect(surfaceVm).toMatch(/onRetryDesign/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    expect(inspector).not.toMatch(/<div\b/)
    expect(inspector).not.toMatch(/<span\b/)
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
    const shellVm = readFileSync(join(REPO, 'apps/studio/src/shell/shell-page-vm.ts'), 'utf8')
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

  it('design surface error strip has Retry', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/design-surface/design-surface.sail'),
      'utf8',
    )
    expect(surface).toMatch(/data-design-error/)
    expect(surface).toMatch(/data-retry-design/)
    expect(surface).toMatch(/Retry/)
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
    __setThemePathForTests('apps/studio/tokens/themes/save-gate.json')
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
  it('previewInApp posts theme-preview, applies density attrs, emits status', () => {
    const base = {
      $extensions: { ship: { themeId: 'ship', density: 'compact' } },
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
      expect(document.documentElement.getAttribute('data-density')).toBe('compact')
      expect(document.documentElement.getAttribute('data-theme')).toBe('ship')
      expect(
        posted.some(
          (m) =>
            m &&
            typeof m === 'object' &&
            (m as { type?: string; channel?: string }).type === 'theme-preview' &&
            (m as { channel?: string }).channel === 'sail-studio',
        ),
      ).toBe(true)
      expect(
        getComputedStyle(document.documentElement).getPropertyValue('--radius-lg').trim(),
      ).toBe('10px')
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
    expect(listed.some((r) => r.path === '$extensions.ship.density')).toBe(true)
  })
})
