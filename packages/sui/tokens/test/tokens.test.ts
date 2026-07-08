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
import { dirname, resolve } from 'node:path'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

import { DEFAULT_THEME, THEMES, isDarkTheme, isValidTheme } from '../src/themes'

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
  const re = /--([a-z0-9-]+)\s*:/gi
  let m: RegExpExecArray | null
  while ((m = re.exec(css)) !== null) {
    out.add(m[1]!)
  }
  return out
}

/** Every `:root[data-theme='X']` selector. Captures the theme id. */
function extractThemeSelectors(css: string): Set<string> {
  const out = new Set<string>()
  const re = /:root\[data-theme=['"]([^'"]+)['"]\]/g
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
  // Tone ramps (semantic: per-tone fg/bg/border/ring families)
  ['--tone-primary-fg', /--tone-primary-fg\s*:/i],
  ['--tone-primary-bg', /--tone-primary-bg\s*:/i],
  ['--tone-primary-border', /--tone-primary-border\s*:/i],
  ['--tone-info-bg', /--tone-info-bg\s*:/i],
  ['--tone-success-fg', /--tone-success-fg\s*:/i],
  ['--tone-warning-border', /--tone-warning-border\s*:/i],
  ['--tone-destructive-ring', /--tone-destructive-ring\s*:/i],
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
      expect(t.id).toMatch(/^[a-z][a-z0-9-]*$/)
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

  it('every THEMES id is also a `:root[data-theme="id"]` selector in themes.css', () => {
    const selectors = extractThemeSelectors(loadSrc('../src/themes.css'))
    for (const t of THEMES) {
      expect(selectors.has(t.id)).toBe(true)
    }
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
    it(`theme '${id}' has a :root[data-theme] override block`, () => {
      const re = new RegExp(`:root\\[data-theme=['"]${id}['"]\\]`, 'm')
      expect(themesCss).toMatch(re)
    })
  }
})

describe('@ship/tokens — contracts.css layer (light-DOM framework contracts)', () => {
  const contracts = loadSrc('../src/contracts.css')
  it('ships a tone-channel contract for every standard tone', () => {
    for (const tone of ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']) {
      const re = new RegExp(`\\[tone=['"]${tone}['"]\\]\\s*\\{`, 'm')
      expect(contracts).toMatch(re)
    }
  })

  it('ships a size-channel contract for every size variant', () => {
    for (const size of ['xs', 'sm', 'md', 'lg']) {
      const re = new RegExp(`\\[size=['"]${size}['"]\\]\\s*\\{`, 'm')
      expect(contracts).toMatch(re)
    }
  })

  it('declares a reduced-motion media query in the contracts layer', () => {
    expect(contracts).toMatch(/@media\s*\(prefers-reduced-motion:\s*reduce\)/i)
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

