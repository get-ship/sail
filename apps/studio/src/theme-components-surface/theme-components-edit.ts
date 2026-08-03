/**
 * Themes → Components facet — L1 skinning from token **registry** SSOT.
 * - List only faces that have component.* tokens (or explicit global hooks).
 * - Composite faces (tabs, toast, radio) collapse sub-tags.
 * - Writable knobs from registry; global color.* can be edited from here too.
 * - Paths shown as DTCG paths — never CSS var names as product labels.
 * - No free-text radius as the primary (or only) control.
 */
import { signal } from '@ship/sail-runtime'
import {
  L1_CATALOG,
  allKnobs,
  clearTokenValue,
  getDesignState,
  getPathValue,
  l1Label,
  setTokenValue,
  toPickerHex,
  type L1Id,
  type TokenKnob,
} from '@ship/studio/design'
import type { ThemeSession } from '../theme-session/theme-session'
import {
  buildStageGates,
  viewerMetaFor,
  type ViewerPropRow,
} from './theme-l1-viewer-meta'

/** Registry tag → face id (collapse subcomponents). */
const FACE_MERGE: Record<string, string> = {
  tab: 'tabs',
  'tab-list': 'tabs',
  'tab-panel': 'tabs',
  toast: 'toast',
  'toast-region': 'toast',
  radio: 'radio-group',
  'radio-group': 'radio-group',
}

/** Globals useful when customizing a face (override global or set component leaf). */
const FACE_GLOBALS: Record<string, { path: string; label: string }[]> = {
  alert: [
    { path: 'color.success.default', label: 'Success (global)' },
    { path: 'color.warning.default', label: 'Warning (global)' },
    { path: 'color.destructive.default', label: 'Danger (global)' },
    { path: 'color.info.default', label: 'Info (global)' },
    { path: 'color.muted.foreground', label: 'Muted (global)' },
    { path: 'color.surface.1', label: 'Surface 1 (global)' },
    { path: 'color.foreground', label: 'Foreground (global)' },
  ],
  badge: [
    { path: 'color.primary.default', label: 'Primary (global)' },
    { path: 'color.muted.foreground', label: 'Muted (global)' },
  ],
  button: [
    { path: 'color.primary.default', label: 'Primary (global)' },
    { path: 'color.secondary.default', label: 'Secondary (global)' },
    { path: 'color.destructive.default', label: 'Danger (global)' },
    { path: 'color.success.default', label: 'Success (global)' },
    { path: 'radius.lg', label: 'Radius lg (global)' },
  ],
  card: [
    { path: 'color.surface.3', label: 'Card surface (global)' },
    { path: 'color.foreground', label: 'Foreground (global)' },
    { path: 'radius.lg', label: 'Radius lg (global)' },
  ],
  switch: [
    { path: 'color.primary.default', label: 'Primary (global)' },
    { path: 'color.border.default', label: 'Border (global)' },
  ],
  tabs: [
    { path: 'color.primary.default', label: 'Primary (global)' },
    { path: 'color.muted.foreground', label: 'Muted (global)' },
    { path: 'color.surface.1', label: 'Surface 1 (global)' },
  ],
  input: [
    { path: 'color.border.default', label: 'Border (global)' },
    { path: 'color.ring.default', label: 'Ring (global)' },
    { path: 'radius.md', label: 'Radius md (global)' },
  ],
}

const FACE_HINTS: Record<string, string> = {
  alert: 'Callouts · tone maps to globals + component leaves',
  badge: 'Status chips',
  button: 'Actions — globals until component.button.* exists',
  card: 'Surfaces · glass leaves',
  tabs: 'Tabs (list + tab + panel as one face)',
  toast: 'Toasts + region',
  'radio-group': 'Radio set',
  switch: 'Binary toggles',
  chip: 'Filters and tags',
  avatar: 'Face / initials',
  input: 'Fields (globals + control radius)',
}

type FaceDef = {
  id: string
  label: string
  /** Registry tags under this face */
  tags: string[]
  /** Prefer stage key for specimen */
  stageTag: string
}

function buildFaces(): FaceDef[] {
  const knobs = allKnobs().filter((k) => k.path.startsWith('component.'))
  const tags = new Set<string>()
  for (const k of knobs) {
    const parts = k.path.split('.')
    if (parts.length >= 3) tags.add(parts[1]!)
  }
  // Faces that only have global hooks (e.g. button) still appear.
  for (const id of Object.keys(FACE_GLOBALS)) tags.add(id)

  const byFace = new Map<string, Set<string>>()
  for (const tag of tags) {
    const face = FACE_MERGE[tag] ?? tag
    if (!byFace.has(face)) byFace.set(face, new Set())
    byFace.get(face)!.add(tag)
  }

  const faces: FaceDef[] = []
  for (const [id, set] of [...byFace.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
    const tagsArr = [...set]
    const stageTag =
      id === 'tabs' ? 'tabs' : id === 'toast' ? 'toast' : id === 'radio-group' ? 'radio-group' : tagsArr[0]!
    faces.push({
      id,
      label: id === 'tabs' ? 'Tabs' : id === 'radio-group' ? 'Radio group' : l1Label(id),
      tags: tagsArr,
      stageTag: (L1_CATALOG as readonly string[]).includes(stageTag) ? stageTag : tagsArr[0]!,
    })
  }
  return faces
}

const FACES = buildFaces()

function knobsForFace(face: FaceDef): TokenKnob[] {
  const set = new Set(face.tags)
  return allKnobs().filter((k) => {
    if (!k.path.startsWith('component.')) return false
    const tag = k.path.split('.')[1]
    return tag != null && set.has(tag)
  })
}

function isColorKnob(k: TokenKnob): boolean {
  return k.kind === 'color'
}

function hexForPath(
  doc: Record<string, unknown> | null,
  path: string,
  mode: 'light' | 'dark',
  fallback = '#888888',
): string {
  const raw = getPathValue(doc, path, mode)
  if (!raw) return fallback
  try {
    return toPickerHex(String(raw))
  } catch {
    return fallback
  }
}

export function createComponentsEdit(session: ThemeSession) {
  const selectedId = signal(FACES[0]?.id ?? 'alert')
  const selectedLabel = signal(FACES[0]?.label ?? 'Alert')
  const filter = signal('')
  const variant = signal('')
  const size = signal('')
  const tone = signal('')

  const seedVariant = (faceId: string) => {
    const face = FACES.find((f) => f.id === faceId)
    const m = viewerMetaFor((face?.stageTag ?? faceId) as L1Id)
    // Prefer solid/filled first when present
    const prefer =
      m.variants.find((v) => v === 'solid') ??
      m.variants.find((v) => v === 'md') ??
      m.variants[0] ??
      ''
    variant.set(prefer)
    size.set(m.variantAttr === 'size' ? prefer : 'md')
    tone.set('primary')
  }

  const sync = () => {
    const face = FACES.find((f) => f.id === selectedId())
    selectedLabel.set(face?.label ?? selectedId())
  }

  session.onSync(sync)
  seedVariant(selectedId())
  sync()

  const componentRows = () => {
    const q = filter().trim().toLowerCase()
    return FACES.filter((f) => {
      if (!q) return true
      return (
        f.id.includes(q) ||
        f.label.toLowerCase().includes(q) ||
        (FACE_HINTS[f.id] ?? '').toLowerCase().includes(q)
      )
    }).map((f) => {
      const knobs = knobsForFace(f)
      const ds = getDesignState()
      let overrides = 0
      for (const k of knobs) {
        if (getPathValue(ds.themeDoc, k.path, ds.modeId) != null) overrides += 1
      }
      return {
        key: f.id,
        id: f.id,
        label: f.label,
        hint: FACE_HINTS[f.id] ?? `${knobs.length} component tokens`,
        selected: selectedId() === f.id,
        hasOverride: overrides > 0,
        tokenCount: knobs.length,
        overrideCount: overrides,
      }
    })
  }

  const onFilterInput = (e: Event) => {
    filter.set((e.target as HTMLInputElement).value)
  }

  const onSelect = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[data-l1]') as HTMLElement | null
    const id = el?.dataset.l1
    if (!id || !FACES.some((f) => f.id === id)) return
    selectedId.set(id)
    seedVariant(id)
    sync()
  }

  const face = () => FACES.find((f) => f.id === selectedId())

  const componentKnobs = () => {
    const f = face()
    if (!f) return [] as {
      key: string
      path: string
      label: string
      kind: string
      hex: string
      raw: string
      isColor: boolean
      isOverridden: boolean
      scope: string
    }[]
    const ds = getDesignState()
    const mode = session.writeMode()
    return knobsForFace(f).map((k) => {
      const raw = getPathValue(ds.themeDoc, k.path, mode)
      const isColor = isColorKnob(k)
      return {
        key: k.path,
        path: k.path,
        label: k.label || k.path.split('.').slice(2).join('.'),
        kind: k.kind,
        hex: isColor ? hexForPath(ds.themeDoc, k.path, mode) : '',
        raw: raw != null ? String(raw) : 'inherit / default',
        isColor,
        isOverridden: raw != null,
        scope: 'component',
      }
    })
  }

  const globalKnobs = () => {
    const globals = FACE_GLOBALS[selectedId()] ?? []
    const ds = getDesignState()
    const mode = session.writeMode()
    return globals.map((g) => {
      const raw = getPathValue(ds.themeDoc, g.path, mode)
      const isColor = g.path.includes('color.') || g.path.startsWith('color.')
      const isRadius = g.path.startsWith('radius.')
      return {
        key: g.path,
        path: g.path,
        label: g.label,
        kind: isColor ? 'color' : isRadius ? 'dimension' : 'other',
        hex: isColor ? hexForPath(ds.themeDoc, g.path, mode) : '',
        raw: raw != null ? String(raw) : 'theme default',
        isColor,
        isRadius,
        isOverridden: false,
        scope: 'global',
      }
    })
  }

  const colorFromEvent = (e: Event): string => {
    const ce = e as CustomEvent<{ value?: string }>
    const t = e.target as HTMLElement & { value?: string }
    return String(ce.detail?.value ?? t?.value ?? '').trim()
  }

  const pathFromEvent = (e: Event): string | null => {
    const t = e.target as HTMLElement | null
    return (
      t?.getAttribute('data-token-path') ||
      t?.closest?.('[data-token-path]')?.getAttribute('data-token-path') ||
      null
    )
  }

  const onComponentColor = (e: Event) => {
    if (!session.guardEdit()) return
    const path = pathFromEvent(e)
    const hex = colorFromEvent(e)
    if (!path || !hex) return
    void setTokenValue(path, hex).then(() => {
      session.setNote(`Updated ${path}`, 'success')
      session.sync()
    })
  }

  const onGlobalColor = (e: Event) => {
    if (!session.guardEdit()) return
    const path = pathFromEvent(e)
    const hex = colorFromEvent(e)
    if (!path || !hex) return
    void setTokenValue(path, hex).then(() => {
      session.setNote(`Global ${path} updated`, 'success')
      session.sync()
    })
  }

  const onClearComponent = (e: Event) => {
    if (!session.guardEdit()) return
    const path = pathFromEvent(e)
    if (!path) return
    void clearTokenValue(path).then(() => {
      session.setNote(`Cleared override ${path}`, 'success')
      session.sync()
    })
  }

  const onVariantPick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[data-variant]') as HTMLElement | null
    const v = el?.dataset.variant
    if (v != null) variant.set(v)
  }
  const onSizePick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[data-size]') as HTMLElement | null
    const v = el?.dataset.size
    if (v != null) size.set(v)
  }
  const onTonePick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[data-tone]') as HTMLElement | null
    const v = el?.dataset.tone
    if (v != null) tone.set(v)
  }

  const stageTag = (): L1Id => {
    const f = face()
    const t = f?.stageTag ?? 'button'
    return ((L1_CATALOG as readonly string[]).includes(t) ? t : 'button') as L1Id
  }

  const meta = () => viewerMetaFor(stageTag())
  const stageGates = () => buildStageGates(() => stageTag())
  // Flatten stage gates for template: components.stage.button() etc.
  const stage = new Proxy(
    {} as Record<string, () => boolean>,
    {
      get(_t, prop: string) {
        if (typeof prop !== 'string') return () => false
        const gates = stageGates() as Record<string, () => boolean>
        return gates[prop] ?? (() => false)
      },
    },
  )

  const viewerBlurb = () => meta().blurb
  const sampleText = signal('Sample')
  const onSampleInput = (e: Event) => {
    sampleText.set((e.target as HTMLInputElement).value)
  }
  const notify = signal(true)
  const load = signal(40)

  const variantChips = () => {
    const m = meta()
    const cur = variant()
    return m.variants.map((v) => ({
      key: v,
      v,
      active: v === cur,
      btnVariant: (v === cur ? 'solid' : 'outline') as 'solid' | 'outline',
    }))
  }
  const sizeChips = () => {
    // Size ladder when variantAttr is size; otherwise empty
    const m = meta()
    if (m.variantAttr !== 'size') return [] as { key: string; v: string; active: boolean; btnVariant: 'solid' | 'outline' }[]
    const cur = size()
    return m.variants.map((v) => ({
      key: v,
      v,
      active: v === cur,
      btnVariant: (v === cur ? 'solid' : 'outline') as 'solid' | 'outline',
    }))
  }
  const toneChips = () => {
    const tones = ['primary', 'accent', 'muted', 'success', 'warning', 'destructive']
    const cur = tone()
    return tones.map((v) => ({
      key: v,
      v,
      active: v === cur,
      btnVariant: (v === cur ? 'solid' : 'outline') as 'solid' | 'outline',
    }))
  }

  const hasComponentKnobs = () => componentKnobs().length > 0
  const hasGlobalKnobs = () => globalKnobs().length > 0
  const faceCount = () => FACES.length
  const registryNote = () =>
    hasComponentKnobs()
      ? `${componentKnobs().length} component token${componentKnobs().length === 1 ? '' : 's'} in registry`
      : 'No component-scoped tokens in registry — edit related globals below'

  return {
    filter,
    selectedId,
    selectedLabel,
    componentRows,
    onFilterInput,
    onSelect,
    componentKnobs,
    globalKnobs,
    hasComponentKnobs,
    hasGlobalKnobs,
    onComponentColor,
    onGlobalColor,
    onClearComponent,
    variant,
    size,
    tone,
    onVariantPick,
    onSizePick,
    onTonePick,
    variantChips,
    sizeChips,
    toneChips,
    hasVariants: () => meta().variants.length > 1 && meta().variantAttr !== 'size',
    hasSizes: () => meta().variantAttr === 'size' && meta().variants.length > 1,
    hasTones: () => true,
    viewerVariantAttr: () => meta().variantAttr || 'variant',
    viewerSizeAttr: () => 'size',
    viewerToneAttr: () => 'tone',
    stageGates,
    stage,
    viewerBlurb,
    sampleText,
    onSampleInput,
    notify,
    load,
    faceCount,
    registryNote,
    l1Count: faceCount,
    // legacy template bindings removed: radiusPath, free-text radius
    radiusPath: () => '',
    overrideCount: () =>
      componentRows().reduce((n, r) => n + (r.hasOverride ? 1 : 0), 0),
  }
}

export type { L1Id }
export type ViewerPropRowReexport = ViewerPropRow
