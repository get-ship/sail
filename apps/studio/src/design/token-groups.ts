/**
 * Token group inventory from the DTCG registry (~400 leaves).
 * Groups drive Design primary nav; knobs are type-aware editors.
 */
import registry from '@ship/tokens/token-registry.json'

export type TokenKind =
  | 'color'
  | 'dimension'
  | 'number'
  | 'string'
  | 'fontFamily'
  | 'duration'
  | 'other'

export type TokenKnob = {
  path: string
  cssName: string
  type: string
  kind: TokenKind
  base: string
  unit?: string
  min?: number
  max?: number
  step?: number
  label: string
}

export type TokenGroup = {
  id: string
  label: string
  knobs: TokenKnob[]
}

type TokenEntry = {
  path?: string
  cssName?: string
  type?: string
  value?: string
}

type CompositeEntry = {
  name: string
  path: string
  type: string
  members: Record<string, string>
}

export type CompositeRef = {
  /** Public identity (`dialog.border`). */
  name: string
  /** Storage path (`component.dialog.border`). */
  path: string
  /** Member slot when `path` is a member leaf (`color`), else empty. */
  member: string
}

const GROUP_ORDER = [
  'color',
  'fill',
  'ink',
  'border',
  'space',
  'size',
  'type',
  'typography',
  'radius',
  'elevation',
  'shadow',
  'motion',
  'component',
  'icon',
  'density',
  'control',
  'interaction',
  'panel',
  'z',
  'scrollbar',
  'breakpoint',
] as const

const GROUP_LABELS: Record<string, string> = {
  color: 'Color',
  fill: 'Fill',
  ink: 'Ink',
  border: 'Border',
  space: 'Space',
  size: 'Size',
  type: 'Type',
  typography: 'Text',
  radius: 'Radius',
  elevation: 'Elevation',
  shadow: 'Shadow',
  motion: 'Motion',
  component: 'Components',
  icon: 'Icons',
  density: 'Density',
  control: 'Controls',
  interaction: 'Interaction',
  panel: 'Panels',
  z: 'Stacking',
  scrollbar: 'Scrollbar',
  breakpoint: 'Breakpoints',
}

function classify(type: string, value: string): TokenKind {
  const t = (type || '').toLowerCase()
  if (t === 'color') return 'color'
  if (t === 'dimension') return 'dimension'
  if (t === 'duration') return 'duration'
  if (t === 'fontfamily' || t === 'font-family') return 'fontFamily'
  if (t === 'number') return 'number'
  if (/^\d+(\.\d+)?(px|rem|em|ms|s|%)$/.test(value.trim())) return 'dimension'
  if (
    value.startsWith('#') ||
    value.startsWith('oklch') ||
    value.startsWith('rgb') ||
    value.startsWith('color-mix')
  ) {
    return 'color'
  }
  return 'other'
}

function parseDim(value: string): { num: number; unit: string } | null {
  const m = String(value)
    .trim()
    .match(/^(-?\d+(?:\.\d+)?)(px|rem|em|ms|s|%)?$/)
  if (!m) return null
  return { num: Number(m[1]), unit: m[2] || '' }
}

function dimBounds(
  unit: string,
  path: string,
): { min: number; max: number; step: number } {
  if (unit === 'ms') return { min: 0, max: 2000, step: 10 }
  if (unit === 's') return { min: 0, max: 5, step: 0.05 }
  if (unit === '%') return { min: 0, max: 100, step: 1 }
  if (path.startsWith('space.') || path.startsWith('radius.')) {
    return { min: 0, max: 64, step: 1 }
  }
  if (path.startsWith('icon.')) return { min: 8, max: 48, step: 1 }
  return { min: 0, max: 128, step: 1 }
}

/** Human label from DTCG path — never the raw oklch string. */
export function humanLabel(path: string): string {
  const parts = path.split('.').filter((p) => p && p !== 'default')
  if (parts.length === 0) return path
  const tail = parts[parts.length - 1] ?? path
  const prev = parts.length > 1 ? (parts[parts.length - 2] ?? '') : ''
  if (/^\d+$/.test(tail)) return prev ? `${titleCase(prev)} ${tail}` : tail
  if (tail === 'sm' || tail === 'md' || tail === 'lg' || tail === 'xl') {
    const size =
      tail === 'sm' ? 'Small' : tail === 'md' ? 'Medium' : tail === 'lg' ? 'Large' : 'Extra large'
    return prev ? `${titleCase(prev)} · ${size}` : size
  }
  // SPEC-D1: primary UI uses short graph label (chip / chip.foreground), not
  // raw component.chip.fg as the hero string.
  if (parts[0] === 'component' && parts.length >= 2) {
    const pack = parts[1]!
    const leaf = parts.length >= 3 ? parts.slice(2).join('.') : ''
    if (!leaf || leaf === 'bg' || leaf === 'track') return pack
    if (leaf === 'fg') return `${pack}.foreground`
    return `${pack}.${leaf}`
  }
  return titleCase(tail)
}

function titleCase(s: string): string {
  return s
    .replace(/[-_]/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .split(' ')
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ')
}

/**
 * Short color readout for UI — never primary-display raw oklch.
 * Falls back to css var name.
 */
export function colorSummary(value: string, cssName: string): string {
  const v = value.trim()
  const m = v.match(/^oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)/i)
  if (m) {
    const L = Number(m[1])
    const C = Number(m[2])
    const H = Number(m[3])
    const light = L >= 0.75 ? 'light' : L >= 0.45 ? 'mid' : 'dark'
    const chroma = C < 0.04 ? 'neutral' : C < 0.12 ? 'soft' : 'vivid'
    return `${light} ${chroma} · hue ${Math.round(H)}`
  }
  if (v.startsWith('color-mix') || v.startsWith('var(')) return cssName
  if (v.startsWith('#')) return cssName
  return cssName || 'color'
}

function pathGroup(path: string): string {
  return path.split('.')[0] || 'other'
}

function listComposites(): CompositeEntry[] {
  const raw = (registry as { composites?: Record<string, Omit<CompositeEntry, 'name'>> }).composites
    || {}
  return Object.entries(raw).map(([name, entry]) => ({
    name,
    path: entry.path || name,
    type: entry.type || '',
    members: entry.members || {},
  }))
}

const ALL_COMPOSITES = listComposites()

/** Registry composite for a storage path, public identity, or member css/path. */
export function compositeForPath(path: string): CompositeRef | null {
  const bare = path.replace(/^modes\.(light|dark)\./, '').replace(/\.\$value$/, '')
  if (!bare) return null
  for (const c of ALL_COMPOSITES) {
    if (bare === c.path || bare === c.name) {
      return { name: c.name, path: c.path, member: '' }
    }
    for (const [slot, css] of Object.entries(c.members)) {
      if (bare === `${c.path}.${slot}` || bare === css || bare === `${c.name}.${slot}`) {
        return { name: c.name, path: c.path, member: slot }
      }
    }
  }
  return null
}

/** Inspector label: folded composite members as one face (`dialog.border · color`). */
export function componentKnobLabel(path: string, fallback: string): string {
  const c = compositeForPath(path)
  if (c?.member) return `${c.name} · ${c.member}`
  if (c) return c.name
  return fallback
}

function buildKnobs(): TokenKnob[] {
  const raw = (registry as { tokens?: Record<string, TokenEntry> }).tokens || {}
  const out: TokenKnob[] = []
  for (const [cssKey, entry] of Object.entries(raw)) {
    const path = entry.path || cssKey
    const cssName = entry.cssName || cssKey
    const type = entry.type || 'unknown'
    const base = String(entry.value ?? '')
    const kind = classify(type, base)
    const dim = parseDim(base)
    const knob: TokenKnob = {
      path,
      cssName,
      type,
      kind,
      base,
      label: componentKnobLabel(path, humanLabel(path)),
    }
    if (dim && (kind === 'dimension' || kind === 'duration' || kind === 'number')) {
      const b = dimBounds(dim.unit, path)
      knob.unit = dim.unit
      knob.min = b.min
      knob.max = b.max
      knob.step = b.step
    }
    out.push(knob)
  }
  out.sort((a, b) => a.path.localeCompare(b.path))
  return out
}

const ALL_KNOBS = buildKnobs()
const BY_PATH = new Map(ALL_KNOBS.map((k) => [k.path, k]))
const BY_CSS = new Map(ALL_KNOBS.map((k) => [k.cssName, k]))

/**
 * SPEC-D1 rest-only: component storage leaves whose leaf id contains hover/focus
 * are not Style surface slots and L1 does not spend them. Park from studio
 * editable rest knobs until a later state-surface decision (costume write =
 * silent no-op for look).
 *
 * Pack names like `hover-card` are fine — only the leaf segment is checked.
 */
export function isParkedComponentStateLeaf(path: string): boolean {
  const bare = path
    .replace(/^modes\.(light|dark)\./, '')
    .replace(/\.\$value$/, '')
  if (!bare.startsWith('component.')) return false
  const parts = bare.split('.')
  if (parts.length < 3) return false
  const leaf = parts.slice(2).join('.')
  return leaf.includes('hover') || leaf.includes('focus')
}

export function listTokenGroups(): TokenGroup[] {
  const map = new Map<string, TokenKnob[]>()
  for (const k of ALL_KNOBS) {
    if (isParkedComponentStateLeaf(k.path)) continue
    const id = pathGroup(k.path)
    if (!map.has(id)) map.set(id, [])
    map.get(id)!.push(k)
  }
  const ids = [
    ...GROUP_ORDER.filter((g) => map.has(g)),
    ...[...map.keys()].filter((g) => !(GROUP_ORDER as readonly string[]).includes(g)).sort(),
  ]
  return ids.map((id) => ({
    id,
    label: GROUP_LABELS[id] || titleCase(id),
    knobs: map.get(id) || [],
  }))
}

export function groupKnobs(groupId: string): TokenKnob[] {
  return ALL_KNOBS.filter((k) => {
    if (pathGroup(k.path) !== groupId) return false
    // Components facet / design group: hide parked state storage leaves.
    if (groupId === 'component' && isParkedComponentStateLeaf(k.path)) return false
    return true
  })
}


export function getKnob(path: string): TokenKnob | undefined {
  return BY_PATH.get(path) || BY_CSS.get(path)
}

export function allKnobs(): TokenKnob[] {
  return ALL_KNOBS
}

export function tokenStats(): { tokenCount: number; groupCount: number } {
  return { tokenCount: ALL_KNOBS.length, groupCount: listTokenGroups().length }
}

/** Groups that use the scale-family slider engine. */
export function isScaleFamilyGroup(groupId: string): boolean {
  return groupId === 'radius' || groupId === 'icon'
}

export const DEFAULT_GROUP_ID = 'color'
