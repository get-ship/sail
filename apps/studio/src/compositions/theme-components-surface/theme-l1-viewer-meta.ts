/**
 * Viewer chrome meta for Themes Components — mirrors sail-site DETAILS
 * (variant chips + blurbs) without importing the docs app.
 * Stage keys are camelCase zero-arg gates: stage.button(), stage.menuItem().
 */
import type { L1Id } from '@ship/studio/design'
import { L1_CATALOG, l1Label } from '@ship/studio/design'

export type ViewerPropRow = { name: string; type: string; desc: string }

export type L1ViewerMeta = {
  /** CamelCase stage gate key (stage.<key>()). */
  key: string
  blurb: string
  variants: string[]
  variantAttr: string
}

/**
 * Stage gate keys — match sail-site DETAILS `key` where it diverges from
 * camelCase (e.g. switch → switchToggle; progress → progressBar) so templates
 * can reuse the same `stage.<key>()` call shape.
 */
const STAGE_KEY_OVERRIDES: Partial<Record<L1Id, string>> = {
  switch: 'switchToggle',
  progress: 'progressBar',
}

function kebabToCamel(id: string): string {
  if (id === 'a') return 'a'
  return id.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase())
}

export function stageKeyFor(id: L1Id): string {
  return STAGE_KEY_OVERRIDES[id] ?? kebabToCamel(id)
}

/** Hand-curated variant chips (attr + values) where the docs surface has them. */
const VARIANT_META: Partial<
  Record<L1Id, { variants: string[]; variantAttr: string; blurb?: string }>
> = {
  button: {
    // solid first so the live stage seeds filled, not bare default chrome
    variants: ['solid', 'soft', 'outline', 'ghost', 'default'],
    variantAttr: 'variant',
    blurb: 'Tone × variant platform button — press and focus states included.',
  },
  switch: {
    variants: ['md', 'sm', 'lg'],
    variantAttr: 'size',
    blurb: 'Checkbox in switch costume; size chips drive the track scale.',
  },
  slider: {
    variants: ['primary', 'success', 'warning', 'destructive'],
    variantAttr: 'tone',
    blurb: 'Native range, themed track and thumb.',
  },
  'toggle-group': {
    variants: ['outline', 'ghost', 'segmented'],
    variantAttr: 'variant',
    blurb: 'Segmented toggles with roving focus.',
  },
  label: {
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    blurb: 'Real label — for= focuses the control.',
  },
  alert: {
    variants: ['info', 'success', 'warning', 'destructive'],
    variantAttr: 'tone',
    blurb: 'Inline status with the right ARIA role.',
  },
  badge: {
    variants: ['soft', 'solid', 'outline'],
    variantAttr: 'variant',
    blurb: 'Status chip — tone + variant.',
  },
  chip: {
    variants: ['solid', 'soft', 'outline', 'ghost'],
    variantAttr: 'variant',
    blurb: 'Filter / selection pill.',
  },
  card: {
    variants: ['default', 'outline'],
    variantAttr: 'variant',
    blurb: 'Surface with optional outline chrome.',
  },
  avatar: {
    variants: ['md', 'sm', 'lg'],
    variantAttr: 'size',
    blurb: 'Face / initials glyph box.',
  },
  icon: {
    variants: ['xs', 'sm', 'md', 'lg', 'xl'],
    variantAttr: 'size',
    blurb: 'Glyph host · size from theme icon scale; ink currentColor (no component.icon paint leaves).',
  },
  spinner: {
    variants: ['xs', 'sm', 'md', 'lg'],
    variantAttr: 'size',
    blurb: 'Loading ring · rest uses component.spinner.fg + track; tone paints active arc.',
  },
  meter: {
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    blurb: 'Native gauge · component.meter.track + optimum/suboptimum/sub-suboptimum bands.',
  },
  progress: {
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    blurb: 'Native bar · component.progress.track + fill; omit value = indeterminate.',
  },
  skeleton: {
    variants: ['text', 'line', 'block', 'circle'],
    variantAttr: 'shape',
    blurb: 'Loading sheen · component.skeleton.bg / highlight.',
  },
  'error-state': {
    variants: ['default', 'inline'],
    variantAttr: 'variant',
    blurb:
      'Recovery surface · rest uses component.error-state bg/fg/border; icon + inline border brand destructive; code + details via recoverySurface.',
  },
  separator: {
    variants: ['solid', 'dashed'],
    variantAttr: 'variant',
    blurb: 'Divider · rest uses component.separator.line; tone optional.',
  },
  kbd: {
    variants: [],
    variantAttr: '',
    blurb: 'Keycap · component.kbd bg/fg/border; no size/variant props.',
  },
  prose: {
    variants: [],
    variantAttr: '',
    blurb: 'Typography scope · component.prose.fg; nested tags via type map; no size/variant props.',
  },
  tooltip: {
    variants: ['top', 'bottom', 'left', 'right'],
    variantAttr: 'placement',
    blurb: 'Hover/focus bubble.',
  },
  tabs: {
    variants: ['line', 'pill', 'enclosed'],
    variantAttr: 'variant',
    blurb: 'Tab strip + panels (variant on tab-list).',
  },
  pagination: {
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    blurb: 'Page controls with ARIA current.',
  },
}

const DEFAULT_BLURB =
  'Live component under the theme preview — radius and component tokens paint here.'

export function viewerMetaFor(id: L1Id): L1ViewerMeta {
  const v = VARIANT_META[id]
  return {
    key: stageKeyFor(id),
    blurb: v?.blurb ?? `${l1Label(id)} — live specimen under theme paint.`,
    variants: v?.variants ?? [],
    variantAttr: v?.variantAttr ?? '',
  }
}

/** Build stage zero-arg gates: stage.button(), stage.menuItem(), … */
export function buildStageGates(selectedId: () => L1Id): Record<string, () => boolean> {
  return Object.fromEntries(
    L1_CATALOG.map((id) => [stageKeyFor(id), () => selectedId() === id]),
  ) as Record<string, () => boolean>
}

export { DEFAULT_BLURB }
