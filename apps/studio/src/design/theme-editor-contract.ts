/**
 * THEMES-1 studio editor contract — every control maps to a schema field.
 *
 * Studio is THE authoring interface. Hand-editing DTCG is unsupported.
 * This table is the edit contract: UI control → theme schema field path.
 * Nothing in the themes surfaces may write outside this set.
 *
 * Schema envelope: packages/sail/tokens theme-schema / generated themes-meta
 * (THEME_ROOT_KEYS, THEME_SAIL_KEYS, THEME_MODE_GROUP_KEYS).
 */

import type { ThemeFacet } from '../compositions/theme-session/theme-nav'

/** Logical schema field category for a control. */
export type SchemaFieldKind =
  | 'identity'
  | 'mode'
  | 'scale'
  | 'component'
  | 'meta'

export type ThemeEditorControl = {
  /** Stable control id (facet + role). */
  id: string
  /** Facet that owns the control. */
  facet: ThemeFacet
  /** Human label (matches product UI). */
  label: string
  /** Schema field kind. */
  kind: SchemaFieldKind
  /**
   * DTCG / envelope path written (dot path).
   * `modes.{mode}.…` for mode-scoped scales; `$extensions.sail.…` for meta.
   */
  schemaPath: string
  /** THEME_MODE_GROUP_KEYS entry when kind is scale|component. */
  modeGroup?: string
}

/**
 * Canonical control → schema field map for the five theme facets.
 * Exhaustive vs product controls; drift tests lock membership.
 */
export const THEME_EDITOR_CONTROLS: readonly ThemeEditorControl[] = [
  // ── Overview ────────────────────────────────────────────────────────────
  {
    id: 'overview.themeId',
    facet: 'overview',
    label: 'Theme id',
    kind: 'identity',
    schemaPath: '$extensions.sail.themeId',
  },
  {
    id: 'overview.defaultMode',
    facet: 'overview',
    label: 'Default mode',
    kind: 'mode',
    schemaPath: '$extensions.sail.defaultMode',
  },
  {
    id: 'overview.supportedModes',
    facet: 'overview',
    label: 'Supported modes',
    kind: 'mode',
    schemaPath: '$extensions.sail.supportedModes',
  },
  {
    id: 'overview.density',
    facet: 'overview',
    label: 'Density',
    kind: 'meta',
    schemaPath: '$extensions.sail.density',
  },
  {
    id: 'overview.primary',
    facet: 'overview',
    label: 'Primary',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.primary.default',
    modeGroup: 'color',
  },
  {
    id: 'overview.secondary',
    facet: 'overview',
    label: 'Secondary',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.secondary.default',
    modeGroup: 'color',
  },
  {
    id: 'overview.accent',
    facet: 'overview',
    label: 'Accent',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.accent.default',
    modeGroup: 'color',
  },
  {
    id: 'overview.type.display',
    facet: 'overview',
    label: 'Display font',
    kind: 'scale',
    schemaPath: 'modes.{mode}.typography.fontFamily.display',
    modeGroup: 'typography',
  },
  {
    id: 'overview.type.body',
    facet: 'overview',
    label: 'Body font',
    kind: 'scale',
    schemaPath: 'modes.{mode}.typography.fontFamily.body',
    modeGroup: 'typography',
  },
  {
    id: 'overview.type.mono',
    facet: 'overview',
    label: 'Mono font',
    kind: 'scale',
    schemaPath: 'modes.{mode}.typography.fontFamily.mono',
    modeGroup: 'typography',
  },

  // ── Color ───────────────────────────────────────────────────────────────
  {
    id: 'color.anchors',
    facet: 'color',
    label: 'Chromatic anchors',
    kind: 'scale',
    schemaPath: 'anchors.{mode}.*',
    modeGroup: 'color',
  },
  {
    id: 'color.brand',
    facet: 'color',
    label: 'Brand colors',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.*',
    modeGroup: 'color',
  },
  {
    id: 'color.surfaces',
    facet: 'color',
    label: 'Surfaces',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.surface.*',
    modeGroup: 'color',
  },
  {
    id: 'color.semantic',
    facet: 'color',
    label: 'Semantic tones',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.{destructive,success,warning,info}.*',
    modeGroup: 'color',
  },

  // ── Shape ───────────────────────────────────────────────────────────────
  {
    id: 'shape.radius',
    facet: 'shape',
    label: 'Radius scale',
    kind: 'scale',
    schemaPath: 'modes.{mode}.radius.*',
    modeGroup: 'radius',
  },
  {
    id: 'shape.shadow',
    facet: 'shape',
    label: 'Elevation / shadow',
    kind: 'scale',
    schemaPath: 'modes.{mode}.shadow.*',
    modeGroup: 'shadow',
  },
  {
    id: 'shape.interaction',
    facet: 'shape',
    label: 'Interaction strength',
    kind: 'scale',
    schemaPath: 'modes.{mode}.interaction.*',
    modeGroup: 'interaction',
  },

  // ── Assets ──────────────────────────────────────────────────────────────
  {
    id: 'assets.type',
    facet: 'assets',
    label: 'Type roles',
    kind: 'scale',
    schemaPath: 'modes.{mode}.typography.*',
    modeGroup: 'typography',
  },
  {
    id: 'assets.icons',
    facet: 'assets',
    label: 'Icon pack / scale',
    kind: 'scale',
    schemaPath: 'modes.{mode}.icon.*',
    modeGroup: 'icon',
  },

  // ── Components ──────────────────────────────────────────────────────────
  {
    id: 'components.token',
    facet: 'components',
    label: 'Face theme role',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.*|modes.{mode}.typography.*|modes.{mode}.radius.*',
    modeGroup: 'color',
  },
  {
    id: 'components.globalFallback',
    facet: 'components',
    label: 'Related global (visible fallback)',
    kind: 'scale',
    schemaPath: 'modes.{mode}.color.*|modes.{mode}.radius.*',
    modeGroup: 'color',
  },
] as const

/** Every control id (for exhaustive checks). */
export const THEME_EDITOR_CONTROL_IDS = THEME_EDITOR_CONTROLS.map((c) => c.id)

/** Controls for one facet. */
export function controlsForFacet(facet: ThemeFacet): ThemeEditorControl[] {
  return THEME_EDITOR_CONTROLS.filter((c) => c.facet === facet)
}

/**
 * True when a write path is allowed by the editor contract.
 * Accepts concrete paths (modes.light.color.primary.default) or template forms.
 *
 * Mode-scoped paths require an explicit control with `modeGroup === group`.
 * Controls that omit `modeGroup` (identity/mode/meta) never authorize modes.*.
 */
export function isContractWritePath(path: string): boolean {
  const p = path.trim()
  if (!p) return false
  // Envelope / meta
  if (p.startsWith('anchors.')) return true
  if (p.startsWith('$extensions.sail.')) {
    const key = p.slice('$extensions.sail.'.length).split('.')[0]
    return (
      key === 'themeId' ||
      key === 'defaultMode' ||
      key === 'supportedModes' ||
      key === 'density' ||
      key === 'layer' ||
      key === 'ordered'
    )
  }
  // Mode-scoped groups — only groups with a contract control (not every schema group)
  const m = /^modes\.(light|dark)\.([a-z][a-z0-9-]*)/.exec(p)
  if (!m) return false
  const group = m[2]!
  return THEME_EDITOR_CONTROLS.some((c) => c.modeGroup === group)
}

/**
 * Schema fields covered by the contract (for THEMES-1 acceptance: every
 * schema field has a control; every control maps a schema field).
 */
export function contractSchemaCoverage(): {
  identity: string[]
  mode: string[]
  scaleGroups: string[]
  component: boolean
} {
  return {
    identity: ['$extensions.sail.themeId'],
    mode: ['$extensions.sail.defaultMode', '$extensions.sail.supportedModes'],
    scaleGroups: [
      ...new Set(
        THEME_EDITOR_CONTROLS.map((c) => c.modeGroup).filter(
          (g): g is string => typeof g === 'string' && g !== 'component',
        ),
      ),
    ],
    component: THEME_EDITOR_CONTROLS.some((c) => c.kind === 'component'),
  }
}
