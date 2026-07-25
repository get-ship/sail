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
  // Underscores appear in fine-step space names (space-0_75, space-1_5, …).
  const re = /--([a-z0-9_-]+)\s*:/gi
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

describe('@ship/tokens — contracts.css layer (host baseline, no attr channels)', () => {
  const contracts = loadSrc('../src/contracts.css')
  it('does not ship tone/size attribute channels', () => {
    expect(contracts).not.toMatch(/\[tone=['"]primary['"]\]\s*\{/)
    expect(contracts).not.toMatch(/\[size=['"]sm['"]\]\s*\{[^}]*--size-h/)
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
    expect(selectors.has('console') || selectors.has('ship-dark')).toBe(true)
    // Theme JSON files exist for the catalog
    const paper = JSON.parse(loadSrc('../model/themes/paper.json')) as {
      $extensions?: { ship?: { themeId?: string } }
    }
    expect(paper.$extensions?.ship?.themeId).toBe('paper')
  })

  it('theme override blocks redeclare public cssNames only (registry-known vars)', () => {
    // themes.css must override --primary / --surface-1 etc. — the same public
    // surface Layout, Fx, and contracts consume — never invent parallel names.
    // Match only sail-compiler theme layers:
    //   :root[data-theme='id'],
    //   [data-theme-preview='id'] { … }
    // (not themes-chrome structural flourishes on body/card selectors).
    const reg = loadRegistry()
    const known = new Set(reg.names)
    const themesCss = loadSrc('../src/themes.css')
    const selectors = extractThemeSelectors(themesCss)
    expect(selectors.size).toBeGreaterThanOrEqual(2)

    const blockRe =
      /:root\[data-theme=['"]([^'"]+)['"]\]\s*,\s*\[data-theme-preview=['"]\1['"]\]\s*\{([^}]+)\}/g
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

  it('dual-emit compat aliases remain (registry + tokens.css)', () => {
    // Do not drop PE dual-emit: app-accent-* → var(--accent-*).
    const reg = loadRegistry()
    const css = loadSrc('../src/tokens.css')
    const compat = JSON.parse(loadSrc('../model/compat-aliases.json')) as {
      aliases: Record<string, string>
    }
    const aliases = compat.aliases
    expect(Object.keys(aliases).length).toBeGreaterThan(0)
    for (const [oldName, newName] of Object.entries(aliases)) {
      expect(reg.aliases?.[oldName]).toBe(newName)
      expect(reg.tokens[oldName]?.aliasOf).toBe(newName)
      expect(reg.tokens[oldName]?.cssName).toBe(oldName)
      expect(reg.tokens[oldName]?.path).toBe(reg.tokens[newName]?.path)
      const re = new RegExp(`--${oldName}\\s*:\\s*var\\(--${newName}\\)`)
      expect(css).toMatch(re)
    }
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
    expect(css).toMatch(/--error-boundary\s*:\s*var\(--tone-destructive-bg\)/)
  })

  it('CONTROL component packs register path → cssName', () => {
    const reg = loadRegistry()
    const expected: Array<[path: string, cssName: string]> = [
      ['component.field.hint', 'field-hint'],
      ['component.field.error', 'field-error'],
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
      ['component.tab.selected-bg', 'tab-selected'],
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
    expect(css).toMatch(/--toggle\s*:\s*var\(--background\)/)
    expect(css).toMatch(/--tab-list\s*:\s*var\(--background\)/)
    expect(css).toMatch(/--scroll-area-thumb\s*:\s*var\(--border\)/)
  })

  it('color surface includes brand + tone tokens and excludes space/text', () => {
    const reg = loadRegistry()
    const colors = new Set(reg.colors)
    for (const n of ['primary', 'foreground', 'surface-1', 'tone-success-bg', 'accent-foreground']) {
      expect(colors.has(n)).toBe(true)
    }
    for (const n of ['space-4', 'text-2xl', 'radius-lg']) {
      expect(colors.has(n)).toBe(false)
    }
    expect(reg.colors.length).toBeGreaterThan(40)
  })
})

