
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
  const ship = ext?.ship as Record<string, unknown> | undefined
  const id = ship?.themeId
  return typeof id === 'string' && id ? id : fallback
}

export function setThemeId(doc: Record<string, unknown>, id: string): Record<string, unknown> {
  const next = structuredClone(doc) as Record<string, unknown>
  const ext = {
    ...((next.$extensions as object) ?? {}),
  } as Record<string, unknown>
  const ship = {
    ...((ext.ship as object) ?? {}),
    themeId: id,
    layer: (ext.ship as Record<string, unknown> | undefined)?.layer ?? 'theme',
  }
  ext.ship = ship
  next.$extensions = ext
  return next
}

export function densityOf(doc: Record<string, unknown> | null): DensityId {
  if (!doc) return 'default'
  const ext = doc.$extensions as Record<string, unknown> | undefined
  const ship = ext?.ship as Record<string, unknown> | undefined
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
    ...((ext.ship as object) ?? {}),
    density,
  }
  ext.ship = ship
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
  for (const comp of ['button', 'card', 'input']) {
    const path = `component.${comp}.radius`
    const raw = getPathValue(doc, path, modeId)
    const isOverride = raw != null
    const name = comp.charAt(0).toUpperCase() + comp.slice(1)
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

/** Set $value at dotted path, creating intermediate objects. Mode-aware. */
export function setPathValue(
  doc: Record<string, unknown>,
  path: string,
  value: string,
  type = 'dimension',
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  const target = resolveWritePath(doc, path, modeId)
  const next = structuredClone(doc) as Record<string, unknown>
  const parts = target.split('.')
  let cur: Record<string, unknown> = next
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i]
    const child = cur[p]
    if (!child || typeof child !== 'object' || Array.isArray(child)) {
      cur[p] = {}
    }
    cur = cur[p] as Record<string, unknown>
  }
  const leaf = parts[parts.length - 1]
  const prev = (cur[leaf] as Record<string, unknown> | undefined) ?? {}
  cur[leaf] = { ...prev, $value: value, $type: prev.$type ?? type }
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
    const child = cur[p]
    if (!child || typeof child !== 'object') return next
    cur = child as Record<string, unknown>
  }
  delete cur[parts[parts.length - 1]]
  return next
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
  // Studio / ship themes: heading.font (display), body.font / font.body, mono stacks.
  const display =
    getPathValue(doc, 'typography.heading.font', modeId) ??
    getPathValue(doc, 'typography.font.display', modeId) ??
    getPathValue(doc, 'typography.font.display-app', modeId)
  const body =
    getPathValue(doc, 'typography.body.font', modeId) ??
    getPathValue(doc, 'typography.font.body', modeId) ??
    getPathValue(doc, 'typography.font.body-app', modeId)
  const mono =
    getPathValue(doc, 'typography.font.mono', modeId) ??
    getPathValue(doc, 'typography.font.mono-app', modeId) ??
    getPathValue(doc, 'typography.mono.font', modeId)
  return {
    display: familyName(display) || defaults.display,
    body: familyName(body) || defaults.body,
    mono: familyName(mono) || defaults.mono,
  }
}

function familyName(raw: string | null): string {
  if (!raw) return ''
  // Token alias — not a face name; leave empty so caller uses default.
  if (raw.trim().startsWith('{')) return ''
  // "\"Syne Variable\", …" or bare Fraunces, ui-serif, Georgia
  const m = raw.match(/"([^"]+)"/)
  if (m) return m[1].replace(/\s+Variable$/, '')
  return raw.split(',')[0]?.replace(/"/g, '').trim() ?? ''
}

/** Patch typography faces into theme + app stacks (mode-aware). */
export function patchTypeRoles(
  doc: Record<string, unknown>,
  roles: TypeRoles,
  modeId: 'light' | 'dark' = 'light',
): Record<string, unknown> {
  const stack = (name: string, fallback: string) =>
    `"${name}", "${fallback}", sans-serif`
  const displayStack = (name: string) =>
    `${name}, ui-serif, Georgia, serif`
  const mono = (name: string) => `"${name}", ui-monospace, monospace`
  // Prefer heading.font when the theme already uses it (studio/ship).
  const hasHeading =
    getPathValue(doc, 'typography.heading.font', modeId) != null ||
    !!(doc.modes && typeof doc.modes === 'object')
  let next = doc
  if (hasHeading) {
    next = setPathValue(
      next,
      'typography.heading.font',
      displayStack(roles.display),
      'fontFamily',
      modeId,
    )
  }
  next = setPathValue(
    next,
    'typography.font.display-app',
    stack(roles.display, 'Inter'),
    'fontFamily',
    modeId,
  )
  next = setPathValue(
    next,
    'typography.font.body-app',
    stack(roles.body, 'Inter'),
    'fontFamily',
    modeId,
  )
  // body.font alias target when present as concrete stack later
  next = setPathValue(
    next,
    'typography.body.font',
    stack(roles.body, 'Inter'),
    'fontFamily',
    modeId,
  )
  return setPathValue(
    next,
    'typography.font.mono-app',
    mono(roles.mono),
    'fontFamily',
    modeId,
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
 * Includes `$value` tokens and intentional ship extensions (`$extensions.ship.density`).
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
      const ship = (v as Record<string, unknown>).ship
      if (ship && typeof ship === 'object' && !Array.isArray(ship)) {
        const dens = (ship as Record<string, unknown>).density
        if (typeof dens === 'string' && dens) {
          const densPath = prefix
            ? `${prefix}.$extensions.ship.density`
            : '$extensions.ship.density'
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
        : path.startsWith('component.') || path === '$extensions.ship.density'
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
  const norm = projectRoot.replace(/\\/g, '/').replace(/\/$/, '')
  if (norm.startsWith('apps/')) return `${norm}/tokens/themes`
  const m = norm.match(/\/(apps\/[^/]+)$/)
  if (m) return `${m[1]}/tokens/themes`
  // Fallback: studio
  return 'apps/studio/tokens/themes'
}

export { formatPx, parseDimensionPx }
