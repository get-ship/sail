// lockstep.ts — the prop-change → render wiring + the no-CEM degradation logic
// for <sui-component-viewer> (build-spec §The viewing components #6 / Slice 5,
// the soft-300 split named up front).
//
// Pure functions over plain values + a signal — NO DOM, NO events emitted here,
// NO view markup. The controller owns the host listeners + emit; this module
// owns the SHAPE of the lockstep: how a child event mutates the shared
// `propValues` store, and the props/slots metadata derivation for the folded
// table. Keeping it controller-free keeps the controller a pure lifecycle shell.

/** A CEM `attributes[]` entry — the host-baked shape from a component's
 *  `.custom-elements.json` (verified `cem.rs` `attr_entry`): `name` +
 *  `type:{text}` + optional `default`/`values`. */
export interface CemAttr {
  name: string
  type?: { text?: string }
  default?: string
  values?: string[]
}

/** The shared propValues snapshot the viewer publishes under
 *  `'viewer:propValues'` and every child `sui-prop-*` reads via `useShared`. */
export type PropValues = Record<string, unknown>

/** The minimal signal surface the lockstep mutates: a getter + `.set(value)`. */
export interface ValueSignal {
  (): PropValues
  set: (next: PropValues) => void
}

/**
 * No-CEM gate (build-spec degradation contract). The viewer renders source +
 * live render + error surfaces ALWAYS; the CEM-driven surfaces (knobs / matrix /
 * folded props-table) are HIDDEN — not errored — when the CEM is
 * absent/empty (the agentic-live path where fresh source has no compiled CEM).
 */
export function degradeMode(cem: readonly CemAttr[] | null | undefined): boolean {
  return cem == null || cem.length === 0
}

/**
 * Wire the cross-child lockstep over the shared `propValues` signal. Returns the
 * three pure detail-handlers the controller's delegated host listeners call.
 * Each merges the child's announced values into the shared store via `merge`
 * (the controller passes identity or a clamp); the shared signal is the single
 * source the stage render / matrix highlight / snippet all re-derive from.
 *
 * The handlers NEVER touch the source signal — the read-only-over-source
 * invariant (the no-IDE fence) is structural here: there is no source param.
 */
export function wireLockstep(
  values: ValueSignal,
  merge: (next: PropValues) => PropValues,
): {
  onPropChange: (detail: { name?: string; value?: unknown; values?: PropValues } | undefined) => void
  onCellClick: (detail: { props?: PropValues; cellId?: string } | undefined) => void
  onReset: (detail: { values?: PropValues } | undefined) => void
} {
  const write = (next: PropValues | undefined): void => {
    if (!next) return
    values.set(merge({ ...next }))
  }
  return {
    // A single knob edit announces the FULL snapshot in `values` — wire it
    // straight in (no bookkeeping, the sui-prop-knobs contract).
    onPropChange: (detail) => write(detail?.values),
    // A matrix cell click announces that cell's axis props — round-trip them
    // into the shared store so the knobs + stage move to that combination.
    onCellClick: (detail) => write(detail?.props),
    // A defaults re-seed (fresh target) announces the seeded snapshot.
    onReset: (detail) => write(detail?.values),
  }
}

/** A folded-props-table row: name + a humanized type + the displayed default
 *  (`—` when the prop declares no default). Pure derivation over the CEM. */
export interface PropRow {
  name: string
  type: string
  def: string
}

/** Build the folded props-table rows from the CEM (name / type / default, with
 *  an em-dash for an absent-or-empty default). Declaration order. */
export function buildPropRows(cem: readonly CemAttr[] | null | undefined): PropRow[] {
  return (cem ?? []).map((attr) => ({
    name: attr.name,
    type: attr.type?.text ?? '—',
    def: attr.default != null && attr.default !== '' ? attr.default : '—',
  }))
}

/** The `'N props · M slots'` meta label (build-spec folded-table contract).
 *  Slots are daemon-only on the static path → `slotCount` is 0 there. */
export function metaLabel(propCount: number, slotCount: number): string {
  const props = `${propCount} ${propCount === 1 ? 'prop' : 'props'}`
  const slots = `${slotCount} ${slotCount === 1 ? 'slot' : 'slots'}`
  return `${props} · ${slots}`
}
