// @ship/tokens — public surface contract.
//
// The design substrate is tiered:
//   1. PRIMITIVE   — spacing scale, radii, typography, motion curves,
//                    z-index, icon/control/panel dimensional scales.
//                    These never change per theme.
//   2. SEMANTIC    — surface stack (--surface-{0..3}), tone ramps
//                    (--tone-*-{fg,bg,bg-solid,border,ring}), brand
//                    palette (--primary, --accent, --foreground, ...).
//                    Themes override this layer; components read from
//                    here at the HIGHEST level that fits.
//   3. COMPONENT   — panel paddings/clamp/width, control heights/stroke,
//                    edge highlights. The overlay/dialog/drawer/sheet
//                    shells + popover/menu quartet read from this tier.
//
// Every test here parses the .css files directly — the assert is on
// the SHIPPED source-of-truth text, not on a runtime computed value
// (the runtime reads var() at paint time and would just defer to the
// same files). This catches a dropped var, a typo'd name, and a
// reskin-by-token that left a tier out.
import { dirname, join, resolve } from 'node:path'
import { readdirSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

import {
  CORE_THEME_IDS,
  DEFAULT_THEME,
  DEFAULT_MODE,
  THEME_MODE_STORAGE_KEY,
  THEME_OVERRIDE_ORDER,
  THEME_ROOT_KEYS,
  THEMES,
  applyMode,
  applyTheme,
  isCoreTheme,
  isDarkTheme,
  isValidTheme,
  modeCapability,
  modesForTheme,
  paintMode,
  persistMode,
  pickerModesForTheme,
  readModePreference,
  resolveOverrideLayer,
  resolveTokenLayer,
  subscribeSystemPreference,
  validateThemeDoc,
} from '../src/themes'
import { THEME_META } from '../src/generated/themes-meta'

// Top-level file reads work fine from inside a test or describe body
// (vitest's esbuild transform evaluates `import.meta.url` lazily); at
// module init in a TS-orphan package they error with "URL must be a
// non-empty 'file:' path" because there's no tsconfig.json to anchor
// the import-meta rewrite. A small lazy helper is the boring fix.
const loadSrc = (rel: string): string => {
  // The path is resolved relative to the package's `src/` dir at the
  // call site — caller passes `tokens.css` (sibling of this module).
  return readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), rel), 'utf8')
}

/** Every `--foo` token declaration in the given CSS. */
function extractVarDecls(css: string): Set<string> {
  const out = new Set<string>()
  // Match `--name:` followed by anything to the end of the line OR a
  // semicolon. The CSS uses single-line declarations exclusively.
  // Underscores appear in fine-step space names (space-0_75, space-1_5, …).
  const re = /--([a-z0-9_-]+)\s*:/gi
  let m: RegExpExecArray | null
  while ((m = re.exec(css)) !== null) {
    out.add(m[1]!)
  }
  return out
}

/** Every `:root[theme='X']` selector. Captures the theme id. */
function extractThemeSelectors(css: string): Set<string> {
  const out = new Set<string>()
  const re = /:root\[theme=['"]([^'"]+)['"]\]/g
  let m: RegExpExecArray | null
  while ((m = re.exec(css)) !== null) {
    out.add(m[1]!)
  }
  return out
}

const PRIMITIVE_TIER: ReadonlyArray<[name: string, sample: RegExp]> = [
  // Spacing scale
  ['--space-1', /--space-1\s*:\s*4px/i],
  ['--space-2', /--space-2\s*:\s*8px/i],
  ['--space-4', /--space-4\s*:\s*16px/i],
  // Radii
  ['--radius-sm', /--radius-sm\s*:\s*4px/i],
  ['--radius-md', /--radius-md\s*:\s*6px/i],
  ['--radius-lg', /--radius-lg\s*:\s*10px/i],
  ['--radius-full', /--radius-full\s*:\s*9999px/i],
  // Motion curves
  ['--ease-out', /--ease-out\s*:\s*cubic-bezier/i],
  ['--ease-spring', /--ease-spring\s*:\s*cubic-bezier/i],
  ['--duration-fast', /--duration-fast\s*:\s*120ms/i],
  ['--duration-slow', /--duration-slow\s*:\s*280ms/i],
  // Typography scale
  ['--text-xs', /--text-xs\s*:\s*11px/i],
  ['--text-base', /--text-base\s*:\s*13px/i],
  // Z-index
  ['--z-dropdown', /--z-dropdown\s*:\s*10/i],
  ['--z-popover', /--z-popover\s*:\s*30/i],
  ['--z-dialog', /--z-dialog\s*:\s*40/i],
] as const

const SEMANTIC_TIER: ReadonlyArray<[name: string, sample: RegExp]> = [
  // Surface stack
  ['--surface-0', /--surface-0\s*:/i],
  ['--surface-1', /--surface-1\s*:/i],
  ['--surface-2', /--surface-2\s*:/i],
  ['--surface-3', /--surface-3\s*:/i],
  // Overlay channel (semantic — the spec's "an --overlay-*")
  ['--surface-overlay', /--surface-overlay\s*:/i],
  ['--backdrop', /--backdrop\s*:/i],
  // Semantic hues + soft surfaces (no tone-* ramp tokens)
  ['--primary', /--primary\s*:/i],
  ['--hover-bg', /--hover-bg\s*:/i],
  ['--success', /--success\s*:/i],
  ['--warning', /--warning\s*:/i],
  ['--destructive', /--destructive\s*:/i],
  // Brand palette
  ['--primary', /--primary\s*:/i],
  ['--foreground', /--foreground\s*:/i],
  ['--muted-foreground', /--muted-foreground\s*:/i],
] as const

const COMPONENT_TIER: ReadonlyArray<[name: string, sample: RegExp]> = [
  // Panel paddings (overlay shells: dialog/drawer/sheet/toast + popover/menu)
  ['--panel-pad-x', /--panel-pad-x\s*:/i],
  ['--panel-head-pad-y', /--panel-head-pad-y\s*:/i],
  ['--panel-body-pad-y', /--panel-body-pad-y\s*:/i],
  ['--panel-foot-pad-y', /--panel-foot-pad-y\s*:/i],
  ['--panel-close-inset', /--panel-close-inset\s*:/i],
  // Panel clamps
  ['--panel-max-w-md', /--panel-max-w-md\s*:/i],
  ['--panel-min-w-md', /--panel-min-w-md\s*:/i],
  ['--panel-rail-w', /--panel-rail-w\s*:/i],
  // Control heights + stroke
  ['--control-h-md', /--control-h-md\s*:/i],
  ['--control-stroke', /--control-stroke\s*:/i],
  // Edge highlight (filled surfaces)
  ['--edge-highlight', /--edge-highlight\s*:/i],
] as const

/** Tokens the spec calls out by name — these MUST exist in every theme.
 *  Stored as BARE names (no leading --); the regex prepends it. */
const REPRSENTATIVE_PER_THEME = [
  'surface-1',
  'surface-overlay',
  'panel-pad-x',
] as const

describe('@ship/tokens — theme catalog (THEMES export)', () => {
  it('every theme has a unique id, label, and 4-tuple swatches', () => {
    const ids = new Set<string>()
    for (const t of THEMES) {
      expect(t.id).toMatch(/^[a-z0-9][a-z0-9-]*$/)
      expect(t.label).toBeTruthy()
      expect(t.swatches).toHaveLength(4)
      expect(t.swatches.every((s) => typeof s === 'string' && s.length > 0)).toBe(true)
      expect(ids.has(t.id)).toBe(false)
      ids.add(t.id)
    }
  })

  it('THEMES is non-empty (the spec called for "the six themes resolve" — assert >= 6)', () => {
    expect(THEMES.length).toBeGreaterThanOrEqual(6)
  })

  it('isValidTheme / isDarkTheme accept every THEMES id and reject unrelated strings', () => {
    for (const t of THEMES) {
      expect(isValidTheme(t.id)).toBe(true)
      expect(isDarkTheme(t.id)).toBe(t.dark)
    }
    expect(isValidTheme('not-a-real-theme')).toBe(false)
    expect(isDarkTheme('not-a-real-theme')).toBe(false)
  })

  it('DEFAULT_THEME is a real theme id', () => {
    expect(isValidTheme(DEFAULT_THEME)).toBe(true)
  })

  // U2.1a — catalog mode honesty + Sail default theme
  it('DEFAULT_THEME is sail (Sail brand, not monorepo ship)', () => {
    expect(DEFAULT_THEME).toBe('sail')
  })

  it('CORE_THEME_IDS includes sail + neutral (ship is catalog)', () => {
    expect([...CORE_THEME_IDS].sort()).toEqual(['neutral', 'sail'].sort())
    expect(isCoreTheme('sail')).toBe(true)
    expect(isCoreTheme('neutral')).toBe(true)
    expect(isCoreTheme('ship')).toBe(false)
  })

  it('paper and brut are light-only (catalog mode honesty)', () => {
    for (const id of ['paper', 'brut'] as const) {
      expect(modesForTheme(id)).toEqual(['light'])
      const meta = THEME_META.find((t) => t.id === id)
      expect(meta?.modes).toEqual(['light'])
      expect(meta?.capability).toBe('light')
      expect(modeCapability(id)).toBe('light')
      const theme = THEMES.find((t) => t.id === id)
      expect(theme?.modes).toEqual(['light'])
      expect(theme?.capability).toBe('light')
      const picker = pickerModesForTheme(id)
      expect(picker.allowSystem).toBe(false)
      expect(picker.paintModes).toEqual(['light'])
    }
  })

  it('sail is dual-mode light+dark and is a real catalog entry', () => {
    expect(isValidTheme('sail')).toBe(true)
    expect(modesForTheme('sail')).toEqual(['light', 'dark'])
    const meta = THEME_META.find((t) => t.id === 'sail')
    expect(meta?.modes).toEqual(['light', 'dark'])
    expect(meta?.defaultMode).toBe('light')
    expect(meta?.capability).toBe('both')
    expect(modeCapability('sail')).toBe('both')
    const theme = THEMES.find((t) => t.id === 'sail')
    expect(theme?.label).toBe('Sail')
    expect(theme?.tier).toBe('core')
    expect(theme?.capability).toBe('both')
    expect(pickerModesForTheme('sail').allowSystem).toBe(true)
  })

  it('THEMES-1: every catalog theme has capability matching modes', () => {
    for (const t of THEME_META) {
      const hasL = t.modes.includes('light')
      const hasD = t.modes.includes('dark')
      const expected =
        hasL && hasD ? 'both' : hasD ? 'dark' : 'light'
      expect(t.capability, t.id).toBe(expected)
      expect(modeCapability(t.id)).toBe(expected)
      // Single-mode themes never pretend system / both.
      if (expected !== 'both') {
        expect(pickerModesForTheme(t.id).allowSystem).toBe(false)
      }
    }
  })

  it('THEMES-1: schema lock rejects unknown root keys naming valid ones', () => {
    expect(THEME_ROOT_KEYS).toContain('$extensions')
    expect(THEME_ROOT_KEYS).toContain('modes')
    const bad = {
      $extensions: { ship: { themeId: 'x', defaultMode: 'light', layer: 'theme' } },
      modes: { light: { color: {} } },
      palette: true,
    }
    const errs = validateThemeDoc(bad, 'x.json')
    expect(errs.length).toBeGreaterThan(0)
    expect(errs.some((e) => e.message.includes('palette'))).toBe(true)
    expect(
      errs.some(
        (e) =>
          e.message.includes('valid:') || e.message.includes('closest valid key'),
      ),
    ).toBe(true)
  })

  it('THEMES-1: schema lock accepts a minimal dual-mode theme', () => {
    const ok = {
      $description: 'test',
      $extensions: {
        ship: {
          themeId: 'probe',
          defaultMode: 'light',
          layer: 'theme',
          supportedModes: ['light', 'dark'],
        },
      },
      modes: {
        light: { color: {}, component: {} },
        dark: { color: {}, component: {} },
      },
    }
    expect(validateThemeDoc(ok)).toEqual([])
  })

  it('THEMES-1: override order is instance > component > global > baked', () => {
    expect([...THEME_OVERRIDE_ORDER]).toEqual([
      'instance',
      'component',
      'global',
      'baked',
    ])
    expect(resolveOverrideLayer({ baked: true })).toBe('baked')
    expect(resolveOverrideLayer({ global: true, baked: true })).toBe('global')
    expect(
      resolveOverrideLayer({ component: true, global: true, baked: true }),
    ).toBe('component')
    expect(
      resolveOverrideLayer({
        instance: true,
        component: true,
        global: true,
        baked: true,
      }),
    ).toBe('instance')
    expect(
      resolveTokenLayer({
        instance: false,
        componentOverride: true,
        globalOverride: true,
      }).layer,
    ).toBe('component')
  })

  it('every THEMES id is also a `:root[theme="id"]` selector in themes.css', () => {
    const selectors = extractThemeSelectors(loadSrc('../src/themes.css'))
    for (const t of THEMES) {
      expect(selectors.has(t.id)).toBe(true)
    }
  })

  it('applyTheme(DEFAULT_THEME) sets theme + data-mode light without .dark', () => {
    const root = document.documentElement
    root.removeAttribute('theme')
    root.removeAttribute('data-mode')
    root.classList.remove('dark')
    applyTheme(DEFAULT_THEME, root)
    expect(root.getAttribute('theme')).toBe(DEFAULT_THEME)
    expect(root.dataset.mode).toBe(DEFAULT_MODE)
    expect(root.classList.contains('dark')).toBe(false)
  })

  it('applyMode(dark) sets data-mode dark without .dark class', () => {
    const root = document.documentElement
    applyTheme('sail', root, 'light')
    applyMode('dark', root)
    expect(root.getAttribute('theme')).toBe('sail')
    expect(root.dataset.mode).toBe('dark')
    expect(root.classList.contains('dark')).toBe(false)
  })

  it('applyMode(dark) on paper clamps to light (single-mode lock)', () => {
    const root = document.documentElement
    applyTheme('paper', root, 'light')
    applyMode('dark', root)
    expect(root.getAttribute('theme')).toBe('paper')
    expect(root.dataset.mode).toBe('light')
  })

  it('never sets data-mode=system on document (paintMode resolves)', () => {
    const root = document.documentElement
    applyTheme('sail', root, 'light')
    applyMode('system', root)
    expect(root.dataset.mode).not.toBe('system')
    expect(['light', 'dark']).toContain(root.dataset.mode)
    applyTheme('console', root, 'system')
    expect(root.dataset.mode).not.toBe('system')
    expect(['light', 'dark']).toContain(root.dataset.mode)
  })

  it('paintMode maps system via prefers-color-scheme (mock)', () => {
    const original = window.matchMedia
    window.matchMedia = ((query: string) =>
      ({
        matches: query.includes('dark') ? true : false,
        media: query,
        addEventListener: () => {},
        removeEventListener: () => {},
        addListener: () => {},
        removeListener: () => {},
        dispatchEvent: () => false,
        onchange: null,
      })) as typeof window.matchMedia
    try {
      expect(paintMode('system')).toBe('dark')
      expect(paintMode('light')).toBe('light')
      expect(paintMode('dark')).toBe('dark')
      expect(paintMode(null)).toBe('light')
    } finally {
      window.matchMedia = original
    }
  })

  it('persistMode + readModePreference keep system intent', () => {
    try {
      localStorage.clear()
    } catch {
      /* */
    }
    persistMode('system')
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe('system')
    expect(readModePreference()).toBe('system')
    persistMode('dark')
    expect(readModePreference()).toBe('dark')
    persistMode('not-a-mode')
    // Unknown stored string normalizes via paintMode → light
    expect(readModePreference()).toBe('light')
  })

  it("applyMode('system') paints light|dark and stores preference system", () => {
    const root = document.documentElement
    try {
      localStorage.clear()
    } catch {
      /* */
    }
    applyTheme('sail', root, 'light')
    applyMode('system', root)
    expect(root.dataset.mode).not.toBe('system')
    expect(['light', 'dark']).toContain(root.dataset.mode)
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe('system')
    expect(readModePreference()).toBe('system')
  })

  it("applyMode('dark') stores preference dark (not only paint)", () => {
    const root = document.documentElement
    try {
      localStorage.clear()
    } catch {
      /* */
    }
    applyTheme('sail', root, 'light')
    applyMode('dark', root)
    expect(root.dataset.mode).toBe('dark')
    expect(localStorage.getItem(THEME_MODE_STORAGE_KEY)).toBe('dark')
  })

  it('subscribeSystemPreference fires only while preference is system', () => {
    try {
      localStorage.clear()
    } catch {
      /* */
    }
    type Listener = () => void
    let listener: Listener | null = null
    const original = window.matchMedia
    window.matchMedia = ((query: string) =>
      ({
        matches: false,
        media: query,
        addEventListener: (_: string, cb: EventListener) => {
          listener = cb as Listener
        },
        removeEventListener: () => {
          listener = null
        },
        addListener: (cb: Listener) => {
          listener = cb
        },
        removeListener: () => {
          listener = null
        },
        dispatchEvent: () => false,
        onchange: null,
      })) as typeof window.matchMedia

    const paints: string[] = []
    try {
      persistMode('system')
      const unsub = subscribeSystemPreference((p) => paints.push(p))
      expect(listener).toBeTypeOf('function')
      listener?.()
      expect(paints.length).toBe(1)
      expect(['light', 'dark']).toContain(paints[0])

      paints.length = 0
      persistMode('dark')
      listener?.()
      expect(paints.length).toBe(0)

      unsub()
      expect(listener).toBeNull()
    } finally {
      window.matchMedia = original
    }
  })

  it('no twin theme ids — ship-dark is not a valid theme', () => {
    expect(isValidTheme('ship-dark')).toBe(false)
    expect(isValidTheme('ship-light')).toBe(false)
    expect(isValidTheme('neutral-light')).toBe(false)
    expect(isValidTheme('neutral-dark')).toBe(false)
    expect(isValidTheme('sail-dark')).toBe(false)
  })
})

describe('@ship/tokens — primitive tier (theme-invariant)', () => {
  const tokens = loadSrc('../src/tokens.css')
  for (const [name, sample] of PRIMITIVE_TIER) {
    it(`declares ${name} in tokens.css`, () => {
      expect(tokens).toMatch(sample)
    })
  }
})

describe('@ship/tokens — semantic tier (theme-overridable)', () => {
  const tokens = loadSrc('../src/tokens.css')
  for (const [name, sample] of SEMANTIC_TIER) {
    it(`declares ${name} in tokens.css`, () => {
      expect(tokens).toMatch(sample)
    })
  }
})

describe('@ship/tokens — component tier (panel + control family)', () => {
  const tokens = loadSrc('../src/tokens.css')
  for (const [name, sample] of COMPONENT_TIER) {
    it(`declares ${name} in tokens.css`, () => {
      expect(tokens).toMatch(sample)
    })
  }
})

describe('@ship/tokens — representative token in every theme (reskin-by-token guard)', () => {
  // The spec demands: "a representative token from each tier exists
  // in every theme". The :root default layer covers the absence case
  // for a theme that only flips a couple of values — the assertion
  // is that the SELECTOR block for each theme is present in
  // themes.css (a theme that didn't even get a block is a
  // regression: the catalog lies about its presence).
  const themesCss = loadSrc('../src/themes.css')
  for (const id of THEMES.map((t) => t.id)) {
    it(`theme '${id}' has a :root[theme] override block`, () => {
      const re = new RegExp(`:root\\[theme=['"]${id}['"]\\]`, 'm')
      expect(themesCss).toMatch(re)
    })
  }
})

describe('@ship/tokens — sail-base.css layer (host baseline + platform tone)', () => {
  const base = loadSrc('../src/sail-base.css')
  it('has no [tone=] universal channel; no --size-h channel', () => {
    // Tone is an ordinary element prop on native <button> / <a>. Status-dot
    // helpers may use span[dot][tone=…]; strip those and ban residual channels.
    const withoutDotTones = base.replace(/span\[dot\]\[tone=['"][^'"]+['"]\]/g, 'span[dot]')
    expect(withoutDotTones).not.toMatch(/\[tone=['"]/)
    expect(base).not.toMatch(/--tone\s*:/)
    // --size-h channel stays gone; size maps to control-h/text tokens directly.
    expect(base).not.toMatch(/\[size=['"]sm['"]\]\s*\{[^}]*--size-h/)
  })

  it('declares a reduced-motion media query in the sail-base layer', () => {
    expect(base).toMatch(/@media\s*\(prefers-reduced-motion:\s*reduce\)/i)
  })
})

describe('@ship/tokens — invariant: no two tier tokens share a name (catch typos)', () => {
  it('primitive / semantic / component tier name sets are disjoint', () => {
    const prim = new Set(PRIMITIVE_TIER.map(([n]) => n))
    const sem = new Set(SEMANTIC_TIER.map(([n]) => n))
    const comp = new Set(COMPONENT_TIER.map(([n]) => n))
    for (const n of prim) {
      expect(sem.has(n)).toBe(false)
      expect(comp.has(n)).toBe(false)
    }
    for (const n of sem) {
      expect(comp.has(n)).toBe(false)
    }
  })

  it('every representative token in REPRSENTATIVE_PER_THEME is in tokens.css', () => {
    const tokens = loadSrc('../src/tokens.css')
    for (const name of REPRSENTATIVE_PER_THEME) {
      const re = new RegExp(`--${name}\\s*:`, 'm')
      expect(tokens).toMatch(re)
    }
  })
})

describe('@ship/tokens — public surface footprint', () => {
  it('exports a non-trivial set of custom properties in tokens.css', () => {
    const decls = extractVarDecls(loadSrc('../src/tokens.css'))
    // Sanity: a real design system has hundreds of tokens. Below 100
    // is a regression signal — the spec demands the tiered families
    // are present.
    expect(decls.size).toBeGreaterThan(100)
  })

  it('exports a non-trivial set of theme overrides in themes.css', () => {
    const decls = extractVarDecls(loadSrc('../src/themes.css'))
    expect(decls.size).toBeGreaterThan(0)
  })
})


describe('@ship/tokens — theme values are DTCG, not CSS vars', () => {
  /** Whole-value `var(--token)` belongs in emit, not theme JSON. Author as `{token}`. */
  it('no pure CSS var(--x) as whole $value in model/themes', () => {
    const dir = resolve(dirname(fileURLToPath(import.meta.url)), '../model/themes')
    const pureVar = /^var\(\s*--[a-zA-Z0-9_-]+\s*\)$/
    const varFallback = /^var\(\s*--[a-zA-Z0-9_-]+\s*,/
    const leaks: string[] = []
    for (const file of readdirSync(dir).filter((f) => f.endsWith('.json'))) {
      const walk = (o: unknown, pth: string) => {
        if (!o || typeof o !== 'object') return
        const obj = o as Record<string, unknown>
        if ('$value' in obj && typeof obj.$value === 'string') {
          const v = obj.$value.trim()
          if (pureVar.test(v) || varFallback.test(v)) {
            leaks.push(`${file}:${pth}=${v}`)
          }
          return
        }
        for (const [k, v] of Object.entries(obj)) {
          if (k.startsWith('$')) continue
          walk(v, pth ? `${pth}.${k}` : k)
        }
      }
      walk(JSON.parse(readFileSync(join(dir, file), 'utf8')), '')
    }
    expect(leaks, leaks.join('\n')).toEqual([])
  })
})

describe('@ship/tokens — DTCG SSOT / generate drift lock', () => {
  type RegistryToken = {
    type: string
    value: string
    path?: string
    cssName?: string
    aliasOf?: string
  }
  type Registry = {
    version?: number
    names: string[]
    colors: string[]
    tokens: Record<string, RegistryToken>
    paths?: Record<string, string>
    aliases?: Record<string, string>
  }

  const loadRegistry = (): Registry =>
    JSON.parse(loadSrc('../src/token-registry.json')) as Registry

  /** Walk base + model/components/* → public cssNames (via $extensions.ship.cssName or path join). */
  function modelBaseCssNames(): Set<string> {
    const names = new Set<string>()
    const walk = (node: unknown, path: string[]) => {
      if (!node || typeof node !== 'object' || Array.isArray(node)) return
      const obj = node as Record<string, unknown>
      if (Object.prototype.hasOwnProperty.call(obj, '$value')) {
        const ext = obj.$extensions as { ship?: { cssName?: string } } | undefined
        const cssName = ext?.ship?.cssName ?? path.join('-')
        names.add(cssName)
        return
      }
      for (const [k, v] of Object.entries(obj)) {
        if (k.startsWith('$')) continue
        walk(v, [...path, k])
      }
    }
    walk(JSON.parse(loadSrc('../model/base.json')) as Record<string, unknown>, [])
    // Component packs merge into the base layer at generate time.
    const componentsDir = resolve(
      dirname(fileURLToPath(import.meta.url)),
      '../model/components',
    )
    try {
      for (const file of readdirSync(componentsDir)) {
        if (!file.endsWith('.json')) continue
        walk(
          JSON.parse(readFileSync(join(componentsDir, file), 'utf8')) as Record<string, unknown>,
          [],
        )
      }
    } catch {
      // no components/ dir yet
    }
    return names
  }

  it('tokens.css carries the GENERATED header (not hand SSOT)', () => {
    const css = loadSrc('../src/tokens.css')
    expect(css).toMatch(/GENERATED FILE/)
    expect(css).toMatch(/packages\/sail\/tokens\/model/)
  })

  it('themes.css carries the GENERATED header (theme overrides from model/themes)', () => {
    const css = loadSrc('../src/themes.css')
    expect(css).toMatch(/GENERATED FILE/)
    expect(css).toMatch(/model\/themes/)
  })

  it('token-registry.json canonical names equal base + component source cssNames', () => {
    const reg = loadRegistry()
    const model = modelBaseCssNames()
    expect(model.size).toBeGreaterThan(100)
    // Canonical tokens (non-alias) match model
    const canonical = reg.names.filter((n) => !reg.tokens[n]?.aliasOf)
    expect(canonical.length).toBe(model.size)
    for (const n of model) {
      expect(canonical.includes(n)).toBe(true)
    }
    for (const n of canonical) {
      expect(model.has(n)).toBe(true)
    }
  })

  it('registry entries expose path + cssName for sx/J2', () => {
    const reg = loadRegistry()
    expect(reg.version ?? 1).toBeGreaterThanOrEqual(2)
    expect(reg.tokens['space-4']?.path).toBe('space.4')
    expect(reg.tokens['space-4']?.cssName).toBe('space-4')
    expect(reg.tokens['primary']?.path).toMatch(/^color\./)
    expect(reg.paths?.['space.4']).toBe('space-4')
    // Accents: path color.accent.* → cssName accent-* (no load-bearing app- prefix)
    expect(reg.tokens['accent-dashboard']?.path).toBe('color.accent.dashboard')
    expect(reg.names.includes('accent-dashboard')).toBe(true)
  })

  it('every registry leaf is studio-ready (path + cssName on all names)', () => {
    // Studio / J2 / sx resolve tokens via path ↔ cssName. A leaf missing
    // either field is unusable in the design-system editor.
    const reg = loadRegistry()
    expect(reg.names.length).toBeGreaterThan(100)
    expect(Object.keys(reg.tokens).length).toBe(reg.names.length)
    const paths = reg.paths ?? {}
    for (const name of reg.names) {
      const leaf = reg.tokens[name]
      expect(leaf, `missing tokens[${name}]`).toBeTruthy()
      expect(typeof leaf.path).toBe('string')
      expect(leaf.path!.length).toBeGreaterThan(0)
      expect(leaf.path).toMatch(/^[a-z0-9]+(?:[._-][a-z0-9]+)*$/i)
      expect(typeof leaf.cssName).toBe('string')
      expect(leaf.cssName).toBe(name)
      if (leaf.aliasOf) {
        // Dual-emit aliases share the target's DTCG path; not listed in paths.
        expect(reg.tokens[leaf.aliasOf]).toBeTruthy()
        expect(paths[leaf.path!]).toBe(leaf.aliasOf)
      } else {
        expect(paths[leaf.path!]).toBe(name)
      }
    }
    // paths index covers exactly the canonical (non-alias) set
    const canonical = reg.names.filter((n) => !reg.tokens[n]?.aliasOf)
    expect(Object.keys(paths).length).toBe(canonical.length)
  })

  it('emits ≥2 theme override blocks from model/themes', () => {
    const selectors = extractThemeSelectors(loadSrc('../src/themes.css'))
    expect(selectors.size).toBeGreaterThanOrEqual(2)
    expect(selectors.has('paper')).toBe(true)
    expect(selectors.has('console') || selectors.has('ship')).toBe(true)
    // Theme JSON files exist for the catalog
    const paper = JSON.parse(loadSrc('../model/themes/paper.json')) as {
      $extensions?: { ship?: { themeId?: string } }
    }
    expect(paper.$extensions?.ship?.themeId).toBe('paper')
  })

  it('theme override blocks redeclare public cssNames only (registry-known vars)', () => {
    // themes.css must override --primary / --surface-1 etc. — the same public
    // surface Layout, Fx, and contracts consume — never invent parallel names.
    // Match modes-in-file theme layers:
    //   :root[theme='id'][data-mode='…'],
    //   [theme='id'][data-mode='…'] { … }
    // (not themes-chrome structural flourishes on body/card selectors).
    const reg = loadRegistry()
    const known = new Set(reg.names)
    const themesCss = loadSrc('../src/themes.css')
    const selectors = extractThemeSelectors(themesCss)
    expect(selectors.size).toBeGreaterThanOrEqual(2)
    // Modes-only: every theme block must mention data-mode.
    expect(themesCss).toMatch(/data-mode=['"]light['"]/)
    expect(themesCss).toMatch(/data-mode=['"]dark['"]/)
    expect(themesCss).not.toMatch(/theme=['"]ship-light['"]/)
    expect(themesCss).not.toMatch(/theme=['"]ship-dark['"]/)

    // Capture selector blocks that start with :root[theme=…] (mode-aware).
    const blockRe =
      /:root\[theme=['"]([^'"]+)['"]\][^{]*\{([^}]+)\}/g
    let m: RegExpExecArray | null
    let blocks = 0
    const overrideNames = new Set<string>()
    while ((m = blockRe.exec(themesCss)) !== null) {
      blocks++
      const body = m[2]!
      const declRe = /--([a-z0-9_-]+)\s*:/gi
      let d: RegExpExecArray | null
      while ((d = declRe.exec(body)) !== null) {
        overrideNames.add(d[1]!)
      }
    }
    expect(blocks).toBeGreaterThanOrEqual(2)
    expect(overrideNames.size).toBeGreaterThan(10)
    for (const n of overrideNames) {
      expect(known.has(n), `theme override --${n} not in token-registry names`).toBe(true)
    }
    // Public brand + surface tokens still overridden somewhere
    for (const n of ['primary', 'foreground', 'background', 'surface-0', 'surface-1']) {
      expect(overrideNames.has(n)).toBe(true)
    }
  })

  it('generated tokens.css preserves protected consumer CSS names', () => {
    const protectedDoc = JSON.parse(loadSrc('./protected-css-names.json')) as { names: string[] }
    const cssNames = extractVarDecls(loadSrc('../src/tokens.css'))
    expect(protectedDoc.names.length).toBeGreaterThan(100)
    for (const n of protectedDoc.names) {
      expect(cssNames.has(n)).toBe(true)
    }
  })

  it('compat aliases scrubbed (TOKENS-1 — app-accent-* cut, no dual-emit tombstones)', () => {
    const reg = loadRegistry()
    const compat = JSON.parse(loadSrc('../model/compat-aliases.json')) as {
      aliases: Record<string, string>
    }
    expect(Object.keys(compat.aliases).length).toBe(0)
    expect(Object.keys(reg.aliases ?? {}).length).toBe(0)
    const names = reg.names as string[]
    expect(names.some((n) => n.startsWith('app-accent-'))).toBe(false)
  })

  it('products.css is generated from model/products.json (system brand-hue axis)', () => {
    const productsCss = loadSrc('../src/products.css')
    expect(productsCss).toMatch(/GENERATED FILE/)
    expect(productsCss).toMatch(/Source of truth: packages\/sail\/tokens\/model\/products\.json/)
    for (const [id, hue] of [
      ['ship', '195'],
      ['sail', '27'],
      ['deck', '285'],
      ['components', '155'],
      ['ai', '255'],
    ] as const) {
      const re = new RegExp(
        String.raw`:root\[data-product='${id}'\]\s*\{\s*--brand-hue:\s*${hue};`,
      )
      expect(productsCss).toMatch(re)
    }
    const reg = loadRegistry() as {
      products?: {
        channel: string
        entries: Record<string, { hue: number }>
      }
    }
    expect(reg.products?.channel).toBe('brand-hue')
    expect(reg.products?.entries.ship?.hue).toBe(195)
    expect(reg.products?.entries.sail?.hue).toBe(27)
  })

  it('tokens.css :root name set equals registry names (emit parity)', () => {
    const reg = loadRegistry()
    const cssNames = extractVarDecls(loadSrc('../src/tokens.css'))
    // CSS also redeclares names under .dark / density — set equality on unique names
    for (const n of reg.names) {
      expect(cssNames.has(n)).toBe(true)
    }
    // Every CSS custom property must be a known token (no orphan decls)
    for (const n of cssNames) {
      expect(reg.names.includes(n)).toBe(true)
    }
  })

  it('registry values for Tailwind space ramp match emitted CSS', () => {
    const reg = loadRegistry()
    const css = loadSrc('../src/tokens.css')
    const ramp: Array<[string, string]> = [
      ['space-1', '4px'],
      ['space-7', '28px'],
      ['space-8', '32px'],
      ['space-64', '256px'],
    ]
    for (const [name, want] of ramp) {
      expect(reg.tokens[name]?.value).toBe(want)
      const re = new RegExp(`--${name}\\s*:\\s*${want}\\s*;`)
      expect(css).toMatch(re)
    }
  })

  it('NAV/CHROME/MISC component packs register path → cssName', () => {
    const reg = loadRegistry()
    const expected: Array<[path: string, cssName: string]> = [
      ['component.card.bg', 'card'],
      ['component.navbar.bg', 'navbar'],
      ['component.navbar.fg', 'navbar-foreground'],
      ['component.navbar.border', 'navbar-border'],
      ['component.menu-item.fg', 'menu-item-foreground'],
      ['component.menu-item.hover-bg', 'menu-item-hover-bg'],
      ['component.dropdown.bg', 'dropdown'],
      ['component.dropdown.fg', 'dropdown-foreground'],
      ['component.context-menu.bg', 'context-menu'],
      ['component.context-menu.fg', 'context-menu-foreground'],
      ['component.combobox.bg', 'combobox'],
      ['component.combobox.fg', 'combobox-foreground'],
      ['component.page.fg', 'page-foreground'],
      ['component.page.hover-bg', 'page-hover-bg'],
      ['component.page.current-bg', 'page-current-bg'],
      ['component.page.current-fg', 'page-current-foreground'],
      ['component.prose.fg', 'prose-foreground'],
      ['component.code-block.bg', 'code-block'],
      ['component.code-block.fg', 'code-block-foreground'],
      ['component.theme-picker.bg', 'theme-picker'],
      ['component.theme-picker.fg', 'theme-picker-foreground'],
      ['component.error-boundary.bg', 'error-boundary'],
      ['component.error-boundary.border', 'error-boundary-border'],
      ['component.error-boundary.fg', 'error-boundary-foreground'],
    ]
    for (const [path, cssName] of expected) {
      expect(reg.paths?.[path], path).toBe(cssName)
      expect(reg.tokens[cssName]?.path).toBe(path)
      expect(reg.tokens[cssName]?.cssName).toBe(cssName)
    }
    const css = loadSrc('../src/tokens.css')
    expect(css).toMatch(/--navbar\s*:\s*var\(--background\)/)
    expect(css).toMatch(/--combobox\s*:\s*var\(--popover\)/)
    expect(css).toMatch(/--code-block\s*:\s*var\(--surface-1\)/)
    expect(css).toMatch(/--error-boundary\s*:\s*var\(--muted\)/)
  })

  it('CONTROL component packs register path → cssName', () => {
    const reg = loadRegistry()
    const expected: Array<[path: string, cssName: string]> = [
      ['component.field.bg', 'field'],
      ['component.field.fg', 'field-foreground'],
      ['component.field.border', 'field-border'],
      ['component.field.border-hover', 'field-border-hover'],
      ['component.field.hint', 'field-hint'],
      ['component.field.error', 'field-error'],
      ['component.button.bg', 'button'],
      ['component.button.fg', 'button-foreground'],
      ['component.button.border', 'button-border'],
      ['component.button.hover-bg', 'button-hover-bg'],
      ['component.label.fg', 'label-foreground'],
      ['component.label.required', 'label-required'],
      ['component.switch.track', 'switch'],
      ['component.switch.border', 'switch-border'],
      ['component.switch.thumb', 'switch-thumb'],
      ['component.toggle.bg', 'toggle'],
      ['component.toggle.fg', 'toggle-foreground'],
      ['component.toggle.border', 'toggle-border'],
      ['component.toggle-group.bg', 'toggle-group'],
      ['component.toggle-group.border', 'toggle-group-border'],
      ['component.radio.fg', 'radio-foreground'],
      ['component.tab.fg', 'tab-foreground'],
      ['component.tab.selected-fg', 'tab-selected-foreground'],
      ['component.tab.selected-bg', 'tab-selected-bg'],
      ['component.tab.border', 'tab-border'],
      ['component.tab-list.bg', 'tab-list'],
      ['component.tab-list.border', 'tab-list-border'],
      ['component.tab-list.rail', 'tab-list-rail'],
      ['component.scroll-area.thumb', 'scroll-area-thumb'],
      ['component.scroll-area.thumb-hover', 'scroll-area-thumb-hover'],
    ]
    for (const [path, cssName] of expected) {
      expect(reg.paths?.[path], path).toBe(cssName)
      expect(reg.tokens[cssName]?.path).toBe(path)
      expect(reg.tokens[cssName]?.cssName).toBe(cssName)
    }
    const css = loadSrc('../src/tokens.css')
    expect(css).toMatch(/--switch\s*:\s*var\(--muted\)/)
    expect(css).toMatch(/--label-foreground\s*:\s*var\(--foreground\)/)
    expect(css).toMatch(/--field-hint\s*:\s*var\(--muted-foreground\)/)
    expect(css).toMatch(/--field\s*:\s*var\(--background\)/)
    expect(css).toMatch(/--button\s*:\s*var\(--background\)/)
    expect(css).toMatch(/--toggle\s*:\s*var\(--background\)/)
    expect(css).toMatch(/--tab-list\s*:\s*var\(--background\)/)
    expect(css).toMatch(/--scroll-area-thumb\s*:\s*var\(--border\)/)
  })

  it('L1-token-totality surface packs register path → cssName', () => {
    const reg = loadRegistry()
    const expected: Array<[path: string, cssName: string]> = [
      ['component.dialog.bg', 'dialog'],
      ['component.dialog.fg', 'dialog-foreground'],
      ['component.dialog.border', 'dialog-border'],
      ['component.accordion.bg', 'accordion'],
      ['component.accordion.fg', 'accordion-foreground'],
      ['component.accordion.border', 'accordion-border'],
      ['component.accordion.hover-bg', 'accordion-hover-bg'],
      ['component.accordion.chevron', 'accordion-chevron'],
      ['component.skeleton.bg', 'skeleton'],
      ['component.skeleton.highlight', 'skeleton-highlight'],
      ['component.kbd.bg', 'kbd'],
      ['component.kbd.fg', 'kbd-foreground'],
      ['component.kbd.border', 'kbd-border'],
      // Slice #2 residual packs
      ['component.table.fg', 'table-foreground'],
      ['component.table.muted-fg', 'table-muted-foreground'],
      ['component.table.stripe', 'table-stripe'],
      ['component.table.sticky-bg', 'table-sticky-bg'],
      ['component.progress.track', 'progress'],
      ['component.progress.fill', 'progress-fill'],
      ['component.meter.track', 'meter'],
      ['component.meter.optimum', 'meter-optimum'],
      ['component.meter.suboptimum', 'meter-suboptimum'],
      ['component.meter.sub-suboptimum', 'meter-sub-suboptimum'],
      ['component.slider.track', 'slider-track'],
      ['component.slider.fill', 'slider-fill'],
      ['component.slider.track-border', 'slider-track-border'],
      ['component.breadcrumb.fg', 'breadcrumb-foreground'],
      ['component.breadcrumb.current', 'breadcrumb-current'],
      ['component.fieldset.border', 'fieldset-border'],
      ['component.legend.fg', 'legend-foreground'],
      // Slice #3 residual pack extensions
      ['component.alert.border', 'alert-border'],
      ['component.toast.stripe', 'toast-stripe'],
      ['component.navbar.border', 'navbar-border'],
      ['component.empty-state.title', 'empty-state-title'],
      ['component.tab.border', 'tab-border'],
      // Slice #4 — panel borders + dialog anatomy
      ['component.popover.border', 'popover-border'],
      ['component.dropdown.border', 'dropdown-border'],
      ['component.context-menu.border', 'context-menu-border'],
      ['component.hover-card.border', 'hover-card-border'],
      ['component.dialog.divider', 'dialog-divider'],
      ['component.dialog.footer', 'dialog-footer'],
      ['component.dialog.muted-fg', 'dialog-muted-foreground'],
    ]
    for (const [path, cssName] of expected) {
      expect(reg.paths?.[path], path).toBe(cssName)
      expect(reg.tokens[cssName]?.path).toBe(path)
    }
    const css = loadSrc('../src/tokens.css')
    expect(css).toMatch(/--dialog\s*:\s*var\(--surface-1\)/)
    expect(css).toMatch(/--accordion\s*:\s*var\(--surface-1\)/)
    expect(css).toMatch(/--skeleton\s*:\s*var\(--muted\)/)
    expect(css).toMatch(/--kbd\s*:\s*var\(--muted\)/)
    expect(css).toMatch(/--table-foreground\s*:\s*var\(--foreground\)/)
    expect(css).toMatch(/--progress\s*:\s*var\(--muted\)/)
    expect(css).toMatch(/--progress-fill\s*:\s*var\(--primary\)/)
    expect(css).toMatch(/--meter\s*:\s*var\(--muted\)/)
    expect(css).toMatch(/--slider-track\s*:\s*var\(--muted\)/)
    expect(css).toMatch(/--breadcrumb-foreground\s*:\s*var\(--muted-foreground\)/)
    expect(css).toMatch(/--fieldset-border\s*:\s*var\(--border\)/)
    expect(css).toMatch(/--legend-foreground\s*:\s*var\(--foreground\)/)
    expect(css).toMatch(/--alert-border\s*:\s*var\(--border\)/)
    expect(css).toMatch(/--toast-stripe\s*:\s*var\(--foreground\)/)
    expect(css).toMatch(/--navbar-border\s*:\s*var\(--border\)/)
    expect(css).toMatch(/--empty-state-title\s*:\s*var\(--foreground\)/)
    expect(css).toMatch(/--tab-border\s*:\s*var\(--border\)/)
    expect(css).toMatch(/--popover-border\s*:\s*var\(--border\)/)
    expect(css).toMatch(/--dropdown-border\s*:\s*var\(--border\)/)
    expect(css).toMatch(/--context-menu-border\s*:\s*var\(--border\)/)
    expect(css).toMatch(/--hover-card-border\s*:\s*var\(--popover-border\)/)
    expect(css).toMatch(/--dialog-divider\s*:\s*var\(--border-subtle\)/)
    expect(css).toMatch(/--dialog-footer\s*:\s*var\(--muted\)/)
    expect(css).toMatch(/--dialog-muted-foreground\s*:\s*var\(--muted-foreground\)/)
  })

  it('color surface includes brand + tone tokens and excludes space/text', () => {
    const reg = loadRegistry()
    const colors = new Set(reg.colors)
    for (const n of ['primary', 'foreground', 'surface-1', 'hover-bg', 'accent-foreground']) {
      expect(colors.has(n)).toBe(true)
    }
    for (const n of ['space-4', 'text-2xl', 'radius-lg']) {
      expect(colors.has(n)).toBe(false)
    }
    expect(reg.colors.length).toBeGreaterThan(40)
  })

  it('generated/unions.ts carries GENERATED header and closed-set exports', () => {
    const unions = loadSrc('../src/generated/unions.ts')
    expect(unions).toMatch(/GENERATED FILE/)
    expect(unions).toMatch(/export type CssTokenName =/)
    expect(unions).toMatch(/export type ColorTokenName =/)
    expect(unions).toMatch(/export type ToneName =/)
    expect(unions).toMatch(/export type ThemeId =/)
    expect(unions).toMatch(/export const TONE_NAMES/)
    expect(unions).toMatch(/TONE_NAMES_SOURCE/)
    expect(unions).toMatch(/export const THEME_IDS/)
    expect(unions).toMatch(/export type ControlHeight =/)
    expect(unions).toMatch(/export type SpaceToken =/)
    expect(unions).toMatch(/export type SemanticColorName =/)
    // Tone membership from DTCG color.tone ramps
    for (const tone of ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']) {
      expect(unions).toContain(`'${tone}'`)
    }
    // Registry names appear in CssTokenName
    expect(unions).toContain("'space-4'")
    expect(unions).toContain("'primary'")
    // Theme ids from model/themes (modes-in-file — no twin ids)
    expect(unions).toContain("'ship'")
    expect(unions).toContain("'paper'")
    expect(unions).not.toContain("'ship-light'")
    expect(unions).not.toContain("'ship-dark'")
  })
})

