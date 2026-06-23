// <sui-prop-matrix> controller — the state-sweep lifecycle (build-spec §3).
//
// Controller-purity: this file does NO view-markup DOM construction. It owns
// LIFECYCLE only — wiring host `prop:*` (the CEM `attributes[]`, the bound
// `compile()`, the source, the active prop-values, the slot markup) into
// signals, deriving the cartesian cell grid, computing the active highlight,
// and emitting `sui-matrix-cell-click`. Each cell's live mount is delegated to
// a per-cell `<sui-render>` in the template (the composed engine) — the matrix
// never createElements a specimen itself. No `innerHTML`/`outerHTML`/
// `insertAdjacentHTML`.

import { controller, signal, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-prop-matrix.sui'
import { deriveAxes, buildCells, cellMatchesActive, type CemAttr, type MatrixCell } from './matrix'
import type { CompileFn } from '../wasm-bootstrap'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    // Identity prop:* inputs (assigned as JS properties in mount). Null until
    // the host supplies them — the template hides the grid until cells exist.
    const props = signal<CemAttr[] | null>(null)
    const compile = signal<CompileFn | null>(null)
    const source = signal<string | null>(null)
    const active = signal<Record<string, unknown> | null>(null)
    const slotContent = signal<string | null>(null)

    // The cartesian grid, derived from the CEM. Pure over `props` only — the
    // active highlight is NOT snapshotted into the cell (that would not refresh
    // on a keyed `{#for}` recycle, the sui-table per-row lesson). Instead the
    // template binds `data-active` against the TRACKED `activeId()` signal, so a
    // recycled cell re-evaluates the match in place when `active` changes.
    const cells = derived<MatrixCell[]>(() => buildCells(deriveAxes(props() ?? [])))

    // The id of the cell matching the current `active` prop-values, or '' for
    // none. A tracked read the template compares each `cell.id` against — the
    // single reactive source the active highlight depends on.
    const activeId = derived<string>(() => {
      const act = active()
      const hit = cells().find((cell) => cellMatchesActive(cell, act))
      return hit ? hit.id : ''
    })

    const gridLabel = derived<string>(() => {
      const n = cells().length
      return n === 0 ? 'no states' : `${n} states`
    })

    return { props, compile, source, active, slotContent, cells, activeId, gridLabel }
  },

  mount(state, host) {
    // ── identity prop:* → signal wiring (mirrors sui-render's defineHostProp) ──
    // A structured/identity value (the CEM array, the bound `compile` function,
    // the active record, the slot string) is assigned to the host PROPERTY, not
    // a serialized attribute. `signal.set(fn)` interprets `fn` as the updater
    // form, so a function-valued prop (`compile`) must be stored via the
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
    defineHostProp('props', state.$signals.props)
    defineHostProp('compile', state.$signals.compile)
    defineHostProp('source', state.$signals.source)
    defineHostProp('active', state.$signals.active)
    defineHostProp('slotContent', state.$signals.slotContent)

    // ── per-cell click → sui-matrix-cell-click {props,cellId} ────────────────
    // Delegated shadow listener (the sui-table `data-row-id` pattern): a click
    // on a `[data-matrix-cell]` reads its `data-cell-id`, looks up that cell's
    // axis props, and emits. Lets the host sync the clicked cell's props back
    // into sui-prop-knobs (the studio cell→knob round-trip). `host.emit!`
    // defaults bubbles+composed so the event crosses the shadow boundary. The
    // host listens via the exact name `on:sui-matrix-cell-click`.
    const shadow = host.shadowRoot
    const onCellClick = (e: Event): void => {
      const cellEl = (e.target as HTMLElement).closest('[data-matrix-cell]') as HTMLElement | null
      if (!cellEl) return
      const cellId = cellEl.getAttribute('data-cell-id') ?? ''
      const cell = state.cells.find((c: MatrixCell) => c.id === cellId)
      if (!cell) return
      host.emit!('sui-matrix-cell-click', { props: cell.props, cellId })
    }
    shadow?.addEventListener('click', onCellClick)

    return () => {
      shadow?.removeEventListener('click', onCellClick)
    }
  },
})
