// <sui-component-viewer> controller — THE composition primitive (build-spec
// §The viewing components #6 / Slice 5).
//
// Controller-purity: this file does NO view-markup DOM construction. It owns
// LIFECYCLE only — wiring host `prop:*` into signals, publishing the shared
// `propValues` store under the stable 'viewer:propValues' key, and wiring the
// cross-child lockstep (a knob/matrix/reset event → the shared store → the
// stage render + matrix highlight + snippet, all re-derived). The composition
// (sui-render + sui-prop-knobs + sui-prop-matrix + sui-source-panel + the
// folded props/slots {#for} + sui-tabs/sui-tab-panel + the internal
// sui-error-boundary) lives in the template. No `innerHTML`/`outerHTML`/
// `insertAdjacentHTML`.
//
// READ-ONLY-OVER-SOURCE (the no-IDE fence): this controller NEVER calls `.set`
// on the source signal. Source is an injected, host-owned, read-only signal —
// the only thing the viewer mutates is `propValues`. The lockstep helper has no
// source param, so the invariant is structural (test-enforced, Slice 5/7).
//
// Structured inputs (the live source signal, the CEM `attributes[]`, the bound
// `compile()`, slots/examples) arrive via `prop:*` IDENTITY — set as JS
// properties on the host, NEVER a json/object attribute (`<prop type=json>`
// does not exist).

import { controller, signal, derived, isSignal, provide } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-component-viewer.sui'
import {
  wireLockstep,
  degradeMode,
  buildPropRows,
  metaLabel,
  type CemAttr,
  type PropValues,
  type ValueSignal,
} from './lockstep'
import type { CompileFn } from '../wasm-bootstrap'

const SHARED_KEY = 'viewer:propValues'

/** Read a source prop that may be a raw string OR a live signal identity
 *  (the agentic-live path). A signal is read reactively; a string is constant.
 *  NEVER written — read-only-over-source. */
function readSource(src: unknown): string {
  if (isSignal(src)) return (src as () => unknown)() as string
  return typeof src === 'string' ? src : ''
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    // ── identity prop:* inputs (assigned as JS properties in mount) ──
    const sourceProp = signal<unknown>(null) // a string OR a live signal (identity)
    const cem = signal<CemAttr[] | null>(null)
    const compile = signal<CompileFn | null>(null)
    const slots = signal<string | null>(null)

    // ── the shared single-source-of-truth propValues store ──
    // Published under 'viewer:propValues'; child sui-prop-* read it via
    // useShared (with a prop:value fallback when somehow detached).
    const propValues = signal<PropValues>({})

    // ── the live source TEXT, read reactively from the source prop ──
    // (a flip of the host's source signal re-derives this → the stage render
    // re-mounts — the agentic-live path. Read-only: never `.set`.)
    const sourceText = derived<string>(() => readSource(sourceProp()))

    // ── degradation gate (no CEM ⇒ hide knobs/matrix/props-table, not error) ──
    const degraded = derived<boolean>(() => degradeMode(cem()))
    const showInspector = derived<boolean>(() => !degraded())

    // ── the folded props/slots table (pure derivation over the CEM) ──
    const propRows = derived(() => buildPropRows(cem()))
    const slotCount = derived<number>(() => 0) // slots are daemon-only on the static path
    const meta = derived<string>(() => metaLabel(propRows().length, slotCount()))
    const hasSlots = derived<boolean>(() => slotCount() > 0)

    return {
      sourceProp,
      cem,
      compile,
      slots,
      propValues,
      sourceText,
      degraded,
      showInspector,
      propRows,
      slotCount,
      meta,
      hasSlots,
    }
  },

  mount(state, host) {
    // ── identity prop:* → signal wiring (mirrors sui-render's defineHostProp) ──
    // A structured/identity value (the source signal, the CEM array, the bound
    // `compile` function, the slot string) is assigned to the host PROPERTY, not
    // a serialized attribute. `signal.set(fn)` interprets `fn` as the updater
    // form, so a function- or signal-valued prop must be stored via the
    // `() => value` updater closure to land the literal reference.
    type AnySig = { (): unknown; set: (u: (p: unknown) => unknown) => void }
    const defineHostProp = (name: string, sig: { (): unknown; set: (v: never) => void }): void => {
      const s = sig as unknown as AnySig
      const existing = (host as unknown as Record<string, unknown>)[name]
      if (existing !== undefined) s.set(() => existing)
      Object.defineProperty(host, name, {
        configurable: true,
        get: () => s(),
        set: (v: unknown) => s.set(() => v),
      })
    }
    defineHostProp('source', state.$signals.sourceProp)
    defineHostProp('cem', state.$signals.cem)
    defineHostProp('compile', state.$signals.compile)
    defineHostProp('slots', state.$signals.slots)

    // ── publish the shared propValues store under the stable key ──
    // The auto-publish keys by tagName; the spec mandates 'viewer:propValues',
    // so provide it explicitly. A wrapped value-signal (getter + `.set`) is the
    // child contract (sui-prop-knobs' useShared expects `{ (): values; set }`).
    const shared = state.$signals.propValues as unknown as ValueSignal
    provide(host as Element, SHARED_KEY, shared)

    // ── the cross-child lockstep (helper owns the SHAPE; controller the wiring) ──
    // merge = identity here (the viewer trusts a child's announced snapshot).
    const lock = wireLockstep(shared, (next) => next)

    // The composed children bubble composed events to the host shadow root. One
    // delegated listener per event type mirrors the announced values into the
    // shared store; the stage render / matrix highlight / snippet all re-derive
    // off `propValues`. The events keep bubbling past the host (composed:true),
    // so a host listening on <sui-component-viewer> still sees the raw child
    // events for its own record/log — the viewer does NOT stopPropagation and
    // does NOT re-emit (no double-fire, no dedup bookkeeping).
    const onPropChange = (e: Event): void =>
      lock.onPropChange((e as CustomEvent).detail as Parameters<typeof lock.onPropChange>[0])
    const onCellClick = (e: Event): void =>
      lock.onCellClick((e as CustomEvent).detail as Parameters<typeof lock.onCellClick>[0])
    const onReset = (e: Event): void =>
      lock.onReset((e as CustomEvent).detail as Parameters<typeof lock.onReset>[0])

    host.addEventListener('sui-prop-change', onPropChange)
    host.addEventListener('sui-matrix-cell-click', onCellClick)
    host.addEventListener('sui-prop-reset', onReset)

    return () => {
      host.removeEventListener('sui-prop-change', onPropChange)
      host.removeEventListener('sui-matrix-cell-click', onCellClick)
      host.removeEventListener('sui-prop-reset', onReset)
    }
  },
})
