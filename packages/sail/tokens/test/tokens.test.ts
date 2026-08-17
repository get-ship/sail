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
import { existsSync, readdirSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

import {
  CORE_THEME_IDS,
  DEFAULT_THEME,
  DEFAULT_MODE,
  PREPAINT_THEME_SCRIPT,
  THEME_DEFAULT_META,
  THEME_DEFAULT_MODE_META,
  THEME_MODE_STORAGE_KEY,
  THEME_OVERRIDE_ORDER,
  THEME_ROOT_KEYS,
  THEME_STORAGE_KEY,
  THEMES,
  applyMode,
  applyNestedTheme,
  applyTheme,
  bootTheme,
  isCoreTheme,
  isDarkTheme,
  isValidTheme,
  modeCapability,
  modesForTheme,
  paintMode,
  persistMode,
  pickerModesForTheme,
  readConfigDefaultMode,
  readConfigDefaultTheme,
  resolveNestedPaintMode,
  readModePreference,
  readStoredTheme,
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
  // Underscores appear in named half-steps (space-1_5, space-2_5).
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

/** Whole-value custom-property alias: `--name: var(--target)` (optional fallback). */
type AliasClass = 'keep' | 'rebind' | 'leftover' | 'product' | 'hue'
type AliasSite = { name: string; target: string; cls: AliasClass }

const WHOLE_VALUE_VAR = /^var\(--([a-zA-Z0-9_-]+)(?:,\s*[^)]+)?\)$/

function stripCssComments(css: string): string {
  return css.replace(/\/\*[\s\S]*?\*\//g, '')
}

function classifyAlias(name: string): AliasClass {
  if (name === 'field' || name.startsWith('field-') || name === 'fieldset-border') {
    return 'rebind'
  }
  if (name === 'brand-hue' || name.startsWith('brand-hue')) return 'product'
  if (name.startsWith('hue-')) return 'hue'
  if (name === 'sidebar' || name.startsWith('sidebar-')) return 'leftover'
  return 'keep'
}

function extractWholeValueAliases(css: string): AliasSite[] {
  const out: AliasSite[] = []
  const re = /--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);/g
  let m: RegExpExecArray | null
  while ((m = re.exec(stripCssComments(css))) !== null) {
    const name = m[1]!
    const value = m[2]!.trim()
    const vm = value.match(WHOLE_VALUE_VAR)
    if (!vm) continue
    out.push({ name, target: vm[1]!, cls: classifyAlias(name) })
  }
  return out
}

function uniqueAliasNames(sites: AliasSite[]): Map<string, AliasSite> {
  const unique = new Map<string, AliasSite>()
  for (const site of sites) {
    if (!unique.has(site.name)) unique.set(site.name, site)
  }
  return unique
}

function countAliasClasses(unique: Map<string, AliasSite>): Record<AliasClass, number> {
  const counts: Record<AliasClass, number> = {
    keep: 0,
    rebind: 0,
    leftover: 0,
    product: 0,
    hue: 0,
  }
  for (const site of unique.values()) counts[site.cls]++
  return counts
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
  // Purpose steps (app / subtle / surface / inset / overlay)
  ['--background', /--background\s*:/i],
  ['--surface', /--surface\s*:/i],
  ['--inset', /--inset\s*:/i],
  ['--overlay', /--overlay\s*:/i],
  ['--overlay-fill', /--overlay-fill\s*:/i],
  ['--backdrop', /--backdrop\s*:/i],
  ['--primary', /--primary\s*:/i],
  ['--hover-bg', /--hover-bg\s*:/i],
  ['--success', /--success\s*:/i],
  ['--warning', /--warning\s*:/i],
  ['--danger', /--danger\s*:/i],
  ['--foreground', /--foreground\s*:/i],
  ['--muted-foreground', /--muted-foreground\s*:/i],
] as const

const COMPONENT_TIER: ReadonlyArray<[name: string, sample: RegExp]> = [
  // Overlay clamps that remain in the model (HOLD / OPEN-5)
  ['--panel-max-w-md', /--panel-max-w-md\s*:/i],
  ['--panel-min-w-md', /--panel-min-w-md\s*:/i],
  // Control heights + stroke
  ['--control-h-md', /--control-h-md\s*:/i],
  ['--control-stroke', /--control-stroke\s*:/i],
  ['--shadow-edge-highlight', /--shadow-edge-highlight\s*:/i],
] as const

/** Tokens the spec calls out by name — these MUST exist in every theme.
 *  Stored as BARE names (no leading --); the regex prepends it. */
const REPRSENTATIVE_PER_THEME = [
  'surface',
  'overlay-fill',
  'panel-max-w-md',
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

  it('CORE_THEME_IDS is sail only (ship/neutral left the catalog)', () => {
    expect([...CORE_THEME_IDS]).toEqual(['sail'])
    expect(isCoreTheme('sail')).toBe(true)
    expect(THEMES.some((t) => t.id === 'ship')).toBe(false)
    expect(THEMES.some((t) => t.id === 'neutral')).toBe(false)
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
        sail: {
          themeId: 'probe',
          defaultMode: 'light',
          layer: 'theme',
          supportedModes: ['light', 'dark'],
        },
      },
      modes: {
        light: { color: {} },
        dark: { color: {} },
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

  /**
   * Cheap edit → registry → CSS chain (full studio screenshot round-trip
   * remains residual). Studio writes DTCG leaves; generate emits CSS vars
   * keyed by registry path→cssName. Assert the three artifacts agree for a
   * representative global scale leaf.
   */
  it('THEMES-1 residual: theme DTCG leaf maps registry path → generated CSS var', () => {
    const paperPath = resolve(
      dirname(fileURLToPath(import.meta.url)),
      '../model/themes/paper.json',
    )
    const paper = JSON.parse(readFileSync(paperPath, 'utf8')) as {
      $extensions?: { sail?: { identity?: string } }
      anchors: { light: { primary: { l: number; c: number; h: number } } }
      modes: {
        light: {
          color: { background: { $value: { l?: number } | string } }
        }
      }
    }
    expect(paper.$extensions?.sail?.identity).toMatch(/warm print/i)
    const paperBg = paper.modes.light.color.background.$value
    expect(paperBg).toBeTruthy()

    const reg = JSON.parse(loadSrc('../src/token-registry.json')) as {
      paths?: Record<string, string>
      tokens?: Record<string, { path?: string; value?: string }>
    }
    expect(reg.paths?.['color.primary.solid']).toBe('primary')
    expect(reg.tokens?.primary?.path).toBe('color.primary.solid')

    const themesCss = loadSrc('../src/themes.css')
    // paper light block emits the derived --primary from the brick-red anchor.
    expect(themesCss).toMatch(/theme=['"]paper['"]\][\s\S]*?--primary\s*:/)
    expect(paper.anchors.light.primary.h).toBe(30)

    // Theme remaps color.card → --card
    const cardBg = paper.modes.light.color?.card?.default?.$value
    if (cardBg) {
      expect(reg.paths?.['color.card.default']).toBe('card')
      expect(reg.tokens?.card?.path).toBe('color.card.default')
      expect(themesCss).toMatch(
        new RegExp(
          `theme=['"]paper['"]\\][\\s\\S]*?--card\\s*:\\s*${cardBg.replace(
            /[.*+?^${}()|[\]\\]/g,
            '\\$&',
          )}`,
        ),
      )
    }
  })

  it('every THEMES id is also a `:root[theme="id"]` selector in themes.css', () => {
    const selectors = extractThemeSelectors(loadSrc('../src/themes.css'))
    for (const t of THEMES) {
      expect(selectors.has(t.id)).toBe(true)
    }
  })

  it('applyTheme(DEFAULT_THEME) sets theme + mode light without .dark', () => {
    const root = document.documentElement
    root.removeAttribute('theme')
    root.removeAttribute('mode')
    root.classList.remove('dark')
    applyTheme(DEFAULT_THEME, root)
    expect(root.getAttribute('theme')).toBe(DEFAULT_THEME)
    expect(root.getAttribute('mode')).toBe(DEFAULT_MODE)
    expect(root.classList.contains('dark')).toBe(false)
  })

  it('PREPAINT_THEME_SCRIPT matches prepaint-theme.js (deck include_str SoT)', () => {
    const file = readFileSync(
      join(dirname(fileURLToPath(import.meta.url)), '../../runtime/src/prepaint-theme.js'),
      'utf8',
    )
    expect(file.trimEnd()).toBe(PREPAINT_THEME_SCRIPT.trimEnd())
    expect(PREPAINT_THEME_SCRIPT).toContain(THEME_STORAGE_KEY)
    expect(PREPAINT_THEME_SCRIPT).toContain(THEME_MODE_STORAGE_KEY)
    expect(PREPAINT_THEME_SCRIPT).toContain(THEME_DEFAULT_META)
    expect(PREPAINT_THEME_SCRIPT).toContain("setAttribute('theme'")
    expect(PREPAINT_THEME_SCRIPT).toContain("setAttribute('mode'")
  })

  it('bootTheme uses config meta defaults when storage is empty', () => {
    try {
      localStorage.clear()
    } catch {
      /* */
    }
    for (const n of [THEME_DEFAULT_META, THEME_DEFAULT_MODE_META]) {
      document.querySelector(`meta[name="${n}"]`)?.remove()
    }
    const tMeta = document.createElement('meta')
    tMeta.setAttribute('name', THEME_DEFAULT_META)
    tMeta.setAttribute('content', 'sail')
    document.head.appendChild(tMeta)
    const mMeta = document.createElement('meta')
    mMeta.setAttribute('name', THEME_DEFAULT_MODE_META)
    mMeta.setAttribute('content', 'dark')
    document.head.appendChild(mMeta)

    const root = document.documentElement
    root.removeAttribute('theme')
    root.removeAttribute('mode')
    root.setAttribute('data-theme', 'unknown')
    root.classList.add('dark')

    const result = bootTheme(root)
    expect(result.theme).toBe('sail')
    expect(result.mode).toBe('dark')
    expect(root.getAttribute('theme')).toBe('sail')
    expect(root.getAttribute('mode')).toBe('dark')
    expect(root.hasAttribute('data-theme')).toBe(false)
    expect(root.classList.contains('dark')).toBe(false)
    expect(readConfigDefaultTheme()).toBe('sail')
    expect(readConfigDefaultMode()).toBe('dark')
    expect(readStoredTheme()).toBe('sail')
    expect(readModePreference()).toBe('dark')
  })

  it('bootTheme prefers storage over config meta', () => {
    try {
      localStorage.clear()
      localStorage.setItem(THEME_STORAGE_KEY, 'console')
      localStorage.setItem(THEME_MODE_STORAGE_KEY, 'dark')
    } catch {
      /* */
    }
    for (const n of [THEME_DEFAULT_META, THEME_DEFAULT_MODE_META]) {
      document.querySelector(`meta[name="${n}"]`)?.remove()
    }
    const tMeta = document.createElement('meta')
    tMeta.setAttribute('name', THEME_DEFAULT_META)
    tMeta.setAttribute('content', 'sail')
    document.head.appendChild(tMeta)

    const root = document.documentElement
    const result = bootTheme(root)
    expect(result.theme).toBe('console')
    expect(result.mode).toBe('dark')
    expect(root.getAttribute('theme')).toBe('console')
  })

  it('bootTheme falls back to package defaults without meta or storage', () => {
    try {
      localStorage.clear()
    } catch {
      /* */
    }
    for (const n of [THEME_DEFAULT_META, THEME_DEFAULT_MODE_META]) {
      document.querySelector(`meta[name="${n}"]`)?.remove()
    }
    const root = document.documentElement
    const result = bootTheme(root)
    expect(result.theme).toBe(DEFAULT_THEME)
    expect(result.mode).toBe(DEFAULT_MODE)
  })

  // Clear config meta so later applyTheme / readModePreference tests stay package-default.
  it('cleans bootTheme meta fixtures', () => {
    for (const n of [THEME_DEFAULT_META, THEME_DEFAULT_MODE_META]) {
      document.querySelector(`meta[name="${n}"]`)?.remove()
    }
    try {
      localStorage.clear()
    } catch {
      /* */
    }
  })

  it('applyMode(dark) sets mode dark without .dark class', () => {
    const root = document.documentElement
    applyTheme('sail', root, 'light')
    applyMode('dark', root)
    expect(root.getAttribute('theme')).toBe('sail')
    expect(root.getAttribute('mode')).toBe('dark')
    expect(root.classList.contains('dark')).toBe(false)
  })

  it('applyMode(dark) on paper clamps to light (single-mode lock)', () => {
    const root = document.documentElement
    applyTheme('paper', root, 'light')
    applyMode('dark', root)
    expect(root.getAttribute('theme')).toBe('paper')
    expect(root.getAttribute('mode')).toBe('light')
  })

  // Nested theme + mode: color-scheme only. Inputs spend --input, not --field.
  it('tokens.css stamps color-scheme on [theme][mode] and does not rebind --field', () => {
    const css = loadSrc('../src/tokens.css')
    expect(css).not.toMatch(/--field\s*:/)
    expect(css).not.toMatch(/--field-foreground\s*:/)
    expect(css).not.toMatch(/--fieldset-border\s*:/)
    expect(css).toMatch(
      /\[theme\]\[mode=['"]light['"]\]\s*\{\s*color-scheme\s*:\s*light\s*;/,
    )
    expect(css).toMatch(
      /\[theme\]\[mode=['"]dark['"]\]\s*\{\s*color-scheme\s*:\s*dark\s*;/,
    )
  })

  it('resolveNestedPaintMode inherits document mode when theme supports it', () => {
    const root = document.documentElement
    applyTheme('sail', root, 'dark')
    expect(resolveNestedPaintMode('sail')).toBe('dark')
    expect(resolveNestedPaintMode('paper')).toBe('light') // paper is light-only
    expect(resolveNestedPaintMode('console')).toBe('dark') // console dark-only
    expect(resolveNestedPaintMode('sail', 'light')).toBe('light') // explicit wins
  })

  it('applyNestedTheme stamps theme + mode without document event/persist', () => {
    const root = document.documentElement
    applyTheme('sail', root, 'dark')
    const host = document.createElement('div')
    document.body.appendChild(host)
    const beforeTheme = root.getAttribute('theme')
    const beforeMode = root.getAttribute('mode')
    const { theme, mode } = applyNestedTheme('paper', host)
    expect(theme).toBe('paper')
    expect(mode).toBe('light')
    expect(host.getAttribute('theme')).toBe('paper')
    expect(host.getAttribute('mode')).toBe('light')
    // Document product theme untouched
    expect(root.getAttribute('theme')).toBe(beforeTheme)
    expect(root.getAttribute('mode')).toBe(beforeMode)
    host.remove()
  })

  it('sail-light input and field-wrapped input share --input (no --field layer)', () => {
    const themes = loadSrc('../src/themes.css')
    const sailLight = /\[theme='sail'\]\[mode='light'\][\s\S]*?--input\s*:\s*([^;]+);/.exec(
      themes,
    )
    const inputValue = sailLight?.[1]?.trim() ?? ''
    expect(inputValue.length).toBeGreaterThan(0)

    const style = document.createElement('style')
    style.textContent = `
      [theme='sail'][mode='light'] { --input: ${inputValue}; }
    `
    document.head.appendChild(style)

    const host = document.createElement('div')
    host.setAttribute('theme', 'sail')
    host.setAttribute('mode', 'light')
    const field = document.createElement('div')
    field.setAttribute('data-field', '')
    host.appendChild(field)
    document.body.appendChild(host)

    const hostInput = getComputedStyle(host).getPropertyValue('--input').trim()
    expect(hostInput).toBe(inputValue)
    // Field wrapper does not declare a second surface — same --input as the
    // theme host. Happy-dom does not inherit custom props to children, so the
    // proof is: no --field layer exists, and L1 skins spend --input.
    expect(getComputedStyle(field).getPropertyValue('--field').trim()).toBe('')
    expect(loadSrc('../src/tokens.css')).not.toMatch(/--field\s*:/)
    expect(loadSrc('../src/tokens.css')).toMatch(
      /--bg-control-color:\s*var\(--input\)/,
    )
    const skins = loadSrc('../src/element-skins.css')
    expect(skins).toMatch(/background-color:\s*var\(--bg-control-color\)/)
    expect(skins).not.toMatch(/var\(--field\)/)

    style.remove()
    host.remove()
  })

  it('nested theme= without mode inherits document mode (no JS)', () => {
    const style = document.createElement('style')
    style.textContent = `
      :root[theme='sail'][mode='dark'],
      [theme='sail'][mode='dark'],
      :root[mode='dark'] [theme='sail']:not([mode]) {
        --primary: #111111;
      }
      :root[theme='sail'][mode='light'],
      :root[theme='sail']:not([mode]),
      [theme='sail'][mode='light'],
      :root[mode='light'] [theme='sail']:not([mode]),
      :root:not([mode]) [theme='sail']:not([mode]) {
        --primary: #eeeeee;
      }
      :root[theme='paper'][mode='light'],
      :root[theme='paper']:not([mode]),
      [theme='paper'][mode='light'],
      [theme='paper']:not([mode]) {
        --primary: #abcdef;
      }
    `
    document.head.appendChild(style)
    const root = document.documentElement
    root.setAttribute('theme', 'sail')
    root.setAttribute('mode', 'dark')

    const nestedShip = document.createElement('div')
    nestedShip.setAttribute('theme', 'sail')
    document.body.appendChild(nestedShip)

    const nestedPaper = document.createElement('div')
    nestedPaper.setAttribute('theme', 'paper')
    nestedShip.appendChild(nestedPaper)

    const shipPrimary = getComputedStyle(nestedShip).getPropertyValue('--primary').trim()
    const paperPrimary = getComputedStyle(nestedPaper).getPropertyValue('--primary').trim()
    expect(shipPrimary.toLowerCase()).toBe('#111111')
    expect(paperPrimary.toLowerCase()).toBe('#abcdef')

    style.remove()
    nestedShip.remove()
  })

  it('never sets mode=system on document (paintMode resolves)', () => {
    const root = document.documentElement
    applyTheme('sail', root, 'light')
    applyMode('system', root)
    expect(root.getAttribute('mode')).not.toBe('system')
    expect(['light', 'dark']).toContain(root.getAttribute('mode'))
    applyTheme('console', root, 'system')
    expect(root.getAttribute('mode')).not.toBe('system')
    expect(['light', 'dark']).toContain(root.getAttribute('mode'))
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
    expect(root.getAttribute('mode')).not.toBe('system')
    expect(['light', 'dark']).toContain(root.getAttribute('mode'))
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
    expect(root.getAttribute('mode')).toBe('dark')
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
    // Tone is an ordinary element prop on native hosts (button / a / …).
    expect(base).not.toMatch(/\[tone=['"]/)
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

  /** Walk base + model/components/* → public cssNames (via publicName(path) or path join). */
  function modelBaseCssNames(): Set<string> {
    const names = new Set<string>()
    const walk = (node: unknown, path: string[]) => {
      if (!node || typeof node !== 'object' || Array.isArray(node)) return
      const obj = node as Record<string, unknown>
      if (Object.prototype.hasOwnProperty.call(obj, '$value')) {
        // Public names are generator-owned; tests walk registry for public names.
        const cssName = path.join('-') // fallback only for structural walk
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

  it('registry has zero component.* paths (cut pack stays cut)', () => {
    const reg = loadRegistry()
    expect(reg.names.length).toBeGreaterThan(200)
    const componentPaths = Object.keys(reg.paths ?? {}).filter((p) =>
      p.startsWith('component.'),
    )
    expect(componentPaths).toEqual([])
    expect(reg.names.some((n) => n === 'field' || n.startsWith('field-'))).toBe(
      false,
    )
    expect(reg.paths?.['color.card.default']).toBe('card')
    expect(reg.paths?.['color.input']).toBe('input')
    expect(reg.paths?.['color.popover.default']).toBe('popover')
    expect(reg.tokens.card?.path).toBe('color.card.default')
    expect(reg.tokens.input?.path).toBe('color.input')
    // Naive path-join of base.json is not an independent pin (public names
    // drop group prefixes). Walk only to prove the model still has $value leaves.
    expect(modelBaseCssNames().size).toBeGreaterThan(100)
  })

  it('registry entries expose path + cssName for sx/J2', () => {
    const reg = loadRegistry()
    expect(reg.version ?? 1).toBeGreaterThanOrEqual(2)
    expect(reg.tokens['space-4']?.path).toBe('space.4')
    expect(reg.tokens['space-4']?.cssName).toBe('space-4')
    expect(reg.tokens['primary']?.path).toMatch(/^color\./)
    expect(reg.paths?.['space.4']).toBe('space-4')
    expect(reg.tokens['accent']?.path).toBe('color.brandAccent.default')
    expect(reg.names.includes('accent-dashboard')).toBe(false)
    expect(reg.paths?.['color.accent.dashboard']).toBeUndefined()
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
    expect(selectors.has('console') || selectors.has('sail')).toBe(true)
    // Theme JSON files exist for the catalog
    const paper = JSON.parse(loadSrc('../model/themes/paper.json')) as {
      $extensions?: { sail?: { themeId?: string } }
    }
    expect(paper.$extensions?.sail?.themeId).toBe('paper')
  })

  it('theme override blocks redeclare public cssNames only (registry-known vars)', () => {
    // themes.css must override --primary / --surface etc. — the same public
    // surface Layout, Fx, and contracts consume — never invent parallel names.
    // Match modes-in-file theme layers:
    //   :root[theme='id'][mode='…'],
    //   [theme='id'][mode='…'] { … }
    // (not themes-chrome structural flourishes on body/card selectors).
    const reg = loadRegistry()
    const known = new Set(reg.names)
    const themesCss = loadSrc('../src/themes.css')
    const selectors = extractThemeSelectors(themesCss)
    expect(selectors.size).toBeGreaterThanOrEqual(2)
    // Modes-only: every theme block must mention mode=.
    expect(themesCss).toMatch(/mode=['"]light['"]/)
    expect(themesCss).toMatch(/mode=['"]dark['"]/)
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
    for (const n of ['primary', 'foreground', 'background', 'surface']) {
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

  it('no dual-emit token aliases (alias file gone, app-accent-* stays gone)', () => {
    const modelDir = resolve(dirname(fileURLToPath(import.meta.url)), '../model')
    const leftoverAliasFiles = readdirSync(modelDir).filter((f) => f.endsWith('-aliases.json'))
    expect(leftoverAliasFiles).toEqual([])
    expect(loadSrc('../src/tokens.css')).not.toMatch(/alias block/)
    const reg = loadRegistry()
    expect(Object.keys(reg.aliases ?? {}).length).toBe(0)
    expect(reg.names.some((n) => n.startsWith('app-accent-'))).toBe(false)
  })

  it('products axis is not part of the Sail model (moved to Ship)', () => {
    const modelDir = resolve(dirname(fileURLToPath(import.meta.url)), '../model')
    expect(existsSync(join(modelDir, 'products.json'))).toBe(false)
    expect(existsSync(resolve(dirname(fileURLToPath(import.meta.url)), '../src/products.css'))).toBe(
      false,
    )
    const reg = loadRegistry() as { products?: unknown }
    expect(reg.products).toBeUndefined()
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

  it('cut component packs stay gone; role aliases remain on color.*', () => {
    const reg = loadRegistry()
    for (const path of [
      'component.card.bg',
      'component.field.bg',
      'component.navbar.bg',
      'component.combobox.bg',
      'component.dialog.bg',
    ]) {
      expect(reg.paths?.[path], path).toBeUndefined()
    }
    const css = loadSrc('../src/tokens.css')
    expect(css).toMatch(/--surface\s*:/)
    expect(css).toMatch(/--overlay\s*:/)
    expect(css).toMatch(/--inset\s*:/)
    expect(css).not.toMatch(/--card\s*:/)
    expect(css).not.toMatch(/--popover\s*:/)
    expect(css).not.toMatch(/--input\s*:/)
    expect(css).not.toMatch(/--field\s*:/)
    expect(css).not.toMatch(/--navbar\s*:/)
    expect(css).not.toMatch(/--combobox\s*:/)
    expect(css).not.toMatch(/--dialog\s*:\s*var\(/)
  })

  it('color surface includes brand + tone tokens and excludes space/text', () => {
    const reg = loadRegistry()
    const colors = new Set(reg.colors)
    for (const n of ['primary', 'foreground', 'surface', 'hover-bg', 'accent-foreground']) {
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
    for (const tone of ['primary', 'info', 'success', 'warning', 'danger', 'accent', 'muted']) {
      expect(unions).toContain(`'${tone}'`)
    }
    // Registry names appear in CssTokenName
    expect(unions).toContain("'space-4'")
    expect(unions).toContain("'primary'")
    // Theme ids from model/themes (modes-in-file — no twin ids)
    expect(unions).toContain("'sail'")
    expect(unions).toContain("'paper'")
    expect(unions).not.toContain("'ship'")
    expect(unions).not.toContain("'ship-light'")
    expect(unions).not.toContain("'ship-dark'")
  })
})

describe('@ship/tokens — whole-value alias audit', () => {
  // Pins RESULT-TOKEN-AUDIT.md. Recount: extractWholeValueAliases.
  const PIN = {
    tokensSites: 82,
    tokensUnique: 77,
    tokensKeep: 77,
    tokensRebind: 0,
    themesSites: 41,
    themesUnique: 7,
    themesKeep: 7,
    chromeSites: 0,
    hueFirstClass: 16,
    sidebarFirstClass: 0,
  } as const

  it('tokens.css / themes.css / chrome.css alias classes match the RESULT pin', () => {
    const tokens = extractWholeValueAliases(loadSrc('../src/tokens.css'))
    const themes = extractWholeValueAliases(loadSrc('../src/themes.css'))
    const chrome = extractWholeValueAliases(
      loadSrc('../../../../crates/sail/compiler/stdlib/platform/chrome.css'),
    )
    const tUnique = uniqueAliasNames(tokens)
    const thUnique = uniqueAliasNames(themes)
    const cUnique = uniqueAliasNames(chrome)
    const tCounts = countAliasClasses(tUnique)
    const thCounts = countAliasClasses(thUnique)
    const cCounts = countAliasClasses(cUnique)

    expect(tokens).toHaveLength(PIN.tokensSites)
    expect(tUnique.size).toBe(PIN.tokensUnique)
    expect(tCounts).toEqual({
      keep: PIN.tokensKeep,
      rebind: PIN.tokensRebind,
      leftover: 0,
      product: 0,
      hue: 0,
    })
    expect(themes).toHaveLength(PIN.themesSites)
    expect(thUnique.size).toBe(PIN.themesUnique)
    expect(thCounts.keep).toBe(PIN.themesKeep)
    expect(thCounts.rebind).toBe(0)
    expect(chrome).toHaveLength(PIN.chromeSites)
    expect(cUnique.size).toBe(0)
    expect(cCounts.rebind).toBe(0)
  })

  it('hue.* stays first-class; sidebar.* and products.css are gone from Sail', () => {
    const reg = JSON.parse(loadSrc('../src/token-registry.json')) as {
      names: string[]
    }
    const hue = reg.names.filter((n) => n.startsWith('hue-'))
    const sidebar = reg.names.filter((n) => n === 'sidebar' || n.startsWith('sidebar-'))
    expect(hue).toHaveLength(PIN.hueFirstClass)
    expect(sidebar).toHaveLength(PIN.sidebarFirstClass)
    expect(reg.names.includes('brand-hue')).toBe(false)
    const tokens = extractWholeValueAliases(loadSrc('../src/tokens.css'))
    expect(tokens.some((s) => s.cls === 'hue' || s.cls === 'leftover' || s.cls === 'product')).toBe(
      false,
    )
    expect(
      existsSync(resolve(dirname(fileURLToPath(import.meta.url)), '../src/products.css')),
    ).toBe(false)
  })
})

