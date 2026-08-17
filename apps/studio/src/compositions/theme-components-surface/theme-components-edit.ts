import { closestAttr, inputValue, type UiEvent } from '../ui-event'
/**
 * Themes → Components facet — L1 specimen + live theme roles.
 * Faces edit color / type / radius roles. Per-tag leaves are gone.
 */
import { signal } from '@ship/sail-runtime'
import { L1_CATALOG, clearTokenValue, getDesignState, getPathValue, l1Label, setTokenValue, type L1Id } from '../../design/index.ts'
import type { ThemeSession } from '../theme-session/theme-session'
import { effectiveGlobalColor } from './theme-component-colors'
import {
  buildStageGates,
  viewerMetaFor,
  type ViewerPropRow,
} from './theme-l1-viewer-meta'

/** Shared control background — input / select / textarea / field. */
const FIELD_ROLES: { path: string; label: string }[] = [
  { path: 'color.input', label: 'Input' },
  { path: 'color.foreground', label: 'Foreground' },
  { path: 'color.border.default', label: 'Border' },
  { path: 'color.destructive.default', label: 'Invalid' },
  { path: 'radius.md', label: 'Radius' },
]

/** Overlay background — popover / dropdown / dialog / hover-card / tooltip. */
const OVERLAY_ROLES: { path: string; label: string }[] = [
  { path: 'color.popover.default', label: 'Popover' },
  { path: 'color.popover.foreground', label: 'Popover foreground' },
]

/** Live roles a face spends. Empty = this face has nothing dedicated to edit. */
export const FACE_ROLES: Record<string, { path: string; label: string }[]> = {
  alert: [
    { path: 'color.info.default', label: 'Info' },
    { path: 'color.success.default', label: 'Success' },
    { path: 'color.warning.default', label: 'Warning' },
    { path: 'color.destructive.default', label: 'Danger' },
    { path: 'color.muted.foreground', label: 'Muted' },
    { path: 'color.surface.1', label: 'Surface' },
    { path: 'color.foreground', label: 'Foreground' },
  ],
  badge: [
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'color.muted.foreground', label: 'Muted' },
  ],
  chip: [
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'color.muted.foreground', label: 'Muted' },
    { path: 'color.border.default', label: 'Border' },
  ],
  kbd: [
    { path: 'color.muted.default', label: 'Muted' },
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.border.default', label: 'Border' },
  ],
  'code-block': [
    { path: 'color.surface.1', label: 'Surface' },
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.border.default', label: 'Border' },
  ],
  legend: [{ path: 'color.foreground', label: 'Foreground' }],
  prose: [
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'color.muted.foreground', label: 'Muted' },
  ],
  spinner: [
    { path: 'color.muted.foreground', label: 'Muted' },
    { path: 'color.border.subtle', label: 'Track' },
    { path: 'color.primary.default', label: 'Primary' },
  ],
  'empty-state': [
    { path: 'color.muted.default', label: 'Muted' },
    { path: 'color.muted.foreground', label: 'Muted foreground' },
    { path: 'color.border.default', label: 'Border' },
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.primary.default', label: 'Primary' },
  ],
  meter: [
    { path: 'color.muted.default', label: 'Track' },
    { path: 'color.success.default', label: 'Success' },
    { path: 'color.warning.default', label: 'Warning' },
    { path: 'color.destructive.default', label: 'Danger' },
  ],
  label: [
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.destructive.default', label: 'Required' },
    { path: 'typography.label.weight', label: 'Label weight' },
    { path: 'typography.label.font', label: 'Label font' },
  ],
  progress: [
    { path: 'color.muted.default', label: 'Track' },
    { path: 'color.primary.default', label: 'Fill' },
  ],
  'scroll-area': [
    { path: 'color.border.default', label: 'Thumb' },
    { path: 'color.border.strong', label: 'Hover thumb' },
  ],
  'error-state': [
    { path: 'color.muted.default', label: 'Muted' },
    { path: 'color.muted.foreground', label: 'Muted foreground' },
    { path: 'color.border.default', label: 'Border' },
    { path: 'color.destructive.default', label: 'Danger' },
    { path: 'color.foreground', label: 'Foreground' },
  ],
  icon: [
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'icon.md', label: 'Icon md' },
    { path: 'icon.xl', label: 'Icon xl' },
  ],
  button: [
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'color.background', label: 'Background' },
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.destructive.default', label: 'Danger' },
    { path: 'radius.lg', label: 'Radius' },
  ],
  a: [
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.background', label: 'Background' },
    { path: 'color.muted.foreground', label: 'Muted' },
    { path: 'radius.md', label: 'Radius' },
  ],
  card: [
    { path: 'color.card.default', label: 'Card' },
    { path: 'color.card.foreground', label: 'Card foreground' },
    { path: 'filter.backdrop', label: 'Card backdrop' },
    { path: 'image.texture', label: 'Card texture' },
    { path: 'radius.lg', label: 'Radius' },
  ],
  popover: OVERLAY_ROLES,
  dropdown: OVERLAY_ROLES,
  dialog: OVERLAY_ROLES,
  'hover-card': OVERLAY_ROLES,
  tooltip: OVERLAY_ROLES,
  switch: [
    { path: 'color.muted.default', label: 'Track' },
    { path: 'color.border.strong', label: 'Border' },
    { path: 'color.primary.default', label: 'Checked' },
  ],
  slider: [
    { path: 'color.muted.default', label: 'Track' },
    { path: 'color.primary.default', label: 'Fill' },
    { path: 'color.border.default', label: 'Track border' },
    { path: 'color.input', label: 'Thumb' },
  ],
  tabs: [
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'color.muted.foreground', label: 'Muted' },
    { path: 'color.surface.1', label: 'Surface' },
  ],
  input: FIELD_ROLES,
  textarea: FIELD_ROLES,
  select: FIELD_ROLES,
  field: FIELD_ROLES,
  combobox: FIELD_ROLES,
  fieldset: [
    { path: 'color.border.default', label: 'Border' },
    { path: 'radius.md', label: 'Radius' },
  ],
  radio: [
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.primary.default', label: 'Checked' },
    { path: 'color.input', label: 'Face' },
  ],
  checkbox: [
    { path: 'color.foreground', label: 'Foreground' },
    { path: 'color.primary.default', label: 'Checked' },
    { path: 'color.input', label: 'Face' },
  ],
  ticker: [
    { path: 'color.muted.default', label: 'Muted' },
    { path: 'color.muted.foreground', label: 'Muted foreground' },
    { path: 'color.primary.default', label: 'Primary' },
    { path: 'color.primary.foreground', label: 'Primary foreground' },
    { path: 'radius.md', label: 'Radius' },
  ],
}

const FACE_HINTS: Record<string, string> = {
  alert: 'Callouts · tone maps to info / success / warning / danger',
  badge: 'Status chips',
  button: 'Actions · default is solid primary',
  a: 'Links · bare text or button-link chrome',
  card: 'Surfaces · card background and foreground',
  tabs: 'Tabs (list + tab + panel as one face)',
  toast: 'Toasts + region',
  'radio-group': 'Radio set',
  switch: 'Binary toggles · muted track + primary checked',
  radio: 'Radio option · foreground',
  checkbox: 'Boolean check · input face + primary checked',
  ticker: 'Marquee host · duration/reverse · author paint',
  chip: 'Filters and tags',
  kbd: 'Keycaps · muted background + border',
  'code-block': 'Source block · surface background + foreground',
  legend: 'Fieldset caption · foreground',
  prose: 'Typography scope · foreground + primary links',
  spinner: 'Indeterminate wait · muted arc + primary tone',
  'empty-state': 'Nothing-here · muted surface · tone on icon',
  meter: 'Gauge · muted track + semantic bands',
  label: 'Form labels · foreground + label type role',
  progress: 'Bar · muted track + primary fill',
  'scroll-area': 'Overflow · border thumb',
  slider: 'Range · muted track + primary fill',
  'error-state': 'Recovery · muted surface + destructive',
  icon: 'Glyph host · icon scale · currentColor',
  avatar: 'Face / initials',
  input: 'Native input · input background + foreground + border',
  textarea: 'Multiline · shared input background',
  select: 'Native closed picker · shared input background',
  field: 'Field wrapper · shared input background',
  fieldset: 'Form group · border',
  popover: 'Overlay · popover background',
  dropdown: 'Overlay · popover background',
  dialog: 'Overlay · popover background',
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
  return L1_CATALOG.map((id) => ({
    id,
    label: id === 'tabs' ? 'Tabs' : l1Label(id),
    tags: [id],
    stageTag: id,
  }))
}

const FACES = buildFaces()

/** Tags that have a live stage template. Others show an honest empty. */
const STAGE_SPECIMENS = new Set([
  'a',
  'accordion',
  'alert',
  'avatar',
  'badge',
  'breadcrumb',
  'button',
  'card',
  'checkbox',
  'chip',
  'code-block',
  'combobox',
  'context-menu',
  'dialog',
  'dropdown',
  'empty-state',
  'error-boundary',
  'error-state',
  'field',
  'fieldset',
  'hover-card',
  'icon',
  'input',
  'kbd',
  'label',
  'legend',
  'menu-item',
  'meter',
  'navbar',
  'page',
  'pagination',
  'popover',
  'progress',
  'prose',
  'radio',
  'radio-group',
  'scroll-area',
  'select',
  'separator',
  'skeleton',
  'slider',
  'spinner',
  'switch',
  'tab',
  'tab-list',
  'tab-panel',
  'table',
  'tabs',
  'textarea',
  'theme-picker',
  'ticker',
  'toast',
  'toast-region',
  'toggle',
  'toggle-group',
  'tooltip',
])

function rolesForFace(faceId: string): { path: string; label: string }[] {
  return FACE_ROLES[faceId] ?? []
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
  const seedId = L1_CATALOG.includes('button') ? 'button' : (FACES[0]?.id ?? 'alert')
  const selectedId = signal(seedId)
  const selectedLabel = signal(l1Label(seedId))
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
      const next = rolesForFace(f.id).map((g) => {
        const isColor = g.path.includes('color.') || g.path.startsWith('color.')
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
            isOverridden: eff.isOverridden,
            layer: eff.layer,
            scope: 'role',
          }
        }
        const raw = getPathValue(ds.themeDoc, g.path, mode)
        const display = raw != null ? String(raw) : 'Default'
        const isOverridden = raw != null
        return {
          key: knobRenderKey(g.path, display, '', isOverridden),
          path: g.path,
          label: g.label,
          kind: g.path.startsWith('radius.') ? 'dimension' : 'other',
          hex: '',
          canPickColor: false,
          hexLabel: display,
          raw: display,
          isColor: false,
          isOverridden,
          layer: isOverridden ? 'global' : 'baked',
          scope: 'role',
        }
      })
      componentKnobList.set(next)
    }

    globalKnobList.set([])

    const bits: Record<string, number> = {}
    for (const faceDef of FACES) {
      let overrides = 0
      for (const k of rolesForFace(faceDef.id)) {
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
      const roles = rolesForFace(f.id)
      const overrides = bits[f.id] ?? 0
      return {
        key: f.id,
        id: f.id,
        label: f.label,
        hint: FACE_HINTS[f.id] ?? (roles.length > 0 ? `${roles.length} theme roles` : 'Inherits the theme'),
        selected: selectedId() === f.id,
        hasOverride: overrides > 0,
        tokenCount: roles.length,
        overrideCount: overrides,
      }
    })
  }

  const onFilterInput = (e: UiEvent) => {
    filter.set(inputValue(e))
  }

  const onSelect = (e: UiEvent) => {
    // Rail rows author `l1=` (data-* banned). dataset.l1 never sees plain attrs.
    const id = closestAttr(e, '[l1]', 'l1')
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

  const onClearComponent = (e: UiEvent) => {
    if (!session.guardEdit()) return
    const path = closestAttr(e, '[token-path]', 'token-path')
    if (!path) return
    void clearTokenValue(path).then(() => {
      session.setNote(`Cleared override ${path}`, 'success')
      session.sync()
    })
  }

  const onVariantPick = (e: UiEvent) => {
    // Plain `variant=` attr (not data-variant) — same as rail l1 / facet doors.
    const v = closestAttr(e, '[variant]', 'variant')
    if (v) variant.set(v)
  }
  const onSizePick = (e: UiEvent) => {
    const v = closestAttr(e, '[size]', 'size')
    if (v) size.set(v)
  }
  const onTonePick = (e: UiEvent) => {
    const v = closestAttr(e, '[tone]', 'tone')
    if (v) tone.set(v)
  }

  const stageTag = (): L1Id => {
    const f = face()
    const t = f?.stageTag ?? 'button'
    return (L1_CATALOG.includes(t) ? t : 'button') as L1Id
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
  const onSampleInput = (e: UiEvent) => {
    sampleText.set(inputValue(e))
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
  const hasSpecimen = () => STAGE_SPECIMENS.has(selectedId())
  const faceCount = () => FACES.length
  const registryNote = () => {
    const n = componentKnobList().length
    return n > 0
      ? `${n} theme role${n === 1 ? '' : 's'}`
      : 'No dedicated theme roles — this face inherits the theme'
  }

  const pageNo = signal('1')
  const onPageChange = (detail: { page?: string }) => {
    if (detail?.page) pageNo.set(String(detail.page))
  }

  return {
    filterQuery: filter,
    selectedId,
    selectedLabel,
    componentRows,
    onFilterInput,
    onSelect,
    componentKnobs,
    globalKnobs,
    hasComponentKnobs,
    hasGlobalKnobs,
    hasSpecimen,
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
