/**
 * L1 surface contract — axes SSOT for docs knobs, Studio faces, visual rows.
 * Axes come from prop-registry / stdlib `props:` only (no invent).
 * See SPEC-L1-SURFACE-SCAFFOLD.md.
 */

export type AxisControl =
  | 'select'
  | 'slider'
  | 'swatches'
  | 'switch'
  | 'text'
  | 'number'
  | 'none'

export interface AxisDef {
  /** Enum values when the prop is a closed union. */
  values?: readonly string[]
  default?: string | boolean | number
  /** Registry-shaped control hint (select/slider/…). */
  control: AxisControl
  /**
   * Docs UI widget when it differs from registry control (e.g. label string prop
   * driven by a decorative/role switch in the configurator).
   */
  docsControl?: AxisControl
  /** True when omit is a valid rest path (e.g. tone → primary / muted roles). */
  optional?: boolean
  /** Handler / identity props not exposed as docs knobs. */
  event?: boolean
  /** Skip checker “must have knob” (value sliders demo-only, name pickers, etc.). */
  docsSkip?: boolean
}

export interface FaceLeaf {
  /** Color role this host spends, e.g. input, card, popover. */
  path: string
  label: string
}

export interface SurfaceContract {
  slug: string
  /** SOURCES tag, or null for alias/action/keywords packages. */
  tag: string | null
  kind: 'host' | 'alias' | 'action' | 'keywords'
  axes: Record<string, AxisDef>
  /**
   * Docs-only demo knobs that are not host props (e.g. progress omit-value).
   */
  demoOnly?: readonly string[]
  /** Color roles this host spends (host paint only). */
  faces: readonly FaceLeaf[]
  visual: {
    /** Docs configurator includes a zero-config / default rest cell. */
    defaultRow: boolean
    /** Paint-affecting axes visual rows should cross. */
    cross: readonly string[]
  }
  examples: boolean
}
