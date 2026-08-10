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
  control: AxisControl
  /** True when omit is a valid rest path (e.g. tone → component.* leaves). */
  optional?: boolean
  /** Handler / identity props not exposed as docs knobs. */
  event?: boolean
  /** Skip checker “must have knob” (value sliders demo-only, name pickers, etc.). */
  docsSkip?: boolean
}

export interface FaceLeaf {
  /** DTCG path, e.g. component.button.bg */
  path: string
  label: string
}

export interface SurfaceContract {
  slug: string
  /** SOURCES tag, or null for alias/action/keywords packages. */
  tag: string | null
  kind: 'host' | 'alias' | 'action' | 'keywords'
  axes: Record<string, AxisDef>
  /** component.* leaves this host actually paints (host paint only). */
  faces: readonly FaceLeaf[]
  visual: {
    /** Zero-config / default row required in l1-conformance rows(). */
    defaultRow: boolean
    /** Paint-affecting axes visual rows should cross. */
    cross: readonly string[]
  }
  examples: boolean
}
