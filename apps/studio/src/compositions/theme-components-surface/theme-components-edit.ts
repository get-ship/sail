/**
 * Themes → Components facet — component-scoped DTCG overrides for this theme.
 * - List faces with component.* tokens (or explicit global hooks).
 * - Composite faces (tabs, toast) collapse sub-tags; radio is its own face.
 * - Defaults: model/registry aliases → theme color paths (never var(--*) labels).
 * - Overrides write component.* leaves; Clear removes them.
 */
import { signal } from '@ship/sail-runtime'
import {
  L1_CATALOG,
  allKnobs,
  clearTokenValue,
  getDesignState,
  getPathValue,
  isParkedComponentStateLeaf,
  l1Label,
  setTokenValue,
  type L1Id,
  type TokenKnob,
} from '@ship/studio/design'
import type { ThemeSession } from '../theme-session/theme-session'
import {
  effectiveComponentColor,
  effectiveGlobalColor,
} from './theme-component-colors'
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
  // radio keeps its own face (component.radio.fg). radio-group is layout +
  // closed action composition — audited separately; no component.radio-group.* yet.
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
  chip: [
    { path: 'color.primary.default', label: 'Primary (global)' },
    { path: 'color.muted.foreground', label: 'Muted (global)' },
    { path: 'color.border.default', label: 'Border (global)' },
  ],
  kbd: [
    { path: 'color.muted.default', label: 'Muted (global)' },
    { path: 'color.foreground', label: 'Foreground (global)' },
    { path: 'color.border.default', label: 'Border (global)' },
  ],
  'code-block': [
    { path: 'color.surface.1', label: 'Surface 1 (global · rest bg)' },
    { path: 'color.foreground', label: 'Foreground (global · rest ink)' },
    { path: 'color.border.default', label: 'Border (global · frame)' },
  ],
  legend: [
    { path: 'color.foreground', label: 'Foreground (global · rest caption ink)' },
  ],
  prose: [
    { path: 'color.foreground', label: 'Foreground (global · rest host ink)' },
    { path: 'color.primary.default', label: 'Primary (global · links / quote rule)' },
    { path: 'color.muted.foreground', label: 'Muted fg (global · quote / markers)' },
  ],
  spinner: [
    { path: 'color.muted.foreground', label: 'Muted fg (global · rest arc)' },
    { path: 'color.border.subtle', label: 'Border subtle (global · track)' },
    { path: 'color.primary.default', label: 'Primary (global · toned)' },
  ],
  'empty-state': [
    { path: 'color.muted.default', label: 'Muted (global · rest bg)' },
    { path: 'color.muted.foreground', label: 'Muted fg (global · rest ink)' },
    { path: 'color.border.default', label: 'Border (global · rest frame)' },
    { path: 'color.foreground', label: 'Foreground (global · title)' },
    { path: 'color.primary.default', label: 'Primary (global · toned icon)' },
  ],
  meter: [
    { path: 'color.muted.default', label: 'Muted (global · track)' },
    { path: 'color.success.default', label: 'Success (global · optimum)' },
    { path: 'color.warning.default', label: 'Warning (global · suboptimum)' },
    { path: 'color.destructive.default', label: 'Danger (global · even-less-good)' },
  ],
  label: [
    { path: 'color.foreground', label: 'Foreground (global · rest ink)' },
    { path: 'color.destructive.default', label: 'Danger (global · required marker)' },
  ],
  progress: [
    { path: 'color.muted.default', label: 'Muted (global · track)' },
    { path: 'color.primary.default', label: 'Primary (global · fill)' },
  ],
  'error-state': [
    { path: 'color.muted.default', label: 'Muted (global · rest bg alias)' },
    { path: 'color.muted.foreground', label: 'Muted fg (global · rest ink)' },
    { path: 'color.border.default', label: 'Border (global · rest edge)' },
    { path: 'color.destructive.default', label: 'Danger (global · icon / inline)' },
    { path: 'color.foreground', label: 'Foreground (global · title)' },
  ],
  icon: [
    { path: 'color.foreground', label: 'Foreground (global · currentColor ink)' },
    { path: 'color.primary.default', label: 'Primary (global · parent fg)' },
    { path: 'icon.md', label: 'Icon md (global size scale)' },
    { path: 'icon.xl', label: 'Icon xl (global size scale)' },
  ],
  button: [
    { path: 'color.primary.default', label: 'Primary (global)' },
    { path: 'color.secondary.default', label: 'Secondary (global)' },
    { path: 'color.destructive.default', label: 'Danger (global)' },
    { path: 'color.success.default', label: 'Success (global)' },
    { path: 'radius.lg', label: 'Radius lg (global)' },
  ],
  a: [
    { path: 'color.primary.default', label: 'Primary (global · toned links)' },
    { path: 'color.foreground', label: 'Foreground (global · solid invert)' },
    { path: 'color.muted.foreground', label: 'Muted fg (global · rest ink alias)' },
    { path: 'color.border.default', label: 'Border (global · rest button.border)' },
    { path: 'radius.md', label: 'Radius md (global · button-link)' },
  ],
  card: [
    { path: 'color.surface.3', label: 'Card surface (global)' },
    { path: 'color.foreground', label: 'Foreground (global)' },
    { path: 'radius.lg', label: 'Radius lg (global)' },
  ],
  switch: [
    { path: 'color.muted.default', label: 'Muted (global · rest track alias)' },
    { path: 'color.border.strong', label: 'Border strong (global · rest border alias)' },
    { path: 'color.primary.default', label: 'Primary (global · checked track default)' },
  ],
  slider: [
    { path: 'color.muted.default', label: 'Muted (global · track alias)' },
    { path: 'color.primary.default', label: 'Primary (global · fill alias)' },
    { path: 'color.border.default', label: 'Border (global · track-border alias)' },
    { path: 'color.background', label: 'Background (global · field/thumb face alias)' },
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
  fieldset: [
    { path: 'color.border.default', label: 'Border (global · rest fieldset.border)' },
    { path: 'radius.md', label: 'Radius md (global · group chrome)' },
  ],
  radio: [
    { path: 'color.foreground', label: 'Foreground (global · rest radio.fg)' },
    { path: 'color.primary.default', label: 'Primary (global · checked face)' },
    { path: 'color.background', label: 'Background (global · control face)' },
  ],
}

const FACE_HINTS: Record<string, string> = {
  alert: 'Callouts · tone maps to globals + component leaves',
  badge: 'Status chips',
  button: 'Actions — globals until component.button.* exists',
  a: 'Links · bare text or button-link chrome · component.button.* rest',
  card: 'Surfaces · glass leaves',
  tabs: 'Tabs (list + tab + panel as one face)',
  toast: 'Toasts + region',
  'radio-group': 'Radio set',
  switch: 'Binary toggles · component.switch.track + border + thumb',
  radio: 'Radio option · component.radio.fg leaf',
  switch: 'Binary toggles',
  chip: 'Filters and tags',
  kbd: 'Keycaps · component.kbd.* faces',
  'code-block': 'Source block · component.code-block bg/fg · .tok-* via --syntax-*',
  legend: 'Fieldset caption · component.legend.fg leaf',
  prose: 'Typography scope · component.prose.fg host leaf',
  spinner: 'Indeterminate wait · fg + track leaves',
  'empty-state': 'Nothing-here · bg/fg/border/title leaves · tone on icon',
  meter: 'Gauge · track + optimum band leaves',
  label: 'Form labels · component.label.fg + required leaves',
  progress: 'Bar · track + fill leaves',
  slider: 'Range · track + fill + track-border leaves · single thumb',
  'error-state': 'Recovery · component.error-state leaves + brand destructive',
  icon: 'Glyph host · icon scale sizes · currentColor ink',
  avatar: 'Face / initials',
  input: 'Fields (globals + control radius)',
  fieldset: 'Form group · component.fieldset.border leaf',
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
    const stageTag = id === 'tabs' ? 'tabs' : id === 'toast' ? 'toast' : tagsArr[0]!
    faces.push({
      id,
      label: id === 'tabs' ? 'Tabs' : l1Label(id),
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
    // SPEC-D1: parked hover/focus storage leaves are not editable rest slots.
    if (isParkedComponentStateLeaf(k.path)) return false
    const tag = k.path.split('.')[1]
    return tag != null && set.has(tag)
  })
}

function isColorKnob(k: TokenKnob): boolean {
  return k.kind === 'color'
}

type ComponentKnobRow = {
  /** for-key — must change when display resolves so Sail rebinds text/swatch */
  key: string
  path: string
  label: string
  kind: string
  hex: string
  canPickColor: boolean
  hexLabel: string
  raw: string
  isColor: boolean
  isOverridden: boolean
  /** THEMES-1: which override layer won (instance > component > global > baked). */
  layer: string
  scope: string
}

type GlobalKnobRow = {
  key: string
  path: string
  label: string
  kind: string
  hex: string
  canPickColor: boolean
  hexLabel: string
  raw: string
  isColor: boolean
  isRadius: boolean
  isOverridden: boolean
  layer: string
  scope: string
}

/** Stable path + display fingerprint so keyed <for> remounts when hex resolves. */
function knobRenderKey(path: string, hexLabel: string, hex: string, overridden: boolean): string {
  return `${path}|${overridden ? 'o' : 'd'}|${hexLabel}|${hex}`
}

export function createComponentsEdit(session: ThemeSession) {
  const selectedId = signal(FACES[0]?.id ?? 'alert')
  const selectedLabel = signal(FACES[0]?.label ?? 'Alert')
  const filter = signal('')
  const variant = signal('')
  const size = signal('')
  const tone = signal('')
  /**
   * Signal-backed knob lists — rebuilt on every session/design sync.
   * Plain getters that only call getDesignState() during first paint freeze on
   * themeDoc=null (openTheme later fills the store without changing selectedId).
   */
  const componentKnobList = signal<ComponentKnobRow[]>([])
  const globalKnobList = signal<GlobalKnobRow[]>([])
  /** Rail override badges + override count — same freeze risk without a signal. */
  const faceOverrideBits = signal<Record<string, number>>({})

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

  const face = () => FACES.find((f) => f.id === selectedId())

  const rebuildInspector = () => {
    const f = face()
    const ds = getDesignState()
    const mode = session.writeMode()

    if (!f) {
      componentKnobList.set([])
    } else {
      const next = knobsForFace(f).map((k) => {
        const isColor = isColorKnob(k)
        if (isColor) {
          const eff = effectiveComponentColor(ds.themeDoc, k, mode)
          return {
            key: knobRenderKey(k.path, eff.hexLabel, eff.hex, eff.isOverridden),
            path: k.path,
            label: k.label || k.path.split('.').slice(2).join('.'),
            kind: k.kind,
            // Resolved hex only — never invent #888 / #fff when default is unresolvable.
            hex: eff.hex,
            canPickColor: true,
            hexLabel: eff.hexLabel,
            raw: eff.raw,
            isColor: true,
            isOverridden: eff.isOverridden,
            layer: eff.layer,
            scope: 'component',
          }
        }
        const raw = getPathValue(ds.themeDoc, k.path, mode)
        const hexLabel = ''
        const isOverridden = raw != null
        const display = raw != null ? String(raw) : 'Default'
        return {
          key: knobRenderKey(k.path, display, '', isOverridden),
          path: k.path,
          label: k.label || k.path.split('.').slice(2).join('.'),
          kind: k.kind,
          hex: '',
          canPickColor: false,
          hexLabel,
          raw: display,
          isColor: false,
          isOverridden,
          layer: isOverridden ? 'component' : 'baked',
          scope: 'component',
        }
      })
      componentKnobList.set(next)
    }

    const globals = FACE_GLOBALS[selectedId()] ?? []
    globalKnobList.set(
      globals.map((g) => {
        const isColor = g.path.includes('color.') || g.path.startsWith('color.')
        const isRadius = g.path.startsWith('radius.')
        if (isColor) {
          const eff = effectiveGlobalColor(ds.themeDoc, g.path, mode)
          return {
            key: knobRenderKey(g.path, eff.hexLabel, eff.hex, eff.isOverridden),
            path: g.path,
            label: g.label,
            kind: 'color',
            hex: eff.hex,
            canPickColor: true,
            hexLabel: eff.hexLabel,
            raw: eff.raw,
            isColor: true,
            isRadius: false,
            isOverridden: eff.isOverridden,
            layer: eff.layer,
            scope: 'global',
          }
        }
        const raw = getPathValue(ds.themeDoc, g.path, mode)
        const display = raw != null ? String(raw) : 'Default'
        return {
          key: knobRenderKey(g.path, display, '', false),
          path: g.path,
          label: g.label,
          kind: isRadius ? 'dimension' : 'other',
          hex: '',
          canPickColor: false,
          hexLabel: '',
          raw: display,
          isColor: false,
          isRadius,
          isOverridden: raw != null,
          layer: raw != null ? 'global' : 'baked',
          scope: 'global',
        }
      }),
    )

    const bits: Record<string, number> = {}
    for (const faceDef of FACES) {
      let overrides = 0
      for (const k of knobsForFace(faceDef)) {
        if (getPathValue(ds.themeDoc, k.path, ds.modeId) != null) overrides += 1
      }
      bits[faceDef.id] = overrides
    }
    faceOverrideBits.set(bits)
  }

  const sync = () => {
    const f = FACES.find((x) => x.id === selectedId())
    selectedLabel.set(f?.label ?? selectedId())
    rebuildInspector()
  }

  session.onSync(sync)
  seedVariant(selectedId())
  sync()

  const componentRows = () => {
    const q = filter().trim().toLowerCase()
    const bits = faceOverrideBits()
    return FACES.filter((f) => {
      if (!q) return true
      return (
        f.id.includes(q) ||
        f.label.toLowerCase().includes(q) ||
        (FACE_HINTS[f.id] ?? '').toLowerCase().includes(q)
      )
    }).map((f) => {
      const knobs = knobsForFace(f)
      const overrides = bits[f.id] ?? 0
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
    // Rail rows author `l1=` (data-* banned). dataset.l1 never sees plain attrs.
    const el = (e.target as Element | null)?.closest?.('[l1]') as HTMLElement | null
    const id = el?.getAttribute('l1') ?? ''
    if (!id || !FACES.some((f) => f.id === id)) return
    selectedId.set(id)
    seedVariant(id)
    sync()
  }

  // Expose signals directly so template `components.componentKnobs()` tracks them.
  // Wrapping as () => list() can fail to re-bind keyed <for> after openTheme.
  const componentKnobs = componentKnobList
  const globalKnobs = globalKnobList

  const onComponentColor = (detail: { value?: string; token?: string }) => {
    if (!session.guardEdit()) return
    const path = detail?.token ?? null
    const hex = String(detail?.value ?? '').trim()
    if (!path || !hex) return
    void setTokenValue(path, hex).then(() => {
      session.setNote(`Updated ${path}`, 'success')
      session.sync()
    })
  }

  const onGlobalColor = (detail: { value?: string; token?: string }) => {
    if (!session.guardEdit()) return
    const path = detail?.token ?? null
    const hex = String(detail?.value ?? '').trim()
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
    // Plain `variant=` attr (not data-variant) — same as rail l1 / facet doors.
    const el = (e.target as Element | null)?.closest?.('[variant]') as HTMLElement | null
    const v = el?.getAttribute('variant')
    if (v != null) variant.set(v)
  }
  const onSizePick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[size]') as HTMLElement | null
    const v = el?.getAttribute('size')
    if (v != null) size.set(v)
  }
  const onTonePick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[tone]') as HTMLElement | null
    const v = el?.getAttribute('tone')
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

  const hasComponentKnobs = () => componentKnobList().length > 0
  const hasGlobalKnobs = () => globalKnobList().length > 0
  const faceCount = () => FACES.length
  const registryNote = () => {
    const n = componentKnobList().length
    return n > 0
      ? `${n} component token${n === 1 ? '' : 's'} in registry`
      : 'No component-scoped tokens in registry — edit related globals below'
  }

  const pageNo = signal('1')
  const onPageChange = (detail: { page?: string }) => {
    if (detail?.page) pageNo.set(String(detail.page))
  }

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
    pageNo,
    onPageChange,
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
export {
  dtcgDefaultAlias,
  effectiveComponentColor,
  effectiveGlobalColor,
  resolveColorRaw,
  assertNoCssExhaustLabel,
} from './theme-component-colors'
