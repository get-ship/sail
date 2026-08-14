
import {
  applyRadiusToDoc,
  applyScaleToDoc,
  formatPx,
  iconFamilyFromDoc,
  parseDimensionPx,
  radiusFamilyFromDoc,
} from './scale'
import type {
  DensityId,
  DesignDiffRow,
  InheritNode,
  ScaleFamily,
  TypeRoles,
} from './types'

export function parseThemeJson(content: string): Record<string, unknown> {
  const v = JSON.parse(content) as unknown
  if (!v || typeof v !== 'object' || Array.isArray(v)) {
    throw new Error('theme JSON root must be an object')
  }
  return v as Record<string, unknown>
}

export function stringifyTheme(doc: Record<string, unknown>): string {
  return `${JSON.stringify(doc, null, 2)}\n`
}

export function themeIdOf(doc: Record<string, unknown>, fallback: string): string {
  const ext = doc.$extensions as Record<string, unknown> | undefined
  const ship = ext?.sail as Record<string, unknown> | undefined
  const id = ship?.themeId
  return typeof id === 'string' && id ? id : fallback
}

export function setThemeId(doc: Record<string, unknown>, id: string): Record<string, unknown> {
  const next = structuredClone(doc) as Record<string, unknown>
  const ext = {
    ...((next.$extensions as object) ?? {}),
  } as Record<string, unknown>
  const ship = {
    ...((ext.sail as object) ?? {}),
    themeId: id,
    layer: (ext.sail as Record<string, unknown> | undefined)?.layer ?? 'theme',
  }
  ext.sail = ship
  next.$extensions = ext
  return next
}

export function densityOf(doc: Record<string, unknown> | null): DensityId {
  if (!doc) return 'default'
  const ext = doc.$extensions as Record<string, unknown> | undefined
  const ship = ext?.sail as Record<string, unknown> | undefined
  const d = ship?.density
  if (d === 'compact' || d === 'spacious' || d === 'default') return d
  return 'default'
}

export function setDensity(
  doc: Record<string, unknown>,
  density: DensityId,
): Record<string, unknown> {
  const next = structuredClone(doc) as Record<string, unknown>
  const ext = {
    ...((next.$extensions as object) ?? {}),
  } as Record<string, unknown>
  const ship = {
    ...((ext.sail as object) ?? {}),
    density,
  }
  ext.sail = ship
  next.$extensions = ext
  return next
}

/** Read a scale group object from root or modes (prefer active mode). */
function extractScaleGroup(
  doc: Record<string, unknown> | null,
  group: string,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> | null {
  if (!doc) return null
  const modes = doc.modes as Record<string, unknown> | undefined
  const active = modes?.[modeId] as Record<string, unknown> | undefined
  if (active?.[group] && typeof active[group] === 'object') {
    return active[group] as Record<string, unknown>
  }
  if (doc[group] && typeof doc[group] === 'object' && !Array.isArray(doc[group])) {
    return doc[group] as Record<string, unknown>
  }
  const other = modeId === 'dark' ? 'light' : 'dark'
  const fallback = modes?.[other] as Record<string, unknown> | undefined
  if (fallback?.[group] && typeof fallback[group] === 'object') {
    return fallback[group] as Record<string, unknown>
  }
  return null
}

/** Read radius family from active mode, then root, then other mode. */
export function extractRadius(
  doc: Record<string, unknown> | null,
  modeId: 'light' | 'dark' = 'light',
): ScaleFamily {
  return radiusFamilyFromDoc(extractScaleGroup(doc, 'radius', modeId))
}

export function patchRadius(
  doc: Record<string, unknown>,
  family: ScaleFamily,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  return applyRadiusToDoc(doc, family, modeId)
}

export function extractIconScale(
  doc: Record<string, unknown> | null,
  modeId: 'light' | 'dark' = 'light',
): ScaleFamily {
  return iconFamilyFromDoc(extractScaleGroup(doc, 'icon', modeId))
}

export function patchIconScale(
  doc: Record<string, unknown>,
  family: ScaleFamily,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  return applyScaleToDoc(doc, { ...family, path: 'icon' }, modeId)
}

/**
 * Inheritance chain for radius: base catalog → theme scale → component override.
 * Components: button, card, input (specimen-relevant).
 */
/**
 * Full Sail L1 stdlib catalog (Themes Components rail SSOT).
 * Keep in sync with `crates/sail/compiler/src/stdlib.rs` `SOURCES` tags.
 * Tag `a` is the anchor element (file: anchor.sail).
 */
export const L1_CATALOG = [
  'spinner',
  'skeleton',
  'button',
  'input',
  'textarea',
  'select',
  'dialog',
  'accordion',
  'table',
  'fieldset',
  'legend',
  'progress',
  'meter',
  'a',
  'separator',
  'kbd',
  'icon',
  'label',
  'card',
  'switch',
  'badge',
  'avatar',
  'tooltip',
  'alert',
  'empty-state',
  'error-state',
  'error-boundary',
  'code-block',
  'chip',
  'field',
  'slider',
  'radio',
  'radio-group',
  'tabs',
  'tab-list',
  'tab',
  'tab-panel',
  'toggle-group',
  'toggle',
  'scroll-area',
  'ticker',
  'theme-picker',
  'navbar',
  'pagination',
  'page',
  'breadcrumb',
  'popover',
  'prose',
  'combobox',
  'dropdown',
  'menu-item',
  'context-menu',
  'hover-card',
  'toast-region',
  'toast',
] as const

export type L1Id = (typeof L1_CATALOG)[number]

/** Display label for an L1 tag (`a` → Anchor, `menu-item` → Menu Item). */
export function l1Label(id: string): string {
  if (id === 'a') return 'Anchor'
  return id
    .split('-')
    .map((p) => (p ? p.charAt(0).toUpperCase() + p.slice(1) : p))
    .join(' ')
}

/**
 * Inheritance chain for radius: base catalog → theme scale → per-L1 override.
 * One node per L1 in {@link L1_CATALOG} so Themes Components + Design stay aligned.
 */
export function radiusInheritance(
  doc: Record<string, unknown> | null,
  modeId: 'light' | 'dark' = 'light',
): InheritNode[] {
  const family = extractRadius(doc, modeId)
  const lg = family.steps.find((s) => s.key === 'lg')
  const globalVal = lg?.raw ?? '10px'
  const nodes: InheritNode[] = [
    {
      id: 'base',
      label: 'Base · catalog',
      value: '10px',
      isOverride: false,
      path: 'radius.lg',
      canReset: false,
    },
    {
      id: 'theme',
      label: `Theme · ${parseDimensionPx(globalVal) ?? globalVal}`,
      value: globalVal,
      isOverride: false,
      path: 'radius.lg',
      canReset: false,
    },
  ]
  for (const comp of L1_CATALOG) {
    const path = `component.${comp}.radius`
    const raw = getPathValue(doc, path, modeId)
    const isOverride = raw != null
    const name = l1Label(comp)
    nodes.push({
      id: comp,
      label: isOverride
        ? `Override · ${name} · ${raw}`
        : `Override · ${name} · inherits theme`,
      value: raw,
      isOverride,
      path,
      canReset: isOverride,
    })
  }
  return nodes
}

function walkPath(doc: Record<string, unknown> | null, path: string): unknown {
  if (!doc || !path) return null
  const parts = path.split('.')
  let cur: unknown = doc
  for (const p of parts) {
    if (!cur || typeof cur !== 'object' || Array.isArray(cur)) return null
    cur = (cur as Record<string, unknown>)[p]
  }
  return cur
}

function leafValue(node: unknown): string | null {
  if (!node || typeof node !== 'object' || Array.isArray(node)) return null
  const v = (node as Record<string, unknown>).$value
  return v != null ? String(v) : null
}

/**
 * Read candidates for a logical token path.
 * Prefer root, then active mode, then the other mode (read fallback only).
 */
export function candidatePaths(
  path: string,
  modeId: 'light' | 'dark' = 'light',
): string[] {
  if (!path || path.startsWith('$') || path.startsWith('modes.')) return [path]
  const other = modeId === 'dark' ? 'light' : 'dark'
  const out = [path, `modes.${modeId}.${path}`, `modes.${other}.${path}`]
  return [...new Set(out)]
}

/**
 * Write candidates — never the inactive mode.
 * Prefer existing root leaf, then active mode leaf; create under active when modes-in-file.
 */
export function writeCandidatePaths(
  path: string,
  modeId: 'light' | 'dark' = 'light',
): string[] {
  if (!path || path.startsWith('$') || path.startsWith('modes.')) return [path]
  return [path, `modes.${modeId}.${path}`]
}

/** Resolve $value at logical path, walking modes when needed (read may fall back). */
export function getPathValue(
  doc: Record<string, unknown> | null,
  path: string,
  modeId: 'light' | 'dark' = 'light',
): string | null {
  if (!doc) return null
  for (const p of candidatePaths(path, modeId)) {
    const v = leafValue(walkPath(doc, p))
    if (v != null) return v
  }
  return null
}

/** Physical path for write-back — never clobbers the inactive mode. */
export function resolveWritePath(
  doc: Record<string, unknown> | null,
  path: string,
  modeId: 'light' | 'dark' = 'light',
): string {
  if (!doc) return path
  for (const p of writeCandidatePaths(path, modeId)) {
    if (leafValue(walkPath(doc, p)) != null) return p
  }
  // Create under active mode when the theme is modes-in-file.
  if (doc.modes && typeof doc.modes === 'object') {
    return `modes.${modeId}.${path}`
  }
  return path
}

/** Set $value at dotted path, creating intermediate objects. Mode-aware.
 * Public CSS names come from generator `publicName(path)` at lower time —
 * do not stamp `$extensions.*.cssName` on theme leaves.
 */
export function setPathValue(
  doc: Record<string, unknown>,
  path: string,
  value: string,
  type = 'dimension',
  modeId: 'light' | 'dark' = 'light',
  _cssName?: string,
): Record<string, unknown> {
  const target = resolveWritePath(doc, path, modeId)
  const next = structuredClone(doc) as Record<string, unknown>
  const parts = target.split('.')
  let cur: Record<string, unknown> = next
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i]
    if (p == null) continue
    const child = cur[p]
    if (!child || typeof child !== 'object' || Array.isArray(child)) {
      cur[p] = {}
    }
    cur = cur[p] as Record<string, unknown>
  }
  const leaf = parts[parts.length - 1]
  if (leaf == null) return next
  const prev = (cur[leaf] as Record<string, unknown> | undefined) ?? {}
  // Preserve non-cssName extensions if present; never stamp cssName.
  const prevExt =
    prev.$extensions && typeof prev.$extensions === 'object' && !Array.isArray(prev.$extensions)
      ? { ...(prev.$extensions as Record<string, unknown>) }
      : undefined
  if (prevExt) {
    const sail = prevExt.sail
    if (sail && typeof sail === 'object' && !Array.isArray(sail)) {
      const nextSail = { ...(sail as Record<string, unknown>) }
      delete nextSail.cssName
      if (Object.keys(nextSail).length === 0) delete prevExt.sail
      else prevExt.sail = nextSail
    }
    if (Object.keys(prevExt).length === 0) {
      cur[leaf] = {
        ...prev,
        $value: value,
        $type: prev.$type ?? type,
      }
      delete (cur[leaf] as Record<string, unknown>).$extensions
    } else {
      cur[leaf] = {
        ...prev,
        $value: value,
        $type: prev.$type ?? type,
        $extensions: prevExt,
      }
    }
  } else {
    cur[leaf] = {
      ...prev,
      $value: value,
      $type: prev.$type ?? type,
    }
    delete (cur[leaf] as Record<string, unknown>).$extensions
  }
  return next
}

/** Remove a leaf path (reset inheritance). Mode-aware. */
export function deletePath(
  doc: Record<string, unknown>,
  path: string,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  const target = resolveWritePath(doc, path, modeId)
  const next = structuredClone(doc) as Record<string, unknown>
  const parts = target.split('.')
  let cur: Record<string, unknown> = next
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i]
    if (p == null) return next
    const child = cur[p]
    if (!child || typeof child !== 'object') return next
    cur = child as Record<string, unknown>
  }
  const delLeaf = parts[parts.length - 1]
  if (delLeaf != null) delete cur[delLeaf]
  return next
}

/**
 * Public cssName → DTCG path for type-role leaves (token-registry paths).
 * Used to walk `{alias}` when extracting faces from a theme doc.
 */
const FONT_CSS_TO_PATH: Record<string, string> = {
  'display-font': 'typography.displayFont',
  'heading-font': 'typography.heading.font',
  'body-font': 'typography.body.font',
  'mono-font': 'typography.monoFont',
  'font-display': 'typography.font.display',
  'font-display-app': 'typography.font.display-app',
  'font-body': 'typography.font.body',
  'font-body-app': 'typography.font.body-app',
  'font-mono': 'typography.font.mono',
  'font-mono-app': 'typography.font.mono-app',
}

/**
 * Base-layer stack names that themes alias into (tokens model base / fonts.ts).
 * Not product CSS — identity for honest role labels when the DTCG leaf is `{cssName}`.
 */
const PLATFORM_STACK_FACE: Record<string, string> = {
  'font-press-start': 'Press Start 2P',
  'font-press-start-app': 'Press Start 2P',
  'font-display-app': 'Syne',
  'font-display': 'Syne',
  'font-body-app': 'DM Sans',
  'font-body': 'DM Sans',
  'font-mono-app': 'JetBrains Mono',
  'font-mono': 'JetBrains Mono',
}

export function extractTypeRoles(
  doc: Record<string, unknown> | null,
  modeId: 'light' | 'dark' = 'light',
): TypeRoles {
  const defaults: TypeRoles = {
    display: 'Fraunces',
    body: 'DM Sans',
    mono: 'JetBrains Mono',
  }
  if (!doc) return defaults
  // Prefer public role leaves (sx uses --display-font / --body-font / --mono-font).
  // Include displayFont — 8bit/brut bind display here; heading alone is often an alias.
  const displayRaw =
    getPathValue(doc, 'typography.displayFont', modeId) ??
    getPathValue(doc, 'typography.heading.font', modeId) ??
    getPathValue(doc, 'typography.font.display', modeId) ??
    getPathValue(doc, 'typography.font.display-app', modeId)
  const bodyRaw =
    getPathValue(doc, 'typography.body.font', modeId) ??
    getPathValue(doc, 'typography.font.body', modeId) ??
    getPathValue(doc, 'typography.font.body-app', modeId)
  const monoRaw =
    getPathValue(doc, 'typography.monoFont', modeId) ??
    getPathValue(doc, 'typography.font.mono', modeId) ??
    getPathValue(doc, 'typography.font.mono-app', modeId) ??
    getPathValue(doc, 'typography.mono.font', modeId)
  return {
    display: resolveFaceName(doc, displayRaw, modeId) || defaults.display,
    body: resolveFaceName(doc, bodyRaw, modeId) || defaults.body,
    mono: resolveFaceName(doc, monoRaw, modeId) || defaults.mono,
  }
}

/**
 * True CSS cascade form of a token (generator emit). Never a UI face name.
 * Product model uses DTCG `{font-display}` or concrete stacks; CSS uses var(--…).
 */
function cssVarTokenName(raw: string): string | null {
  const m = raw.trim().match(/^var\(\s*--([a-zA-Z0-9_-]+)\s*(?:,[^)]*)?\)$/)
  return m?.[1] ?? null
}

/** First family name from a concrete stack: `"Fraunces", …` or bare `Fraunces, serif`. */
function familyName(raw: string | null): string {
  if (!raw) return ''
  const t = raw.trim()
  // Reject aliases + generated cascade — never treat CSS as a font face label.
  if (!t || t.startsWith('{') || t.includes('var(') || t.startsWith('--')) return ''
  const m = t.match(/"([^"]+)"/)
  if (m?.[1]) return m[1].replace(/\s+Variable$/, '')
  const first = t.split(',')[0]?.replace(/"/g, '').trim() ?? ''
  if (!first || first.includes('var(') || first.startsWith('--')) return ''
  return first
}

/**
 * Resolve a token ref (`{cssName}` or cascade `var(--cssName)`) to a face name.
 */
function resolveTokenRef(
  doc: Record<string, unknown>,
  ref: string,
  modeId: 'light' | 'dark',
  depth: number,
): string {
  if (!ref || depth > 6) return ''
  const platform = PLATFORM_STACK_FACE[ref]
  if (platform) return platform
  const path = FONT_CSS_TO_PATH[ref] ?? (ref.includes('.') ? ref : null)
  if (path) {
    return resolveFaceName(doc, getPathValue(doc, path, modeId), modeId, depth + 1)
  }
  return ''
}

/**
 * Resolve a DTCG font $value to a UI face name.
 * Walks whole-value `{alias}` and cascade `var(--cssName)` (never keeps var() as a face).
 * Never invents Fraunces when the theme aliases to Press Start / mono.
 */
function resolveFaceName(
  doc: Record<string, unknown>,
  raw: string | null,
  modeId: 'light' | 'dark',
  depth = 0,
): string {
  if (!raw) return ''
  const t = raw.trim()
  if (!t) return ''
  if (t.startsWith('{') && t.endsWith('}')) {
    return resolveTokenRef(doc, t.slice(1, -1), modeId, depth)
  }
  // Generated CSS may leak into docs / inspector as var(--font-display). That is emit,
  // not a face — walk the public name like a DTCG alias. Never return "var(--…)".
  const cssName = cssVarTokenName(t)
  if (cssName) {
    return resolveTokenRef(doc, cssName, modeId, depth)
  }
  // Stacks that start with a var (e.g. var(--font-display), sans-serif) — resolve head only.
  if (t.includes('var(')) {
    const head = t.match(/var\(\s*--([a-zA-Z0-9_-]+)/)
    if (head?.[1]) return resolveTokenRef(doc, head[1], modeId, depth)
    return ''
  }
  return familyName(t)
}

/** Patch typography faces into public role leaves + app stacks (mode-aware). */
export function patchTypeRoles(
  doc: Record<string, unknown>,
  roles: TypeRoles,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  // Always quote family names so multi-word faces (Cal Sans, Press Start 2P) parse.
  const stack = (name: string, fallback: string) =>
    `"${name}", "${fallback}", sans-serif`
  const displayStack = (name: string) => `"${name}", ui-serif, Georgia, serif`
  const monoStack = (name: string) => `"${name}", ui-monospace, monospace`

  let next = doc
  // Public cascade names sx fontFamily(display|body|mono|heading) resolve to.
  // Must be **concrete** stacks so (1) applyScope paints faces and (2) SailTheme
  // Generate rewrites theme table leaves — not leftover {font-press-start} aliases.
  next = setPathValue(
    next,
    'typography.displayFont',
    displayStack(roles.display),
    'fontFamily',
    modeId,
    'display-font',
  )
  next = setPathValue(
    next,
    'typography.heading.font',
    displayStack(roles.display),
    'fontFamily',
    modeId,
    'heading-font',
  )
  next = setPathValue(
    next,
    'typography.body.font',
    stack(roles.body, 'Inter'),
    'fontFamily',
    modeId,
    'body-font',
  )
  next = setPathValue(
    next,
    'typography.monoFont',
    monoStack(roles.mono),
    'fontFamily',
    modeId,
    'mono-font',
  )
  // App stacks + mid-tier names themes often rebind (console/8bit → mono/press-start).
  next = setPathValue(
    next,
    'typography.font.display-app',
    stack(roles.display, 'Inter'),
    'fontFamily',
    modeId,
    'font-display-app',
  )
  next = setPathValue(
    next,
    'typography.font.display',
    stack(roles.display, 'Inter'),
    'fontFamily',
    modeId,
    'font-display',
  )
  next = setPathValue(
    next,
    'typography.font.body-app',
    stack(roles.body, 'Inter'),
    'fontFamily',
    modeId,
    'font-body-app',
  )
  next = setPathValue(
    next,
    'typography.font.body',
    stack(roles.body, 'Inter'),
    'fontFamily',
    modeId,
    'font-body',
  )
  next = setPathValue(
    next,
    'typography.font.mono-app',
    monoStack(roles.mono),
    'fontFamily',
    modeId,
    'font-mono-app',
  )
  return setPathValue(
    next,
    'typography.font.mono',
    monoStack(roles.mono),
    'fontFamily',
    modeId,
    'font-mono',
  )
}

/** Diff two theme docs as path → $value rows (shallow leaf walk + ship density). */
export function diffThemeDocs(
  from: Record<string, unknown> | null,
  to: Record<string, unknown> | null,
): DesignDiffRow[] {
  const a = flattenValues(from)
  const b = flattenValues(to)
  const keys = new Set([...a.keys(), ...b.keys()])
  const rows: DesignDiffRow[] = []
  for (const path of [...keys].sort()) {
    const fa = a.get(path) ?? ''
    const tb = b.get(path) ?? ''
    if (fa !== tb) rows.push({ path, from: fa, to: tb })
  }
  return rows
}

/**
 * Flatten DTCG leaves for export/diff.
 * Includes `$value` tokens and intentional ship extensions (`$extensions.sail.density`).
 * Never reads CSS.
 */
export function flattenValues(
  doc: Record<string, unknown> | null,
  prefix = '',
  out = new Map<string, string>(),
): Map<string, string> {
  if (!doc) return out
  for (const [k, v] of Object.entries(doc)) {
    // Intentional ship density mode — density done-when requires this path in diffs.
    if (k === '$extensions' && v && typeof v === 'object' && !Array.isArray(v)) {
      const ship = (v as Record<string, unknown>).sail
      if (ship && typeof ship === 'object' && !Array.isArray(ship)) {
        const dens = (ship as Record<string, unknown>).density
        if (typeof dens === 'string' && dens) {
          const densPath = prefix
            ? `${prefix}.$extensions.sail.density`
            : '$extensions.sail.density'
          out.set(densPath, dens)
        }
      }
      continue
    }
    if (k.startsWith('$') && k !== '$value') continue
    const path = prefix ? `${prefix}.${k}` : k
    if (v && typeof v === 'object' && !Array.isArray(v)) {
      const obj = v as Record<string, unknown>
      if ('$value' in obj) {
        out.set(path, String(obj.$value))
      } else {
        flattenValues(obj, path, out)
      }
    }
  }
  return out
}

/** Dirty path count vs baseline, or component + density overrides. */
export function countOverrides(
  doc: Record<string, unknown> | null,
  baseline: Record<string, unknown> | null = null,
): number {
  if (!doc) return 0
  if (baseline) {
    return diffThemeDocs(baseline, doc).length
  }
  let n = 0
  const flat = flattenValues(doc)
  for (const path of flat.keys()) {
    if (path.startsWith('component.')) n++
  }
  if (densityOf(doc) !== 'default') n++
  return n
}

/** Token inventory rows (search + overrides-only). */
export function listTokenRows(
  doc: Record<string, unknown> | null,
  baseline: Record<string, unknown> | null,
  opts?: { search?: string; overridesOnly?: boolean },
): { path: string; value: string; dirty: boolean }[] {
  const flat = flattenValues(doc)
  const base = flattenValues(baseline)
  const q = (opts?.search ?? '').trim().toLowerCase()
  const rows: { path: string; value: string; dirty: boolean }[] = []
  for (const [path, value] of [...flat.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
    const dirty = (base.get(path) ?? '') !== value || (base.has(path) === false && !!value)
    // When baseline missing, treat component + density as dirty candidates for filter.
    const isOverride =
      baseline != null
        ? dirty
        : path.startsWith('component.') || path === '$extensions.sail.density'
    if (opts?.overridesOnly && !isOverride) continue
    if (q && !path.toLowerCase().includes(q) && !value.toLowerCase().includes(q)) continue
    rows.push({ path, value, dirty: baseline != null ? dirty : isOverride })
  }
  return rows
}

/** Catalog theme basenames → ThemeFile list. */
export function themeFilesFromList(
  files: string[],
  dir: string,
  catalog: boolean,
): { file: string; id: string; path: string; catalog: boolean }[] {
  return files
    .filter((f) => f.endsWith('.json') && !f.startsWith('.'))
    .map((file) => ({
      file,
      id: file.replace(/\.json$/, ''),
      path: `${dir.replace(/\/$/, '')}/${file}`,
      catalog,
    }))
    .sort((a, b) => a.id.localeCompare(b.id))
}

export function projectThemesDir(projectRoot: string): string {
  // projectRoot may be apps/studio or absolute; keep workspace-relative when possible.
  // Project theme JSON lives at <project>/themes (EXT-1 E4).
  const norm = projectRoot.replace(/\\/g, '/').replace(/\/$/, '')
  if (norm.startsWith('apps/')) return `${norm}/themes`
  const m = norm.match(/\/(apps\/[^/]+(?:\/[^/]+)*)$/)
  if (m) return `${m[1]}/themes`
  if (norm && !norm.startsWith('/') && !norm.includes(':')) {
    // Workspace-relative external or monorepo project root.
    return `${norm}/themes`
  }
  // Fallback: studio theme id
  return 'apps/studio/themes'
}

export { formatPx, parseDimensionPx }
