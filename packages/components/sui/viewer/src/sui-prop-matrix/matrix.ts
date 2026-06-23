// State-sweep derivation for <sui-prop-matrix> (build-spec §3).
//
// Pure transform over the CEM `attributes[]` (cem.rs `attr_entry`):
//   deriveAxes — primary = `variant` if present; secondary = first of
//                `['tone','size']`; else the sole axis `tone ?? size ?? first
//                enum`.
//   buildCells — the full cartesian product of the derived axes → one labelled
//                cell per combination. Each cell carries a STABLE id (the
//                `{#for} key=`) and its merged axis props (the props each
//                per-cell `<sui-render>` applies; the `sui-matrix-cell-click`
//                payload). NO `JSON.stringify` of the props — they flow by
//                identity (`prop:*`), never a serialized attribute.
//
// This is the LOGIC extracted from the studio `matrix.ts`, reproduced here so
// the viewing components are the one home for the sweep (no parallel system).

/** The raw CEM attribute shape (`cem.rs` `attr_entry`): the matrix reads
 *  `values[]` to find enum axes. Received via `prop:props` identity, NOT a
 *  serialized attribute. */
export interface CemAttr {
  name: string
  type?: { text?: string }
  default?: string
  /** Closed set of allowed values when the prop is a union — the axis values. */
  values?: string[]
}

export interface MatrixAxis {
  name: string
  values: string[]
}

export interface MatrixCell {
  /** Stable unique id — the `{#for} key=` and the click payload's `cellId`. */
  id: string
  /** Human label (axis values joined). */
  label: string
  /** The merged axis prop-values applied to this cell's specimen. */
  props: Record<string, string>
}

// When a component has `variant` + another enum, prefer this order for the
// second axis. `tone` is the canonical pair (button, badge, alert); `size` is
// the fallback for tone-less components (input, kbd).
const SECONDARY_AXIS_CANDIDATES = ['tone', 'size'] as const

type EnumAttr = CemAttr & { values: string[] }

function enumAttrs(props: readonly CemAttr[]): EnumAttr[] {
  return props.filter((p): p is EnumAttr => Array.isArray(p.values) && p.values.length > 0)
}

export function deriveAxes(props: readonly CemAttr[]): MatrixAxis[] {
  const enums = enumAttrs(props)
  if (enums.length === 0) return []

  const variant = enums.find((p) => p.name === 'variant')
  if (variant) {
    const secondary = SECONDARY_AXIS_CANDIDATES.map((name) =>
      enums.find((p) => p.name === name && p !== variant),
    ).find((p): p is EnumAttr => p !== undefined)
    if (secondary) {
      return [
        { name: variant.name, values: variant.values },
        { name: secondary.name, values: secondary.values },
      ]
    }
    return [{ name: variant.name, values: variant.values }]
  }

  // No variant: prefer `tone`, then `size`, then the first enum, as the sole axis.
  const fallback =
    enums.find((p) => p.name === 'tone') ?? enums.find((p) => p.name === 'size') ?? enums[0]
  if (!fallback) return []
  return [{ name: fallback.name, values: fallback.values }]
}

export function buildCells(axes: readonly MatrixAxis[]): MatrixCell[] {
  const [a, b] = axes
  if (!a) return []

  if (!b) {
    return a.values.map((v) => ({
      id: `${a.name}-${v}`,
      label: v,
      props: { [a.name]: v },
    }))
  }

  const out: MatrixCell[] = []
  for (const av of a.values) {
    for (const bv of b.values) {
      out.push({
        id: `${a.name}-${av}-${b.name}-${bv}`,
        label: `${av} · ${bv}`,
        props: { [a.name]: av, [b.name]: bv },
      })
    }
  }
  return out
}

/** Does `active` (current knob values) match this cell's axis props? A cell is
 *  the active highlight when EVERY axis value it fixes equals `active`'s. */
export function cellMatchesActive(
  cell: MatrixCell,
  active: Record<string, unknown> | null,
): boolean {
  if (!active) return false
  for (const [k, v] of Object.entries(cell.props)) {
    if (String(active[k]) !== v) return false
  }
  return true
}
